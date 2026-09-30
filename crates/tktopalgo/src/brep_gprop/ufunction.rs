// Rust translation of ModelingAlgorithms/TKTopAlgo/BRepGProp/BRepGProp_UFunction.hxx/.lxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepGProp_UFunction`: the integrand of the inner (U) integral of the
//! Gauss-Kronrod volume properties.
use super::face::BRepGPropFace;
use rustycad_tkernel::standard::Failure;
use rustycad_tkgeombase::gprop::ValueType;
use rustycad_tkmath::gp::{Point3, Xyz};
use rustycad_tkmath::math::Function;

/// `BRepGProp_UFunction`: the value of the integrand along U at a fixed V, for the
/// volume, a coordinate of the centre or a moment of inertia, of the region between
/// the face and a point (`is_by_point`, coefficients the shift of the inertia point) or
/// a plane (four coefficients). An exception of the surface is kept and `value`
/// answers false (docs/topalgo-design.md section 9).
///
/// ```
/// use rustycad_tkgeombase::gprop::ValueType;
/// use rustycad_tkmath::gp::Point3;
/// use rustycad_tkmath::math::Function;
/// use rustycad_tktopalgo::brep_gprop::{BRepGPropFace, BRepGPropUFunction};
/// let face = BRepGPropFace::new(false);
/// let mut f = BRepGPropUFunction::new(&face, Point3::default(), true, &[0.0; 3]);
/// let mut value = 0.0;
/// assert!(!f.value(0.5, &mut value)); // GProp_Unknown
/// ```
pub struct BRepGPropUFunction {
    surface: BRepGPropFace,
    vertex: Point3,
    coeffs: Vec<f64>,
    v_param: f64,
    value_type: ValueType,
    is_by_point: bool,
    failure: Option<Failure>,
}

impl BRepGPropUFunction {
    /// `BRepGProp_UFunction(theSurface, theVertex, IsByPoint, theCoeffs)`: a copy of the
    /// face tool, value type GProp_Unknown, V 0;
    /// `BRepGPropUFunction::new(&face, vertex, true, &[0.0; 3])`.
    pub fn new(surface: &BRepGPropFace, vertex: Point3, is_by_point: bool, coeffs: &[f64]) -> Self {
        Self {
            surface: surface.clone(),
            vertex,
            coeffs: coeffs.to_vec(),
            v_param: 0.,
            value_type: ValueType::Unknown,
            is_by_point,
            failure: None,
        }
    }

    /// `SetValueType(theType)`; `f.set_value_type(ValueType::Mass)`.
    pub fn set_value_type(&mut self, value_type: ValueType) {
        self.value_type = value_type;
    }

    /// `SetVParam(theVParam)`: the fixed V of the integral; `f.set_v_param(0.5)`.
    pub fn set_v_param(&mut self, v_param: f64) {
        self.v_param = v_param;
    }

    /// The exception a `value` call kept (and forgets it); `f.take_failure()`.
    pub fn take_failure(&mut self) -> Option<Failure> {
        self.failure.take()
    }

    /// `VolumeValue(X, thePMP0, theS, theD1)`: the volume integrand, the point minus the
    /// vertex, and by plane the normal and distance factors (0 by point).
    fn volume_value(&self, x: f64) -> Result<(f64, Xyz, f64, f64), Failure> {
        let (point, normal) = self.surface.normal(x, self.v_param)?;
        let pmp0 = point.xyz() - self.vertex.xyz();
        if self.is_by_point {
            return Ok((pmp0.dot(*normal.xyz()), pmp0, 0.0, 0.0));
        }
        let c = &self.coeffs;
        let s = normal.x() * c[0] + normal.y() * c[1] + normal.z() * c[2];
        let d1 = pmp0.x() * c[0] + pmp0.y() * c[1] + pmp0.z() * c[2] - c[3];
        Ok((s * d1, pmp0, s, d1))
    }

