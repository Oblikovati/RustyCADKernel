//! A Rust extension hook must never intercept native nonvirtual inherited storage access.
use martensite_tkmeshvs::{
    MeshVSDrawer, MeshVSDrawerAttribute as K, MeshVSDrawerSource, MeshVSTool as T,
};
use rustycad_tkernel::standard::Handle;
use rustycad_tkservice::graphic3d::{Graphic3dAspectsLineMembers, Graphic3dMaterialAspect};
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};
struct DrawerAccessorControl {
    actual: MeshVSDrawer,
    redirected: MeshVSDrawer,
    accessor_calls: Arc<AtomicUsize>,
}
rustycad_tkernel::impl_transient!(DrawerAccessorControl,"DrawerAccessorControl",MeshVSDrawer,actual,interfaces:[dyn MeshVSDrawerSource]);
impl MeshVSDrawerSource for DrawerAccessorControl {
    fn drawer_base(&self) -> &MeshVSDrawer {
        self.accessor_calls.fetch_add(1, Ordering::SeqCst);
        &self.redirected
    }
}
fn control(integer: i32) -> Handle<DrawerAccessorControl> {
    let actual = MeshVSDrawer::default();
    actual.set_integer(77, integer);
    actual.set_double(K::BeamWidth as i32, 2.5);
    let redirected = MeshVSDrawer::default();
    redirected.set_integer(77, integer + 100);
    redirected.set_double(K::BeamWidth as i32, 8.75);
    Handle::new(DrawerAccessorControl {
        actual,
        redirected,
        accessor_calls: Arc::new(AtomicUsize::new(0)),
    })
}
fn as_source(owner: &Handle<DrawerAccessorControl>) -> Handle<dyn MeshVSDrawerSource> {
    owner.clone().into_cast()
}
fn factory_presence(source: &Handle<dyn MeshVSDrawerSource>) -> [bool; 4] {
    let material = Graphic3dMaterialAspect::default();
    [
        !T::create_aspect_fill_area3d(source, true)
            .unwrap()
            .is_null(),
        !T::create_aspect_fill_area3d_with_material(source, &material, true)
            .unwrap()
            .is_null(),
        !T::create_aspect_marker3d(source, true).unwrap().is_null(),
        !T::create_aspect_text3d(source, true).unwrap().is_null(),
    ]
}
#[test]
fn qualified_tool_getters_ignore_overridable_accessor_and_preserve_real_counts() {
    let owner = control(42);
    let source = as_source(&owner);
    let line = T::create_aspect_line3d(&source, true).unwrap();
    assert_eq!(line.line_width(), 2.5);
    assert_eq!(factory_presence(&source), [true; 4]);
    assert_eq!(owner.accessor_calls.load(Ordering::SeqCst), 0);
    assert_eq!(owner.ref_count(), 2);
    assert_eq!(line.ref_count(), 1);
}
#[test]
fn default_assign_uses_actual_registered_receiver_and_source_base_fields() {
    let target = control(17);
    let source = control(42);
    let target_source = as_source(&target);
    let source_arg = as_source(&source);
    target_source.assign(&source_arg);
    let mut integer = 0;
    assert!(target.actual.get_integer(77, &mut integer));
    assert_eq!(integer, 42);
    assert!(target.redirected.get_integer(77, &mut integer));
    assert_eq!(integer, 117);
    assert_eq!(target.accessor_calls.load(Ordering::SeqCst), 0);
    assert_eq!(source.accessor_calls.load(Ordering::SeqCst), 0);
    assert_eq!((target.ref_count(), source.ref_count()), (2, 2));
}
#[test]
fn source_nonvirtual_convenience_projects_real_base_without_accessor_callback() {
    let owner = control(42);
    let source = as_source(&owner);
    source.set_integer(77, 51);
    let mut integer = 0;
    assert!(owner.actual.get_integer(77, &mut integer));
    assert_eq!(integer, 51);
    assert!(owner.redirected.get_integer(77, &mut integer));
    assert_eq!(integer, 142);
    assert_eq!(owner.accessor_calls.load(Ordering::SeqCst), 0);
}

fn factory_row(owner: &Handle<DrawerAccessorControl>) -> String {
    let source = as_source(owner);
    let line = T::create_aspect_line3d(&source, true).unwrap();
    let present = factory_presence(&source).map(u8::from);
    format!(
        "factories {} {} {} {} {} {} {} {}",
        line.line_width(),
        present[0],
        present[1],
        present[2],
        present[3],
        owner.accessor_calls.load(Ordering::SeqCst),
        owner.ref_count(),
        line.ref_count()
    )
}
#[test]
fn registered_projection_replays_actual_native_derived_control() {
    let owner = control(42);
    let fixture = include_str!("fixtures/drawer-projection-native.txt");
    assert_eq!(factory_row(&owner), fixture.lines().next().unwrap());
}
