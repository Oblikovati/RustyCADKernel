// Rust translation of ChFi2d.hxx/.cxx and ChFi2d_ConstructionError.hxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! ChFi2d: the fillets and chamfers of planar wires and faces: the analytical and the
//! iterative fillet algorithms, their API, the chamfer API, and the builder that edits
//! the contour of a planar face.
mod ana_fillet;
mod builder;
mod chamfer_api;
mod fillet_algo;
mod fillet_api;
#[cfg(test)]
mod tests;

pub use ana_fillet::ChFi2dAnaFilletAlgo;
pub use builder::ChFi2dBuilder;
pub use chamfer_api::ChFi2dChamferAPI;
pub use fillet_algo::{ChFi2dFilletAlgo, FilletPoint};
pub use fillet_api::ChFi2dFilletAPI;

use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{self, Edge, Face, Shape, ShapeIterator, Vertex, Wire};
use rustycad_tkbrep::toptools::IndexedDataMapOfShapeListOfShape;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;

/// `ChFi2d_ConstructionError`: the status of the 2d fillets and chamfers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum ConstructionError {
    /// `ChFi2d_NotPlanar`: the face is not planar.
    NotPlanar,
    /// `ChFi2d_NoFace`: the face is null.
    NoFace,
    /// `ChFi2d_InitialisationError`: the two faces used for the initialisation are
    /// uncompatible.
    InitialisationError,
    /// `ChFi2d_ParametersError`: a distance or an angle of a chamfer is not positive.
    ParametersError,
    /// `ChFi2d_Ready`: the initialization has been successful.
    Ready,
    /// `ChFi2d_IsDone`.
    IsDone,
    /// `ChFi2d_ComputationError`: the algorithm could not find a solution.
    ComputationError,
    /// `ChFi2d_ConnexionError`: the vertex is not connected to two edges.
    ConnexionError,
    /// `ChFi2d_TangencyError`: the two edges at the vertex are tangent.
    TangencyError,
    /// `ChFi2d_FirstEdgeDegenerated`: the first edge is degenerated.
    FirstEdgeDegenerated,
    /// `ChFi2d_LastEdgeDegenerated`: the last edge is degenerated.
    LastEdgeDegenerated,
    /// `ChFi2d_BothEdgesDegenerated`: the two edges are degenerated.
    BothEdgesDegenerated,
    /// `ChFi2d_NotAuthorized`: an edge at the vertex is a fillet or a chamfer, or is not
    /// a line or a circle.
    NotAuthorized,
}

/// `ChFi2d::CommonVertex(E1, E2, V)`: the vertex the two edges share (the first one of
/// `e1` first), or `None`.
///
/// ```
/// use rustycad_tkbrep::topods::Edge;
/// use rustycad_tkfillet::chfi2d::common_vertex;
/// assert!(common_vertex(&Edge::default(), &Edge::default()).is_some());
/// ```
pub fn common_vertex(e1: &Edge, e2: &Edge) -> Option<Vertex> {
    let (first1, last1) = topexp::vertices(e1, false);
    let (first2, last2) = topexp::vertices(e2, false);
    if first1.is_same(&first2) || first1.is_same(&last2) {
        return Some(first1);
    }
    if last1.is_same(&first2) || last1.is_same(&last2) {
        return Some(last1);
    }
    None
}

/// `ChFi2d::FindConnectedEdges(F, V, E1, E2)`: the two edges of the face at the vertex
/// (written as they are found, even when the status is a connexion error); IsDone
/// when there are exactly two. `let status = find_connected_edges(&f, &v, &mut e1, &mut e2)?;`.
pub fn find_connected_edges(
    f: &Face,
    v: &Vertex,
    e1: &mut Edge,
    e2: &mut Edge,
) -> Result<ConstructionError, Failure> {
    let mut vertex_map = IndexedDataMapOfShapeListOfShape::default();
    topexp::map_shapes_and_ancestors(f, ShapeEnum::Vertex, ShapeEnum::Edge, &mut vertex_map)?;
    let Ok(edges) = vertex_map.find_from_key(v) else {
        return Ok(ConstructionError::ConnexionError);
    };
    let mut iterator = edges.iter();
    let Some(first) = iterator.next() else {
        return Ok(ConstructionError::ConnexionError);
    };
    *e1 = topods::edge(first)?;
    let Some(second) = iterator.next() else {
        return Ok(ConstructionError::ConnexionError);
    };
    *e2 = topods::edge(second)?;
    if iterator.next().is_some() {
        return Ok(ConstructionError::ConnexionError);
    }
    Ok(ConstructionError::IsDone)
}

/// The first two edges of a wire (`TopoDS_Iterator` over its children); the null edge
/// for each one missing.
fn first_two_edges(wire: &Wire) -> Result<(Edge, Edge), Failure> {
    let (mut e1, mut e2) = (Edge::default(), Edge::default());
    let mut itr = ShapeIterator::new(wire, true, true);
    while itr.more() {
        let child: &Shape = itr.value()?;
        if e1.is_null() {
            e1 = topods::edge(child)?;
        } else if e2.is_null() {
            e2 = topods::edge(child)?;
        } else {
            break;
        }
        itr.next();
    }
    Ok((e1, e2))
}
