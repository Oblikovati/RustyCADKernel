// Rust translation of the D0, D1 and D2 members of
// ModelingAlgorithms/TKGeomAlgo/GeomFill/GeomFill_LocationDraft.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The evaluators of `GeomFill_LocationDraft`: the placement from the draft
//! trihedron and, with a stop surface, the trace of the generatrix on it found by a
//! Newton search started from the sampled traces, with its derivatives.
use super::function_draft::GeomFillFunctionDraft;
use super::location_draft::{GeomFillLocationDraft, NB_PTS, generatrix, generatrix_direction};
use super::location_helpers::columns;
use super::location_law::{LocationD0, LocationD1, LocationD2, LocationPoles2d};
use super::sweep_failures::sweep_failure;
use super::tensor::GeomFillTensor;
use super::trihedron_law::{GeomFillTrihedronLaw, TrihedronD0, TrihedronD1, TrihedronD2};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::real::is_equal;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::arrays::put;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkmath::gp::{Matrix3, Point2, Point3, Vector2, Vector3};
use rustycad_tkmath::math::{Gauss, Matrix, NewtonFunctionSetRoot, Vector};

/// The Newton settings of one evaluator: `XTol`, `FTol`, the iteration limit.
struct NewtonSettings {
    x_tol: f64,
    f_tol: f64,
    iter: i32,
}

/// The Newton settings of the trace search of `D0`, `D1` and `D2`.
const NEWTON_D0: NewtonSettings = NewtonSettings {
    x_tol: 0.00001,
    f_tol: 0.0000001,
    iter: 100,
};
const NEWTON_D1: NewtonSettings = NewtonSettings {
    x_tol: 0.0001,
    f_tol: 0.000001,
    iter: 100,
};
const NEWTON_D2: NewtonSettings = NewtonSettings {
    x_tol: 0.0001,
    f_tol: 0.000001,
    iter: 150,
};

/// The frame values of a law evaluation.
fn value_frame(tangent: Vector3, normal: Vector3, binormal: Vector3) -> TrihedronD0 {
    TrihedronD0 {
        tangent,
        normal,
        binormal,
    }
}

/// The two 2d poles of the trace root `r` (`(u, v)` on the surface, `(w, t)`).
fn put_root(poles2d: &mut Array1<'_, Point2>, r: &Vector<'_>, param: f64) -> Result<(), Failure> {
    put(poles2d, 1, Point2::new(r[2], r[3]))?;
    put(poles2d, 2, Point2::new(r[1], param))
}

/// `myPoles2d->Value(index)` with the native Standard_OutOfRange check.
fn pole(poles: &[Point2], index: i32) -> Result<Point2, Failure> {
    if index < 1 || index as usize > poles.len() {
        return Err(sweep_failure(
            "Standard_OutOfRange",
            &format!(
                "GeomFill_LocationDraft: myPoles2d({index}) outside 1..{}",
                poles.len()
            ),
        ));
    }
    Ok(poles[index as usize - 1])
}

/// `(W0, U0, V0)`: the start of the trace search, interpolated between the sampled
/// traces around `param` (0 beyond the last sample).
fn start_point(poles: &[Point2], param: f64) -> Result<[f64; 3], Failure> {
    let last = 2 * NB_PTS;
    let mut ii = 1;
    while ii < last && pole(poles, ii)?.y() < param {
        ii += 2;
    }
    if ii < last && !is_equal(pole(poles, ii)?.y(), param) {
        let t1 = pole(poles, ii)?.y();
        let t2 = pole(poles, ii - 2)?.y();
        let paramt1 = (param - t1) / (t2 - t1);
        let t2param = (t2 - param) / (t2 - t1);
        let w0 = pole(poles, ii - 2)?.x() * paramt1 + pole(poles, ii)?.x() * t2param;
        let u0 = pole(poles, ii - 1)?.x() * paramt1 + pole(poles, ii + 1)?.x() * t2param;
        let v0 = pole(poles, ii - 1)?.y() * paramt1 + pole(poles, ii + 1)?.y() * t2param;
        return Ok([w0, u0, v0]);
    }
    if ii < last && is_equal(pole(poles, ii)?.y(), param) {
        return Ok([
            pole(poles, ii)?.x(),
            pole(poles, ii + 1)?.x(),
            pole(poles, ii + 1)?.y(),
        ]);
    }
    Ok([0.0, 0.0, 0.0])
}

fn vector3(values: [f64; 3]) -> Result<Vector<'static>, Failure> {
    let mut v = Vector::with_bounds(1, 3)?;
    for (index, value) in values.iter().enumerate() {
        v[index as i32 + 1] = *value;
    }
    Ok(v)
}

fn zero_matrix() -> Result<Matrix<'static>, Failure> {
    super::sweep_failures::matrix_with_bounds(1, 3, 1, 3)
}

