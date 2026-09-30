// Rust translation of ModelingAlgorithms/TKGeomAlgo/GeomFill/GeomFill_ConstantBiNormal.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomFill_ConstantBiNormal`: a frame whose binormal is a fixed direction, the
//! tangent being the curve tangent projected normal to it.
use super::frame_math::{dderiv, fderiv};
use super::frenet::GeomFillFrenet;
use super::sweep_failures::live;
use super::trihedron_law::{
    GeomFillTrihedronLaw, GeomFillTrihedronLawBase, TrihedronD0, TrihedronD1, TrihedronD2,
};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient};
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkmath::geom_abs::{CurveType, Shape};
use rustycad_tkmath::gp::{Axis1, Direction3, Vector3};

/// `GeomFill_ConstantBiNormal`: the Frenet frame re-orthogonalized on a constant
/// binormal. `let law = GeomFillConstantBiNormal::new(dir); law.set_curve(&curve)?;`.
pub struct GeomFillConstantBiNormal {
    parent: StandardTransient,
    base: GeomFillTrihedronLawBase,
    bn: Vector3,
    frenet: Handle<GeomFillFrenet>,
}
impl_transient!(
    GeomFillConstantBiNormal,
    "GeomFill_ConstantBiNormal",
    dyn GeomFillTrihedronLaw,
    parent,
    interfaces: [dyn GeomFillTrihedronLaw]
);

impl GeomFillConstantBiNormal {
    /// `GeomFill_ConstantBiNormal(BiNormal)`; `GeomFillConstantBiNormal::new(dir)`.
    pub fn new(binormal: Direction3) -> Self {
        Self {
            parent: StandardTransient::default(),
            base: GeomFillTrihedronLawBase::default(),
            bn: Vector3::from(binormal),
            frenet: Handle::new(GeomFillFrenet::new()),
        }
    }

    fn frenet(&self) -> &GeomFillFrenet {
        self.frenet
            .get()
            .expect("the constructor creates the Frenet law")
    }
}

/// The D0 re-orthogonalization shared with `GetAverageLaw`: the normal from
/// `B ^ T` when it is not null, else the tangent from `N ^ B`.
fn orthogonalize(frame: &mut TrihedronD0, bn: Vector3) -> Result<(), Failure> {
    frame.binormal = bn;
    if frame.binormal.crossed(frame.tangent).magnitude() > precision::confusion() {
        frame.normal = frame.binormal.crossed(frame.tangent).normalized()?;
        frame.tangent = frame.normal.crossed(frame.binormal);
    } else {
        frame.tangent = frame.normal.crossed(frame.binormal).normalized()?;
        frame.normal = frame.binormal.crossed(frame.tangent);
    }
    Ok(())
}

/// The axis of a conic, or `None` for other kinds (and the line case apart).
fn conic_axis(curve: &dyn Adaptor3dCurve) -> Result<Option<Axis1>, Failure> {
    Ok(match curve.get_type()? {
        CurveType::Circle => Some(curve.circle()?.axis()),
        CurveType::Ellipse => Some(curve.ellipse()?.axis()),
        CurveType::Hyperbola => Some(curve.hyperbola()?.axis()),
        CurveType::Parabola => Some(curve.parabola()?.axis()),
        _ => None,
    })
}

