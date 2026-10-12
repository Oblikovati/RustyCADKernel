//! Native triangulation policy used by AIS textured and shaded presentations at OCCT 3d097a0.
use crate::prs3d::{Prs3d, Prs3dDrawer};
use rustycad_tkbrep::{
    brep::BRepTool,
    brep_tools,
    topexp::Explorer,
    topods::{self, Shape, ShapeIterator},
};
use rustycad_tkernel::{
    message::ProgressRange,
    precision,
    standard::{Failure, Handle},
};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::{bnd::BoundingBox3, poly::mesh_purpose};
use rustycad_tkmesh::brep_mesh::BRepMeshDiscretFactory;
use rustycad_tkservice::aspect::AspectTypeOfDeflection;
use rustycad_tktopalgo::brep_bnd_lib::BRepBndLib;

/// Owned meshing boundary; `provider.perform(&shape,deflection,angle)?` preserves native factory choice.
pub trait StdPrsShapeMesher: Send + Sync {
    /// Return true when a real factory algorithm ran; `mesher.perform(&shape,0.1,0.5)?`.
    fn perform(&self, shape: &Shape, deflection: f64, angle: f64) -> Result<bool, Failure>;
}
/// Native registered factory adapter; `StdPrsNativeShapeMesher.perform(&shape,0.1,0.5)?`.
pub struct StdPrsNativeShapeMesher;
impl StdPrsShapeMesher for StdPrsNativeShapeMesher {
    fn perform(&self, shape: &Shape, deflection: f64, angle: f64) -> Result<bool, Failure> {
        let algorithm = BRepMeshDiscretFactory::get().discret(shape, deflection, angle);
        let Some(algorithm) = algorithm.get() else {
            return Ok(false);
        };
        algorithm.perform(&ProgressRange::default())?;
        Ok(true)
    }
}
/// Native shape triangulation tools; `StdPrsToolTriangulatedShape::is_triangulated(&shape)?`.
pub struct StdPrsToolTriangulatedShape;
impl StdPrsToolTriangulatedShape {
    /// Every explored face has a triangulation; empty and null shapes are true. `Self::is_triangulated(&shape)?`.
    pub fn is_triangulated(shape: &Shape) -> Result<bool, Failure> {
        for face in Explorer::new(shape, ShapeEnum::Face, ShapeEnum::Shape)? {
            if BRepTool::triangulation(&topods::face(&face)?, mesh_purpose::NONE)?
                .0
                .is_null()
            {
                return Ok(false);
            }
        }
        Ok(true)
    }
    /// Native closed-solid policy ignores one-dimensional subshapes; `Self::is_closed(&shape)?`.
    pub fn is_closed(shape: &Shape) -> Result<bool, Failure> {
        if shape.is_null() {
            return Ok(true);
        }
        match shape.shape_type()? {
            ShapeEnum::Shell | ShapeEnum::Face => Ok(false),
            ShapeEnum::Wire | ShapeEnum::Edge | ShapeEnum::Vertex => Ok(true),
            ShapeEnum::Solid => closed_triangulated_solid(shape),
            _ => recursively_closed_children(shape),
        }
    }
    /// Relative deflection stores the computed absolute deviation only for finite bounds; `Self::get_deflection(&shape,&drawer)?`.
    pub fn get_deflection(shape: &Shape, drawer: &Handle<Prs3dDrawer>) -> Result<f64, Failure> {
        if drawer.type_of_deflection() != AspectTypeOfDeflection::Relative {
            return Ok(drawer.maximal_chordial_deviation());
        }
        let mut bounds = BoundingBox3::default();
        BRepBndLib::add(shape, &mut bounds, false)?;
        if bounds.is_void() || (bounds.is_open() && !bounds.has_finite_part()) {
            return Ok(drawer.maximal_chordial_deviation());
        }
        let deviation = Prs3d::get_deflection_of_box(
            &bounds,
            drawer.deviation_coefficient(),
            drawer.maximal_chordial_deviation(),
        );
        drawer.set_maximal_chordial_deviation(deviation);
        Ok(deviation)
    }
    /// Validate native mesh deflection and free edges; `Self::is_tessellated(&shape,&drawer)?`.
    pub fn is_tessellated(shape: &Shape, drawer: &Handle<Prs3dDrawer>) -> Result<bool, Failure> {
        let deflection = Self::get_deflection(shape, drawer)?;
        brep_tools::triangulation(shape, deflection, true)
    }
    /// Native factory default, with an owned injectable companion; `Self::tessellate(&shape,&drawer)?`.
    pub fn tessellate(shape: &Shape, drawer: &Handle<Prs3dDrawer>) -> Result<bool, Failure> {
        let mesher = StdPrsNativeShapeMesher;
        Self::tessellate_with_mesher(shape, drawer, &mesher)
    }
    /// Skip complete meshes before asking the injected factory; `Self::tessellate_with_mesher(&shape,&drawer,&mesher)?`.
    pub fn tessellate_with_mesher(
        shape: &Shape,
        drawer: &Handle<Prs3dDrawer>,
        mesher: &dyn StdPrsShapeMesher,
    ) -> Result<bool, Failure> {
        if Self::is_tessellated(shape, drawer)? {
            return Ok(false);
        }
        let deflection = Self::get_deflection(shape, drawer)?;
        mesher.perform(shape, deflection, drawer.deviation_angle())
    }
    /// Clean shared meshes only when own deviation changed; `Self::clear_on_own_deflection_change(&shape,&drawer,true)?`.
    pub fn clear_on_own_deflection_change(
        shape: &Shape,
        drawer: &Handle<Prs3dDrawer>,
        reset: bool,
    ) -> Result<(), Failure> {
        if !drawer.is_auto_triangulation() || shape.is_null() {
            return Ok(());
        }
        if !own_shape_deviation_changed(drawer) {
            return Ok(());
        }
        brep_tools::clean(shape, false)?;
        if reset {
            drawer.update_previous_deviation_angle();
            drawer.update_previous_deviation_coefficient();
        }
        Ok(())
    }
}
fn recursively_closed_children(shape: &Shape) -> Result<bool, Failure> {
    for child in ShapeIterator::new(shape, true, true) {
        if !StdPrsToolTriangulatedShape::is_closed(&child)? {
            return Ok(false);
        }
    }
    Ok(true)
}
fn closed_triangulated_solid(shape: &Shape) -> Result<bool, Failure> {
    if !BRepTool::is_closed(shape)? {
        return Ok(false);
    }
    for child in ShapeIterator::new(shape, true, true) {
        if child.is_null() {
            continue;
        }
        if child.shape_type()? == ShapeEnum::Face
            || !StdPrsToolTriangulatedShape::is_triangulated(&child)?
        {
            return Ok(false);
        }
    }
    Ok(true)
}
fn own_shape_deviation_changed(drawer: &Handle<Prs3dDrawer>) -> bool {
    // OCCT 3d097a0 uses <=: infinite current/previous values yield NaN and require cleaning.
    let angle_unchanged = !drawer.has_own_deviation_angle()
        || (drawer.deviation_angle() - drawer.previous_deviation_angle()).abs()
            <= precision::angular();
    let coefficient_unchanged = !drawer.has_own_deviation_coefficient()
        || (drawer.deviation_coefficient() - drawer.previous_deviation_coefficient()).abs()
            <= precision::confusion();
    !(angle_unchanged && coefficient_unchanged)
}
