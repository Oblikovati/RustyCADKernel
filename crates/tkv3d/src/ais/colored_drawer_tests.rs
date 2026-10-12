use super::{AisColoredDrawer, AisColoredDrawerParent};
use crate::prs3d::{Prs3dDrawer, Prs3dLineAspect};
use rustycad_tkernel::{
    quantity::{Color, NamedColor},
    standard::{Handle, RuntimeTyped},
};
use rustycad_tkservice::{
    aspect::AspectTypeOfLine,
    graphic3d::{Graphic3dAspectsLineMembers, Graphic3dAspectsMembers},
};
use std::fmt::Write;

fn linked_colored_drawer() -> (Handle<Prs3dDrawer>, Handle<AisColoredDrawer>) {
    let defaults = Handle::new(Prs3dDrawer::new());
    defaults.set_discretisation(31);
    let line =
        Prs3dLineAspect::new(Color::named(NamedColor::BLUE), AspectTypeOfLine::Solid, 2.5).unwrap();
    defaults.set_line_aspect(Handle::new(line));
    let drawer = Handle::new(AisColoredDrawer::new(&defaults));
    (defaults, drawer)
}
fn ownership_bits(drawer: &AisColoredDrawer) -> String {
    let flags = [
        drawer.is_hidden(),
        drawer.has_own_material(),
        drawer.has_own_color(),
        drawer.has_own_transparency(),
        drawer.has_own_width(),
    ];
    flags
        .into_iter()
        .map(|flag| if flag { '1' } else { '0' })
        .collect()
}
fn record_flags(output: &mut String, index: i32, drawer: &AisColoredDrawer) {
    let bits = ownership_bits(drawer);
    let width = drawer.line_aspect().aspect().line_width();
    let discretisation = drawer.discretisation();
    let owns_line = i32::from(drawer.has_own_line_aspect());
    let owns_shading = i32::from(drawer.has_own_shading_aspect());
    let color = drawer.line_aspect().aspect().interior_color();
    let blue = i32::from(color == Color::named(NamedColor::BLUE));
    writeln!(
        output,
        "{index} {bits} {discretisation} {owns_line} {width} {owns_shading} {blue}"
    )
    .unwrap();
}
fn mark_ownership(drawer: &AisColoredDrawer, argument: f64) {
    drawer.set_hidden(true);
    drawer.set_own_material();
    drawer.set_own_color(Color::named(NamedColor::RED));
    drawer.set_own_transparency(argument);
    drawer.set_own_width(argument);
}
fn clear_ownership(drawer: &AisColoredDrawer) {
    drawer.set_hidden(false);
    drawer.unset_own_material();
    drawer.unset_own_color();
    drawer.unset_own_transparency();
    drawer.unset_own_width();
}
fn record_aliases(
    output: &mut String,
    drawer: &Handle<AisColoredDrawer>,
    defaults: &Handle<Prs3dDrawer>,
) {
    let base: Handle<AisColoredDrawerParent> = drawer.cast();
    let back: Handle<AisColoredDrawer> = base.cast();
    let aliases = [back == *drawer, base == *drawer, drawer.link() == *defaults];
    let descriptor = AisColoredDrawer::static_type();
    let names = format!(
        "{} {}",
        descriptor.name(),
        descriptor.parent().unwrap().name()
    );
    let counts = format!("{} {}", drawer.ref_count(), defaults.ref_count());
    let alias_bits = aliases.map(i32::from);
    let [first, second, third] = alias_bits;
    writeln!(output, "aliases {first} {second} {third} {counts} {names}").unwrap();
}
fn record_public_fields(output: &mut String, drawer: &AisColoredDrawer) {
    let retained_flags = &drawer.flags;
    drawer.set_hidden(true);
    write!(
        output,
        "field {} ",
        i32::from(retained_flags.read().is_hidden)
    )
    .unwrap();
    *retained_flags.write() = super::AisColoredDrawerFlags {
        is_hidden: false,
        has_own_material: true,
        has_own_color: true,
        has_own_transparency: true,
        has_own_width: true,
    };
    record_flags(output, 40, drawer);
}
#[test]
fn colored_drawer_native_flags_aliases_and_ignored_arguments() {
    let (defaults, drawer) = linked_colored_drawer();
    let mut output = String::new();
    record_flags(&mut output, 0, &drawer);
    replay_ignored_arguments(&mut output, &drawer);
    defaults.set_discretisation(73);
    record_flags(&mut output, 20, &drawer);
    drawer.set_discretisation(91);
    record_flags(&mut output, 21, &drawer);
    record_aliases(&mut output, &drawer, &defaults);
    record_public_fields(&mut output, &drawer);
    record_unlinked(&mut output);
    record_types(&mut output, &drawer);
    assert_eq!(
        output,
        include_str!("../../tests/fixtures/colored-drawer/native-flags.txt")
    );
}
fn replay_ignored_arguments(output: &mut String, drawer: &AisColoredDrawer) {
    for (index, argument) in [-2., 0., 2., f64::INFINITY, f64::NAN]
        .into_iter()
        .enumerate()
    {
        mark_ownership(drawer, argument);
        record_flags(output, 1 + 2 * index as i32, drawer);
        clear_ownership(drawer);
        record_flags(output, 2 + 2 * index as i32, drawer);
    }
}
fn record_unlinked(output: &mut String) {
    let drawer = AisColoredDrawer::new(&Handle::default());
    let linked = i32::from(drawer.has_link());
    let bits = ownership_bits(&drawer);
    writeln!(output, "unlinked {linked} {bits}").unwrap();
}

fn record_types(output: &mut String, drawer: &Handle<AisColoredDrawer>) {
    let name = AisColoredDrawer::TYPE_NAME;
    let same = std::ptr::eq(
        AisColoredDrawer::static_type(),
        drawer.dynamic_type().unwrap(),
    );
    writeln!(output, "types {name} {}", i32::from(same)).unwrap();
}
