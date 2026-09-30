// Rust translation of BRepLib::BoundingVertex and BRepLib_ComparePoints (BRepLib.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepLib::BoundingVertex`: the sphere around the tolerance spheres of vertices.
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topods::{Shape, Vertex};
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::standard::real::real_epsilon;
use rustycad_tkmath::gp::{Point3, Vector3, Xyz};
use std::cmp::Ordering;

/// `BRepLib::BoundingVertex(theLV, theNewCenter, theNewTol)`: the center and radius of
/// a sphere around the tolerance spheres of the vertices: for two, the smallest one
/// (the larger sphere when it holds the other); for more, the mean of the points (summed
/// in lexicographic order, issue 0027540) with the largest distance plus tolerance.
/// None for fewer than two vertices (natively the outputs are left unchanged);
/// `brep_lib::bounding_vertex(&vertices)?`.
pub fn bounding_vertex(vertices: &List<Shape>) -> Result<Option<(Point3, f64)>, Failure> {
    match vertices.extent() {
        count if count < 2 => Ok(None),
        2 => {
            let first = Vertex::unchecked(vertices.first().expect("two vertices").clone());
            let last = Vertex::unchecked(vertices.last().expect("two vertices").clone());
            two_vertices(&first, &last).map(Some)
        }
        _ => many_vertices(vertices).map(Some),
    }
}

/// The bounding sphere of two tolerance spheres.
fn two_vertices(v0: &Vertex, v1: &Vertex) -> Result<(Point3, f64), Failure> {
    let eps: f64 = real_epsilon();
    let p: [Point3; 2] = [BRepTool::pnt(v0)?, BRepTool::pnt(v1)?];
    let r: [f64; 2] = [
        BRepTool::vertex_tolerance(v0)?,
        BRepTool::vertex_tolerance(v1)?,
    ];
    // m: max R, n: min R
    let (m, n): (usize, usize) = if r[0] < r[1] { (1, 0) } else { (0, 1) };
    let dr: f64 = r[m] - r[n]; // dR >= 0.
    let vd = Vector3::between(p[m], p[n]);
    let d: f64 = vd.magnitude();
    if d <= dr || d < eps {
        return Ok((p[m], r[m]));
    }
    let rr: f64 = 0.5 * (r[m] + r[n] + d);
    let xyz: Xyz = (p[m].xyz() + p[n].xyz() - *vd.xyz() * (dr / d)) * 0.5;
    Ok((Point3::new(xyz.x(), xyz.y(), xyz.z()), rr))
}

/// `BRepLib_ComparePoints`: the lexicographic order of the coordinates.
fn compare_points(p1: &Point3, p2: &Point3) -> Ordering {
    for (a, b) in p1.coordinates().iter().zip(p2.coordinates()) {
        if a < b {
            return Ordering::Less;
        }
        if a > b {
            return Ordering::Greater;
        }
    }
    Ordering::Equal
}

/// The bounding sphere of three or more tolerance spheres.
fn many_vertices(vertices: &List<Shape>) -> Result<(Point3, f64), Failure> {
    let mut points: Vec<Point3> = Vec::new();
    for shape in vertices.iter() {
        points.push(BRepTool::pnt(&Vertex::unchecked(shape.clone()))?);
    }
    points.sort_by(compare_points);
    let mut sum = Xyz::new(0., 0., 0.);
    for point in &points {
        sum += point.xyz();
    }
    let mean: Xyz = sum / points.len() as f64;
    let center = Point3::new(mean.x(), mean.y(), mean.z());
    // compute the tolerance for the new vertex
    let mut dmax: f64 = -1.;
    for shape in vertices.iter() {
        let vertex = Vertex::unchecked(shape.clone());
        let pi: Point3 = BRepTool::pnt(&vertex)?;
        let ti: f64 = BRepTool::vertex_tolerance(&vertex)?;
        let di: f64 = center.square_distance(&pi).sqrt() + ti;
        if di > dmax {
            dmax = di;
        }
    }
    Ok((center, dmax))
}
