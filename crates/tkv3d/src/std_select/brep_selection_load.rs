//! Native selection loading allocates genuine BRep owners and applies selectable origins after all sensitive construction.
use super::brep_sensitive_compute::{BrepSensitiveBuild, compute_brep_sensitive};
use super::{StdSelectBRepOwner, StdSelectBrepSelectionTool};
use crate::select_mgr::{SelectMgrSelectableObject, SelectMgrSelection};
use rustycad_tkbrep::{brep_tools, topexp, topods::Shape, toptools::IndexedMapOfShape};
use rustycad_tkernel::{
    precision,
    standard::{Failure, Handle, HandleReference, catch_native_failure},
};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmesh::brep_mesh::BRepMeshIncrementalMesh;
impl StdSelectBrepSelectionTool {
    /// Append native selection entities and actual BRep owners; `Self::load(&selection,&shape,ShapeEnum::Face,0.1,0.5,false,-1,9,500.)?`.
    #[allow(clippy::too_many_arguments)] // Pinned native overload and defaults.
    pub fn load(
        selection: &dyn HandleReference<SelectMgrSelection>,
        shape: &Shape,
        kind: ShapeEnum,
        deflection: f64,
        angle: f64,
        auto_triangulation: bool,
        priority: i32,
        edge_points: i32,
        max_parameter: f64,
    ) -> Result<(), Failure> {
        let tolerances = (deflection, angle, max_parameter);
        let sampling = (auto_triangulation, priority, edge_points);
        let criteria = brep_load_criteria(tolerances, sampling);
        load_brep_shape_checked(selection, shape, kind, criteria)
    }

    /// Load first, then assign the origin to all old and new owners; `Self::load_with_selectable(&selection,&object,&shape,ShapeEnum::Shape,0.1,0.5,false,-1,9,500.)?`.
    #[allow(clippy::too_many_arguments)] // Actual native origin overload.
    pub fn load_with_selectable(
        selection: &dyn HandleReference<SelectMgrSelection>,
        selectable: &Handle<dyn SelectMgrSelectableObject>,
        shape: &Shape,
        kind: ShapeEnum,
        deflection: f64,
        angle: f64,
        auto_triangulation: bool,
        priority: i32,
        edge_points: i32,
        max_parameter: f64,
    ) -> Result<(), Failure> {
        let tolerances = (deflection, angle, max_parameter);
        let sampling = (auto_triangulation, priority, edge_points);
        let criteria = brep_load_criteria(tolerances, sampling);
        load_brep_origin_checked(selection, selectable, shape, kind, criteria)
    }
}
type BrepLoadCriteria = (f64, f64, bool, i32, i32, f64);
fn load_brep_shape_checked(
    selection: &dyn HandleReference<SelectMgrSelection>,
    shape: &Shape,
    kind: ShapeEnum,
    criteria: BrepLoadCriteria,
) -> Result<(), Failure> {
    let priority = brep_load_priority(shape, kind, criteria.3)?;
    let owner = Handle::default();
    let build = brep_load_sensitive_build(&owner, selection, criteria);
    catch_native_failure(|| load_brep_geometry(shape, kind, priority, &build))
}
fn brep_load_criteria(tolerances: (f64, f64, f64), sampling: (bool, i32, i32)) -> BrepLoadCriteria {
    let (deflection, angle, max_parameter) = tolerances;
    let (auto_triangulation, priority, edge_points) = sampling;
    (
        deflection,
        angle,
        auto_triangulation,
        priority,
        edge_points,
        max_parameter,
    )
}
fn brep_load_priority(shape: &Shape, kind: ShapeEnum, requested: i32) -> Result<i32, Failure> {
    if requested == -1 {
        return StdSelectBrepSelectionTool::get_standard_priority(shape, kind);
    }
    Ok(requested)
}
fn brep_load_sensitive_build<'a>(
    owner: &'a Handle<dyn crate::select_mgr::SelectMgrEntityOwnerSource>,
    selection: &'a dyn HandleReference<SelectMgrSelection>,
    criteria: BrepLoadCriteria,
) -> BrepSensitiveBuild<'a> {
    let (deflection, angle, auto_triangulation, _, edge_points, max_parameter) = criteria;
    let sampling = (
        deflection,
        angle,
        edge_points,
        max_parameter,
        auto_triangulation,
    );
    BrepSensitiveBuild::from_arguments((owner, selection), sampling)
}

fn load_brep_origin_checked(
    selection: &dyn HandleReference<SelectMgrSelection>,
    selectable: &Handle<dyn SelectMgrSelectableObject>,
    shape: &Shape,
    kind: ShapeEnum,
    criteria: BrepLoadCriteria,
) -> Result<(), Failure> {
    load_brep_shape_checked(selection, shape, kind, criteria)?;
    set_brep_selection_origin(&selection.observe(), selectable);
    Ok(())
}

fn load_brep_geometry(
    shape: &Shape,
    kind: ShapeEnum,
    priority: i32,
    build: &BrepSensitiveBuild<'_>,
) -> Result<(), Failure> {
    if build.auto_triangulation && !brep_tools::triangulation(shape, precision::infinite(), true)? {
        BRepMeshIncrementalMesh::with_shape(shape, build.deflection, false, build.angle, false)?;
    }
    if kind == ShapeEnum::Compound || kind == ShapeEnum::Shape {
        return load_brep_owned_shape(shape, priority, false, build);
    }
    let mut shapes = IndexedMapOfShape::default();
    topexp::map_shapes_of_type(shape, kind, &mut shapes)?;
    let decomposed = shapes.extent() != 1 || !shape.is_equal(shapes.find_key(1)?);
    for index in 1..=shapes.extent() {
        load_brep_owned_shape(shapes.find_key(index)?, priority, decomposed, build)?;
    }
    Ok(())
}
fn load_brep_owned_shape(
    shape: &Shape,
    priority: i32,
    decomposed: bool,
    build: &BrepSensitiveBuild<'_>,
) -> Result<(), Failure> {
    let owner = StdSelectBRepOwner::with_shape(shape, priority, decomposed).into_cast();
    let build = BrepSensitiveBuild {
        owner: &owner,
        ..*build
    };
    compute_brep_sensitive(shape, &build)
}
pub(crate) fn set_brep_selection_origin(
    selection: &SelectMgrSelection,
    selectable: &Handle<dyn SelectMgrSelectableObject>,
) {
    // DynamicArray iterator freezes its end but rereads existing Value slots after callbacks.
    let extent = selection.native_entity_count();
    for index in 0..extent {
        if let Some(owner) = selection.native_owner_at(index) {
            owner.set_selectable(selectable);
        }
    }
}
