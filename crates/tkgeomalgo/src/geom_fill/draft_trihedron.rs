// Rust translation of ModelingAlgorithms/TKGeomAlgo/GeomFill/GeomFill_DraftTrihedron.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomFill_DraftTrihedron`: the frame of a draft, whose normal makes a fixed angle
//! with a direction `B`.
use super::frame_math::dderiv;
use super::frenet::{average_frame, frame_shape};
use super::sweep_failures::live;
use super::trihedron_law::{
    GeomFillTrihedronLaw, GeomFillTrihedronLawBase, TrihedronD0, TrihedronD1, TrihedronD2,
};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkmath::geom_abs::{CurveType, Shape};
use rustycad_tkmath::gp::{Axis1, Vector3};
use std::f64::consts::PI;

/// `myAngle` and `myCos`.
#[derive(Clone, Copy, Default)]
struct DraftAngle {
    angle: f64,
    cos: f64,
}

/// `GeomFill_DraftTrihedron`: the draft frame along the curve.
/// `let law = GeomFillDraftTrihedron::new(b, 0.1)?; law.set_curve(&curve)?;`.
pub struct GeomFillDraftTrihedron {
    parent: StandardTransient,
    base: GeomFillTrihedronLawBase,
    b: Vector3,
    angle: SharedState<DraftAngle>,
}
impl_transient!(
    GeomFillDraftTrihedron,
    "GeomFill_DraftTrihedron",
    dyn GeomFillTrihedronLaw,
    parent,
    interfaces: [dyn GeomFillTrihedronLaw]
);

impl GeomFillDraftTrihedron {
    /// `GeomFill_DraftTrihedron(BiNormal, Angle)`: a null `bi_normal` raises from
    /// the normalization.
    pub fn new(bi_normal: Vector3, angle: f64) -> Result<Self, Failure> {
        let law = Self {
            parent: StandardTransient::default(),
            base: GeomFillTrihedronLawBase::default(),
            b: bi_normal.normalized()?,
            angle: SharedState::new(DraftAngle::default()),
        };
        law.set_angle(angle);
        Ok(law)
    }

    /// `SetAngle(Angle)`: `myAngle = PI/2 + Angle`; `law.set_angle(0.2)`.
    pub fn set_angle(&self, angle: f64) {
        let my_angle = PI / 2.0 + angle;
        *self.angle.write() = DraftAngle {
            angle: my_angle,
            cos: my_angle.cos(),
        };
    }

    fn my_cos(&self) -> f64 {
        self.angle.read().cos
    }

    fn trimmed_curve(&self, member: &str) -> Result<Handle<dyn Adaptor3dCurve>, Failure> {
        let trimmed = self.base.trimmed();
        live(&trimmed, member, "myTrimmed")?;
        Ok(trimmed)
    }
}

/// `a * V1 + b * V2`, the two-term `SetLinearForm`.
fn form(a: f64, v1: Vector3, b: f64, v2: Vector3) -> Vector3 {
    let mut out = Vector3::default();
    out.set_linear_form(a, v1, b, v2);
    out
}

/// `-(u . aux) u + aux`, divided by `norm`: the derivative of a unit vector.
fn unit_derivative(u: Vector3, aux: Vector3, norm: f64) -> Vector3 {
    let mut out = Vector3::default();
    out.set_scaled_sum(-(u.dot(aux)), u, aux);
    out /= norm;
    out
}

