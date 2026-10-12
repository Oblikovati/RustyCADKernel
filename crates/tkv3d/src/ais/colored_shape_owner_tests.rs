//! Actual native parent defaults and custom-drawer destructor ordering replay.
use super::*;
use crate::{prs_mgr::*, select_mgr::SelectMgrSelectableObject};
use rustycad_tkernel::standard::{Handle, RuntimeTyped, Transient, WeakHandle};
use rustycad_tkmath::gp::Point3;
use rustycad_tkservice::{graphic3d::Graphic3dStructureManager, null_driver::NullGraphicDriver};
use std::fmt::Write;
use std::sync::{Arc, Mutex};
struct ColoredOwnerReplay {
    object: Handle<AisColoredShape>,
    prs: Handle<PrsMgrPresentation>,
    key: rustycad_tkbrep::topods::Shape,
    rows: Arc<Mutex<String>>,
}
fn colored_owner_replay() -> ColoredOwnerReplay {
    let key = shape_drawer_oracle_inputs::shape_drawer_vertex(Point3::new(1., 2., 3.));
    let object = AisColoredShape::new(&key);
    let driver = NullGraphicDriver::new_handle();
    let structures = Handle::new(Graphic3dStructureManager::new(&driver).unwrap());
    let manager = PrsMgrPresentationManager::new(&structures);
    let prs = PrsMgrPresentation::new(&manager, &object.clone().cast(), 0).unwrap();
    object
        .presentable_base()
        .fields
        .write()
        .presentations
        .push(prs.clone());
    ColoredOwnerReplay {
        object,
        prs,
        key,
        rows: Arc::new(Mutex::new(String::new())),
    }
}
struct ColoredReleaseDrawerFake {
    colored: AisColoredDrawer,
    owner: WeakHandle<AisColoredShape>,
    prs: Handle<PrsMgrPresentation>,
    rows: Arc<Mutex<String>>,
    label: &'static str,
}
rustycad_tkernel::impl_transient!(
    ColoredReleaseDrawerFake,
    "MartensiteColoredReleaseDrawerProbe",
    AisColoredDrawer,
    colored
);
impl Drop for ColoredReleaseDrawerFake {
    fn drop(&mut self) {
        let owner = self.owner.upgrade();
        let dirty = i32::from(self.prs.must_be_updated());
        let extent = owner.custom_aspects_map().len();
        assert_eq!(owner.custom_aspects_map().is_empty(), extent == 0);
        assert_eq!(owner.change_custom_aspects_map().as_const().len(), extent);
        let keys = owner.shape_base().fields.read().shape.tshape().ref_count();
        let links = owner
            .attributes_ref()
            .with_object(|drawer| drawer.unwrap().transient_base().ref_count());
        writeln!(
            self.rows.lock().unwrap(),
            "{} {dirty} {extent} {keys} {links}",
            self.label
        )
        .unwrap();
    }
}
fn install_release_drawer(scene: &ColoredOwnerReplay, label: &'static str) {
    let drawer = Handle::new(ColoredReleaseDrawerFake {
        colored: AisColoredDrawer::new(&scene.object.attributes()),
        owner: scene.object.downgrade(),
        prs: scene.prs.clone(),
        rows: scene.rows.clone(),
        label,
    });
    scene
        .object
        .change_custom_aspects_map()
        .bind(&scene.key, &drawer.cast())
        .unwrap();
    scene.prs.set_update_status(false);
}
fn parent_uv_defaults(scene: &ColoredOwnerReplay) -> [f64; 7] {
    let fields = scene.object.shape_base().fields.read();
    [
        fields.uv_origin.x(),
        fields.uv_origin.y(),
        fields.uv_repeat.x(),
        fields.uv_repeat.y(),
        fields.uv_scale.x(),
        fields.uv_scale.y(),
        fields.initial_angle,
    ]
}
fn record_parent_defaults(scene: &ColoredOwnerReplay) {
    let kind = scene
        .object
        .presentable_base()
        .fields
        .read()
        .type_of_presentation3d as i32;
    let mut rows = scene.rows.lock().unwrap();
    write!(rows, "defaults").unwrap();
    for value in parent_uv_defaults(scene) {
        write!(rows, " {value}").unwrap();
    }
    let fields = scene.object.shape_base().fields.read();
    let dirty = i32::from(fields.recompute_bounds);
    let void = i32::from(fields.bounds.is_void());
    writeln!(rows, " {dirty} {void} {kind}").unwrap();
    drop(fields);
    drop(rows);
    record_parent_identity(scene);
}
fn record_parent_identity(scene: &ColoredOwnerReplay) {
    let fields = scene.object.shape_base().fields.read();
    let count = scene.key.tshape().ref_count();
    let equal = i32::from(fields.shape.is_equal(&scene.key));
    let signature = scene.object.signature();
    let kind = scene.object.kind_of_interactive() as i32;
    writeln!(
        scene.rows.lock().unwrap(),
        "parent {count} {equal} {signature} {kind}"
    )
    .unwrap();
}
fn record_parent_aspects(scene: &ColoredOwnerReplay) {
    let drawer = scene.object.attributes();
    let line = drawer.line_aspect();
    let aliases = [
        drawer.free_boundary_aspect(),
        drawer.un_free_boundary_aspect(),
        drawer.seen_line_aspect(),
        drawer.face_boundary_aspect(),
    ];
    let mut rows = scene.rows.lock().unwrap();
    write!(rows, "aspects").unwrap();
    for aspect in aliases {
        write!(rows, " {}", i32::from(aspect == line)).unwrap();
    }
    writeln!(rows).unwrap();
}
fn replay_release_unset(scene: &ColoredOwnerReplay, unregister: bool) {
    let label = if unregister {
        "remove-release"
    } else {
        "replace-release"
    };
    install_release_drawer(scene, label);
    scene
        .object
        .unset_custom_aspects(&scene.key, unregister)
        .unwrap();
    let extent = scene.object.custom_aspects_map().len();
    let dirty = i32::from(scene.prs.must_be_updated());
    writeln!(scene.rows.lock().unwrap(), "{label}-final {dirty} {extent}").unwrap();
}
fn replay_release_clear(scene: &ColoredOwnerReplay) {
    install_release_drawer(scene, "clear-release");
    scene.object.clear_custom_aspects().unwrap();
    let dirty = i32::from(scene.prs.must_be_updated());
    let extent = scene.object.custom_aspects_map().len();
    writeln!(scene.rows.lock().unwrap(), "clear-final {dirty} {extent}").unwrap();
    scene.prs.set_update_status(false);
    scene.object.clear_custom_aspects().unwrap();
    writeln!(
        scene.rows.lock().unwrap(),
        "empty-clear {}",
        i32::from(scene.prs.must_be_updated())
    )
    .unwrap();
}
#[test]
fn owned_colored_shape_replays_native_parent_and_release_order() {
    let scene = colored_owner_replay();
    record_parent_defaults(&scene);
    record_parent_aspects(&scene);
    record_owner_runtime(&scene);
    record_owner_modes(&scene);
    record_parent_references(&scene);
    record_alias_creation(&scene);
    record_alias_replacement(&scene);
    record_alias_null_slot(&scene);
    replay_release_unset(&scene, false);
    replay_release_unset(&scene, true);
    replay_release_clear(&scene);
    record_owner_destruction(&scene.rows);
    colored_shape_parent_drop::record_parent_destruction(&scene.rows);
    assert_eq!(
        *scene.rows.lock().unwrap(),
        include_str!("../../tests/fixtures/colored-shape/native-owner.txt")
    );
}