impl GeomFillTrihedronLaw for GeomFillConstantBiNormal {
    fn trihedron_base(&self) -> &GeomFillTrihedronLawBase {
        &self.base
    }
    fn copy(&self) -> Result<Handle<dyn GeomFillTrihedronLaw>, Failure> {
        let copy = GeomFillConstantBiNormal::new(Direction3::from_vector(self.bn)?);
        let curve = self.base.curve();
        if curve.is_some() {
            copy.set_curve(&curve)?;
        }
        Ok(Handle::new(copy).cast())
    }
    /// Loads the curve and the Frenet law; false for a null curve.
    fn set_curve(&self, curve: &Handle<dyn Adaptor3dCurve>) -> Result<bool, Failure> {
        self.base.set_curve(curve);
        if curve.is_null() {
            return Ok(false);
        }
        self.frenet().set_curve(curve)
    }
    fn d0(&self, param: f64, frame: &mut TrihedronD0) -> Result<bool, Failure> {
        self.frenet().d0(param, frame)?;
        orthogonalize(frame, self.bn)?;
        Ok(true)
    }
    fn d1(&self, param: f64, frame: &mut TrihedronD1) -> Result<bool, Failure> {
        self.frenet().d1(param, frame)?;
        frame.binormal = self.bn;
        frame.d_binormal = Vector3::new(0.0, 0.0, 0.0);
        if frame.binormal.crossed(frame.tangent).magnitude() > precision::confusion() {
            let f = frame.binormal.crossed(frame.tangent);
            let df = frame.binormal.crossed(frame.d_tangent);
            frame.normal = f.normalized()?;
            frame.d_normal = fderiv(f, df);
            frame.tangent = frame.normal.crossed(frame.binormal);
            frame.d_tangent = frame.d_normal.crossed(frame.binormal);
        } else {
            let f = frame.normal.crossed(frame.binormal);
            let df = frame.d_normal.crossed(frame.binormal);
            frame.tangent = f.normalized()?;
            frame.d_tangent = fderiv(f, df);
            frame.normal = frame.binormal.crossed(frame.tangent);
            frame.d_normal = frame.binormal.crossed(frame.d_tangent);
        }
        Ok(true)
    }
    fn d2(&self, param: f64, frame: &mut TrihedronD2) -> Result<bool, Failure> {
        self.frenet().d2(param, frame)?;
        frame.binormal = self.bn;
        frame.d_binormal = Vector3::new(0.0, 0.0, 0.0);
        frame.d2_binormal = Vector3::new(0.0, 0.0, 0.0);
        let b = frame.binormal;
        if b.crossed(frame.tangent).magnitude() > precision::confusion() {
            let (f, df, d2f) = (
                b.crossed(frame.tangent),
                b.crossed(frame.d_tangent),
                b.crossed(frame.d2_tangent),
            );
            frame.normal = f.normalized()?;
            frame.d_normal = fderiv(f, df);
            frame.d2_normal = dderiv(f, df, d2f);
            frame.tangent = frame.normal.crossed(b);
            frame.d_tangent = frame.d_normal.crossed(b);
            frame.d2_tangent = frame.d2_normal.crossed(b);
        } else {
            let (f, df, d2f) = (
                frame.normal.crossed(b),
                frame.d_normal.crossed(b),
                frame.d2_normal.crossed(b),
            );
            frame.tangent = f.normalized()?;
            frame.d_tangent = fderiv(f, df);
            frame.d2_tangent = dderiv(f, df, d2f);
            frame.normal = b.crossed(frame.tangent);
            frame.d_normal = b.crossed(frame.d_tangent);
            frame.d2_normal = b.crossed(frame.d2_tangent);
        }
        Ok(true)
    }
    fn nb_intervals(&self, shape: Shape) -> Result<i32, Failure> {
        self.frenet().nb_intervals(shape)
    }
    fn intervals(&self, bounds: &mut Array1<'_, f64>, shape: Shape) -> Result<(), Failure> {
        self.frenet().intervals(bounds, shape)
    }
    fn get_average_law(&self) -> Result<TrihedronD0, Failure> {
        let mut frame = self.frenet().get_average_law()?;
        orthogonalize(&mut frame, self.bn)?;
        Ok(frame)
    }
    fn is_constant(&self) -> Result<bool, Failure> {
        self.frenet().is_constant()
    }
    /// Whether the binormal is the axis of a conic path (or normal to a line path).
    fn is_only_by_3d_curve(&self) -> Result<bool, Failure> {
        let curve = self.base.curve();
        let c = live(
            &curve,
            "GeomFill_ConstantBiNormal::IsOnlyBy3dCurve",
            "myCurve",
        )?;
        if c.get_type()? == CurveType::Line {
            let v = Vector3::from(c.line()?.direction());
            return v.is_normal(self.bn, precision::angular());
        }
        match conic_axis(c)? {
            Some(axis) => {
                Vector3::from(axis.direction()).is_parallel(self.bn, precision::angular())
            }
            None => Ok(false),
        }
    }
}
