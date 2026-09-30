// Rust translation of the BRepTools::Update overloads and BRepTools::UpdateFaceUVPoints
// (ModelingData/TKBRep/BRepTools/BRepTools.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepTools::Update`: recomputes the UV points of the edges of unchecked faces.
use crate::brep::BRepTool;
use crate::brep::access::edge_tshape;
use crate::brep::update_curves::as_gcurve;
use crate::topexp::Explorer;
use crate::topods::{
    self, CompSolid, Compound, Edge, Face, Shape, Shell, Solid, TShape, Vertex, Wire,
};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;

/// `BRepTools::Update(const TopoDS_Vertex&)`: does nothing; `update_vertex(&vertex)`.
pub fn update_vertex(_vertex: &Vertex) {}

/// `BRepTools::Update(const TopoDS_Edge&)`: does nothing; `update_edge(&edge)`.
pub fn update_edge(_edge: &Edge) {}

/// `BRepTools::Update(const TopoDS_Wire&)`: does nothing; `update_wire(&wire)`.
pub fn update_wire(_wire: &Wire) {}

/// `BRepTools::Update(const TopoDS_Face&)`: an unchecked face gets the UV points of its
/// edges reset, then its TShape is marked Checked; `update_face(&face)?`.
pub fn update_face(face: &Face) -> Result<(), Failure> {
    if face.checked()? {
        return Ok(());
    }
    update_face_uv_points(face)?;
    face.set_checked(true)
}

/// Updates every face of `shape` (the Shell, Solid, CompSolid and Compound overloads).
fn update_faces(shape: &Shape) -> Result<(), Failure> {
    for face in Explorer::new(shape, ShapeEnum::Face, ShapeEnum::Shape)? {
        update_face(&topods::face(&face)?)?;
    }
    Ok(())
}

/// `BRepTools::Update(const TopoDS_Shell&)`: updates each face; `update_shell(&shell)?`.
pub fn update_shell(shell: &Shell) -> Result<(), Failure> {
    update_faces(shell)
}

/// `BRepTools::Update(const TopoDS_Solid&)`: updates each face; `update_solid(&solid)?`.
pub fn update_solid(solid: &Solid) -> Result<(), Failure> {
    update_faces(solid)
}

/// `BRepTools::Update(const TopoDS_CompSolid&)`: updates each face;
/// `update_comp_solid(&comp_solid)?`.
pub fn update_comp_solid(comp_solid: &CompSolid) -> Result<(), Failure> {
    update_faces(comp_solid)
}

/// `BRepTools::Update(const TopoDS_Compound&)`: updates each face;
/// `update_compound(&compound)?`.
pub fn update_compound(compound: &Compound) -> Result<(), Failure> {
    update_faces(compound)
}

/// `BRepTools::Update(const TopoDS_Shape&)`: dispatches on the shape type (a null shape
/// raises Standard_NullObject, a crash natively); `update(&shape)?`.
pub fn update(shape: &Shape) -> Result<(), Failure> {
    match shape.shape_type()? {
        ShapeEnum::Face => update_face(&topods::face(shape)?),
        ShapeEnum::Shell | ShapeEnum::Solid | ShapeEnum::CompSolid | ShapeEnum::Compound => {
            update_faces(shape)
        }
        _ => Ok(()),
    }
}

/// `BRepTools::UpdateFaceUVPoints(F)`: for each edge of the face, the first parametric
/// representation on the face's located surface recomputes its UV points from the
/// ends of its p-curve; a locked edge stops the whole update;
/// `update_face_uv_points(&face)?`.
pub fn update_face_uv_points(face: &Face) -> Result<(), Failure> {
    let (surface, location) = BRepTool::surface_with_location(face)?;
    for edge in Explorer::new(face, ShapeEnum::Edge, ShapeEnum::Shape)? {
        let tedge = edge_tshape(&edge, "BRepTools::UpdateFaceUVPoints")?;
        if tedge.locked() {
            return Ok(());
        }
        let edge_location = location.predivided(edge.location())?;
        for representation in tedge.curves() {
            let gcurve = as_gcurve(&representation);
            let Some(gcurve) = gcurve.get() else { continue };
            if gcurve.is_curve_on_surface_at(&surface, &edge_location) {
                gcurve.update()?;
                break;
            }
        }
    }
    Ok(())
}