fn record_parent_references(scene: &ColoredOwnerReplay) {
    let replacement = shape_drawer_oracle_inputs::shape_drawer_vertex(Point3::new(4., 5., 6.));
    scene.prs.set_update_status(false);
    record_shape_assignment(scene, &replacement);
    record_drawer_assignment(scene);
    scene.object.set_shape(&scene.key);
}
fn record_shape_assignment(
    scene: &ColoredOwnerReplay,
    replacement: &rustycad_tkbrep::topods::Shape,
) {
    let alias = scene.object.shape_ref();
    scene.object.set_shape(replacement);
    let same = i32::from(alias.is_same_reference(&scene.object.shape_ref()));
    let equal = i32::from(alias.is_equal(replacement));
    let counts = format!(
        "{} {}",
        scene.key.tshape().ref_count(),
        replacement.tshape().ref_count()
    );
    let dirty = i32::from(scene.prs.must_be_updated());
    let row = format!("shape-reference {same} {equal} {counts} {dirty}");
    owner_row(scene, &row);
    let copied = alias.clone_shape();
    assert!(copied.is_equal(replacement));
    assert_eq!(replacement.tshape().ref_count(), 3);
}
fn record_drawer_assignment(scene: &ColoredOwnerReplay) {
    let alias = scene.object.attributes_ref();
    scene
        .object
        .set_attributes(&Handle::new(crate::prs3d::Prs3dDrawer::new()));
    let same = i32::from(alias.is_same_reference(&scene.object.attributes_ref()));
    let equal = i32::from(alias.object_address() == scene.object.attributes().object_address());
    let count = alias.with_object(|drawer| drawer.unwrap().transient_base().ref_count());
    writeln!(
        scene.rows.lock().unwrap(),
        "drawer-reference {same} {equal} {count}"
    )
    .unwrap();
}

