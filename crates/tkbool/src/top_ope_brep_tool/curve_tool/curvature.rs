// Rust translation of the curvature test of TopOpeBRepTool_CurveTool::MakeCurves
// (ModelingAlgorithms/TKBool/TopOpeBRepTool/TopOpeBRepTool_CurveTool.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Whether the poles of an intersection curve are bad for the approximation: confused
//! poles, or a total variation of the discrete curvature derivative above 1000.
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkgeombase::gce::MakeCirc;
use rustycad_tkmath::gp::{Point3, Vector3};

/// The chord-length parameters of the poles, `None` when two consecutive poles are
/// confused.
fn chord_parameters(polc3d: &[Point3]) -> Option<Vec<f64>> {
    let mut par: Vec<f64> = vec![0.];
    for ip in 1..polc3d.len() {
        let dist: f64 = polc3d[ip - 1].distance(&polc3d[ip]);
        if dist < precision::confusion() {
            return None;
        }
        par.push(par[ip - 1] + dist);
    }
    Some(par)
}

/// The curvature of the circle through three poles, 0 when they are aligned within
/// 1e-4.
fn three_point_curvature(p0: Point3, p1: Point3, p2: Point3) -> Result<f64, Failure> {
    let (v1, v2) = (Vector3::between(p0, p1), Vector3::between(p1, p2));
    if v1.is_parallel(v2, 1.0e-4)? {
        return Ok(0.);
    }
    let cir = MakeCirc::through_points(p0, p1, p2)?.value()?;
    Ok(1. / cir.radius())
}

/// The curvatures at the poles: the circle curvature at the ends, the second
/// difference inside.
fn curvatures(polc3d: &[Point3], par: &[f64]) -> Result<Vec<f64>, Failure> {
    let npol = polc3d.len();
    let mut curvature: Vec<f64> = vec![0.; npol];
    if npol > 3 {
        // The ends (ip = 2 and ip = npol - 1 natively) go to Curvature(1) and Curvature(npol).
        curvature[0] = three_point_curvature(polc3d[0], polc3d[1], polc3d[2])?;
        curvature[npol - 1] =
            three_point_curvature(polc3d[npol - 3], polc3d[npol - 2], polc3d[npol - 1])?;
    } else if npol == 3 {
        let kc = three_point_curvature(polc3d[0], polc3d[1], polc3d[2])?;
        (curvature[0], curvature[npol - 1]) = (kc, kc);
    }
    let derivative = |ip: usize| -> (f64, f64, f64) {
        let dt: f64 = par[ip + 1] - par[ip];
        let (a, b) = (polc3d[ip], polc3d[ip + 1]);
        (
            (b.x() - a.x()) / dt,
            (b.y() - a.y()) / dt,
            (b.z() - a.z()) / dt,
        )
    };
    let (mut dx, mut dy, mut dz) = derivative(0);
    for ip in 1..npol - 1 {
        let (dx1, dy1, dz1) = derivative(ip);
        let d2t: f64 = 2. / (par[ip + 1] - par[ip - 1]);
        let (d2x, d2y, d2z) = (d2t * (dx1 - dx), d2t * (dy1 - dy), d2t * (dz1 - dz));
        curvature[ip] = (d2x * d2x + d2y * d2y + d2z * d2z).sqrt();
        (dx, dy, dz) = (dx1, dy1, dz1);
    }
    Ok(curvature)
}

/// Whether the poles are bad for the approximation (an empirical criterion of 1000 on
/// the accumulated curvature jumps; the native loop never updates its previous jump).
pub(super) fn is_bad(polc3d: &[Point3]) -> Result<bool, Failure> {
    if polc3d.len() < 2 {
        // A single pole is read past its array natively.
        return Err(Failure::with_type(
            "Standard_OutOfRange",
            &format!(
                "TopOpeBRepTool_CurveTool: {} pole; expected at least 2",
                polc3d.len()
            ),
            "NCollection_Array1",
        ));
    }
    let Some(par) = chord_parameters(polc3d) else {
        return Ok(true);
    };
    let curvature = curvatures(polc3d, &par)?;
    let crit: f64 = 1000.;
    let npol = polc3d.len();
    let kcprev: f64 = (curvature[1] - curvature[0]) / (par[1] - par[0]);
    let mut ksi: f64 = 0.;
    for ip in 1..npol - 1 {
        let kc: f64 = (curvature[ip + 1] - curvature[ip]) / (par[ip + 1] - par[ip]);
        ksi += (kc - kcprev).abs();
        if ksi > crit {
            return Ok(true);
        }
    }
    Ok(false)
}
