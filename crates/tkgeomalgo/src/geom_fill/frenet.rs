// Rust translation of ModelingAlgorithms/TKGeomAlgo/GeomFill/GeomFill_Frenet.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomFill_Frenet`: the Frenet frame of a curve (tangent, principal normal,
//! binormal), with a special treatment of the points where the curvature vanishes.
//! The singular points are searched by `Init` (`frenet_init.rs`) and evaluated by
//! the `Singular*` members (`frenet_singular.rs`).
use super::frame_math::{fderiv, std_max, std_min};
use super::sweep_failures::{live, out_of_range};
use super::trihedron_law::{
    GeomFillTrihedronLaw, GeomFillTrihedronLawBase, TrihedronD0, TrihedronD1, TrihedronD2,
};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::ncollection::{Array1, LinearVector};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkgeombase::geom_lib;
use rustycad_tkmath::geom_abs::{CurveType, Shape};
use rustycad_tkmath::gp::{self, Axis2, Direction3, Point3, Vector3};

/// The singular points found by `Init` (`mySngl`, `mySnglLen`, `isSngl`).
#[derive(Clone, Default)]
pub(crate) struct FrenetSingularities {
    /// `mySngl`: the singular parameters, 1-based.
    pub sngl: Array1<'static, f64>,
    /// `mySnglLen`: the half width of the neighbourhood of each.
    pub sngl_len: Array1<'static, f64>,
    /// `isSngl`.
    pub is_sngl: bool,
}

/// `GeomFill_Frenet`: the Frenet trihedron law.
///
/// The native member `P` only receives the curve point of each evaluation and is
/// never read, so the port keeps it local.
/// `let law = GeomFillFrenet::new(); law.set_curve(&curve)?; law.d0(0.5, &mut frame)?`.
#[derive(Default)]
pub struct GeomFillFrenet {
    parent: StandardTransient,
    base: GeomFillTrihedronLawBase,
    pub(crate) singular: SharedState<FrenetSingularities>,
}
impl_transient!(
    GeomFillFrenet,
    "GeomFill_Frenet",
    dyn GeomFillTrihedronLaw,
    parent,
    interfaces: [dyn GeomFillTrihedronLaw]
);

/// The continuity of the curve that gives continuity `shape` to the frame; G1 and
/// G2 raise Standard_OutOfRange, as natively.
pub(crate) fn frame_shape(shape: Shape, member: &str) -> Result<Shape, Failure> {
    match shape {
        Shape::C0 => Ok(Shape::C2),
        Shape::C1 => Ok(Shape::C3),
        Shape::C2 | Shape::C3 | Shape::CN => Ok(Shape::CN),
        _ => Err(out_of_range(member, &format!("{shape:?}"))),
    }
}

/// `gp_Ax2(gp_Pnt(0, 0, 0), Tangent)`: the frame of normal `tangent` at the origin.
pub(crate) fn origin_frame(tangent: Vector3) -> Result<Axis2, Failure> {
    Axis2::new(
        Point3::new(0.0, 0.0, 0.0),
        Direction3::from_vector(tangent)?,
    )
}

impl GeomFillFrenet {
    /// `GeomFill_Frenet()`: no curve, no singularity; `GeomFillFrenet::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `myTrimmed`, or the Standard_NullObject of a law without curve.
    pub(crate) fn trimmed(&self, member: &str) -> Result<Handle<dyn Adaptor3dCurve>, Failure> {
        let trimmed = self.base.trimmed();
        live(&trimmed, member, "myTrimmed")?;
        Ok(trimmed)
    }

    /// The regular D0 once the parameter is not singular: the frame of `C'` and
    /// `C''`, or the rotated frame of a nearby parameter where `C'` vanishes.
    fn regular_d0(
        &self,
        the_param: f64,
        delta: f64,
        frame: &mut TrihedronD0,
    ) -> Result<bool, Failure> {
        let trimmed = self.trimmed("GeomFill_Frenet::D0")?;
        let curve = live(&trimmed, "GeomFill_Frenet::D0", "myTrimmed")?;
        let a_param = the_param + delta;
        let (_, tangent, binormal) = curve.d2(a_param)?;
        frame.tangent = tangent;
        frame.binormal = binormal;
        let ndu = frame.tangent.magnitude();
        if ndu <= gp::resolution() {
            return self.null_tangent_d0(curve, the_param, a_param, frame);
        }
        frame.tangent /= ndu;
        frame.binormal = frame.tangent.crossed(frame.binormal);
        let norm = frame.binormal.magnitude();
        if norm <= gp::resolution() {
            frame.binormal = Vector3::from(origin_frame(frame.tangent)?.y_direction().xyz());
        } else {
            frame.binormal.normalize()?;
        }
        frame.normal = frame.binormal;
        frame.normal.cross(frame.tangent);
        Ok(true)
    }

