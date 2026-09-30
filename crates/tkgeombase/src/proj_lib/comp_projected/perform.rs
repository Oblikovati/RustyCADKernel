// Rust translation of ProjLib_CompProjectedCurve::Perform
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Perform`: the result curves of every part. A single point part gives points;
//! an isoparametric part a trimmed 2D line; any other part the approximation of
//! the projection on the surface slightly extended beyond its non-periodic bounds.
use super::eval::eval_d0;
use super::{CompProjectedCurve, CompState, PerformResults};
use crate::approx::{CurveOnSurface, CurveOnSurfaceOptions};
use crate::native_failure::failure;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine, Geom2dTrimmedCurve};
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::geom::GeomCurve;
use rustycad_tkmath::gp::{Direction2, Point2, Point3, StandardDirection2};

/// The state carried from part to part, as the native locals are: the 2D
/// approximation flag (cleared by an iso part for the rest), the extended surface
/// and the last curves.
struct PerformLoop {
    approx2d: bool,
    approx3d: bool,
    hs: Handle<dyn Adaptor3dSurface>,
    pcur2d: Handle<dyn Geom2dCurve>,
    pcur3d: Handle<dyn GeomCurve>,
}

impl CompProjectedCurve {
    /// `Perform()`: fills the per-part results (`ResultIsPoint`, `GetResult2dC`,
    /// ...) according to `SetProj2d` and `SetProj3d`. An isoparametric part clears
    /// the 2D approximation for the parts after it, and a part without an
    /// approximation takes the curves of the previous part, as natively;
    /// `projection.perform()?`.
    pub fn perform(&self) -> Result<(), Failure> {
        let state: CompState = self.snapshot();
        if state.nb_curves == 0 {
            return Ok(());
        }
        let surface: &dyn Adaptor3dSurface = state.surface.get().ok_or_else(null_surface)?;
        let n: usize = state.nb_curves as usize;
        let mut results = PerformResults {
            is_point: Some(vec![false; n]),
            error_2d_u: Some(vec![0.0; n]),
            error_2d_v: Some(vec![0.0; n]),
            error_3d: Some(vec![0.0; n]),
            ..PerformResults::default()
        };
        if state.proj2d {
            results.point_2d = Some(vec![Point2::default(); n]);
            results.curve_2d = Some(vec![Handle::default(); n]);
        }
        if state.proj3d {
            results.point_3d = Some(vec![Point3::default(); n]);
            results.curve_3d = Some(vec![Handle::default(); n]);
        }
        let mut lp = PerformLoop {
            approx2d: state.proj2d,
            approx3d: state.proj3d,
            hs: surface.shallow_copy()?,
            pcur2d: Handle::default(),
            pcur3d: Handle::default(),
        };
        for k in 1..=state.nb_curves {
            self.perform_part(&state, k, &mut lp, &mut results)?;
        }
        self.state.write().results = results;
        Ok(())
    }

    fn perform_part(
        &self,
        state: &CompState,
        k: i32,
        lp: &mut PerformLoop,
        results: &mut PerformResults,
    ) -> Result<(), Failure> {
        let index: usize = (k - 1) as usize;
        let (single, p2d) = self.is_single_pnt(k)?;
        if single {
            let point: Point3 = state
                .surface
                .get()
                .ok_or_else(null_surface)?
                .d0(p2d.x(), p2d.y())?;
            if let Some(points) = results.point_2d.as_mut() {
                points[index] = p2d;
            }
            if let Some(points) = results.point_3d.as_mut() {
                points[index] = point;
            }
            results.is_point.get_or_insert_with(Vec::new)[index] = true;
            return Ok(());
        }
        let (mut udeb, mut ufin) = self.bounds(k)?;
        let hp_cur: Handle<dyn Adaptor2dCurve2d> = if let (true, u_iso) = self.is_u_iso(k)? {
            lp.approx2d = false;
            (udeb, ufin, lp.pcur2d) = iso_line(state, (udeb, ufin), u_iso, true)?;
            Handle::new(Geom2dAdaptorCurve::with_curve(&lp.pcur2d)?).cast()
        } else if let (true, v_iso) = self.is_v_iso(k)? {
            lp.approx2d = false;
            (udeb, ufin, lp.pcur2d) = iso_line(state, (udeb, ufin), v_iso, false)?;
            Handle::new(Geom2dAdaptorCurve::with_curve(&lp.pcur2d)?).cast()
        } else {
            self.extended_copy(state, lp)?
        };
        if lp.approx2d || lp.approx3d {
            approximate(state, lp, &hp_cur, (udeb, ufin), index, results)?;
        }
        if let Some(curves) = results.curve_2d.as_mut() {
            curves[index] = lp.pcur2d.clone();
        }
        if let Some(curves) = results.curve_3d.as_mut() {
            curves[index] = lp.pcur3d.clone();
        }
        Ok(())
    }

    // A shallow copy of this projection on the surface extended by ten parametric
    // tolerances beyond each non-periodic bound (cumulatively, part after part).
    fn extended_copy(
        &self,
        state: &CompState,
        lp: &mut PerformLoop,
    ) -> Result<Handle<dyn Adaptor2dCurve2d>, Failure> {
        let surface: &dyn Adaptor3dSurface = state.surface.get().ok_or_else(null_surface)?;
        if !surface.is_u_periodic()? {
            let du: f64 = 10. * state.tol_u;
            let (u1, u2) = (
                surface.first_u_parameter()? - du,
                surface.last_u_parameter()? + du,
            );
            lp.hs = lp.hs.get().ok_or_else(null_surface)?.u_trim(u1, u2, 0.0)?;
        }
        if !surface.is_v_periodic()? {
            let dv: f64 = 10. * state.tol_v;
            let (v1, v2) = (
                surface.first_v_parameter()? - dv,
                surface.last_v_parameter()? + dv,
            );
            lp.hs = lp.hs.get().ok_or_else(null_surface)?.v_trim(v1, v2, 0.0)?;
        }
        let hp: CompProjectedCurve = self.shallow_projection()?;
        hp.load_surface(&lp.hs);
        Ok(Handle::new(hp).cast())
    }
}

fn null_surface() -> Failure {
    failure(
        "Standard_NullObject",
        "ProjLib_CompProjectedCurve::Perform: null surface",
    )
}

// The trimmed 2D line of an iso part (`u_iso` for a U iso at `value`, else a V iso)
// over the other coordinate of its end points, reversed when decreasing.
fn iso_line(
    state: &CompState,
    (udeb, ufin): (f64, f64),
    value: f64,
    u_iso: bool,
) -> Result<(f64, f64, Handle<dyn Geom2dCurve>), Failure> {
    let (pdeb, pfin) = (eval_d0(state, udeb)?, eval_d0(state, ufin)?);
    let (mut first, mut last) = if u_iso {
        (pdeb.y(), pfin.y())
    } else {
        (pdeb.x(), pfin.x())
    };
    let direction: StandardDirection2 = if first > last {
        first = -first;
        last = -last;
        if u_iso {
            StandardDirection2::NegY
        } else {
            StandardDirection2::NegX
        }
    } else if u_iso {
        StandardDirection2::Y
    } else {
        StandardDirection2::X
    };
    let origin: Point2 = if u_iso {
        Point2::new(value, 0.)
    } else {
        Point2::new(0., value)
    };
    let line: Handle<dyn Geom2dCurve> = Handle::new(Geom2dLine::from_point(
        origin,
        Direction2::from_standard(direction),
    ))
    .cast();
    let trimmed = Geom2dTrimmedCurve::new(&line, first, last, true, true)?;
    Ok((first, last, Handle::new(trimmed).cast()))
}

// The approximation of the part: both curves, or only the asked one.
fn approximate(
    state: &CompState,
    lp: &mut PerformLoop,
    hp_cur: &Handle<dyn Adaptor2dCurve2d>,
    (udeb, ufin): (f64, f64),
    index: usize,
    results: &mut PerformResults,
) -> Result<(), Failure> {
    let (only_2d, only_3d) = if lp.approx2d && lp.approx3d {
        (!lp.approx2d, !lp.approx3d)
    } else {
        (lp.approx2d, lp.approx3d)
    };
    let options = CurveOnSurfaceOptions {
        max_segments: state.max_seg,
        max_degree: state.max_degree,
        continuity: state.continuity,
        only_3d,
        only_2d,
    };
    let mut appr = CurveOnSurface::without_perform(hp_cur, &lp.hs, udeb, ufin, state.tol3d);
    appr.perform(options)?;
    if lp.approx2d {
        lp.pcur2d = appr.curve_2d().cast();
        results.error_2d_u.get_or_insert_with(Vec::new)[index] = appr.max_error_2d_u();
        results.error_2d_v.get_or_insert_with(Vec::new)[index] = appr.max_error_2d_v();
    }
    if lp.approx3d {
        lp.pcur3d = appr.curve_3d().cast();
        results.error_3d.get_or_insert_with(Vec::new)[index] = appr.max_error_3d();
    }
    Ok(())
}
