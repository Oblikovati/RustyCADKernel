//! Actual stored presentations expose native first-mesh wire invalidation and repeated reuse.
#[path = "support/sensitive_display_driver.rs"]
mod mesh_display_driver;
use rustycad_tkbrep::{brep_tools, topods::Shape};
use rustycad_tkernel::{precision, standard::Handle};
use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakeBox;
use rustycad_tkservice::graphic3d::{Graphic3dStructureMembers, Graphic3dStructureSource};
use rustycad_tkv3d::{ais::*, prs_mgr::*, v3d::V3dViewer};
use std::fmt::Write;
struct ColoredMeshPresentationFixture {
    shape: Shape,
    object: Handle<AisColoredShape>,
    manager: Handle<PrsMgrPresentationManager>,
    wire: Handle<PrsMgrPresentation>,
    shade: Handle<PrsMgrPresentation>,
}
impl ColoredMeshPresentationFixture {
    fn new(automatic: bool, iso: bool) -> Self {
        let shape = colored_mesh_box();
        let object = AisColoredShape::new(&shape);
        configure_colored_mesh(&object, automatic, iso);
        let (manager, wire, shade) = attached_colored_mesh_presentations(&object);
        Self {
            shape,
            object,
            manager,
            wire,
            shade,
        }
    }
    fn run(&self, automatic: bool, iso: bool, repeat: usize) -> String {
        self.wire.set_update_status(false);
        self.shade.set_update_status(false);
        self.shade.clear(true);
        let before = brep_tools::triangulation(&self.shape, precision::infinite(), false).unwrap();
        self.object
            .compute(
                &self.manager,
                &self.shade.cast::<dyn Graphic3dStructureSource>(),
                1,
            )
            .unwrap();
        let after = brep_tools::triangulation(&self.shape, precision::infinite(), false).unwrap();
        record_colored_mesh_flags(self, (automatic, iso, repeat), (before, after))
    }
}
fn colored_mesh_box() -> Shape {
    let mut builder = BRepPrimAPIMakeBox::from_sizes(2., 3., 4.).unwrap();
    builder.solid().unwrap().into_shape()
}
fn attached_colored_mesh_presentations(
    object: &Handle<AisColoredShape>,
) -> (
    Handle<PrsMgrPresentationManager>,
    Handle<PrsMgrPresentation>,
    Handle<PrsMgrPresentation>,
) {
    let manager = colored_mesh_presentation_manager();
    let wire = PrsMgrPresentation::new(&manager, &object.cast(), 0).unwrap();
    let shade = PrsMgrPresentation::new(&manager, &object.cast(), 1).unwrap();
    attach_colored_mesh_presentations(object, &wire, &shade);
    (manager, wire, shade)
}
fn attach_colored_mesh_presentations(
    object: &AisColoredShape,
    wire: &Handle<PrsMgrPresentation>,
    shade: &Handle<PrsMgrPresentation>,
) {
    let parent = object.presentable_base();
    parent
        .fields
        .write()
        .presentations
        .extend([wire.clone(), shade.clone()]);
}
fn configure_colored_mesh(object: &AisColoredShape, automatic: bool, iso: bool) {
    let drawer = object.attributes();
    drawer.setup_own_defaults().unwrap();
    drawer.set_auto_triangulation(automatic);
    drawer.set_iso_on_triangulation(iso);
}
fn colored_mesh_presentation_manager() -> Handle<PrsMgrPresentationManager> {
    let (driver, _) = mesh_display_driver::SensitiveDisplayDriver::new_handle(0);
    let viewer = V3dViewer::new(&driver).unwrap();
    PrsMgrPresentationManager::new(&viewer.structure_manager())
}
fn record_colored_mesh_flags(
    fixture: &ColoredMeshPresentationFixture,
    scenario: (bool, bool, usize),
    mesh: (bool, bool),
) -> String {
    let (automatic, iso, repeat) = scenario;
    let wire = i32::from(fixture.wire.must_be_updated());
    let shade = i32::from(fixture.shade.must_be_updated());
    let automatic = i32::from(automatic);
    let iso = i32::from(iso);
    let before = i32::from(mesh.0);
    let after = i32::from(mesh.1);
    format!(
        "tessellate {automatic} {iso} {repeat} {before} {after} {wire} {shade} {}\n",
        fixture.shade.groups().len()
    )
}
#[test]
fn native_colored_first_tessellation_updates_wire_mode_only_when_iso_on_mesh_then_reuses() {
    let mut rows = String::new();
    for (automatic, iso) in [(false, false), (false, true), (true, false), (true, true)] {
        let fixture = ColoredMeshPresentationFixture::new(automatic, iso);
        for repeat in 0..2 {
            write!(rows, "{}", fixture.run(automatic, iso, repeat)).unwrap();
        }
    }
    assert_eq!(
        rows,
        include_str!("fixtures/colored-shape-compute/native-tessellate.txt")
    );
}
