//! Real NewGroup callbacks replace the caller's presentation field between native computation stages.
#[path = "support/colored_presentation_argument_probe.rs"]
mod argument_probe;
#[path = "support/sensitive_display_driver.rs"]
mod display_driver;
use rustycad_tkernel::{
    quantity::{Color, NamedColor},
    standard::{Handle, HandleFieldRef, SharedState, WeakHandle},
};
use rustycad_tkservice::{aspect::AspectTypeOfFacingModel, graphic3d::*};
use rustycad_tkv3d::{
    ais::*,
    prs_mgr::{PrsMgrPresentableObject, PrsMgrPresentableObjectMembers, PrsMgrPresentationManager},
    prs3d::Prs3dShadingAspect,
    v3d::V3dViewer,
};
use std::{fmt::Write, sync::Arc};
struct PresentationSlotReplacementProbe {
    slot: Arc<SharedState<Handle<dyn Graphic3dStructureSource>>>,
    incoming: WeakHandle<dyn Graphic3dStructureSource>,
}
impl display_driver::SensitiveGroupCreationCallback for PresentationSlotReplacementProbe {
    fn group_created(&self) {
        // The source field changes while the original receiver's already allocated group stays local.
        *self.slot.write() = self.incoming.observe().clone();
    }
}
#[test]
fn native_compute_rereads_original_presentation_slot_after_group_creation_callback() {
    let mut rows = String::new();
    for mode in [0, 1] {
        replay_presentation_swap(mode, &mut rows);
    }
    let native = include_str!("fixtures/colored-shape-compute/native-presentation-slot.txt");
    assert_eq!(rows, native);
}
#[test]
fn native_primary_virtual_compute_forwards_original_manager_and_presentation_fields() {
    let rows = Arc::new(SharedState::new(String::new()));
    for mode in [0, 1] {
        replay_virtual_presentation_swap(mode, &rows);
    }
    let native =
        include_str!("fixtures/colored-shape-compute/native-virtual-presentation-slot.txt");
    assert_eq!(*rows.read(), native);
}
struct PresentationSwapFixture {
    previous: Handle<dyn Graphic3dStructureSource>,
    incoming: Handle<dyn Graphic3dStructureSource>,
    slot: Arc<SharedState<Handle<dyn Graphic3dStructureSource>>>,
    trace: display_driver::DisplayTrace,
    _viewer: Handle<V3dViewer>,
}
impl PresentationSwapFixture {
    fn new() -> Self {
        let (driver, trace) = display_driver::SensitiveDisplayDriver::new_handle(-1);
        let viewer = V3dViewer::new(&driver).unwrap();
        let previous = swap_presentation(&viewer);
        let incoming = swap_presentation(&viewer);
        let slot = Arc::new(SharedState::new(previous.clone()));
        // Presentation ownership ends before its native viewer/driver scope.
        Self {
            previous,
            incoming,
            slot,
            trace,
            _viewer: viewer,
        }
    }
    fn arm(&self) {
        self.trace.write().new_group_callback = Some(Box::new(PresentationSlotReplacementProbe {
            slot: self.slot.clone(),
            incoming: self.incoming.downgrade(),
        }));
    }
}
fn swap_presentation(viewer: &V3dViewer) -> Handle<dyn Graphic3dStructureSource> {
    Graphic3dStructure::new(&viewer.structure_manager(), &Handle::default())
        .unwrap()
        .cast()
}
fn replay_presentation_swap(mode: i32, rows: &mut String) {
    let object = presentation_swap_object();
    let fixture = PresentationSwapFixture::new();
    fixture.arm();
    let reference = HandleFieldRef::new(&fixture.slot, presentation_slot_field);
    ais_colored_shape_compute(&*object, &reference, mode).unwrap();
    let replaced = i32::from(*fixture.slot.read() == fixture.incoming);
    let previous = fixture.previous.groups().len();
    let incoming = fixture.incoming.groups().len();
    writeln!(
        rows,
        "presentation-slot {mode} {replaced} {previous} {incoming}"
    )
    .unwrap();
}
fn replay_virtual_presentation_swap(mode: i32, rows: &Arc<SharedState<String>>) {
    let fixture = PresentationSwapFixture::new();
    let previous_manager = PrsMgrPresentationManager::new(&fixture._viewer.structure_manager());
    let incoming_manager = PrsMgrPresentationManager::new(&fixture._viewer.structure_manager());
    let input = virtual_argument_input(&fixture, &previous_manager, &incoming_manager, rows);
    let shape = presentation_swap_shape();
    let object = argument_probe::PresentationArgumentProbe::new(&shape, &input);
    configure_swap_object(&*object, &shape);
    fixture.arm();
    invoke_virtual_argument_compute(&object, &input, mode);
    let previous = fixture.previous.groups().len();
    let incoming = fixture.incoming.groups().len();
    let replaced = i32::from(*fixture.slot.read() == fixture.incoming);
    writeln!(
        &mut *rows.write(),
        "presentation-slot {mode} {replaced} {previous} {incoming}"
    )
    .unwrap();
}
fn virtual_argument_input(
    fixture: &PresentationSwapFixture,
    previous: &Handle<PrsMgrPresentationManager>,
    incoming: &Handle<PrsMgrPresentationManager>,
    rows: &Arc<SharedState<String>>,
) -> Arc<argument_probe::PresentationArgumentInput> {
    Arc::new(argument_probe::PresentationArgumentInput {
        manager: Arc::new(SharedState::new(previous.clone())),
        presentation: fixture.slot.clone(),
        incoming_manager: incoming.downgrade(),
        incoming_presentation: fixture.incoming.downgrade(),
        rows: rows.clone(),
    })
}
fn invoke_virtual_argument_compute(
    object: &Handle<argument_probe::PresentationArgumentProbe>,
    input: &argument_probe::PresentationArgumentInput,
    mode: i32,
) {
    let manager = HandleFieldRef::new(&input.manager, argument_probe::manager_argument_field);
    let presentation = HandleFieldRef::new(
        &input.presentation,
        argument_probe::presentation_argument_field,
    );
    let source = object.observe_as::<dyn PrsMgrPresentableObject>();
    source.compute(&manager, &presentation, mode).unwrap();
}
fn presentation_slot_field(
    value: &Handle<dyn Graphic3dStructureSource>,
) -> &Handle<dyn Graphic3dStructureSource> {
    // Pointer identity belongs to this actual application argument field, not an owning snapshot.
    value
}
fn presentation_swap_object() -> Handle<AisColoredShape> {
    let shape = presentation_swap_shape();
    let object = AisColoredShape::new(&shape);
    configure_swap_object(&*object, &shape);
    object
}
pub(super) fn presentation_swap_shape() -> rustycad_tkbrep::topods::Shape {
    let shape = rustycad_tkprim::brep_prim_api::BRepPrimAPIMakeBox::from_sizes(2., 3., 4.)
        .unwrap()
        .solid()
        .unwrap()
        .into_shape();
    rustycad_tkmesh::brep_mesh::BRepMeshIncrementalMesh::with_shape(&shape, 0.1, false, 0.5, false)
        .unwrap();
    shape
}
fn configure_swap_object<S: AisColoredShapeSource + ?Sized>(
    object: &S,
    shape: &rustycad_tkbrep::topods::Shape,
) {
    object.attributes().setup_own_defaults().unwrap();
    object.attributes().set_auto_triangulation(false);
    object.attributes().set_face_boundary_draw(true);
    configure_swap_custom_face(object, shape);
}
fn configure_swap_custom_face<S: AisColoredShapeSource + ?Sized>(
    object: &S,
    shape: &rustycad_tkbrep::topods::Shape,
) {
    let face = rustycad_tkbrep::topexp::Explorer::new(
        shape,
        rustycad_tkg3d::top_abs::ShapeEnum::Face,
        rustycad_tkg3d::top_abs::ShapeEnum::Shape,
    )
    .unwrap()
    .current()
    .clone();
    let drawer = object.custom_aspects(&face).unwrap();
    drawer.set_shading_aspect(swap_custom_shading());
}
fn swap_custom_shading() -> Handle<Prs3dShadingAspect> {
    let shading = Handle::new(Prs3dShadingAspect::new().unwrap());
    shading
        .set_color(
            Color::named(NamedColor::BLUE),
            AspectTypeOfFacingModel::BothSide,
        )
        .unwrap();
    shading
}