impl GeomFillLocationDraft {
    /// `M(N, B, T)` and the transformation.
    fn placed(&self, m: &mut [&mut Matrix3]) {
        let fields = self.fields.read();
        if fields.with_trans {
            for matrix in m.iter_mut() {
                **matrix *= fields.trans;
            }
        }
    }

    /// The trace of the generatrix of `(p, frame)` on the stop surface: the Newton
    /// root `(w, u, v)` and its function, or `None` when the search fails.
    fn trace_root(
        &self,
        p: Point3,
        frame: &TrihedronD0,
        param: f64,
        settings: NewtonSettings,
    ) -> Result<Option<(Vector<'static>, GeomFillFunctionDraft)>, Failure> {
        let fields = self.fields.read().clone();
        let g = generatrix(p, generatrix_direction(frame, fields.my_angle))?;
        let x = vector3(start_point(&fields.poles2d, param)?)?;
        let mut x_tol = Vector::with_bounds(1, 3)?;
        x_tol.init(settings.x_tol);
        let mut e = GeomFillFunctionDraft::new(&fields.my_surf, &g);
        let mut result = NewtonFunctionSetRoot::new(&e, &x_tol, settings.f_tol, settings.iter)?;
        result.perform(&mut e, &x)?;
        e.take_failure()?;
        if !result.is_done() {
            return Ok(None);
        }
        Ok(Some((result.root()?.clone(), e)))
    }

    /// `D0(Param, M, V)`: the placement on the trimmed path.
    pub(crate) fn draft_d0(&self, param: f64, placement: &mut LocationD0) -> Result<bool, Failure> {
        let trimmed = self.curve_of(true, "GeomFill_LocationDraft::D0")?;
        let p = trimmed.get().expect("checked").d0(param)?;
        placement.v = Vector3::from(p.xyz());
        let mut frame = TrihedronD0::default();
        if !self.law().d0(param, &mut frame)? {
            return Ok(false);
        }
        placement.m = columns(frame.normal, frame.binormal, frame.tangent);
        self.placed(&mut [&mut placement.m]);
        Ok(true)
    }

    /// `D0(Param, M, V, Poles2d)`: on the whole path, with the trace `(u, v)` and
    /// `(w, Param)` when the generatrices meet the stop surface.
    pub(crate) fn draft_d0_poles2d(
        &self,
        param: f64,
        placement: &mut LocationD0,
        poles2d: &mut Array1<'_, Point2>,
    ) -> Result<bool, Failure> {
        let curve = self.curve_of(false, "GeomFill_LocationDraft::D0")?;
        let p = curve.get().expect("checked").d0(param)?;
        placement.v = Vector3::from(p.xyz());
        let mut frame = TrihedronD0::default();
        if !self.law().d0(param, &mut frame)? {
            return Ok(false);
        }
        placement.m = columns(frame.normal, frame.binormal, frame.tangent);
        self.placed(&mut [&mut placement.m]);
        if !self.fields.read().intersec {
            return Ok(true);
        }
        let Some((r, _)) = self.trace_root(p, &frame, param, NEWTON_D0)? else {
            return Ok(false);
        };
        put_root(poles2d, &r, param)?;
        Ok(true)
    }

    /// `DSDT`: the derivative of the trace root, solved from `DEDX DSDT = -DEDT`
    /// (left at zero when the Gauss decomposition fails).
    fn root_speed(
        &self,
        e: &GeomFillFunctionDraft,
        r: &Vector<'_>,
        (param, dn): (f64, Vector3),
    ) -> Result<(Vector<'static>, Matrix<'static>, Option<Vector<'static>>), Failure> {
        let fields = self.fields.read().clone();
        let mut dedt = Vector::filled(1, 3, 0.0)?;
        let trimmed: Handle<dyn Adaptor3dCurve> = fields.my_trimmed.clone();
        e.deriv_t(&trimmed, param, r[1], dn, fields.my_angle, &mut dedt)?;
        let mut dsdt = Vector::filled(1, 3, 0.0)?;
        let mut dedx = zero_matrix()?;
        e.try_derivatives(r, &mut dedx)?;
        let ga = Gauss::new(&dedx)?;
        if !ga.is_done() {
            return Ok((dsdt, dedx, None));
        }
        ga.solve_into(&dedt.opposite(), &mut dsdt)?;
        let speed = dsdt.clone();
        Ok((dsdt, dedx, Some(speed)))
    }
}

/// `DPoles2d(1) = (DSDT(2), DSDT(3))`, `DPoles2d(2) = (DSDT(1), 1)`.
fn store_speed(d_poles2d: &mut Array1<'_, Vector2>, dsdt: &Vector<'_>) -> Result<(), Failure> {
    put(d_poles2d, 1, Vector2::new(dsdt[2], dsdt[3]))?;
    put(d_poles2d, 2, Vector2::new(dsdt[1], 1.0))
}

