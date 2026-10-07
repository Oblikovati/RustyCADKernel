//! HLRTopoBRep_DSFiller.cxx: the curves of a walking contour line: a polyline of the
//! walking points for fewer than 6 of them, else its approximation by BRepApprox.
use super::ContourCurves;
use crate::contap::ContapLine;
use crate::hlr_algo::c_int_truncation;
use crate::hlr_topo_brep::DSFiller;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::real::real_last;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{Geom2dBSplineCurve, Geom2dCurve};
use rustycad_tkg3d::geom::GeomBSplineCurve;
use rustycad_tkgeomalgo::approx_int::ApproxIntRange;
use rustycad_tkgeombase::app_par_curves::MultiCurveEval;
use rustycad_tkgeombase::approx::ParametrizationType;
use rustycad_tkmath::gp::{Point2, Point3};
use rustycad_tktopalgo::brep_approx::{BRepApproxApprox, BRepApproxApproxLine};

fn array<T: Default + Clone>(n: i32) -> Result<Array1<'static, T>, Failure> {
    Array1::with_bounds(1, n).map_err(|e| e.into_failure("HLRTopoBRep_DSFiller::InsertFace"))
}

// The walking points `ipF..=ipL` with knots 1..=nbp (end multiplicities 2), and
// their (u, v) on the surface.
struct WalkingPoints {
    knots: Array1<'static, f64>,
    mults: Array1<'static, i32>,
    points: Array1<'static, Point3>,
    points2d: Array1<'static, Point2>,
}

// The extents of the points (the native Max/Min locals).
#[derive(Clone, Copy)]
struct Extents {
    min: [f64; 5],
    max: [f64; 5],
}

impl Extents {
    fn new() -> Self {
        Self {
            min: [real_last(); 5],
            max: [-real_last(); 5],
        }
    }
    // `if (x < Min) Min = x; ... if (x > Max) Max = x;` per coordinate, in the
    // native order (x, y, z, then u, v).
    fn add(&mut self, slots: std::ops::Range<usize>, values: &[f64]) {
        for (k, x) in slots.clone().zip(values) {
            if *x < self.min[k] {
                self.min[k] = *x;
            }
        }
        for (k, x) in slots.zip(values) {
            if *x > self.max[k] {
                self.max[k] = *x;
            }
        }
    }
}

fn walking_points(
    line: &ContapLine,
    (ip_f, ip_l): (i32, i32),
    ext: &mut Extents,
) -> Result<WalkingPoints, Failure> {
    let nbp = ip_l - ip_f + 1;
    let mut w = WalkingPoints {
        knots: array(nbp)?,
        mults: array(nbp)?,
        points: array(nbp)?,
        points2d: array(nbp)?,
    };
    for i in 1..=nbp {
        w.knots[i] = f64::from(i);
        w.mults[i] = 1;
        let p = line.point(i + ip_f - 1)?.value();
        ext.add(0..3, &[p.x(), p.y(), p.z()]);
        w.points[i] = p;
    }
    w.mults[1] = 2;
    w.mults[nbp] = 2;
    Ok(w)
}

fn walking_points2d(
    line: &ContapLine,
    ip_f: i32,
    w: &mut WalkingPoints,
    ext: &mut Extents,
) -> Result<(), Failure> {
    for i in 1..=w.points2d.length() {
        let (u, v) = line.point(i + ip_f - 1)?.parameters_on_s2();
        ext.add(3..5, &[u, v]);
        w.points2d[i] = Point2::new(u, v);
    }
    Ok(())
}

// `TOL * extent` clamped to [1e-12, 0.1].
fn clamped_tolerance(tol: f64, extent: f64) -> f64 {
    // NaN stays NaN, as the native comparisons leave it.
    (tol * extent).clamp(1e-12, 0.1)
}