    /// The branch of D0 where `C'` vanishes: the frame of a parameter `10 * aDelta`
    /// away, rotated onto the first non-null derivative (or a finite difference).
    fn null_tangent_d0(
        &self,
        curve: &dyn Adaptor3dCurve,
        the_param: f64,
        a_param: f64,
        frame: &mut TrihedronD0,
    ) -> Result<bool, Failure> {
        let an_uinfium = curve.first_parameter()?;
        let a_delta = (curve.last_parameter()? - an_uinfium) * 1.0e-3;
        let a_tn = substitute_tangent(curve, the_param, an_uinfium, a_delta)?;
        let d_par = 10.0 * a_delta;
        let other = if the_param - an_uinfium < d_par {
            a_param + d_par
        } else {
            a_param - d_par
        };
        if !self.d0(other, frame)? {
            return Ok(false);
        }
        rotate_trihedron(frame, a_tn)
    }
}

/// The first non-null derivative of order 2 or 3 at `the_param`, oriented along the
/// curve, or a second-order finite difference of three points.
fn substitute_tangent(
    curve: &dyn Adaptor3dCurve,
    the_param: f64,
    an_uinfium: f64,
    a_delta: f64,
) -> Result<Vector3, Failure> {
    let (mut an_index, mut a_tn) = (1, Vector3::default());
    let mut is_derive_found = false;
    while !is_derive_found {
        an_index += 1;
        a_tn = curve.dn(the_param, an_index)?;
        is_derive_found = a_tn.magnitude() > gp::resolution();
        if an_index >= 3 {
            break;
        }
    }
    if !is_derive_found {
        return finite_difference_tangent(curve, the_param, an_uinfium, a_delta);
    }
    let u = if the_param - an_uinfium < a_delta {
        the_param + a_delta
    } else {
        the_param - a_delta
    };
    let p1 = curve.d0(std_min(the_param, u))?;
    let p2 = curve.d0(std_max(the_param, u))?;
    if a_tn.dot(Vector3::between(p1, p2)) < 0.0 {
        a_tn = -a_tn;
    }
    Ok(a_tn)
}

/// `-3 V1 + 4 V2 - V3` forward (or `V1 - 4 V2 + 3 V3` backward) over steps of
/// `a_delta`.
fn finite_difference_tangent(
    curve: &dyn Adaptor3dCurve,
    the_param: f64,
    an_uinfium: f64,
    a_delta: f64,
) -> Result<Vector3, Failure> {
    let origin = Point3::new(0.0, 0.0, 0.0);
    let grown = the_param - an_uinfium < 2.0 * a_delta;
    let params = if grown {
        [the_param, the_param + a_delta, the_param + 2.0 * a_delta]
    } else {
        [the_param - 2.0 * a_delta, the_param - a_delta, the_param]
    };
    let v1 = Vector3::between(origin, curve.d0(params[0])?);
    let v2 = Vector3::between(origin, curve.d0(params[1])?);
    let v3 = Vector3::between(origin, curve.d0(params[2])?);
    if grown {
        return Ok(-3.0 * v1 + 4.0 * v2 - v3);
    }
    Ok(v1 - 4.0 * v2 + 3.0 * v3)
}

/// `CosAngle(V1, V2)`: the cosine of the angle, clamped to `[-1, 1]`, and 1 for a
/// null vector.
fn cos_angle(v1: Vector3, v2: Vector3) -> f64 {
    let a_tol = gp::resolution();
    let (m1, m2) = (v1.magnitude(), v2.magnitude());
    if m1 <= a_tol || m2 <= a_tol {
        return 1.0;
    }
    (v1.dot(v2) / (m1 * m2)).clamp(-1.0, 1.0)
}

