// Rust translation of the sequence post-processing of ProjLib_CompProjectedCurve::Init
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The post-processing of the parts: poor parts removed, overlapping bounds
//! separated, then per part the maximal distance, the single point flag and the
//! isoparametric flags.
use super::derivatives::Pair;
use super::parts::Parts;
use super::{CompProjectedCurve, CompState};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{Point2, Point3};

// The average (U, V) of a part, summed in order and divided by its length.
fn average(points: &[Point3]) -> (f64, f64) {
    let (mut ave_u, mut ave_v): (f64, f64) = (0., 0.);
    for point in points {
        ave_u += point.y();
        ave_v += point.z();
    }
    let length: f64 = points.len() as f64;
    (ave_u / length, ave_v / length)
}

// The largest distance between the curve points and their projections.
fn max_distance(points: &[Point3], pair: Pair<'_>) -> Result<f64, Failure> {
    let mut max: f64 = 0.;
    for triple in points {
        let on_curve: Point3 = pair.curve.d0(triple.x())?;
        let on_surface: Point3 = pair.surface.d0(triple.y(), triple.z())?;
        let distance: f64 = on_curve.distance(&on_surface);
        if max < distance {
            max = distance;
        }
    }
    Ok(max)
}

// A part within the larger parametric tolerance of its average point.
fn is_single_point(points: &[Point3], tol: f64) -> bool {
    let (ave_u, ave_v) = average(points);
    let pmoy: Point2 = Point2::new(ave_u, ave_v);
    !points
        .iter()
        .any(|p| Point2::new(p.y(), p.z()).distance(pmoy) > tol)
}

// Whether all the U (or V, `coord` 2) of a part are within `tol` of their average.
fn is_iso(points: &[Point3], coord: usize, tol: f64) -> bool {
    let (ave_u, ave_v) = average(points);
    let ave: f64 = if coord == 1 { ave_u } else { ave_v };
    let value = |p: &Point3| if coord == 1 { p.y() } else { p.z() };
    !points.iter().any(|p| (value(p) - ave).abs() > tol)
}

impl CompProjectedCurve {
    /// Steps 1 to 5 of the post-processing of `Init`.
    pub(super) fn post_process(&self) -> Result<(), Failure> {
        let state: CompState = self.snapshot();
        let parts = state.parts()?;
        // 1. Removing poor parts.
        let mut nb_curves: i32 = state.nb_curves;
        let mut ipart: i32 = 1;
        for _ in 1..=state.nb_curves {
            if parts.points(ipart)?.len() < 2 {
                parts.remove_part(ipart)?;
                nb_curves -= 1;
            } else {
                ipart += 1;
            }
        }
        self.state.write().nb_curves = nb_curves;
        if nb_curves == 0 {
            return Ok(());
        }
        // 2. Removing common parts of bounds.
        for i in 1..nb_curves {
            let last: f64 = parts.last_point(i)?.x();
            if last >= parts.points(i + 1)?[0].x() {
                parts.set_first_x(i + 1, last + 1.0e-12)?;
            }
        }
        let all: Vec<Vec<Point3>> = (1..=nb_curves)
            .map(|i| parts.points(i))
            .collect::<Result<_, _>>()?;
        self.store_flags(&state, &all)
    }

    // 3. to 5.: the maximal distances, the single point and the iso flags.
    fn store_flags(&self, state: &CompState, all: &[Vec<Point3>]) -> Result<(), Failure> {
        let pair: Pair<'_> = state.pair()?;
        let mut distances: Vec<f64> = Vec::with_capacity(all.len());
        for points in all {
            distances.push(max_distance(points, pair)?);
        }
        let tol: f64 = if state.tol_u < state.tol_v {
            state.tol_v
        } else {
            state.tol_u
        };
        let sngl: Vec<bool> = all
            .iter()
            .map(|points| is_single_point(points, tol))
            .collect();
        let (mut u_iso, mut v_iso) = (vec![true; all.len()], vec![true; all.len()]);
        for (i, points) in all.iter().enumerate() {
            if sngl[i] || points.len() <= 2 {
                (u_iso[i], v_iso[i]) = (false, false);
                continue;
            }
            u_iso[i] = is_iso(points, 1, state.tol_u);
            v_iso[i] = is_iso(points, 2, state.tol_v);
        }
        let mut target = self.state.write();
        target.max_distance = Some(distances);
        target.sngl_pnts = Some(sngl);
        (target.u_iso, target.v_iso) = (Some(u_iso), Some(v_iso));
        Ok(())
    }
}
