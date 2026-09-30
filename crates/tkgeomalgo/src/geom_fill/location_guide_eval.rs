// Rust translation of the two D0 members and InitX of
// ModelingAlgorithms/TKGeomAlgo/GeomFill/GeomFill_LocationGuide.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The placement of `GeomFill_LocationGuide`: the trihedron frame, turned around
//! the tangent by the contact angle solved on the surface of revolution of the
//! section when the law rotates.
use super::function_guide::GeomFillFunctionGuide;
use super::location_guide::GeomFillLocationGuide;
use super::location_helpers::columns;
use super::location_law::LocationD0;
use super::pipe_error::GeomFillPipeError;
use super::trihedron_law::TrihedronD0;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{Matrix3, Point2, Point3, Vector3, Xyz};
use rustycad_tkmath::math::{FunctionSetRoot, Vector};
use std::f64::consts::PI;

/// A 1-based 3-vector.
pub(crate) fn vector3(values: [f64; 3]) -> Result<Vector<'static>, Failure> {
    let mut v = Vector::with_bounds(1, 3)?;
    for (index, value) in values.iter().enumerate() {
        v[index as i32 + 1] = *value;
    }
    Ok(v)
}

/// The interpolation of `(W, angle, v)` between the two samples around `param`.
fn interpolated_start(rows: &[Vec<Point2>; 2], param: f64) -> [f64; 3] {
    let t = |i: usize| rows[0][i - 1];
    let (mut ideb, mut ifin) = (1, rows[0].len());
    if param == t(ideb).x() {
        ifin = ideb + 1;
    }
    if param == t(ifin).x() {
        ideb = ifin - 1;
    }
    while ideb + 1 != ifin {
        let idemi = (ideb + ifin) / 2;
        let valeur = t(idemi).x();
        if valeur < param {
            ideb = idemi;
        } else if valeur > param {
            ifin = idemi;
        } else {
            ideb = idemi;
            ifin = ideb + 1;
        }
    }
    let (t1, t2) = (t(ideb).x(), t(ifin).x());
    let diff = t2 - t1;
    let (w1, w2) = (t(ideb).y(), t(ifin).y());
    let (p1, p2) = (rows[1][ideb - 1], rows[1][ifin - 1]);
    if diff > 1.0e-7 {
        let (b, a) = ((param - t1) / diff, (t2 - param) / diff);
        return [
            a * w1 + b * w2,
            a * p1.x() + b * p2.x(),
            a * p1.y() + b * p2.y(),
        ];
    }
    [
        (w1 + w2) / 2.0,
        (p1.x() + p2.x()) / 2.0,
        (p1.y() + p2.y()) / 2.0,
    ]
}

impl GeomFillLocationGuide {
    /// `InitX(Param)`: the start of the contact search, brought into the periods of
    /// the guide, of the angle and of a periodic section.
    pub(crate) fn init_x(&self, param: f64) -> Result<(), Failure> {
        let f = self.fields.read().clone();
        let mut x = interpolated_start(&f.poles2d, param);
        let guide = self.guide_curve()?;
        if guide.is_periodic()? {
            x[0] = elclib::in_period(x[0], guide.first_parameter()?, guide.last_parameter()?);
        }
        x[1] = elclib::in_period(x[1], 0.0, 2.0 * PI);
        let sec = super::sweep_failures::live(&f.my_sec, "GeomFill_LocationGuide::InitX", "mySec")?;
        if sec.is_u_periodic()? {
            x[2] = elclib::in_period(x[2], f.uf, f.ul);
        }
        self.fields.write().x = x;
        Ok(())
    }

    /// The D0 of both overloads: the frame of the trihedron law, transformed, then
    /// turned by the contact angle when the law rotates.
    pub(crate) fn guide_d0(&self, param: f64, placement: &mut LocationD0) -> Result<bool, Failure> {
        let curve = self.curve_of(false, "GeomFill_LocationGuide::D0")?;
        let c = curve.get().expect("checked");
        let p = c.d0(param)?;
        placement.v = Vector3::from(p.xyz());
        let mut frame = TrihedronD0::default();
        if !self.law().d0(param, &mut frame)? {
            self.fields.write().status = self.law().error_status();
            return Ok(false);
        }
        placement.m = columns(frame.normal, frame.binormal, frame.tangent);
        let fields = self.fields.read().clone();
        if fields.with_trans {
            placement.m *= fields.trans;
        }
        if !fields.rotation {
            return Ok(true);
        }
        let u = fields.my_first_s + (param - c.first_parameter()?) * fields.ratio;
        self.init_x(param)?;
        self.contact_rotation(param, p, u, placement)
    }

    /// Solves the contact angle `R(2)` from `X` within `Inf`, `Sup` (100
    /// iterations) and turns the normal and binormal by it; false (and
    /// ImpossibleContact) when the search fails.
    fn contact_rotation(
        &self,
        param: f64,
        p: Point3,
        u: f64,
        placement: &mut LocationD0,
    ) -> Result<bool, Failure> {
        let t: Xyz = placement.m.column(3)?;
        let mut b: Xyz = placement.m.column(2)?;
        let mut n: Xyz = placement.m.column(1)?;
        let f = self.fields.read().clone();
        let mut e = GeomFillFunctionGuide::new(&f.my_sec, &self.my_guide, u)?;
        e.set_param(param, p, t, n)?;
        let mut result = FunctionSetRoot::new(&e, &vector3(f.tol_res)?, 100)?;
        result.perform_bounded(
            &mut e,
            &vector3(f.x)?,
            &vector3(f.inf)?,
            &vector3(f.sup)?,
            false,
        )?;
        e.take_failure()?;
        if !result.is_done() {
            self.fields.write().status = GeomFillPipeError::ImpossibleContact;
            return Ok(false);
        }
        let root = result.root()?;
        let r = [root[1], root[2], root[3]];
        self.fields.write().r = r;
        let mut rot = Matrix3::default();
        rot.set_rotation(t, r[1])?;
        b *= rot;
        n *= rot;
        placement.m = columns(Vector3::from(n), Vector3::from(b), Vector3::from(t));
        Ok(true)
    }
}