impl DSFiller {
    pub(super) fn walking_curves(
        line: &ContapLine,
        (par_f, par_l): (f64, f64),
        with_p_curve: bool,
    ) -> Result<ContourCurves, Failure> {
        // copy the points
        let (ip_f, ip_l) = (c_int_truncation(par_f), c_int_truncation(par_l));
        if ip_l - ip_f < 1 {
            return Ok(None);
        }
        let mut ext = Extents::new();
        let mut w = walking_points(line, (ip_f, ip_l), &mut ext)?;
        let nbp = ip_l - ip_f + 1;
        let app_c = Handle::new(GeomBSplineCurve::new(
            &w.points, &w.knots, &w.mults, 1, false,
        )?);
        let mut app_c2d = Handle::<Geom2dBSplineCurve>::default();
        if with_p_curve {
            walking_points2d(line, ip_f, &mut w, &mut ext)?;
            app_c2d = Handle::new(Geom2dBSplineCurve::new(
                &w.points2d,
                &w.knots,
                &w.mults,
                1,
                false,
            )?);
        }
        let poly = (
            app_c.clone().cast(),
            app_c2d.clone().cast(),
            1.,
            f64::from(nbp),
        );
        if ip_l - ip_f < 5 {
            return Ok(Some(poly));
        }
        Self::approximated(&app_c, &app_c2d, nbp, ext).map(|c| Some(c.unwrap_or(poly)))
    }

    // BRepApprox on the polyline and its p-curve, with tolerances relative to the
    // extents of the points; `None` when it is not done.
    fn approximated(
        app_c: &Handle<GeomBSplineCurve>,
        app_c2d: &Handle<Geom2dBSplineCurve>,
        nbp: i32,
        ext: Extents,
    ) -> Result<ContourCurves, Failure> {
        let c_null = Handle::<Geom2dBSplineCurve>::default();
        let app_line = Handle::new(BRepApproxApproxLine::from_curves(app_c, app_c2d, &c_null));
        let (dmin, dmax, niter, tg, tol) = (4, 8, 0, false, 0.0001);
        let mut approx = BRepApproxApprox::new();
        let (tol3d, tol2d) = approx_tolerances(&ext, tol);
        approx.set_parameters(
            tol3d,
            tol2d,
            dmin,
            dmax,
            niter,
            30,
            tg,
            ParametrizationType::ChordLength,
        );
        let range = ApproxIntRange {
            approx_xyz: true,
            approx_u1v1: true,
            approx_u2v2: false,
            indicemin: 1,
            indicemax: nbp,
        };
        approx.perform(&app_line, range)?;
        if !approx.is_done() {
            return Ok(None);
        }
        let app_val = approx.value(1);
        let mut poles3d = array(app_val.nb_poles())?;
        app_val.curve_3d(1, &mut poles3d)?;
        let (knots, mults) = (app_val.knots()?, app_val.multiplicities()?);
        let c = GeomBSplineCurve::new(&poles3d, &knots, &mults, app_val.degree()?, false)?;
        let app_val2 = approx.value(2);
        let mut poles2d = array(app_val2.nb_poles())?;
        app_val2.curve_2d(2, &mut poles2d)?;
        let (knots2, mults2) = (app_val2.knots()?, app_val2.multiplicities()?);
        let c2d = Geom2dBSplineCurve::new(&poles2d, &knots2, &mults2, app_val2.degree()?, false)?;
        let (first, last) = (c2d.first_parameter(), c2d.last_parameter());
        Ok(Some((
            Handle::new(c).cast(),
            Handle::new(c2d).cast(),
            first,
            last,
        )))
    }
}

// The native extents: `Maxx -= Minx` ...; then `Maxx` takes Maxy when larger, and
// Maxy again when Maxz is larger (a native slip kept as is), `Maxu` takes Maxv.
fn approx_tolerances(ext: &Extents, tol: f64) -> (f64, f64) {
    let d: Vec<f64> = (0..5).map(|k| ext.max[k] - ext.min[k]).collect();
    let (mut maxx, maxy, maxz, mut maxu, maxv) = (d[0], d[1], d[2], d[3], d[4]);
    if maxy > maxx {
        maxx = maxy;
    }
    if maxz > maxx {
        maxx = maxy;
    }
    if maxv > maxu {
        maxu = maxv;
    }
    (clamped_tolerance(tol, maxx), clamped_tolerance(tol, maxu))
}
