//! The approximation loop of `ShapeCustom_BSplineRestriction::ConvertSurface`: growing
//! tolerances, decreasing continuities on exceptions, then more segments (degree priority)
//! or a higher degree.
use super::ShapeCustomBSplineRestriction;
use super::predicates::{continuity_to_integer, integer_to_continuity};
use super::surface::ShapeCustomRestrictionBox;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{
    GeomBSplineSurface, GeomBezierSurface, GeomOffsetSurface, GeomRectangularTrimmedSurface,
    GeomSurface,
};
use rustycad_tkgeombase::geom_convert::ApproxSurface;
use rustycad_tkmath::geom_abs::Shape as Continuity;

/// The loop variables of the native approximation (aCU, aCV, MaxSeg, MaxDeg).
struct ApproxLoop {
    cu: i32,
    cv: i32,
    max_seg: i32,
    max_deg: i32,
    shift: (f64, f64),
}

/// The outcome of one GeomConvert_ApproxSurface attempt inside the native `try`.
enum Attempt {
    Accepted,
    Rejected(f64),
    /// An exception; the error when it was measured before the exception (native newTol).
    Raised(Option<f64>),
}

impl ShapeCustomBSplineRestriction {
    /// The tail of `ConvertSurface`: B-splines and Beziers within the limits are kept;
    /// any other surface is approximated (an offset with continuity C0) on the box, the
    /// surface first trimmed to it when it is not periodic in that direction.
    pub(super) fn approximate_surface(
        &self,
        original: &Handle<dyn GeomSurface>,
        current: &Handle<dyn GeomSurface>,
        result: &mut Handle<dyn GeomSurface>,
        bounds: ShapeCustomRestrictionBox,
    ) -> Result<bool, Failure> {
        if self.within_limits(current) {
            return Ok(false);
        }
        let state = self.read();
        let mut target = current.clone();
        let shift = trimmed_to_box(&mut target, bounds)?;
        let continuity = if current.cast::<GeomOffsetSurface>().is_null() {
            state.continuity_3d
        } else {
            Continuity::C0
        };
        let start =
            continuity_to_integer(continuity).min(continuity_to_integer(target.continuity()));
        let start = if start == 0 {
            continuity_to_integer(continuity)
        } else {
            start
        };
        let mut l = ApproxLoop {
            cu: start,
            cv: start,
            max_seg: state.nb_max_seg,
            max_deg: state.max_degree,
            shift,
        };
        loop {
            if self.tolerance_rounds(original, &target, result, &mut l)? {
                return Ok(true);
            }
            if !self.widen_limits(&mut l)? {
                return Ok(false);
            }
        }
    }

    fn within_limits(&self, surface: &Handle<dyn GeomSurface>) -> bool {
        let state = self.read();
        let rational = |u: bool, v: bool| state.rational && (u || v);
        if let Some(s) = surface.cast::<GeomBSplineSurface>().get() {
            let segments = (s.nb_u_knots() - 1) * (s.nb_v_knots() - 1);
            return s.u_degree() <= state.max_degree
                && s.v_degree() <= state.max_degree
                && segments <= state.nb_max_seg
                && !rational(s.is_u_rational(), s.is_v_rational());
        }
        if let Some(s) = surface.cast::<GeomBezierSurface>().get() {
            return s.u_degree() <= state.max_degree
                && s.v_degree() <= state.max_degree
                && 1 <= state.nb_max_seg
                && !rational(s.is_u_rational(), s.is_v_rational());
        }
        false
    }

    // Up to ten tolerances myTol3d * i / 2, stopping when the error stops decreasing.
    fn tolerance_rounds(
        &self,
        original: &Handle<dyn GeomSurface>,
        target: &Handle<dyn GeomSurface>,
        result: &mut Handle<dyn GeomSurface>,
        l: &mut ApproxLoop,
    ) -> Result<bool, Failure> {
        let (mut previous, mut newest) = (f64::MAX, 0.);
        for i in 1..=10 {
            let tol = self.read().tol_3d * f64::from(i) / 2.;
            loop {
                let raised = match self.attempt(original, target, result, l, tol) {
                    Attempt::Accepted => return Ok(true),
                    Attempt::Rejected(error) => {
                        newest = error;
                        break;
                    }
                    Attempt::Raised(error) => error,
                };
                newest = raised.unwrap_or(newest);
                if l.cu == 0 && l.cv == 0 {
                    break;
                }
                l.cu -= i32::from(l.cu > 0);
                l.cv -= i32::from(l.cv > 0);
            }
            if previous <= newest {
                break;
            }
            previous = newest;
        }
        Ok(false)
    }