fn owner_row(scene: &ColoredOwnerReplay, row: &str) {
    let mut rows = scene.rows.lock().unwrap();
    writeln!(rows, "{row}").unwrap();
}

fn record_alias_creation(scene: &ColoredOwnerReplay) {
    let retained = scene.object.custom_aspects_map();
    let mutable = scene.object.change_custom_aspects_map();
    scene.prs.set_update_status(false);
    scene
        .object
        .unset_custom_aspects(&scene.key, false)
        .unwrap();
    let dirty = i32::from(scene.prs.must_be_updated());
    owner_row(scene, &format!("missing-unset {dirty} {}", retained.len()));
    let child = record_created_alias(scene, &retained, &mutable);
    record_reused_alias(scene, &child);
}
fn record_created_alias(
    scene: &ColoredOwnerReplay,
    retained: &AisShapeDrawerMapRef<'_>,
    mutable: &AisShapeDrawerMapMut<'_>,
) -> Handle<AisColoredDrawer> {
    let child = scene.object.custom_aspects(&scene.key).unwrap();
    let dirty = i32::from(scene.prs.must_be_updated());
    let same = i32::from(retained.is_same_reference(&mutable.as_const()));
    owner_row(
        scene,
        &format!("created-alias {dirty} {} {same}", retained.len()),
    );
    child
}
fn record_reused_alias(scene: &ColoredOwnerReplay, child: &Handle<AisColoredDrawer>) {
    scene.prs.set_update_status(false);
    let repeat = scene.object.custom_aspects(&scene.key.reversed()).unwrap();
    let dirty = i32::from(scene.prs.must_be_updated());
    owner_row(
        scene,
        &format!("reused-alias {dirty} {}", i32::from(repeat == *child)),
    );
}
fn record_alias_replacement(scene: &ColoredOwnerReplay) {
    let retained = scene.object.custom_aspects_map();
    let alias = retained.node_ref(&scene.key).unwrap();
    let address = alias.value_reference_address().unwrap();
    let previous = alias.clone_drawer().unwrap();
    previous.set_hidden(true);
    scene.prs.set_update_status(false);
    scene
        .object
        .unset_custom_aspects(&scene.key, false)
        .unwrap();
    record_replaced_slot(scene, &alias, address, &previous);
    scene.prs.set_update_status(false);
    scene.object.unset_custom_aspects(&scene.key, true).unwrap();
    let dirty = i32::from(scene.prs.must_be_updated());
    owner_row(scene, &format!("removed-alias {dirty} {}", retained.len()));
}
fn record_replaced_slot(
    scene: &ColoredOwnerReplay,
    alias: &AisShapeDrawerNodeRef<'_>,
    address: *const (),
    previous: &Handle<AisColoredDrawer>,
) {
    let dirty = i32::from(scene.prs.must_be_updated());
    let current = scene
        .object
        .custom_aspects_map()
        .node_ref(&scene.key)
        .unwrap();
    let same = i32::from(current.value_reference_address().unwrap() == address);
    let changed = i32::from(alias.drawer_address().unwrap() != previous.object_address());
    let hidden = i32::from(alias.clone_drawer().unwrap().is_hidden());
    owner_row(
        scene,
        &format!("replaced-alias {dirty} {same} {changed} {hidden}"),
    );
}
fn record_alias_null_slot(scene: &ColoredOwnerReplay) {
    let mutable = scene.object.change_custom_aspects_map();
    scene.prs.set_update_status(false);
    mutable
        .bind(&scene.key.reversed(), &Handle::default())
        .unwrap();
    let dirty = i32::from(scene.prs.must_be_updated());
    owner_row(
        scene,
        &format!("raw-bind {dirty} {}", mutable.as_const().len()),
    );
    let child = scene.object.custom_aspects(&scene.key).unwrap();
    record_null_filled(scene, &mutable);
    assert!(!child.is_null());
    scene.object.clear_custom_aspects().unwrap();
}
fn record_null_filled(scene: &ColoredOwnerReplay, mutable: &AisShapeDrawerMapMut<'_>) {
    let dirty = i32::from(scene.prs.must_be_updated());
    let map = mutable.as_const();
    let orientation = map.node_ref(&scene.key).unwrap().orientation().unwrap() as i32;
    owner_row(
        scene,
        &format!("filled-null {dirty} {} {orientation}", map.len()),
    );
}