impl GeomFillTrihedronLaw for GeomFillDraftTrihedron {
    fn trihedron_base(&self) -> &GeomFillTrihedronLawBase {
        &self.base
    }
    /// A draft law of the same direction and angle on the same curve.
    fn copy(&self) -> Result<Handle<dyn GeomFillTrihedronLaw>, Failure> {
        let copy = GeomFillDraftTrihedron::new(self.b, self.angle.read().angle - PI / 2.0)?;
        copy.set_curve(&self.base.curve())?;
        Ok(Handle::new(copy).cast())
    }
    /// False (outputs partly written) where the tangent is parallel to `B`.
    fn d0(&self, param: f64, frame: &mut TrihedronD0) -> Result<bool, Failure> {
        let trimmed = self.trimmed_curve("GeomFill_DraftTrihedron::D0")?;
        let (_, mut t) = live(&trimmed, "GeomFill_DraftTrihedron::D0", "myTrimmed")?.d1(param)?;
        t.normalize()?;
        let mut b = t.crossed(self.b);
        let normb = b.magnitude();
        b /= normb;
        if normb < 1.0e-12 {
            return Ok(false);
        }
        let v = b.crossed(t);
        let mu = self.my_cos();
        frame.normal = form((1.0 - mu * mu).sqrt(), b, mu, v);
        frame.tangent = frame.normal.crossed(self.b);
        frame.tangent.normalize()?;
        frame.binormal = frame.tangent;
        frame.binormal.cross(frame.normal);
        Ok(true)
    }
    fn d1(&self, param: f64, frame: &mut TrihedronD1) -> Result<bool, Failure> {
        let trimmed = self.trimmed_curve("GeomFill_DraftTrihedron::D1")?;
        let (_, mut t, aux) =
            live(&trimmed, "GeomFill_DraftTrihedron::D1", "myTrimmed")?.d2(param)?;
        let norm_t = t.magnitude();
        t /= norm_t;
        let dt = unit_derivative(t, aux, norm_t);
        let mut b = t.crossed(self.b);
        let normb = b.magnitude();
        if normb < 1.0e-12 {
            return Ok(false);
        }
        b /= normb;
        let db = unit_derivative(b, dt.crossed(self.b), normb);
        let v = b.crossed(t);
        let dv = db.crossed(t) + b.crossed(dt);
        let mu = self.my_cos();
        frame.normal = form((1.0 - mu * mu).sqrt(), b, mu, v);
        frame.d_normal = form((1.0 - mu * mu).sqrt(), db, mu, dv);
        frame.tangent = frame.normal.crossed(self.b);
        let norm_tt = frame.tangent.magnitude();
        frame.tangent /= norm_tt;
        frame.d_tangent = unit_derivative(frame.tangent, frame.d_normal.crossed(self.b), norm_tt);
        frame.binormal = frame.tangent;
        frame.binormal.cross(frame.normal);
        frame.d_binormal.set_sum(
            frame.d_tangent.crossed(frame.normal),
            frame.tangent.crossed(frame.d_normal),
        );
        Ok(true)
    }
    fn d2(&self, param: f64, frame: &mut TrihedronD2) -> Result<bool, Failure> {
        let trimmed = self.trimmed_curve("GeomFill_DraftTrihedron::D2")?;
        let (_, mut t, aux, aux2) =
            live(&trimmed, "GeomFill_DraftTrihedron::D2", "myTrimmed")?.d3(param)?;
        let d2t = dderiv(t, aux, aux2);
        let norm_t = t.magnitude();
        t /= norm_t;
        let dt = unit_derivative(t, aux, norm_t);
        let mut b = t.crossed(self.b);
        let normb = b.magnitude();
        if normb < 1.0e-12 {
            return Ok(false);
        }
        let (aux, aux2) = (dt.crossed(self.b), d2t.crossed(self.b));
        let d2b = dderiv(b, aux, aux2);
        b /= normb;
        let db = unit_derivative(b, aux, normb);
        let v = b.crossed(t);
        let dv = db.crossed(t) + b.crossed(dt);
        let d2v = d2b.crossed(t) + 2.0 * db.crossed(dt) + b.crossed(d2t);
        let mu = self.my_cos();
        let rac = (1.0 - mu * mu).sqrt();
        frame.normal = form(rac, b, mu, v);
        frame.d_normal = form(rac, db, mu, dv);
        frame.d2_normal = form(rac, d2b, mu, d2v);
        draft_d2_tangent(frame, self.b);
        Ok(true)
    }
    fn nb_intervals(&self, shape: Shape) -> Result<i32, Failure> {
        let tmp_s = frame_shape(shape, "GeomFill_DraftTrihedron::NbIntervals")?;
        let curve = self.base.curve();
        live(&curve, "GeomFill_DraftTrihedron::NbIntervals", "myCurve")?.nb_intervals(tmp_s)
    }
    fn intervals(&self, bounds: &mut Array1<'_, f64>, shape: Shape) -> Result<(), Failure> {
        let tmp_s = frame_shape(shape, "GeomFill_DraftTrihedron::Intervals")?;
        let curve = self.base.curve();
        live(&curve, "GeomFill_DraftTrihedron::Intervals", "myCurve")?.intervals(bounds, tmp_s)
    }
    /// The plain mean of 21 frames, not normalized, as natively.
    fn get_average_law(&self) -> Result<TrihedronD0, Failure> {
        let mut average = average_frame(self, &self.base.trimmed())?;
        average.normal /= 21.0;
        average.binormal /= 21.0;
        average.tangent /= 21.0;
        Ok(average)
    }
    /// Whether the curve is a line.
    fn is_constant(&self) -> Result<bool, Failure> {
        let curve = self.base.curve();
        Ok(
            live(&curve, "GeomFill_DraftTrihedron::IsConstant", "myCurve")?.get_type()?
                == CurveType::Line,
        )
    }
    /// Whether `B` is parallel to the direction of a line path or the axis of a
    /// conic path.
    fn is_only_by_3d_curve(&self) -> Result<bool, Failure> {
        let curve = self.base.curve();
        let c = live(
            &curve,
            "GeomFill_DraftTrihedron::IsOnlyBy3dCurve",
            "myCurve",
        )?;
        let axis: Axis1 = match c.get_type()? {
            CurveType::Circle => c.circle()?.axis(),
            CurveType::Ellipse => c.ellipse()?.axis(),
            CurveType::Hyperbola => c.hyperbola()?.axis(),
            CurveType::Parabola => c.parabola()?.axis(),
            CurveType::Line => {
                return Vector3::from(c.line()?.direction())
                    .is_parallel(self.b, precision::angular());
            }
            _ => return Ok(false),
        };
        Vector3::from(axis.direction()).is_parallel(self.b, precision::angular())
    }
}

/// The tangent, binormal and their derivatives of the D2 member from the normal
/// and its derivatives.
fn draft_d2_tangent(frame: &mut TrihedronD2, b: Vector3) {
    frame.tangent = frame.normal.crossed(b);
    let norm_t = frame.tangent.magnitude();
    let aux = frame.d_normal.crossed(b);
    let aux2 = frame.d2_normal.crossed(b);
    frame.d2_tangent = dderiv(frame.tangent, aux, aux2);
    frame.tangent /= norm_t;
    frame.d_tangent = unit_derivative(frame.tangent, aux, norm_t);
    frame.binormal = frame.tangent;
    frame.binormal.cross(frame.normal);
    frame.d_binormal.set_sum(
        frame.d_tangent.crossed(frame.normal),
        frame.tangent.crossed(frame.d_normal),
    );
    frame.d2_binormal.set_linear_form_offset(
        1.0,
        frame.d2_tangent.crossed(frame.normal),
        2.0,
        frame.d_tangent.crossed(frame.d_normal),
        frame.tangent.crossed(frame.d2_normal),
    );
}
