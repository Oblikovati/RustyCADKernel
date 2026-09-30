// Rust translation of ModelingData/TKG3d/Geom/Geom_RectangularTrimmedSurface.cxx (SetTrim, UReverse, VReverse) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The trimming state of a rectangular trimmed surface and the private 8-argument
//! `SetTrim` with the reversals it may trigger.
use crate::geom::GeomSurface;
use crate::geom::errors;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::native::std_min;
use rustycad_tkmath::elclib;

/// The fields of Geom_RectangularTrimmedSurface.
pub(super) struct TrimState {
    pub(super) basis: Handle<dyn GeomSurface>,
    pub(super) utrim1: f64,
    pub(super) vtrim1: f64,
    pub(super) utrim2: f64,
    pub(super) vtrim2: f64,
    pub(super) isutrimmed: bool,
    pub(super) isvtrimmed: bool,
}

/// The request of one direction of `SetTrim`: bounds, whether to trim, sense.
#[derive(Clone, Copy)]
pub(super) struct DirectionTrim {
    pub(super) p1: f64,
    pub(super) p2: f64,
    pub(super) trimmed: bool,
    pub(super) sense: bool,
}

// The basis range and periodicity of one direction, and its name for messages.
struct DirectionRange {
    first: f64,
    last: f64,
    periodic: bool,
    name: &'static str,
}

// One direction of the private SetTrim on the stored bounds `t1`, `t2`; returns
// whether the sense is kept. Like the native code, an out of range failure is
// raised after the ordered bounds are stored.
fn trim_direction(
    stored: (&mut f64, &mut f64),
    trim: DirectionTrim,
    range: DirectionRange,
) -> Result<bool, Failure> {
    let (t1, t2) = stored;
    if !trim.trimmed {
        (*t1, *t2) = (range.first, range.last);
        return Ok(true);
    }
    if trim.p1 == trim.p2 {
        return Err(errors::construction(&format!(
            "Geom_RectangularTrimmedSurface::{0}1=={0}2 = {1}; expected distinct parameters",
            range.name, trim.p1
        )));
    }
    if range.periodic {
        let tolerance: f64 = std_min((trim.p2 - trim.p1).abs() / 2.0, precision::p_confusion());
        (*t1, *t2) = elclib::adjust_periodic(range.first, range.last, tolerance, trim.p1, trim.p2);
        return Ok(trim.sense);
    }
    let same: bool = if trim.p1 < trim.p2 {
        trim.sense
    } else {
        !trim.sense
    };
    (*t1, *t2) = if trim.p1 < trim.p2 {
        (trim.p1, trim.p2)
    } else {
        (trim.p2, trim.p1)
    };
    if (range.first - *t1 > precision::p_confusion())
        || (*t2 - range.last > precision::p_confusion())
    {
        return Err(errors::construction(&format!(
            "Geom_RectangularTrimmedSurface::{} parameters out of range: [{}, {}]; expected within [{}, {}]",
            range.name, *t1, *t2, range.first, range.last
        )));
    }
    Ok(same)
}

impl TrimState {
    /// The private `SetTrim(U1, U2, V1, V2, UTrim, VTrim, USense, VSense)`; the U
    /// fields are stored before the V checks can raise, as natively.
    pub(super) fn set_trim(&mut self, u: DirectionTrim, v: DirectionTrim) -> Result<(), Failure> {
        let (udeb, ufin, vdeb, vfin) = self.basis.bounds();
        self.isutrimmed = u.trimmed;
        let u_range = DirectionRange {
            first: udeb,
            last: ufin,
            periodic: u.trimmed && self.basis.is_u_periodic(),
            name: "U",
        };
        let u_same: bool = trim_direction((&mut self.utrim1, &mut self.utrim2), u, u_range)?;
        self.isvtrimmed = v.trimmed;
        let v_range = DirectionRange {
            first: vdeb,
            last: vfin,
            periodic: v.trimmed && self.basis.is_v_periodic(),
            name: "V",
        };
        let v_same: bool = trim_direction((&mut self.vtrim1, &mut self.vtrim2), v, v_range)?;
        if !u_same {
            self.u_reverse()?;
        }
        if !v_same {
            self.v_reverse()?;
        }
        Ok(())
    }
    fn stored_u(&self, p1: f64, p2: f64) -> DirectionTrim {
        DirectionTrim {
            p1,
            p2,
            trimmed: self.isutrimmed,
            sense: true,
        }
    }
    fn stored_v(&self, p1: f64, p2: f64) -> DirectionTrim {
        DirectionTrim {
            p1,
            p2,
            trimmed: self.isvtrimmed,
            sense: true,
        }
    }
    /// `UReverse`: reverse the basis in U and re-trim with the reversed bounds.
    pub(super) fn u_reverse(&mut self) -> Result<(), Failure> {
        let u1: f64 = self.basis.u_reversed_parameter(self.utrim2)?;
        let u2: f64 = self.basis.u_reversed_parameter(self.utrim1)?;
        self.basis.u_reverse()?;
        let (u, v) = (
            self.stored_u(u1, u2),
            self.stored_v(self.vtrim1, self.vtrim2),
        );
        self.set_trim(u, v)
    }
    /// `VReverse`: reverse the basis in V and re-trim with the reversed bounds.
    pub(super) fn v_reverse(&mut self) -> Result<(), Failure> {
        let v1: f64 = self.basis.v_reversed_parameter(self.vtrim2)?;
        let v2: f64 = self.basis.v_reversed_parameter(self.vtrim1)?;
        self.basis.v_reverse()?;
        let (u, v) = (
            self.stored_u(self.utrim1, self.utrim2),
            self.stored_v(v1, v2),
        );
        self.set_trim(u, v)
    }
}