/// The two rotation matrices (rows) of angle `acos(c)` around the unit axis `p`,
/// one for each sign of the sine.
fn rotation_rows(p: [f64; 3], c: f64, s: f64) -> [[Vector3; 3]; 2] {
    let [px, py, pz] = p;
    let a = 1.0 - c;
    let row = |sign: f64| {
        [
            Vector3::new(
                a * px * px + c,
                a * px * py - sign * pz * s,
                a * px * pz + sign * py * s,
            ),
            Vector3::new(
                a * px * py + sign * pz * s,
                a * py * py + c,
                a * py * pz - sign * px * s,
            ),
            Vector3::new(
                a * px * pz - sign * py * s,
                a * py * pz + sign * px * s,
                a * pz * pz + c,
            ),
        ]
    };
    [row(1.0), row(-1.0)]
}

/// `RotateTrihedron(Tangent, Normal, BiNormal, NewTangent)`: turns the frame so that
/// its tangent goes to `new_tangent`; false when they still differ by more than
/// Precision::Angular().
fn rotate_trihedron(frame: &mut TrihedronD0, new_tangent: Vector3) -> Result<bool, Failure> {
    let an_inf_cos = precision::angular().cos();
    let mut an_axis = frame.tangent.crossed(new_tangent);
    let nt = an_axis.magnitude();
    if nt <= gp::resolution() {
        return Ok(true);
    }
    an_axis /= nt;
    let c = cos_angle(frame.tangent, new_tangent);
    let s = (1.0 - c * c).sqrt();
    let rows = rotation_rows([an_axis.x(), an_axis.y(), an_axis.z()], c, s);
    let apply = |v: Vector3, m: &[Vector3; 3]| Vector3::new(v.dot(m[0]), v.dot(m[1]), v.dot(m[2]));
    let (t1, t2) = (
        apply(frame.tangent, &rows[0]),
        apply(frame.tangent, &rows[1]),
    );
    let m = if cos_angle(t1, new_tangent) >= cos_angle(t2, new_tangent) {
        frame.tangent = t1;
        &rows[0]
    } else {
        frame.tangent = t2;
        &rows[1]
    };
    frame.normal = apply(frame.normal, m);
    frame.binormal = apply(frame.binormal, m);
    Ok(cos_angle(frame.tangent, new_tangent) >= an_inf_cos)
}

