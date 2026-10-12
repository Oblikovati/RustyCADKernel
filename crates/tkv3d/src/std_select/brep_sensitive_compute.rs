//! Real native sensitive construction preserves wire partial state and compound free-subshape ordering.
use super::StdSelectBrepSelectionTool;
use crate::{
    select_mgr::{SelectMgrEntityOwnerSource, SelectMgrSelection},
    select3d::*,
};
use rustycad_tkbrep::{
    brep::BRepTool,
    brep_tools::WireExplorer,
    topexp::{self, Explorer},
    topods::{self, Shape},
    toptools::IndexedMapOfShape,
};
use rustycad_tkernel::standard::HandleReference;
use rustycad_tkernel::standard::{Failure, Handle, catch_native_failure};
use rustycad_tkg3d::top_abs::ShapeEnum;
pub(super) struct BrepSensitiveBuild<'a> {
    pub owner: &'a dyn HandleReference<dyn SelectMgrEntityOwnerSource>,
    pub selection: &'a dyn HandleReference<SelectMgrSelection>,
    pub deflection: f64,
    pub angle: f64,
    pub edge_points: i32,
    pub max_parameter: f64,
    pub auto_triangulation: bool,
}
impl StdSelectBrepSelectionTool {
    /// Construct native point/edge/wire/face/solid/compound entities; `Self::compute_sensitive(&shape,&owner,&selection,0.1,0.5,9,500.,false)?`.
    #[allow(clippy::too_many_arguments)] // Exact native geometry construction overload.
    pub fn compute_sensitive(
        shape: &Shape,
        owner: &dyn HandleReference<dyn SelectMgrEntityOwnerSource>,
        selection: &dyn HandleReference<SelectMgrSelection>,
        deflection: f64,
        angle: f64,
        edge_points: i32,
        max_parameter: f64,
        auto_triangulation: bool,
    ) -> Result<(), Failure> {
        let references = (owner, selection);
        let tolerances = (deflection, angle, max_parameter);
        let criteria = brep_sensitive_criteria(tolerances, edge_points, auto_triangulation);
        let build = BrepSensitiveBuild::from_arguments(references, criteria);
        catch_native_failure(|| compute_brep_sensitive(shape, &build))
    }
}
fn brep_sensitive_criteria(
    tolerances: (f64, f64, f64),
    edge_points: i32,
    auto_triangulation: bool,
) -> (f64, f64, i32, f64, bool) {
    let (deflection, angle, max_parameter) = tolerances;
    (
        deflection,
        angle,
        edge_points,
        max_parameter,
        auto_triangulation,
    )
}
impl<'a> BrepSensitiveBuild<'a> {
    pub(super) fn from_arguments(
        references: (
            &'a dyn HandleReference<dyn SelectMgrEntityOwnerSource>,
            &'a dyn HandleReference<SelectMgrSelection>,
        ),
        criteria: (f64, f64, i32, f64, bool),
    ) -> Self {
        let (owner, selection) = references;
        let (deflection, angle, edge_points, max_parameter, auto_triangulation) = criteria;
        Self {
            owner,
            selection,
            deflection,
            angle,
            edge_points,
            max_parameter,
            auto_triangulation,
        }
    }
}
pub(super) fn compute_brep_sensitive(
    shape: &Shape,
    build: &BrepSensitiveBuild<'_>,
) -> Result<(), Failure> {
    match shape.shape_type()? {
        ShapeEnum::Vertex => compute_brep_vertex_entity(shape, build),
        ShapeEnum::Edge => compute_brep_edge_entity(shape, build),
        ShapeEnum::Wire => compute_brep_wire_entity(shape, build),
        ShapeEnum::Face => compute_brep_face_entities(shape, build),
        ShapeEnum::Shell | ShapeEnum::Solid | ShapeEnum::CompSolid => {
            compute_brep_solid_entities(shape, build)
        }
        _ => compute_brep_compound_entities(shape, build),
    }
}
fn compute_brep_vertex_entity(
    shape: &Shape,
    build: &BrepSensitiveBuild<'_>,
) -> Result<(), Failure> {
    let point = BRepTool::pnt(&topods::vertex(shape)?)?;
    let sensitive = Select3DSensitivePoint::new(&build.owner.observe(), point);
    build
        .selection
        .observe()
        .add(&Handle::new(sensitive).cast())
}
fn brep_build_edge(
    shape: &Shape,
    build: &BrepSensitiveBuild<'_>,
    output: &mut Handle<dyn Select3DSensitiveEntity>,
) -> Result<(), Failure> {
    StdSelectBrepSelectionTool::get_edge_sensitive(
        shape,
        build.owner,
        build.selection,
        build.deflection,
        build.angle,
        build.edge_points,
        build.max_parameter,
        output,
    )
}
fn compute_brep_edge_entity(shape: &Shape, build: &BrepSensitiveBuild<'_>) -> Result<(), Failure> {
    let mut output = Handle::default();
    brep_build_edge(shape, build, &mut output)?;
    if !output.is_null() {
        build.selection.observe().add(&output)?;
    }
    Ok(())
}
fn compute_brep_wire_entity(shape: &Shape, build: &BrepSensitiveBuild<'_>) -> Result<(), Failure> {
    let mut explorer = WireExplorer::with_wire(&topods::wire(shape)?)?;
    let mut output = Handle::default();
    let wire = Handle::new(Select3DSensitiveWire::new(&build.owner.observe()));
    build.selection.observe().add(&wire.cast())?;
    while explorer.more() {
        // OCCT 3d097a0 reuses output: missing geometry may repeat the preceding edge entity.
        brep_build_edge(explorer.current(), build, &mut output)?;
        if !output.is_null() {
            wire.add(&output)?;
        }
        explorer.next()?;
    }
    Ok(())
}
fn compute_brep_face_entities(
    shape: &Shape,
    build: &BrepSensitiveBuild<'_>,
) -> Result<(), Failure> {
    let face = topods::face(shape)?;
    let mut entities = Vec::new();
    StdSelectBrepSelectionTool::get_sensitive_for_face(
        &face,
        build.owner,
        &mut entities,
        build.auto_triangulation,
        build.edge_points,
        build.max_parameter,
        true,
    )?;
    for entity in entities {
        build.selection.observe().add(&entity)?;
    }
    Ok(())
}
fn compute_brep_solid_entities(
    shape: &Shape,
    build: &BrepSensitiveBuild<'_>,
) -> Result<(), Failure> {
    let mut faces = IndexedMapOfShape::default();
    topexp::map_shapes_of_type(shape, ShapeEnum::Face, &mut faces)?;
    if StdSelectBrepSelectionTool::get_sensitive_for_cylinder(&faces, build.owner, build.selection)?
    {
        return Ok(());
    }
    compute_brep_mapped_faces(&faces, build)
}
fn compute_brep_mapped_faces(
    faces: &IndexedMapOfShape,
    build: &BrepSensitiveBuild<'_>,
) -> Result<(), Failure> {
    for index in 1..=faces.extent() {
        compute_brep_sensitive(faces.find_key(index)?, build)?;
    }
    Ok(())
}
fn compute_brep_compound_entities(
    shape: &Shape,
    build: &BrepSensitiveBuild<'_>,
) -> Result<(), Failure> {
    compute_brep_free_subshapes(shape, ShapeEnum::Vertex, ShapeEnum::Edge, build)?;
    compute_brep_free_subshapes(shape, ShapeEnum::Edge, ShapeEnum::Face, build)?;
    compute_brep_free_subshapes(shape, ShapeEnum::Wire, ShapeEnum::Face, build)?;
    let mut faces = IndexedMapOfShape::default();
    topexp::map_shapes_of_type(shape, ShapeEnum::Face, &mut faces)?;
    compute_brep_mapped_faces(&faces, build)
}
fn compute_brep_free_subshapes(
    shape: &Shape,
    kind: ShapeEnum,
    avoid: ShapeEnum,
    build: &BrepSensitiveBuild<'_>,
) -> Result<(), Failure> {
    let mut explorer = Explorer::new(shape, kind, avoid)?;
    while explorer.more() {
        compute_brep_sensitive(explorer.current(), build)?;
        explorer.next()?;
    }
    Ok(())
}
