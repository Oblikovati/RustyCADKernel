//! Native selection loading allocates genuine BRep owners and applies selectable origins after all sensitive construction.
use super::brep_sensitive_compute::{BrepSensitiveBuild, compute_brep_sensitive};
use super::{StdSelectBRepOwner, StdSelectBrepSelectionTool};
use crate::select_mgr::{SelectMgrSelectableObject, SelectMgrSelection};
use rustycad_tkbrep::{brep_tools, topexp, topods::Shape, toptools::IndexedMapOfShape};
use rustycad_tkernel::{
    precision,
    standard::{Failure, Handle, catch_native_failure},
};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmesh::brep_mesh::BRepMeshIncrementalMesh;
impl StdSelectBrepSelectionTool {
    /// Append native selection entities and actual BRep owners; `Self::load(&selection,&shape,ShapeEnum::Face,0.1,0.5,false,-1,9,500.)?`.
    #[allow(clippy::too_many_arguments)] // Pinned native overload and defaults.
    pub fn load(
        selection: &Handle<SelectMgrSelection>,
        shape: &Shape,
        kind: ShapeEnum,
        deflection: f64,
        angle: f64,
        auto_triangulation: bool,
        priority: i32,
        edge_points: i32,
        max_parameter: f64,
    ) -> Result<(), Failure> {
        let priority = if priority == -1 {
            Self::get_standard_priority(shape, kind)?
        } else {
            priority
        };
        let build = BrepSensitiveBuild {
            owner: &Handle::default(),
            selection,
            deflection,
            angle,
            edge_points,
            max_parameter,
            auto_triangulation,
        };
        catch_native_failure(|| load_brep_geometry(shape, kind, priority, &build))
    }
    /// Load first, then assign the origin to all old and new owners; `Self::load_with_selectable(&selection,&object,&shape,ShapeEnum::Shape,0.1,0.5,false,-1,9,500.)?`.
    #[allow(clippy::too_many_arguments)] // Actual native origin overload.
    pub fn load_with_selectable(
        selection: &Handle<SelectMgrSelection>,
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
        Self::load(
            selection,
            shape,
            kind,
            deflection,
            angle,
            auto_triangulation,
            priority,
            edge_points,
            max_parameter,
        )?;
        set_brep_selection_origin(selection, selectable);
        Ok(())
    }
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
