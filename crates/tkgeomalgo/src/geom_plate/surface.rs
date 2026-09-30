// Rust translation of ModelingAlgorithms/TKGeomAlgo/GeomPlate/GeomPlate_Surface.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomPlate_Surface`: the initial surface deformed by a solved plate.
use super::live;
use crate::plate::PlatePlate;
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::ncollection::Sequence;
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkernel::standard::{Failure, Handle, RuntimeTyped, SharedState, StandardTransient};
use rustycad_tkg3d::geom::errors::undefined_derivative;
use rustycad_tkg3d::geom::{
    GeomCurve, GeomGeometry, GeomSurface, GeomSurfaceD1, GeomSurfaceD2, GeomSurfaceD3,
    SurfaceBounds, dump_surface,
};
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::{GeneralTransform2, Point3, Transform3, Vector3, Xy, Xyz};
use std::io;

/// The surface `S(u, v) = S0(u, v) + Plate(u, v)` of an initial surface `S0` and a
/// solved plate (`GeomPlate_Surface`); the result of `GeomPlate_BuildPlateSurface`.
///
/// It is a `Geom_Surface` (docs/geomalgo-design.md section 19) whose geometric edits
/// do nothing, as natively: reversing, transforming and the iso curves are empty
/// members, `Copy` keeps the initial surface and the plate but not the bounds.
/// `let s = GeomPlateSurface::new(&plane, &plate); s.set_bounds(0.0, 1.0, 0.0, 1.0)?;`
pub struct GeomPlateSurface {
    parent: StandardTransient,
    surfinter: PlatePlate,
    surfinit: Handle<dyn GeomSurface>,
    bounds: SharedState<SurfaceBounds>,
}
impl_transient!(
    GeomPlateSurface,
    "GeomPlate_Surface",
    dyn GeomSurface,
    parent,
    interfaces: [dyn GeomGeometry, dyn GeomSurface]
);

impl GeomPlateSurface {
    /// The initial surface `surfinit` deformed by a copy of `surfinter`, with null
    /// bounds (`GeomPlate_Surface(Surfinit, Surfinter)`);
    /// `GeomPlateSurface::new(&plane, &plate)`.
    pub fn new(surfinit: &Handle<dyn GeomSurface>, surfinter: &PlatePlate) -> Self {
        Self {
            parent: StandardTransient::default(),
            surfinter: surfinter.clone(),
            surfinit: surfinit.clone(),
            bounds: SharedState::new((0.0, 0.0, 0.0, 0.0)),
        }
    }

    /// The initial surface (`CallSurfinit`); `s.call_surfinit()`.
    pub fn call_surfinit(&self) -> Handle<dyn GeomSurface> {
        self.surfinit.clone()
    }

    /// Sets the bounds that `Bounds` returns (`SetBounds`); Standard_Failure for
    /// decreasing or equal bounds. `s.set_bounds(0.0, 1.0, 0.0, 1.0)?`.
    pub fn set_bounds(&self, umin: f64, umax: f64, vmin: f64, vmax: f64) -> Result<(), Failure> {
        if umin > umax || vmin > vmax {
            return Err(Failure::new(&format!(
                "Bounds haven't the good sense: [{umin}, {umax}] x [{vmin}, {vmax}]; expected increasing bounds"
            )));
        }
        if umin == umax || vmin == vmax {
            return Err(Failure::new(&format!(
                "Bounds are equal: [{umin}, {umax}] x [{vmin}, {vmax}]; expected distinct bounds"
            )));
        }
        *self.bounds.write() = (umin, umax, vmin, vmax);
        Ok(())
    }

    /// The parameter box of the plate constraints (`RealBounds`, `Plate_Plate::UVBox`);
    /// Standard_OutOfRange for a plate without constraint. `let (u1, u2, v1, v2) = s.real_bounds()?;`.
    pub fn real_bounds(&self) -> Result<SurfaceBounds, Failure> {
        self.surfinter.uv_box()
    }

    /// Appends the parameters of the order 0 plate constraints to `seq`
    /// (`Constraints`, `Plate_Plate::UVConstraints`); `s.constraints(&mut seq)`.
    pub fn constraints(&self, seq: &mut Sequence<Xy>) {
        self.surfinter.uv_constraints(seq);
    }

    fn initial(&self) -> Result<&dyn GeomSurface, Failure> {
        live(&self.surfinit, "the initial surface")
    }
}

impl GeomGeometry for GeomPlateSurface {
    /// Does nothing, as natively (`Transform`).
    fn transform(&self, _transform: Transform3) -> Result<(), Failure> {
        Ok(())
    }
    /// A plate surface on the same initial surface and plate, with null bounds
    /// (`Copy`).
    fn copy(&self) -> Result<Handle<dyn GeomGeometry>, Failure> {
        Ok(Handle::new(Self::new(&self.surfinit, &self.surfinter)).cast())
    }
    /// `Geom_Surface::DumpJson`, which the class does not override.
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        dump_surface(stream, depth)
    }
}