fn record_owner_runtime(scene: &ColoredOwnerReplay) {
    let descriptor = scene.object.dynamic_type().unwrap();
    let parent = descriptor.parent().unwrap();
    let grandparent = parent.parent().unwrap();
    let same = i32::from(std::ptr::eq(descriptor, AisColoredShape::static_type()));
    let names = format!(
        "{} {} {}",
        AisColoredShape::TYPE_NAME,
        parent.name(),
        grandparent.name()
    );
    owner_row(scene, &format!("runtime-base {names} {same}"));
}
#[test]
fn partial_colored_shape_explicit_geometry_failures_remain_unclaimed() {
    let scene = colored_owner_replay();
    let presentation = scene.prs.clone().cast();
    let error = scene
        .object
        .compute(&Handle::default(), &presentation, 17)
        .unwrap_err();
    assert_eq!(error.exception_type(), "Standard_NotImplemented");
    assert!(error.message().contains("mode=17"));
    let error = scene
        .object
        .compute_selection(&Handle::default(), 18)
        .unwrap_err();
    assert_eq!(error.exception_type(), "Standard_NotImplemented");
    assert!(error.message().contains("mode=18"));
}

fn record_owner_modes(scene: &ColoredOwnerReplay) {
    let flags: Vec<_> = (-1..=3)
        .map(|mode| i32::from(scene.object.accept_display_mode(mode)).to_string())
        .collect();
    owner_row(scene, &format!("display-modes {}", flags.join(" ")));
}

struct ColoredOwnerDropDrawerFake {
    retained_key: rustycad_tkbrep::topods::Shape,
    colored: AisColoredDrawer,
    rows: Arc<Mutex<String>>,
}
rustycad_tkernel::impl_transient!(
    ColoredOwnerDropDrawerFake,
    "MartensiteColoredOwnerDropDrawer",
    AisColoredDrawer,
    colored
);
impl Drop for ColoredOwnerDropDrawerFake {
    fn drop(&mut self) {
        let keys = self.retained_key.tshape().ref_count();
        let links = self
            .colored
            .link_ref()
            .with_object(|link| link.unwrap().transient_base().ref_count());
        writeln!(self.rows.lock().unwrap(), "owner-release {keys} {links}").unwrap();
    }
}
fn record_owner_destruction(rows: &Arc<Mutex<String>>) {
    let key = shape_drawer_oracle_inputs::shape_drawer_vertex(Point3::new(1., 2., 3.));
    let object = AisColoredShape::new(&key);
    let drawer = owner_drop_drawer(&key, &object.attributes(), rows);
    object
        .change_custom_aspects_map()
        .bind(&key, &drawer.cast())
        .unwrap();
    drop(drawer);
    drop(object);
}
fn owner_drop_drawer(
    key: &rustycad_tkbrep::topods::Shape,
    link: &Handle<crate::prs3d::Prs3dDrawer>,
    rows: &Arc<Mutex<String>>,
) -> Handle<ColoredOwnerDropDrawerFake> {
    let colored = AisColoredDrawer::new(link);
    Handle::new(ColoredOwnerDropDrawerFake {
        colored,
        retained_key: key.clone(),
        rows: rows.clone(),
    })
}