    fn attempt(
        &self,
        original: &Handle<dyn GeomSurface>,
        target: &Handle<dyn GeomSurface>,
        result: &mut Handle<dyn GeomSurface>,
        l: &ApproxLoop,
        tol: f64,
    ) -> Attempt {
        let continuities = (integer_to_continuity(l.cu), integer_to_continuity(l.cv));
        let Ok(approx) = ApproxSurface::from_surface(
            target,
            tol,
            continuities,
            (l.max_deg, l.max_deg),
            l.max_seg,
            0,
        ) else {
            return Attempt::Raised(None);
        };
        let state = self.read();
        if !(approx.max_error() <= state.tol_3d && approx.is_done()) {
            return Attempt::Rejected(approx.max_error());
        }
        *result = approx.surface().cast();
        match self.accept(original, &approx.surface(), approx.max_error(), l.shift) {
            Ok(()) => Attempt::Accepted,
            Err(_) => Attempt::Raised(Some(approx.max_error())),
        }
    }

    fn accept(
        &self,
        original: &Handle<dyn GeomSurface>,
        spline: &Handle<GeomBSplineSurface>,
        error: f64,
        shift: (f64, f64),
    ) -> Result<(), Failure> {
        let s = spline.get().ok_or_else(|| {
            Failure::with_type(
                "Standard_NullObject",
                "ShapeCustom_BSplineRestriction: null approximated surface",
                "",
            )
        })?;
        let spans = (s.nb_u_knots() - 1) * (s.nb_v_knots() - 1);
        if original.is_u_periodic() {
            s.set_u_periodic()?;
        }
        if original.is_v_periodic() {
            s.set_v_periodic()?;
        }
        {
            let mut state = self.state.write();
            state.converted = true;
            state.nb_of_span += spans;
            state.surface_error = state.surface_error.max(error);
        }
        shift_knots(s, shift)
    }

    // Degree priority: lower the continuities or double the segments up to GMaxSeg;
    // otherwise one more pass at GMaxDegree. False is the native `return false`.
    fn widen_limits(&self, l: &mut ApproxLoop) -> Result<bool, Failure> {
        let parameters = self.parameters()?;
        if !self.read().degree_priority {
            if l.max_deg < parameters.g_max_degree() {
                l.max_deg = parameters.g_max_degree();
                return Ok(true);
            }
            return Ok(false);
        }
        if l.max_seg >= parameters.g_max_seg() {
            return Ok(false);
        }
        if l.cv != 0 || l.cu != 0 {
            l.cv -= i32::from(l.cv > 0);
            l.cu -= i32::from(l.cu > 0);
        } else {
            l.max_seg *= 2;
        }
        if l.max_seg > parameters.g_max_seg() {
            l.max_seg = parameters.g_max_seg();
        }
        Ok(true)
    }
}

/// Trims a non-periodic direction to the box when the box differs from the surface
/// bounds; the shifts move the knots of the result back to the box parameters.
fn trimmed_to_box(
    surface: &mut Handle<dyn GeomSurface>,
    b: ShapeCustomRestrictionBox,
) -> Result<(f64, f64), Failure> {
    let (mut u1, mut u2, mut v1, mut v2) = surface.bounds();
    let pc = precision::p_confusion();
    let differs = (u1 - b.uf).abs() > pc
        || (u2 - b.ul).abs() > pc
        || (v1 - b.vf).abs() > pc
        || (v2 - b.vl).abs() > pc;
    if !differs {
        return Ok((0., 0.));
    }
    let mut is_trim = false;
    if !surface.is_u_periodic() {
        (u1, u2, is_trim) = (u1.max(b.uf), u2.min(b.ul), true);
    }
    if !surface.is_v_periodic() {
        (v1, v2, is_trim) = (v1.max(b.vf), v2.min(b.vl), true);
    }
    if !(is_trim && u1 != u2 && v1 != v2) {
        return Ok((0., 0.));
    }
    let trimmed = GeomRectangularTrimmedSurface::new(surface, (u1, u2), (v1, v2), (true, true))?;
    let (ur1, _, vr1, _) = trimmed.bounds();
    *surface = Handle::new(trimmed).cast();
    Ok((u1 - ur1, v1 - vr1))
}

fn shift_knots(s: &GeomBSplineSurface, (du, dv): (f64, f64)) -> Result<(), Failure> {
    if du.abs() > precision::p_confusion() {
        let knots: Vec<f64> = (1..=s.nb_u_knots())
            .map(|j| s.u_knot(j).map(|k| k + du))
            .collect::<Result<_, _>>()?;
        s.set_u_knots(&Array1::from_shared(&knots, 1))?;
    }
    if dv.abs() > precision::p_confusion() {
        let knots: Vec<f64> = (1..=s.nb_v_knots())
            .map(|j| s.v_knot(j).map(|k| k + dv))
            .collect::<Result<_, _>>()?;
        s.set_v_knots(&Array1::from_shared(&knots, 1))?;
    }
    Ok(())
}
