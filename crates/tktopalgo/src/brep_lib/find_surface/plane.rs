// Rust translation of the plane search of BRepLib_FindSurface::Init, fillParams,
// fillPoints and Controle (BRepLib_FindSurface.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The plane of `BRepLib_FindSurface` (OCC157): points sampled along the edges, each
//! weighted by the distances to its neighbours on the same edge, and the plane through
//! their barycenter normal to the eigenvector of the smallest weighted variance.
use super::BRepLibFindSurface;
use crate::brep_lib::make_face::BRepLibMakeFace;
use crate::brep_top_adaptor::BRepTopAdaptorFClass2d;
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorCurve;
use rustycad_tkbrep::topods::{Edge, Shape};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::real::{epsilon, is_equal};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::native::std_max;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::geom::{GeomCurve as _, GeomElementarySurface as _, GeomPlane, GeomSurface};
use rustycad_tkg3d::top_abs::{ShapeEnum, State};
use rustycad_tkmath::geom_abs::{CurveType, Shape as Continuity};
use rustycad_tkmath::gp::{self, Direction3, Point3, Vector3, Xyz};
use rustycad_tkmath::math::{Jacobi, Matrix, Vector};
use rustycad_tkmath::toploc::Location;

/// The sample points and their weights.
type Samples = (Vec<Point3>, Vec<f64>);

/// "no existing surface, search a plane": the fitted plane when it is within the
/// tolerance (5 times it when `tol` is negative), oriented by a closed wire.
pub(super) fn fit_plane(
    finder: &mut BRepLibFindSurface,
    shape: &Shape,
    edges: &[Edge],
    tol: f64,
) -> Result<(), Failure> {
    let (points, weights) = samples(edges)?;
    if points.len() < 3 {
        return Ok(());
    }
    finder.my_location = Location::default();
    let (barycenter, max_weight) = barycenter(&points, &weights);
    let Some(normal) = plane_normal(&points, &weights, barycenter, max_weight)? else {
        return Ok(());
    };
    let plane = GeomPlane::from_point(
        Point3::new(barycenter.x(), barycenter.y(), barycenter.z()),
        Direction3::from_vector(normal)?,
    )?;
    finder.my_tol_reached = controle(&points, &plane)?;
    const WEAKNESS: f64 = 5.0;
    let tolerance: f64 = finder.my_tolerance;
    if finder.my_tol_reached <= tolerance
        || (tol < 0. && finder.my_tol_reached < tolerance * WEAKNESS)
    {
        finder.my_surface = oriented_plane(shape, plane)?;
    }
    Ok(())
}

/// "Step #1": the samples of every non-degenerated edge in explorer order.
fn samples(edges: &[Edge]) -> Result<Samples, Failure> {
    let (mut points, mut weights) = (Vec::new(), Vec::new());
    for edge in edges {
        let curve = BRepAdaptorCurve::with_edge(edge)?;
        let (first, last) = (curve.first_parameter()?, curve.last_parameter()?);
        if is_equal(first, last) {
            // Degenerate
            continue;
        }
        let params = sample_params(&curve, first, last)?;
        fill_points(&curve, &params, &mut points, &mut weights)?;
    }
    Ok((points, weights))
}

/// "Fill the parameters of the sampling points": the knots of a Bezier or B-spline
/// curve split by the degree, the ends of a line, 4 points on a conic, and 15 plus the
/// number of C3 intervals on the other curves.
fn sample_params(curve: &BRepAdaptorCurve, first: f64, last: f64) -> Result<Vec<f64>, Failure> {
    let nb_points: i32 = match curve.get_type()? {
        CurveType::BezierCurve => {
            let bezier = curve.bezier()?;
            let b = bezier.get().expect("a Bezier curve");
            let knots = [b.first_parameter(), b.last_parameter()];
            return Ok(fill_params(&knots, b.degree(), first, last));
        }
        CurveType::BSplineCurve => {
            let bspline = curve.bspline()?;
            let b = bspline.get().expect("a B-spline curve");
            return Ok(fill_params(b.knots().as_slice(), b.degree(), first, last));
        }
        // Two points on a straight segment
        CurveType::Line => return Ok(vec![first, last]),
        // Four points on other analytical curves
        CurveType::Circle | CurveType::Ellipse | CurveType::Hyperbola | CurveType::Parabola => 4,
        // Put some points on other curves
        _ => 15 + curve.nb_intervals(Continuity::C3)?,
    };
    Ok(fill_params(&[first, last], nb_points - 1, first, last))
}

