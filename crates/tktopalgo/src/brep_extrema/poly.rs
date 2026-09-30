// Rust translation of ModelingAlgorithms/TKTopAlgo/BRepExtrema/BRepExtrema_Poly.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepExtrema_Poly`: the minimum distance between the triangulation nodes of the
//! faces of two shapes.
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Shape};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::Point3;
use rustycad_tkmath::poly::mesh_purpose;

/// The nearest pair of triangulation nodes of two shapes and their distance.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct BRepExtremaPolyDistance {
    /// `P1`: the node of the first shape.
    pub p1: Point3,
    /// `P2`: the node of the second shape.
    pub p2: Point3,
    /// `dist`: their distance.
    pub dist: f64,
}

/// `BRepExtrema_Poly` (docs/topalgo-design.md section 13): the package class of
/// `Distance`.
///
/// ```
/// use rustycad_tkbrep::topods::Shape;
/// use rustycad_tktopalgo::brep_extrema::BRepExtremaPoly;
/// // Shapes without triangulated faces give no distance.
/// let (found, _) = BRepExtremaPoly::distance(&Shape::default(), &Shape::default())?;
/// assert!(!found);
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
pub struct BRepExtremaPoly;

impl BRepExtremaPoly {
    /// `Distance(S1, S2, P1, P2, dist)`: the nearest pair of nodes (in world
    /// coordinates) of the triangulations of the faces of `s1` and `s2` (the first
    /// pair of the smallest distance); false when a shape has no triangulated face.
    /// `dist` is Precision::Infinite() and the points are untouched (the origin here)
    /// when no pair is nearer; `let (found, nearest) = BRepExtremaPoly::distance(&a, &b)?;`.
    pub fn distance(s1: &Shape, s2: &Shape) -> Result<(bool, BRepExtremaPolyDistance), Failure> {
        let mut result = BRepExtremaPolyDistance {
            dist: precision::infinite(),
            ..BRepExtremaPolyDistance::default()
        };
        let nodes1: Vec<Point3> = triangulation_nodes(s1)?;
        if nodes1.is_empty() {
            return Ok((false, result));
        }
        let nodes2: Vec<Point3> = triangulation_nodes(s2)?;
        if nodes2.is_empty() {
            return Ok((false, result));
        }
        for pp1 in &nodes1 {
            for pp2 in &nodes2 {
                let d_cur: f64 = pp1.distance(pp2);
                if result.dist > d_cur {
                    result = BRepExtremaPolyDistance {
                        p1: *pp1,
                        p2: *pp2,
                        dist: d_cur,
                    };
                }
            }
        }
        Ok((true, result))
    }
}

/// The nodes of the triangulations of the faces of `shape`, face after face in
/// exploration order, transformed by the face locations.
fn triangulation_nodes(shape: &Shape) -> Result<Vec<Point3>, Failure> {
    let mut nodes: Vec<Point3> = Vec::new();
    for face in Explorer::new(shape, ShapeEnum::Face, ShapeEnum::Shape)? {
        let (triangulation, location) =
            BRepTool::triangulation(&topods::face(&face)?, mesh_purpose::NONE)?;
        let Some(triangulation) = triangulation.get() else {
            continue;
        };
        let transformation = location.transformation();
        for i in 1..=triangulation.nb_nodes() {
            nodes.push(triangulation.node(i)?.transformed(transformation));
        }
    }
    Ok(nodes)
}
