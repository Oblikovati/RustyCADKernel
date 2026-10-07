// Rust translation of ShapeCustom_Surface::ConvertToPeriodic (ShapeCustom_Surface.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `ConvertToPeriodic`: a B-spline surface closed in U or V made periodic in that
//! direction, its clamped end knots first opened by one knot on each side.
use super::ShapeCustomSurface;
use crate::shape_analysis::ShapeAnalysisSurface;
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{GeomBSplineSurface, GeomSurface};

/// The knots and multiplicities of one direction.
type KnotData = (Vec<f64>, Vec<i32>);

impl ShapeCustomSurface {
    /// `ConvertToPeriodic(substitute, preci)`: the periodic form of a B-spline surface
    /// closed (ShapeAnalysis_Surface::IsUClosed/IsVClosed with `preci`) in U or V with more
    /// than 3 poles in that direction; a null handle when nothing was converted.
    ///
    /// Native quirk (kept): when the end multiplicities need no rearrangement, SetUPeriodic
    /// (SetVPeriodic) runs on the stored surface itself, which is modified in place.
    /// Example: `let periodic = tool.convert_to_periodic(false, -1.0)?;`.
    pub fn convert_to_periodic(
        &mut self,
        substitute: bool,
        preci: f64,
    ) -> Result<Handle<dyn GeomSurface>, Failure> {
        let mut spline = self.surface.cast::<GeomBSplineSurface>();
        if spline.is_null() {
            return Ok(Handle::default());
        }
        let analysis = ShapeAnalysisSurface::new(&self.surface)?;
        let (u_closed, v_closed) = (analysis.is_u_closed(preci)?, analysis.is_v_closed(preci)?);
        if !u_closed && !v_closed {
            return Ok(Handle::default());
        }
        let mut converted = false;
        if u_closed {
            converted |= make_periodic(&mut spline, true)?;
        }
        if v_closed {
            converted |= make_periodic(&mut spline, false)?;
        }
        if !converted {
            return Ok(Handle::default());
        }
        let result: Handle<dyn GeomSurface> = spline.cast();
        if substitute {
            self.surface = result.clone();
        }
        Ok(result)
    }
}

/// The per-direction accessors of a B-spline surface.
struct Direction {
    u: bool,
}

impl Direction {
    fn periodic(&self, s: &GeomBSplineSurface) -> bool {
        if self.u {
            s.is_u_periodic()
        } else {
            s.is_v_periodic()
        }
    }
    fn nb_poles(&self, s: &GeomBSplineSurface) -> i32 {
        if self.u {
            s.nb_u_poles()
        } else {
            s.nb_v_poles()
        }
    }
    fn degree(&self, s: &GeomBSplineSurface) -> i32 {
        if self.u { s.u_degree() } else { s.v_degree() }
    }
    fn nb_knots(&self, s: &GeomBSplineSurface) -> i32 {
        if self.u {
            s.nb_u_knots()
        } else {
            s.nb_v_knots()
        }
    }
    fn multiplicity(&self, s: &GeomBSplineSurface, i: i32) -> Result<i32, Failure> {
        if self.u {
            s.u_multiplicity(i)
        } else {
            s.v_multiplicity(i)
        }
    }
    fn knot(&self, s: &GeomBSplineSurface, i: i32) -> Result<f64, Failure> {
        if self.u { s.u_knot(i) } else { s.v_knot(i) }
    }
    fn set_periodic(&self, s: &GeomBSplineSurface) -> Result<(), Failure> {
        if self.u {
            s.set_u_periodic()
        } else {
            s.set_v_periodic()
        }
    }
}

/// One native direction block: false when the direction is already periodic, has at most
/// 3 poles or end multiplicities that cannot be opened.
fn make_periodic(spline: &mut Handle<GeomBSplineSurface>, u: bool) -> Result<bool, Failure> {
    let d = Direction { u };
    let Some(s) = spline.get() else {
        return Ok(false);
    };
    if d.periodic(s) || d.nb_poles(s) <= 3 {
        return Ok(false);
    }
    let (degree, last) = (d.degree(s), d.nb_knots(s));
    let (first_mult, last_mult) = (d.multiplicity(s, 1)?, d.multiplicity(s, last)?);
    let replacement = if first_mult == degree + 1 && last_mult == degree + 1 {
        Some(opened_ends(s, &d)?)
    } else if first_mult > degree || last_mult > degree + 1 {
        return Ok(false);
    } else {
        None
    };
    if let Some(opened) = replacement {
        *spline = Handle::new(opened);
    }
    if let Some(s) = spline.get() {
        d.set_periodic(s)?;
    }
    Ok(true)
}

/// "if degree+1 at ends, first change it to 1 by rearranging knots": one knot added at
/// each end at the mean of the end spans, the old end multiplicities set to the degree.
fn opened_knots(s: &GeomBSplineSurface, d: &Direction) -> Result<KnotData, Failure> {
    let n = d.nb_knots(s);
    let a = 0.5 * (d.knot(s, 2)? - d.knot(s, 1)? + d.knot(s, n)? - d.knot(s, n - 1)?);
    let mut knots = vec![d.knot(s, 1)? - a];
    let mut mults = vec![1];
    for i in 1..=n {
        knots.push(d.knot(s, i)?);
        mults.push(d.multiplicity(s, i)?);
    }
    knots.push(d.knot(s, n)? + a);
    mults.push(1);
    let degree = d.degree(s);
    mults[1] = degree;
    mults[n as usize] = degree;
    Ok((knots, mults))
}

fn knots_of(s: &GeomBSplineSurface, d: &Direction) -> Result<KnotData, Failure> {
    let n = d.nb_knots(s);
    let knots = (1..=n)
        .map(|i| d.knot(s, i))
        .collect::<Result<Vec<_>, _>>()?;
    let mults = (1..=n)
        .map(|i| d.multiplicity(s, i))
        .collect::<Result<Vec<_>, _>>()?;
    Ok((knots, mults))
}

fn opened_ends(s: &GeomBSplineSurface, d: &Direction) -> Result<GeomBSplineSurface, Failure> {
    let opened = opened_knots(s, d)?;
    let other = knots_of(s, &Direction { u: !d.u })?;
    let (u, v) = if d.u {
        (opened, other)
    } else {
        (other, opened)
    };
    let poles: Array2<'static, _> = s.poles();
    let weights = s.weights_array(|w| w.clone());
    let (uk, um) = (Array1::from_shared(&u.0, 1), Array1::from_shared(&u.1, 1));
    let (vk, vm) = (Array1::from_shared(&v.0, 1), Array1::from_shared(&v.1, 1));
    GeomBSplineSurface::with_weights(
        &poles,
        &weights,
        (&uk, &um),
        (&vk, &vm),
        (s.u_degree(), s.v_degree()),
        (s.is_u_periodic(), s.is_v_periodic()),
    )
}