/// `fillParams`: `par_min`, the `max(degree, 1)` steps of each knot span inside the
/// range (each more than Precision::PConfusion() after the previous parameter), then
/// `par_max`.
fn fill_params(knots: &[f64], degree: i32, par_min: f64, par_max: f64) -> Vec<f64> {
    let confusion: f64 = precision::p_confusion();
    let mut params = vec![par_min];
    let mut prev: f64 = par_min;
    let nb_p: i32 = degree.max(1);
    for span in knots
        .windows(2)
        .take_while(|span| span[0] < par_max - confusion)
    {
        if span[1] < par_min + confusion {
            continue;
        }
        let step: f64 = (span[1] - span[0]) / f64::from(nb_p);
        for k in 1..=nb_p {
            let par: f64 = span[0] + f64::from(k) * step;
            if par > par_max - confusion {
                break;
            }
            if par > prev + confusion {
                params.push(par);
                prev = par;
            }
        }
    }
    params.push(par_max);
    params
}

/// `fillPoints`: the point at each parameter, weighted by its distances to the previous
/// and next points of the edge.
fn fill_points(
    curve: &BRepAdaptorCurve,
    params: &[f64],
    points: &mut Vec<Point3>,
    weights: &mut Vec<f64>,
) -> Result<(), Failure> {
    let mut dist_prev: f64 = 0.;
    let mut p_prev: Point3 = curve.value(params[0])?;
    for i in 1..=params.len() {
        let (p_next, dist_next) = match params.get(i) {
            Some(&param) => {
                let next = curve.value(param)?;
                (next, p_prev.distance(&next))
            }
            None => (p_prev, 0.0),
        };
        points.push(p_prev);
        weights.push(dist_prev + dist_next);
        (dist_prev, p_prev) = (dist_next, p_next);
    }
    Ok(())
}

/// "Find the barycenter and normalize weights": the weighted barycenter and the
/// largest weight.
fn barycenter(points: &[Point3], weights: &[f64]) -> (Xyz, f64) {
    let (mut center, mut sum, mut max) = (Xyz::new(0., 0., 0.), 0., 0.);
    for (point, &w) in points.iter().zip(weights) {
        center += w * point.xyz();
        sum += w;
        if w > max {
            max = w;
        }
    }
    (center / sum, max)
}

/// "Fill the matrix": the weighted covariance of the points about the barycenter.
fn covariance(
    points: &[Point3],
    weights: &[f64],
    center: Xyz,
    max_weight: f64,
) -> Result<Matrix<'static>, Failure> {
    let mut m = [[0.0_f64; 3]; 3];
    for (point, &weight) in points.iter().zip(weights) {
        let p = point.xyz() - center;
        let w: f64 = weight / max_weight;
        let c = [p.x(), p.y(), p.z()];
        for (i, j) in [(0, 0), (0, 1), (0, 2), (1, 1), (1, 2), (2, 2)] {
            m[i][j] += w * c[i] * c[j];
        }
    }
    let mut matrix = Matrix::filled(1, 3, 1, 3, 0.).expect("a 3x3 matrix");
    for (i, j) in (0..3).flat_map(|i| (0..3).map(move |j| (i, j))) {
        *matrix.change_value(i as i32 + 1, j as i32 + 1)? = if i <= j { m[i][j] } else { m[j][i] };
    }
    Ok(matrix)
}