impl GeomFillTrihedronLaw for GeomFillFrenet {
    fn trihedron_base(&self) -> &GeomFillTrihedronLawBase {
        &self.base
    }
    /// A new Frenet law on the same curve (when there is one).
    fn copy(&self) -> Result<Handle<dyn GeomFillTrihedronLaw>, Failure> {
        let copy = GeomFillFrenet::new();
        let curve = self.base.curve();
        if curve.is_some() {
            copy.set_curve(&curve)?;
        }
        Ok(Handle::new(copy).cast())
    }
    /// Loads the curve; conics and lines have no singular point, other curves run
    /// `Init`. Always true.
    fn set_curve(&self, curve: &Handle<dyn Adaptor3dCurve>) -> Result<bool, Failure> {
        self.base.set_curve(curve);
        let Some(c) = curve.get() else {
            return Ok(true);
        };
        match c.get_type()? {
            CurveType::Circle
            | CurveType::Ellipse
            | CurveType::Hyperbola
            | CurveType::Parabola
            | CurveType::Line => self.singular.write().is_sngl = false,
            _ => self.init()?,
        }
        Ok(true)
    }
    fn d0(&self, param: f64, frame: &mut TrihedronD0) -> Result<bool, Failure> {
        let mut delta = 0.0;
        if let Some(index) = self.is_singular(param)
            && self.singular_d0(param, index, frame, &mut delta)?
        {
            return Ok(true);
        }
        self.regular_d0(param, delta, frame)
    }
    fn d1(&self, param: f64, frame: &mut TrihedronD1) -> Result<bool, Failure> {
        let mut delta = 0.0;
        if let Some(index) = self.is_singular(param)
            && self.singular_d1(param, index, frame, &mut delta)?
        {
            return Ok(true);
        }
        let trimmed = self.trimmed("GeomFill_Frenet::D1")?;
        let (_, dc1, dc2, dc3) =
            live(&trimmed, "GeomFill_Frenet::D1", "myTrimmed")?.d3(param + delta)?;
        frame.tangent = dc1.normalized()?;
        if frame.tangent.crossed(dc2).magnitude() <= gp::resolution() {
            let axe = origin_frame(frame.tangent)?;
            frame.normal = Vector3::from(axe.x_direction().xyz());
            frame.binormal = Vector3::from(axe.y_direction().xyz());
            (frame.d_tangent, frame.d_normal, frame.d_binormal) = Default::default();
            return Ok(true);
        }
        frame.binormal = frame.tangent.crossed(dc2).normalized()?;
        frame.normal = frame.binormal.crossed(frame.tangent);
        frame.d_tangent = fderiv(dc1, dc2);
        let instead_dc1 = frame.tangent.crossed(dc2);
        let instead_dc2 = frame.d_tangent.crossed(dc2) + frame.tangent.crossed(dc3);
        frame.d_binormal = fderiv(instead_dc1, instead_dc2);
        frame.d_normal =
            frame.d_binormal.crossed(frame.tangent) + frame.binormal.crossed(frame.d_tangent);
        Ok(true)
    }
    fn d2(&self, param: f64, frame: &mut TrihedronD2) -> Result<bool, Failure> {
        let mut delta = 0.0;
        if let Some(index) = self.is_singular(param)
            && self.singular_d2(param, index, frame, &mut delta)?
        {
            return Ok(true);
        }
        self.regular_d2(param + delta, frame)
    }
    fn nb_intervals(&self, shape: Shape) -> Result<i32, Failure> {
        let tmp_s = frame_shape(shape, "GeomFill_Frenet::NbIntervals")?;
        let curve = self.base.curve();
        let c = live(&curve, "GeomFill_Frenet::NbIntervals", "myCurve")?;
        let nb_trimmed = c.nb_intervals(tmp_s)?;
        let singular = self.singular.read().clone();
        if !singular.is_sngl {
            return Ok(nb_trimmed);
        }
        Ok(self
            .fused_intervals(c, tmp_s, nb_trimmed, &singular.sngl)?
            .len() as i32
            - 1)
    }
    fn intervals(&self, bounds: &mut Array1<'_, f64>, shape: Shape) -> Result<(), Failure> {
        let tmp_s = frame_shape(shape, "GeomFill_Frenet::Intervals")?;
        let curve = self.base.curve();
        let c = live(&curve, "GeomFill_Frenet::Intervals", "myCurve")?;
        let singular = self.singular.read().clone();
        if !singular.is_sngl {
            return c.intervals(bounds, tmp_s);
        }
        let nb_trimmed = c.nb_intervals(tmp_s)?;
        let fusion = self.fused_intervals(c, tmp_s, nb_trimmed, &singular.sngl)?;
        copy_fusion(&fusion, bounds)
    }
    fn get_average_law(&self) -> Result<TrihedronD0, Failure> {
        let trimmed = self.trimmed("GeomFill_Frenet::GetAverageLaw")?;
        let mut average = average_frame(self, &trimmed)?;
        average.tangent /= 21.0;
        average.normal /= 21.0;
        average.tangent.normalize()?;
        average.binormal = average.tangent.crossed(average.normal).normalized()?;
        average.normal = average.binormal.crossed(average.tangent);
        Ok(average)
    }
    /// Whether the curve is a line.
    fn is_constant(&self) -> Result<bool, Failure> {
        let curve = self.base.curve();
        Ok(live(&curve, "GeomFill_Frenet::IsConstant", "myCurve")?.get_type()? == CurveType::Line)
    }
    /// Always true.
    fn is_only_by_3d_curve(&self) -> Result<bool, Failure> {
        Ok(true)
    }
}