impl GeomFillLocationDraft {
    /// `D1(...)`.
    pub(crate) fn draft_d1(
        &self,
        param: f64,
        placement: &mut LocationD1,
        poles2d: &mut Array1<'_, Point2>,
        d_poles2d: &mut Array1<'_, Vector2>,
    ) -> Result<bool, Failure> {
        let curve = self.curve_of(false, "GeomFill_LocationDraft::D1")?;
        let (p, dv) = curve.get().expect("checked").d1(param)?;
        placement.dv = dv;
        placement.v = Vector3::from(p.xyz());
        let mut f = TrihedronD1::default();
        if !self.law().d1(param, &mut f)? {
            return Ok(false);
        }
        placement.m = columns(f.normal, f.binormal, f.tangent);
        placement.dm = columns(f.d_normal, f.d_binormal, f.d_tangent);
        self.placed(&mut [&mut placement.m, &mut placement.dm]);
        if !self.fields.read().intersec {
            return Ok(true);
        }
        let frame = value_frame(f.tangent, f.normal, f.binormal);
        let Some((r, e)) = self.trace_root(p, &frame, param, NEWTON_D1)? else {
            return Ok(false);
        };
        put_root(poles2d, &r, param)?;
        if let (_, _, Some(dsdt)) = self.root_speed(&e, &r, (param, f.d_normal))? {
            store_speed(d_poles2d, &dsdt)?;
        }
        Ok(true)
    }

    /// `D2(...)`: true even when the trace search fails, as natively.
    pub(crate) fn draft_d2(
        &self,
        param: f64,
        placement: &mut LocationD2,
        poles: LocationPoles2d<'_, '_>,
    ) -> Result<bool, Failure> {
        let curve = self.curve_of(false, "GeomFill_LocationDraft::D2")?;
        let (p, dv, d2v) = curve.get().expect("checked").d2(param)?;
        (placement.dv, placement.d2v) = (dv, d2v);
        placement.v = Vector3::from(p.xyz());
        let mut f = TrihedronD2::default();
        if !self.law().d2(param, &mut f)? {
            return Ok(false);
        }
        placement.m = columns(f.normal, f.binormal, f.tangent);
        placement.dm = columns(f.d_normal, f.d_binormal, f.d_tangent);
        placement.d2m = columns(f.d2_normal, f.d2_binormal, f.d2_tangent);
        self.placed(&mut [&mut placement.m, &mut placement.dm, &mut placement.d2m]);
        if !self.fields.read().intersec {
            return Ok(true);
        }
        let frame = value_frame(f.tangent, f.normal, f.binormal);
        let Some((r, e)) = self.trace_root(p, &frame, param, NEWTON_D2)? else {
            return Ok(true);
        };
        put_root(poles.poles2d, &r, param)?;
        let (dsdt, dedx, speed) = self.root_speed(&e, &r, (param, f.d_normal))?;
        if let Some(speed) = &speed {
            store_speed(poles.d_poles2d, speed)?;
        }
        self.trace_acceleration(
            &e,
            (&r, &dsdt, &dedx),
            (param, f.d_normal, f.d2_normal),
            poles.d2_poles2d,
        )
    }

    /// `D2SDT2` from `DEDX D2SDT2 = -aT DSDT - 2 D2EDTDX DSDT - D2EDT2`; false when
    /// the Gauss decomposition fails.
    fn trace_acceleration(
        &self,
        e: &GeomFillFunctionDraft,
        (r, dsdt, dedx): (&Vector<'_>, &Vector<'_>, &Matrix<'_>),
        (param, dn, d2n): (f64, Vector3, Vector3),
        d2_poles2d: &mut Array1<'_, Vector2>,
    ) -> Result<bool, Failure> {
        let fields = self.fields.read().clone();
        let mut d2edx2 = GeomFillTensor::new(3, 3, 3)?;
        e.deriv2_x(r, &mut d2edx2)?;
        let mut d2edt2 = Vector::filled(1, 3, 0.0)?;
        e.deriv2_t(
            &fields.my_trimmed,
            param,
            r[1],
            d2n,
            fields.my_angle,
            &mut d2edt2,
        )?;
        let mut d2edtdx = zero_matrix()?;
        e.deriv_tx(dn, fields.my_angle, &mut d2edtdx)?;
        let mut d2sdt2 = Vector::filled(1, 3, 0.0)?;
        let mut at = zero_matrix()?;
        d2edx2.multiply(dsdt, &mut at)?;
        let ga1 = Gauss::new(dedx)?;
        if !ga1.is_done() {
            return Ok(false);
        }
        let first = at.opposite().multiplied_vector(dsdt)?;
        let second = d2edtdx.multiplied(2.0).multiplied_vector(dsdt)?;
        let rhs = first.subtracted(&second)?.subtracted(&d2edt2)?;
        ga1.solve_into(&rhs, &mut d2sdt2)?;
        put(d2_poles2d, 1, Vector2::new(d2sdt2[2], d2sdt2[3]))?;
        put(d2_poles2d, 2, Vector2::new(d2sdt2[1], 0.0))?;
        Ok(true)
    }
}