    /// `CenterMassValue(X, F)`: the integrand of a centre coordinate.
    fn center_mass_value(&self, x: f64) -> Result<Option<f64>, Failure> {
        let (f, pmp0, _, d1) = self.volume_value(x)?;
        let axis = match self.value_type {
            ValueType::CenterMassX => 0,
            ValueType::CenterMassY => 1,
            ValueType::CenterMassZ => 2,
            _ => return Ok(None),
        };
        let coordinate = [pmp0.x(), pmp0.y(), pmp0.z()][axis];
        if self.is_by_point {
            return Ok(Some(f * coordinate));
        }
        Ok(Some(f * (coordinate - 0.5 * self.coeffs[axis] * d1)))
    }

    /// `InertiaValue(X, F)`: the integrand of a moment or product of inertia.
    fn inertia_value(&self, x: f64) -> Result<Option<f64>, Failure> {
        let (f, pmp0, s, d1) = self.volume_value(x)?;
        let Some((first, second, is_moment)) = inertia_axes(self.value_type) else {
            return Ok(None);
        };
        let p = [pmp0.x(), pmp0.y(), pmp0.z()];
        let c = &self.coeffs;
        if self.is_by_point {
            let (param1, param2) = (p[first] - c[first], p[second] - c[second]);
            if is_moment {
                return Ok(Some(f * (param1 * param1 + param2 * param2)));
            }
            return Ok(Some(f * (-param1 * param2)));
        }
        Ok(Some(plane_inertia(
            (p[first], p[second]),
            (c[first], c[second]),
            (s, d1),
            is_moment,
        )))
    }
}

/// The two axes a moment (`is_moment`) or product of inertia reads: XX and YZ read Y
/// and Z, YY and XZ read X and Z, ZZ and XY read X and Y.
fn inertia_axes(value_type: ValueType) -> Option<(usize, usize, bool)> {
    Some(match value_type {
        ValueType::InertiaXX => (1, 2, true),
        ValueType::InertiaYY => (0, 2, true),
        ValueType::InertiaZZ => (0, 1, true),
        ValueType::InertiaXY => (0, 1, false),
        ValueType::InertiaYZ => (1, 2, false),
        ValueType::InertiaXZ => (0, 2, false),
        _ => return None,
    })
}

/// The by-plane inertia integrand of the pair of coordinates `p` and coefficients `c`.
fn plane_inertia(p: (f64, f64), c: (f64, f64), (s, d1): (f64, f64), is_moment: bool) -> f64 {
    let mut d2 = d1 * d1;
    let d3 = d1 * d2 / 3.;
    if is_moment {
        let (p1, p2) = (p.0 - c.0 * d1, p.1 - c.1 * d1);
        let param1 = p1 * p1 * d1 + p1 * c.0 * d2 + c.0 * c.0 * d3;
        let param2 = p2 * p2 * d1 + p2 * c.1 * d2 + c.1 * c.1 * d3;
        return (param1 + param2) * s;
    }
    d2 *= 0.5;
    let (p1, p2) = (p.0 - c.0 * d1, p.1 - c.1 * d1);
    let param1 = p1 * p2 * d1 + (p1 * c.1 + p2 * c.0) * d2 + c.0 * c.1 * d3;
    -param1 * s
}

impl Function for BRepGPropUFunction {
    /// `Value(X, F)`: the integrand of the value type; false for GProp_Unknown and when
    /// the surface fails (the failure is kept).
    fn value(&mut self, x: f64, result: &mut f64) -> bool {
        let computed = match self.value_type {
            ValueType::Mass => self.volume_value(x).map(|(f, _, _, _)| Some(f)),
            ValueType::CenterMassX | ValueType::CenterMassY | ValueType::CenterMassZ => {
                self.center_mass_value(x)
            }
            ValueType::Unknown => Ok(None),
            _ => self.inertia_value(x),
        };
        match computed {
            Ok(Some(value)) => {
                *result = value;
                true
            }
            Ok(None) => false,
            Err(failure) => {
                self.failure = Some(failure);
                false
            }
        }
    }
}