/// The normal of the plane: the eigenvector of the eigenvalue nearest 0 (when within
/// the epsilon of the largest one), else the cross product of the two others, with
/// the components within the epsilon of the largest one set to 0; None when the
/// eigenanalysis fails or the normal is below gp::Resolution().
fn plane_normal(
    points: &[Point3],
    weights: &[f64],
    center: Xyz,
    max_weight: f64,
) -> Result<Option<Vector3>, Failure> {
    let jacobi = Jacobi::new(&covariance(points, weights, center, max_weight)?)?;
    if !jacobi.is_done() {
        return Ok(None);
    }
    let values = jacobi.values()?;
    let (mut e_min, mut e_max, mut isol) = (f64::MAX, -f64::MAX, 0);
    for i in 1..=3 {
        let e: f64 = values.value(i)?.abs();
        if e_min > e {
            (e_min, isol) = (e, i);
        }
        if e_max < e {
            e_max = e;
        }
    }
    if isol == 0 {
        return Ok(None);
    }
    let vec = eigen_normal(&jacobi, isol, e_min <= epsilon(e_max))?;
    if vec.iter().map(|c| c * c).sum::<f64>() < gp::resolution() {
        return Ok(None);
    }
    // Removing very small values
    let max_v: f64 = std_max(vec[0].abs(), std_max(vec[1].abs(), vec[2].abs()));
    let eps: f64 = epsilon(max_v);
    let [x, y, z] = vec.map(|c| if c.abs() <= eps { 0. } else { c });
    Ok(Some(Vector3::new(x, y, z)))
}

/// The eigenvector `isol` (`direct`), or the cross product of the two other ones.
fn eigen_normal(jacobi: &Jacobi, isol: i32, direct: bool) -> Result<[f64; 3], Failure> {
    let column = |index: i32| -> Result<[f64; 3], Failure> {
        let mut v = Vector::filled(1, 3, 0.)?;
        jacobi.vector(index, &mut v)?;
        Ok([*v.value(1)?, *v.value(2)?, *v.value(3)?])
    };
    if direct {
        return column(isol);
    }
    // try using vector product of other axes
    let others: Vec<i32> = (1..=3).filter(|&i| i != isol).collect();
    let (a, b) = (column(others[0])?, column(others[1])?);
    let n = Vector3::new(a[0], a[1], a[2]).crossed(Vector3::new(b[0], b[1], b[2]));
    Ok([n.x(), n.y(), n.z()])
}

/// `Controle`: the largest distance from the points to the plane, from its equation.
fn controle(points: &[Point3], plane: &GeomPlane) -> Result<f64, Failure> {
    let [a, b, c, d] = plane.coefficients()?;
    let mut max_dist: f64 = 0.;
    for point in points {
        let dist: f64 = (a * point.x() + b * point.y() + c * point.z() + d).abs();
        if dist > max_dist {
            max_dist = dist;
        }
    }
    Ok(max_dist)
}

/// "If S is wire, try to orient surface according to orientation of wire": a closed
/// wire whose face on the plane holds the infinite point reverses the plane normal.
fn oriented_plane(shape: &Shape, plane: GeomPlane) -> Result<Handle<dyn GeomSurface>, Failure> {
    let surface: Handle<dyn GeomSurface> = Handle::new(plane.clone()).cast();
    if shape.shape_type()? != ShapeEnum::Wire || !shape.closed()? {
        return Ok(surface);
    }
    let mut face = BRepLibMakeFace::from_surface(&surface, precision::confusion())?.face()?;
    BRepBuilder::default().add(&mut face, shape)?;
    let classifier = BRepTopAdaptorFClass2d::new(&face, 0.)?;
    if classifier.perform_infinite_point()? != State::In {
        return Ok(surface);
    }
    let position = plane.position();
    let normal = position.direction().reversed();
    Ok(Handle::new(GeomPlane::from_point(position.location(), normal)?).cast())
}