impl GeomFillFrenet {
    /// The regular D2 at `the_param`.
    fn regular_d2(&self, the_param: f64, frame: &mut TrihedronD2) -> Result<bool, Failure> {
        let trimmed = self.trimmed("GeomFill_Frenet::D2")?;
        let curve = live(&trimmed, "GeomFill_Frenet::D2", "myTrimmed")?;
        let (_, dc1, dc2, dc3) = curve.d3(the_param)?;
        let dc4 = curve.dn(the_param, 4)?;
        frame.tangent = dc1.normalized()?;
        if frame.tangent.crossed(dc2).magnitude() <= gp::resolution() {
            let axe = origin_frame(frame.tangent)?;
            frame.normal = Vector3::from(axe.x_direction().xyz());
            frame.binormal = Vector3::from(axe.y_direction().xyz());
            (frame.d_tangent, frame.d_normal, frame.d_binormal) = Default::default();
            (frame.d2_tangent, frame.d2_normal, frame.d2_binormal) = Default::default();
            return Ok(true);
        }
        frame.binormal = frame.tangent.crossed(dc2).normalized()?;
        frame.normal = frame.binormal.crossed(frame.tangent);
        frame.d_tangent = fderiv(dc1, dc2);
        frame.d2_tangent = super::frame_math::dderiv(dc1, dc2, dc3);
        let instead = [
            frame.tangent.crossed(dc2),
            frame.d_tangent.crossed(dc2) + frame.tangent.crossed(dc3),
            frame.d2_tangent.crossed(dc2)
                + 2.0 * frame.d_tangent.crossed(dc3)
                + frame.tangent.crossed(dc4),
        ];
        frame.d_binormal = fderiv(instead[0], instead[1]);
        frame.d2_binormal = super::frame_math::dderiv(instead[0], instead[1], instead[2]);
        set_normal_derivatives(frame);
        Ok(true)
    }

    /// `GeomLib::FuseIntervals` of the curve intervals of `tmp_s` and `mySngl`.
    fn fused_intervals(
        &self,
        c: &dyn Adaptor3dCurve,
        tmp_s: Shape,
        nb_trimmed: i32,
        sngl: &Array1<'static, f64>,
    ) -> Result<LinearVector<f64>, Failure> {
        let mut trim_int = rustycad_tkg2d::geom2d::arrays::one_based(nb_trimmed + 1)?;
        c.intervals(&mut trim_int, tmp_s)?;
        let mut fusion = LinearVector::with_capacity(0);
        geom_lib::fuse_intervals(&trim_int, sngl, &mut fusion, precision::p_confusion(), true);
        Ok(fusion)
    }
}

/// `DNormal` and `D2Normal` from the binormal and tangent derivatives.
pub(crate) fn set_normal_derivatives(frame: &mut TrihedronD2) {
    frame.d_normal =
        frame.d_binormal.crossed(frame.tangent) + frame.binormal.crossed(frame.d_tangent);
    frame.d2_normal = frame.d2_binormal.crossed(frame.tangent)
        + 2.0 * frame.d_binormal.crossed(frame.d_tangent)
        + frame.binormal.crossed(frame.d2_tangent);
}

/// `T.ChangeAt(i) = Fusion.Value(i)` from the first slot of `bounds`.
pub(crate) fn copy_fusion(
    fusion: &LinearVector<f64>,
    bounds: &mut Array1<'_, f64>,
) -> Result<(), Failure> {
    let lower = bounds.lower();
    for (index, value) in fusion.as_slice().iter().enumerate() {
        rustycad_tkg2d::geom2d::arrays::put(bounds, lower + index as i32, *value)?;
    }
    Ok(())
}

/// The sum of the 21 frames of `law` at `First + i * Step` (clamped to `Last`),
/// the loop that the GetAverageLaw of several laws repeats.
pub(crate) fn average_frame(
    law: &dyn GeomFillTrihedronLaw,
    trimmed: &Handle<dyn Adaptor3dCurve>,
) -> Result<TrihedronD0, Failure> {
    let curve = live(trimmed, "GeomFill_TrihedronLaw::GetAverageLaw", "myTrimmed")?;
    let step = (curve.last_parameter()? - curve.first_parameter()?) / 20.0;
    let mut sum = TrihedronD0::default();
    let mut frame = TrihedronD0::default();
    for i in 0..=20 {
        let mut param = curve.first_parameter()? + f64::from(i) * step;
        if param > curve.last_parameter()? {
            param = curve.last_parameter()?;
        }
        law.d0(param, &mut frame)?;
        sum.tangent += frame.tangent;
        sum.normal += frame.normal;
        sum.binormal += frame.binormal;
    }
    Ok(sum)
}
