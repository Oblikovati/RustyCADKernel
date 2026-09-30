// Rust translation of ModelingAlgorithms/TKGeomAlgo/GeomFill/GeomFill_TgtOnCoons.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomFill_TgtOnCoons`: the cross tangents of a Coons patch along a boundary,
//! made normal to the boundary's normals.
use super::{GeomFillCoonsAlgPatch, GeomFillTgtField};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient};
use rustycad_tkg2d::geom2d::errors;
use rustycad_tkmath::gp::Vector3;

/// The field of cross tangents of a Coons patch along its boundary `I`
/// (`GeomFill_TgtOnCoons`): the derivative across the boundary (in V for the
/// boundaries 0 and 2, in U for 1 and 3) minus its component along the boundary's
/// normal.
///
/// `let field = GeomFillTgtOnCoons::new(&patch, 0); field.value(0.5)?`
pub struct GeomFillTgtOnCoons {
    parent: StandardTransient,
    my_k: Handle<GeomFillCoonsAlgPatch>,
    ibound: i32,
}
impl_transient!(
    GeomFillTgtOnCoons,
    "GeomFill_TgtOnCoons",
    dyn GeomFillTgtField,
    parent,
    interfaces: [dyn GeomFillTgtField]
);

/// The cross derivative at a boundary point and the fixed parameter it was taken at.
struct Cross {
    tangent: Vector3,
    u: f64,
    v: f64,
}

impl GeomFillTgtOnCoons {
    /// `GeomFill_TgtOnCoons(K, I)`: the field along boundary `i` of the shared patch
    /// `k`; `GeomFillTgtOnCoons::new(&patch, 1)`.
    pub fn new(k: &Handle<GeomFillCoonsAlgPatch>, i: i32) -> Self {
        Self {
            parent: StandardTransient::default(),
            my_k: k.clone(),
            ibound: i,
        }
    }
    fn patch(&self) -> Result<&GeomFillCoonsAlgPatch, Failure> {
        self.my_k
            .get()
            .ok_or_else(|| errors::null_object("GeomFill_TgtOnCoons: null patch"))
    }
    /// The first switch of the evaluators: the parameter of the boundary's side,
    /// read from the bounds of the adjacent boundary (0 outside the four sides).
    fn side_parameters(&self) -> Result<(f64, f64), Failure> {
        let k = self.patch()?;
        let bounds = |i: i32| -> Result<(f64, f64), Failure> { k.bound(i)?.bounds() };
        Ok(match self.ibound {
            0 => (0., bounds(1)?.0),
            1 => (bounds(0)?.1, 0.),
            2 => (0., bounds(1)?.1),
            3 => (bounds(0)?.0, 0.),
            _ => (0., 0.),
        })
    }
    /// The second switch: the cross derivative at `w` (null outside the four sides).
    fn cross(&self, w: f64) -> Result<Cross, Failure> {
        let k = self.patch()?;
        let (u, v) = self.side_parameters()?;
        Ok(match self.ibound {
            0 | 2 => Cross {
                tangent: k.d1v(w, v)?,
                u: w,
                v,
            },
            1 | 3 => Cross {
                tangent: k.d1u(u, w)?,
                u,
                v: w,
            },
            _ => Cross {
                tangent: Vector3::default(),
                u,
                v,
            },
        })
    }
    /// The body shared by the two `D1` overloads: the tangent `T` and its
    /// derivative. Quirk kept: `D1(W)` takes the second-order term from the
    /// corrected tangent, `D1(W, T, DT)` from the uncorrected one (it writes the
    /// corrected tangent to `T` and leaves its local unchanged).
    fn tangent_and_derivative(
        &self,
        w: f64,
        corrected_term: bool,
    ) -> Result<(Vector3, Vector3), Failure> {
        let k = self.patch()?;
        let Cross { tangent, u, v } = self.cross(w)?;
        let dtgsc: Vector3 = k.duv(u, v)?;
        let (n, dn) = k.bound(self.ibound)?.d1_norm(w)?;
        let scal: f64 = tangent.dot(n);
        let tgsc: Vector3 = tangent + n * -scal;
        let scaldn: Vector3 = dn * -scal;
        let second: Vector3 = if corrected_term { tgsc } else { tangent };
        let scal2: f64 = -dtgsc.dot(n) - second.dot(dn);
        let temp: Vector3 = n * scal2 + scaldn;
        Ok((tgsc, dtgsc + temp))
    }
}

impl GeomFillTgtField for GeomFillTgtOnCoons {
    /// The cross tangent minus its normal component.
    fn value(&self, w: f64) -> Result<Vector3, Failure> {
        let tgk: Vector3 = self.cross(w)?.tangent;
        let n: Vector3 = self.patch()?.bound(self.ibound)?.norm(w)?;
        let scal: f64 = tgk.dot(n);
        Ok(tgk + n * -scal)
    }
    fn d1(&self, w: f64) -> Result<Vector3, Failure> {
        Ok(self.tangent_and_derivative(w, true)?.1)
    }
    fn d1_with_value(&self, w: f64) -> Result<(Vector3, Vector3), Failure> {
        self.tangent_and_derivative(w, false)
    }
}