impl GeomSurface for GeomPlateSurface {
    /// Does nothing, as natively (`UReverse`).
    fn u_reverse(&self) -> Result<(), Failure> {
        Ok(())
    }
    fn u_reversed_parameter(&self, u: f64) -> Result<f64, Failure> {
        Ok(-u)
    }
    /// Does nothing, as natively (`VReverse`).
    fn v_reverse(&self) -> Result<(), Failure> {
        Ok(())
    }
    fn v_reversed_parameter(&self, v: f64) -> Result<f64, Failure> {
        Ok(-v)
    }
    /// The parameters unchanged, as natively (`TransformParameters`).
    fn transform_parameters(&self, u: f64, v: f64, _transform: Transform3) -> (f64, f64) {
        (u, v)
    }
    /// The identity, as natively (`ParametricTransformation`).
    fn parametric_transformation(
        &self,
        _transform: Transform3,
    ) -> Result<GeneralTransform2, Failure> {
        Ok(GeneralTransform2::default())
    }
    /// The bounds of the initial surface when it is a plate surface itself, else the
    /// bounds set by `SetBounds` (`Bounds`). A null initial surface (a null
    /// dereference natively) gives the set bounds.
    fn bounds(&self) -> SurfaceBounds {
        match self.surfinit.get() {
            Some(init) if self.surfinit.is_instance(Self::static_type()) => init.bounds(),
            _ => *self.bounds.read(),
        }
    }
    fn is_u_closed(&self) -> Result<bool, Failure> {
        Ok(false)
    }
    fn is_v_closed(&self) -> Result<bool, Failure> {
        Ok(false)
    }
    fn is_u_periodic(&self) -> bool {
        false
    }
    fn is_v_periodic(&self) -> bool {
        false
    }
    /// 0: the native member returns `false` converted to a real instead of raising.
    fn u_period(&self) -> Result<f64, Failure> {
        Ok(0.0)
    }
    /// 0, as `u_period`.
    fn v_period(&self) -> Result<f64, Failure> {
        Ok(0.0)
    }
    /// A null curve (`UIso`).
    fn u_iso(&self, _u: f64) -> Result<Handle<dyn GeomCurve>, Failure> {
        Ok(Handle::default())
    }
    /// A null curve (`VIso`).
    fn v_iso(&self, _v: f64) -> Result<Handle<dyn GeomCurve>, Failure> {
        Ok(Handle::default())
    }
    /// `GeomAbs_Shape()`, that is C0 (`Continuity`).
    fn continuity(&self) -> Shape {
        Shape::C0
    }
    /// Standard_Failure, as natively (`IsCNu`).
    fn is_cn_u(&self, _n: i32) -> Result<bool, Failure> {
        Err(Failure::new("IsCNu"))
    }
    /// Standard_Failure, as natively (`IsCNv`).
    fn is_cn_v(&self, _n: i32) -> Result<bool, Failure> {
        Err(Failure::new("IsCNv"))
    }
    fn eval_d0(&self, u: f64, v: f64) -> Result<Point3, Failure> {
        let surf_p: Point3 = self.initial()?.eval_d0(u, v)?;
        let p3: Xyz = self.surfinter.evaluate(Xy::new(u, v))?;
        Ok(Point3::from(p3 + surf_p.xyz()))
    }
    fn eval_d1(&self, u: f64, v: f64) -> Result<GeomSurfaceD1, Failure> {
        let surf_d1: GeomSurfaceD1 = self.initial()?.eval_d1(u, v)?;
        let p1 = Xy::new(u, v);
        let p3: Xyz = self.surfinter.evaluate(p1)?;
        let v2u: Xyz = self.surfinter.evaluate_derivative(p1, 1, 0)?;
        let v2v: Xyz = self.surfinter.evaluate_derivative(p1, 0, 1)?;
        Ok(GeomSurfaceD1 {
            point: Point3::from(p3 + surf_d1.point.xyz()),
            d1u: Vector3::from(*surf_d1.d1u.xyz() + v2u),
            d1v: Vector3::from(*surf_d1.d1v.xyz() + v2v),
        })
    }
    fn eval_d2(&self, u: f64, v: f64) -> Result<GeomSurfaceD2, Failure> {
        let s: GeomSurfaceD2 = self.initial()?.eval_d2(u, v)?;
        let p1 = Xy::new(u, v);
        let plate = &self.surfinter;
        let p3: Xyz = plate.evaluate(p1)?;
        let (du, dv) = (
            plate.evaluate_derivative(p1, 1, 0)?,
            plate.evaluate_derivative(p1, 0, 1)?,
        );
        let (duu, dvv) = (
            plate.evaluate_derivative(p1, 2, 0)?,
            plate.evaluate_derivative(p1, 0, 2)?,
        );
        let duv: Xyz = plate.evaluate_derivative(p1, 1, 1)?;
        Ok(GeomSurfaceD2 {
            point: Point3::from(p3 + s.point.xyz()),
            d1u: Vector3::from(*s.d1u.xyz() + du),
            d1v: Vector3::from(*s.d1v.xyz() + dv),
            d2u: Vector3::from(*s.d2u.xyz() + duu),
            d2v: Vector3::from(*s.d2v.xyz() + dvv),
            d2uv: Vector3::from(*s.d2uv.xyz() + duv),
        })
    }
    /// Geom_UndefinedDerivative, as natively (`EvalD3`).
    fn eval_d3(&self, _u: f64, _v: f64) -> Result<GeomSurfaceD3, Failure> {
        Err(undefined_derivative("GeomPlate_Surface::EvalD3"))
    }
    /// Geom_UndefinedDerivative, as natively (`EvalDN`).
    fn eval_dn(&self, _u: f64, _v: f64, _nu: i32, _nv: i32) -> Result<Vector3, Failure> {
        Err(undefined_derivative("GeomPlate_Surface::EvalDN"))
    }
}
