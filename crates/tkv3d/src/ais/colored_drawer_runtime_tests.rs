//! Named application subclass retains native descriptor ancestry and virtual JSON dispatch.
use super::AisColoredDrawer;
use crate::prs3d::Prs3dDrawer;
use rustycad_tkernel::standard::{Handle, dump::DumpStream};
use rustycad_tkservice::graphic3d::{
    Graphic3dPresentationAttributesBase, Graphic3dPresentationAttributesSource,
};
use std::fmt::Write;

struct ColoredDrawerApplicationFake {
    colored: AisColoredDrawer,
}
rustycad_tkernel::impl_transient!(ColoredDrawerApplicationFake, "MartensiteColoredDrawerProbe", AisColoredDrawer, colored,
    interfaces: [dyn Graphic3dPresentationAttributesSource]);
impl Graphic3dPresentationAttributesSource for ColoredDrawerApplicationFake {
    fn presentation_attributes_base(&self) -> &Graphic3dPresentationAttributesBase {
        let parent = &self.colored;
        parent.presentation_attributes_base()
    }
    fn dump_json(&self, output: &mut DumpStream, _depth: i32) -> std::io::Result<()> {
        output.append("application-json");
        Ok(())
    }
}
fn runtime_names(colored: &Handle<AisColoredDrawer>) -> String {
    let descriptor = colored.dynamic_type().unwrap();
    let parent = descriptor.parent().unwrap();
    let grandparent = parent.parent().unwrap();
    format!(
        "{} {} {}",
        descriptor.name(),
        parent.name(),
        grandparent.name()
    )
}
fn runtime_header(
    output: &mut String,
    application: &Handle<ColoredDrawerApplicationFake>,
    colored: &Handle<AisColoredDrawer>,
    defaults: &Handle<Prs3dDrawer>,
) {
    let names = runtime_names(colored);
    let attributes: Handle<dyn Graphic3dPresentationAttributesSource> = colored.cast();
    let mut json = DumpStream::default();
    attributes.dump_json(&mut json, 0).unwrap();
    drop(attributes);
    let aliases = i32::from(*colored == *application);
    let counts = format!("{} {}", application.ref_count(), defaults.ref_count());
    writeln!(output, "runtime {names} {aliases} {counts} {}", json.text()).unwrap();
}
fn record_independent(output: &mut String, label: &str, drawer: &AisColoredDrawer) {
    let bits = [
        drawer.has_own_material(),
        drawer.has_own_color(),
        drawer.has_own_transparency(),
        drawer.has_own_width(),
    ];
    let serialized: String = bits
        .into_iter()
        .map(|flag| if flag { '1' } else { '0' })
        .collect();
    writeln!(output, "{label} {serialized}").unwrap();
}
fn independent_ownership(output: &mut String, drawer: &AisColoredDrawer) {
    let retained = &drawer.flags;
    drawer.set_own_material();
    assert!(retained.read().has_own_material);
    record_independent(output, "material", drawer);
    drawer.unset_own_material();
    drawer.set_own_color(rustycad_tkernel::quantity::Color::default());
    record_independent(output, "color", drawer);
    drawer.unset_own_color();
    drawer.set_own_transparency(-20.);
    record_independent(output, "transparency", drawer);
    drawer.unset_own_transparency();
    drawer.set_own_width(-20.);
    record_independent(output, "width", drawer);
}
fn inherited_json(output: &mut String, defaults: &Handle<Prs3dDrawer>) {
    let direct = AisColoredDrawer::new(defaults);
    let mut json = DumpStream::default();
    direct.dump_json(&mut json, 0).unwrap();
    writeln!(
        output,
        "inherited-json {}",
        i32::from(json.text().contains("Prs3d_Drawer"))
    )
    .unwrap();
}
fn record_application(output: &mut String, defaults: &Handle<Prs3dDrawer>) {
    let application = Handle::new(ColoredDrawerApplicationFake {
        colored: AisColoredDrawer::new(defaults),
    });
    let base: Handle<Prs3dDrawer> = application.cast();
    let colored: Handle<AisColoredDrawer> = base.cast();
    runtime_header(output, &application, &colored, defaults);
    independent_ownership(output, &colored);
    drop(base);
    drop(colored);
    record_release(output, application, defaults);
}
fn record_release(
    output: &mut String,
    application: Handle<ColoredDrawerApplicationFake>,
    defaults: &Handle<Prs3dDrawer>,
) {
    let counts = format!("{} {}", application.ref_count(), defaults.ref_count());
    writeln!(output, "release {counts}").unwrap();
    drop(application);
    writeln!(output, "destroy {}", defaults.ref_count()).unwrap();
}
#[test]
fn colored_drawer_application_ownership_virtual_dispatch_native_replay() {
    let defaults = Handle::new(Prs3dDrawer::new());
    let mut output = String::new();
    record_application(&mut output, &defaults);
    inherited_json(&mut output, &defaults);
    retained_link(&mut output);
    assert_eq!(
        output,
        include_str!("../../tests/fixtures/colored-drawer/native-runtime.txt")
    );
}

fn link_counts(first: &Handle<Prs3dDrawer>, replacement: &Handle<Prs3dDrawer>) -> String {
    let first_count = first.ref_count();
    format!("{first_count} {}", replacement.ref_count())
}
fn retained_link(output: &mut String) {
    let first = Handle::new(Prs3dDrawer::new());
    let replacement = Handle::new(Prs3dDrawer::new());
    let drawer = AisColoredDrawer::new(&first);
    let alias = drawer.link_ref();
    let second_alias = drawer.link_ref();
    let same_slot = i32::from(alias.is_same_reference(&second_alias));
    let same_target = i32::from(alias.object_address() == first.object_address());
    let counts = link_counts(&first, &replacement);
    writeln!(output, "link-before {same_slot} {same_target} {counts}").unwrap();
    replace_retained_link(output, &drawer, &first, &replacement);
}
fn replace_retained_link(
    output: &mut String,
    drawer: &AisColoredDrawer,
    first: &Handle<Prs3dDrawer>,
    replacement: &Handle<Prs3dDrawer>,
) {
    let alias = drawer.link_ref();
    drawer.set_link(replacement);
    let same_target = i32::from(alias.object_address() == replacement.object_address());
    let counts = link_counts(first, replacement);
    writeln!(output, "link-replaced {same_target} {counts}").unwrap();
    drawer.set_link(&Handle::default());
    let empty = i32::from(alias.is_null());
    let count = replacement.ref_count();
    writeln!(output, "link-cleared {empty} {count}").unwrap();
}
