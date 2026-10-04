// Rust translation of ModelingData/TKG3d/GeomAdaptor/GeomAdaptor_TransformedSurface.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomAdaptor_TransformedSurface`: a `GeomAdaptor_Surface` seen through a
//! transformation.
//!
//! Evaluations transform the results of the original adaptor; the kind queries
//! read an adaptor on the transformed Geom surface, built on first use and kept
//! until the surface or the transformation changes.
use super::surface::GeomAdaptorSurface;
use super::surface_state::LoadRange;
use crate::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use crate::geom::{
    GeomBSplineSurface, GeomBezierSurface, GeomConicalSurface, GeomCylindricalSurface, GeomPlane,
    GeomSphericalSurface, GeomSurface, GeomSurfaceD1, GeomSurfaceD2, GeomSurfaceD3,
    GeomToroidalSurface,
};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkmath::geom_abs::{Shape, SurfaceType};
use rustycad_tkmath::gp::{
    Axis1, Cone3, Cylinder3, Direction3, Plane3, Point3, Sphere3, Torus3, Transform3,
    TransformForm, Vector3,
};

#[derive(Default)]
struct TransformedState {
    trsf: Transform3,
    // myTransformedAdaptor: None until built, reset whenever the input changes.
    transformed: Option<Handle<GeomAdaptorSurface>>,
}

/// A surface adaptor with a transformation (`GeomAdaptor_TransformedSurface`);
/// `GeomAdaptorTransformedSurface::with_surface(&surface, trsf)?`.
pub struct GeomAdaptorTransformedSurface {
    parent: StandardTransient,
    surf: Handle<GeomAdaptorSurface>,
    state: SharedState<TransformedState>,
}
impl Default for GeomAdaptorTransformedSurface {
    fn default() -> Self {
        Self::from_parts(
            Handle::new(GeomAdaptorSurface::default()),
            TransformedState::default(),
        )
    }
}
impl_transient!(
    GeomAdaptorTransformedSurface,
    "GeomAdaptor_TransformedSurface",
    dyn Adaptor3dSurface,
    parent,
    interfaces: [dyn Adaptor3dSurface]
);

impl GeomAdaptorTransformedSurface {
    fn from_parts(surf: Handle<GeomAdaptorSurface>, state: TransformedState) -> Self {
        Self {
            parent: StandardTransient::default(),
            surf,
            state: SharedState::new(state),
        }
    }
    /// An adaptor on `surface` over its bounds, with `trsf`
    /// (`GeomAdaptor_TransformedSurface(S, Trsf)`); `with_surface(&s, trsf)?`.
    pub fn with_surface(
        surface: &Handle<dyn GeomSurface>,
        trsf: Transform3,
    ) -> Result<Self, Failure> {
        let adaptor = Self::default();
        adaptor.load(surface, trsf)?;
        Ok(adaptor)
    }
    /// An adaptor on `surface` over `((U1, U2, V1, V2), (TolU, TolV))`, with `trsf`
    /// (`GeomAdaptor_TransformedSurface(S, U1, U2, V1, V2, Trsf, TolU, TolV)`);
    /// `with_range(&s, ((0.0, 1.0, 0.0, 1.0), (0.0, 0.0)), trsf)?`.
    pub fn with_range(
        surface: &Handle<dyn GeomSurface>,
        range: LoadRange,
        trsf: Transform3,
    ) -> Result<Self, Failure> {
        let adaptor = Self::default();
        adaptor.load_range(surface, range, trsf)?;
        Ok(adaptor)
    }
    /// Load `surface` over its bounds with `trsf` (`Load(S, Trsf)`); `adaptor.load(&s, trsf)?`.
    pub fn load(&self, surface: &Handle<dyn GeomSurface>, trsf: Transform3) -> Result<(), Failure> {
        self.surf.load(surface)?;
        self.set_trsf(trsf);
        Ok(())
    }
    /// Load `surface` over a range with `trsf` (`Load(S, U1, U2, V1, V2, Trsf, TolU, TolV)`);
    /// `adaptor.load_range(&s, ((0.0, 1.0, 0.0, 1.0), (0.0, 0.0)), trsf)?`.
    pub fn load_range(
        &self,
        surface: &Handle<dyn GeomSurface>,
        range: LoadRange,
        trsf: Transform3,
    ) -> Result<(), Failure> {
        self.surf.load_range(surface, range)?;
        self.set_trsf(trsf);
        Ok(())
    }
    /// Replace the transformation and drop the transformed adaptor (`SetTrsf`);
    /// `adaptor.set_trsf(trsf)`.
    pub fn set_trsf(&self, trsf: Transform3) {
        let mut state = self.state.write();
        state.trsf = trsf;
        state.transformed = None;
    }
    /// Whether the transformation is not the identity (`HasTrsf`); `adaptor.has_trsf()`.
    pub fn has_trsf(&self) -> bool {
        self.trsf().form() != TransformForm::Identity
    }
    /// The transformation (`Trsf`); `adaptor.trsf()`.
    pub fn trsf(&self) -> Transform3 {
        self.state.read().trsf
    }
    /// The untransformed adaptor (`Surface`, deprecated natively); `adaptor.surface()`.
    pub fn surface(&self) -> Handle<GeomAdaptorSurface> {
        self.adaptor_surface_original()
    }
    /// The untransformed adaptor (`AdaptorSurfaceOriginal`); `adaptor.adaptor_surface_original()`.
    pub fn adaptor_surface_original(&self) -> Handle<GeomAdaptorSurface> {
        self.surf.clone()
    }
    /// The adaptor on the transformed surface, the original one for an identity
    /// (`AdaptorSurfaceTransformed`); `adaptor.adaptor_surface_transformed()?`.
    pub fn adaptor_surface_transformed(&self) -> Result<Handle<GeomAdaptorSurface>, Failure> {
        self.transformed_adaptor()
    }
    /// The untransformed Geom surface (`GeomSurfaceOriginal`); `adaptor.geom_surface_original()`.
    pub fn geom_surface_original(&self) -> Handle<dyn GeomSurface> {
        self.surf.surface()
    }
    /// The untransformed Geom surface (`GeomSurface`, deprecated natively); `adaptor.geom_surface()`.
    pub fn geom_surface(&self) -> Handle<dyn GeomSurface> {
        self.geom_surface_original()
    }
    /// The transformed Geom surface, the original one for an identity
    /// (`GeomSurfaceTransformed`); `adaptor.geom_surface_transformed()?`.
    pub fn geom_surface_transformed(&self) -> Result<Handle<dyn GeomSurface>, Failure> {
        Ok(self.transformed_adaptor()?.surface())
    }
    /// A copy with a shallow copy of the original adaptor and the transformed adaptor
    /// shared (`ShallowCopy`, as a value, which the derived BRepAdaptor_Surface embeds);
    /// `adaptor.shallow_adaptor()?`.
    pub fn shallow_adaptor(&self) -> Result<Self, Failure> {
        let surf: Handle<GeomAdaptorSurface> = Handle::new(self.surf.shallow_adaptor()?);
        let state = {
            let source = self.state.read();
            TransformedState {
                trsf: source.trsf,
                transformed: source
                    .transformed
                    .as_ref()
                    .map(|adaptor| Handle::new(adaptor.copy_value())),
            }
        };
        Ok(Self::from_parts(surf, state))
    }
    /// The U tolerance of the original adaptor (`ToleranceU`); `adaptor.tolerance_u()`.
    pub fn tolerance_u(&self) -> f64 {
        self.surf.tolerance_u()
    }
    /// The V tolerance of the original adaptor (`ToleranceV`); `adaptor.tolerance_v()`.
    pub fn tolerance_v(&self) -> f64 {
        self.surf.tolerance_v()
    }
    /// Invalidates the protected transformed cache; `adaptor.invalidate_transformed_cache()`.
    pub fn invalidate_transformed_cache(&self) {
        self.state.write().transformed = None;
    }
    /// Builds a missing nonidentity cache; `adaptor.ensure_transformed_cache()?`.
    pub fn ensure_transformed_cache(&self) -> Result<(), Failure> {
        self.ensure_cached_state(&mut self.state.write())
    }
    /// Rebuilds the protected cache even for identity transforms; `adaptor.init_transformed_cache()?`.
    pub fn init_transformed_cache(&self) -> Result<(), Failure> {
        self.initialize_cached_state(&mut self.state.write())
    }
    /// Inspects protected cached surface state without initializing it; `adaptor.cached_transformed_surface()`.
    /// `None` means absent; `Some(null)` means an initialized empty adaptor.
    pub fn cached_transformed_surface(&self) -> Option<Handle<dyn GeomSurface>> {
        self.state
            .read()
            .transformed
            .as_ref()
            .map(|adaptor| adaptor.surface())
    }
    fn ensure_cached_state(&self, state: &mut TransformedState) -> Result<(), Failure> {
        if state.trsf.form() == TransformForm::Identity || state.transformed.is_some() {
            return Ok(());
        }
        self.initialize_cached_state(state)
    }
    fn initialize_cached_state(&self, state: &mut TransformedState) -> Result<(), Failure> {
        state.transformed = Some(Handle::new(self.build_transformed(state.trsf)?));
        Ok(())
    }
    /// The protected adaptor accessor: original for identity, otherwise lazily cached;
    /// `adaptor.transformed_adaptor()?`.
    pub fn transformed_adaptor(&self) -> Result<Handle<GeomAdaptorSurface>, Failure> {
        let mut state = self.state.write();
        if state.trsf.form() == TransformForm::Identity {
            return Ok(self.surf.clone());
        }
        self.ensure_cached_state(&mut state)?;
        Ok(state
            .transformed
            .as_ref()
            .expect("initialized nonidentity cache")
            .clone())
    }
    // initTransformedCache: elementary kinds are rebuilt from their transformed gp
    // geometry, the others are transformed copies; bounds and tolerances are kept.
    fn build_transformed(&self, trsf: Transform3) -> Result<GeomAdaptorSurface, Failure> {
        let original: Handle<dyn GeomSurface> = self.surf.surface();
        if original.is_null() {
            return Ok(GeomAdaptorSurface::default());
        }
        let surface = self.transformed_geometry(original, trsf)?;
        let range: LoadRange = (
            self.surf.bounds()?,
            (self.tolerance_u(), self.tolerance_v()),
        );
        GeomAdaptorSurface::with_range(&surface, range)
    }
    fn transformed_geometry(
        &self,
        original: Handle<dyn GeomSurface>,
        trsf: Transform3,
    ) -> Result<Handle<dyn GeomSurface>, Failure> {
        if trsf.form() == TransformForm::Identity {
            return Ok(original);
        }
        Ok(match self.surf.get_type()? {
            SurfaceType::Plane => {
                Handle::new(GeomPlane::from_pln(self.surf.plane()?.transformed(trsf))).cast()
            }
            SurfaceType::Cylinder => Handle::new(GeomCylindricalSurface::from_cylinder(
                self.surf.cylinder()?.transformed(trsf),
            ))
            .cast(),
            _ => self.transformed_curved_geometry(original, trsf)?,
        })
    }
    fn transformed_curved_geometry(
        &self,
        original: Handle<dyn GeomSurface>,
        trsf: Transform3,
    ) -> Result<Handle<dyn GeomSurface>, Failure> {
        Ok(match self.surf.get_type()? {
            SurfaceType::Cone => Handle::new(GeomConicalSurface::from_cone(
                self.surf.cone()?.transformed(trsf),
            ))
            .cast(),
            SurfaceType::Sphere => Handle::new(GeomSphericalSurface::from_sphere(
                self.surf.sphere()?.transformed(trsf),
            ))
            .cast(),
            SurfaceType::Torus => Handle::new(GeomToroidalSurface::from_torus(
                self.surf.torus()?.transformed(trsf),
            ))
            .cast(),
            _ => original.transformed(trsf)?.cast(),
        })
    }
    // Apply the transformation to an evaluation unless it is the identity.
    fn transformed_by<T>(&self, value: T, apply: impl Fn(T, Transform3) -> T) -> T {
        let trsf: Transform3 = self.trsf();
        if trsf.form() == TransformForm::Identity {
            return value;
        }
        apply(value, trsf)
    }
}

fn d1_transformed(mut result: GeomSurfaceD1, trsf: Transform3) -> GeomSurfaceD1 {
    result.point = result.point.transformed(trsf);
    result.d1u = result.d1u.transformed(trsf);
    result.d1v = result.d1v.transformed(trsf);
    result
}
fn d2_transformed(mut result: GeomSurfaceD2, trsf: Transform3) -> GeomSurfaceD2 {
    result.point = result.point.transformed(trsf);
    for vector in [
        &mut result.d1u,
        &mut result.d1v,
        &mut result.d2u,
        &mut result.d2v,
        &mut result.d2uv,
    ] {
        *vector = vector.transformed(trsf);
    }
    result
}
fn d3_transformed(mut result: GeomSurfaceD3, trsf: Transform3) -> GeomSurfaceD3 {
    result.point = result.point.transformed(trsf);
    let vectors = [
        &mut result.d1u,
        &mut result.d1v,
        &mut result.d2u,
        &mut result.d2v,
        &mut result.d2uv,
        &mut result.d3u,
        &mut result.d3v,
        &mut result.d3uuv,
        &mut result.d3uvv,
    ];
    for vector in vectors {
        *vector = vector.transformed(trsf);
    }
    result
}

impl Adaptor3dSurface for GeomAdaptorTransformedSurface {
    fn shallow_copy(&self) -> Result<Handle<dyn Adaptor3dSurface>, Failure> {
        Ok(Handle::new(self.shallow_adaptor()?).cast())
    }
    fn first_u_parameter(&self) -> Result<f64, Failure> {
        self.surf.first_u_parameter()
    }
    fn last_u_parameter(&self) -> Result<f64, Failure> {
        self.surf.last_u_parameter()
    }
    fn first_v_parameter(&self) -> Result<f64, Failure> {
        self.surf.first_v_parameter()
    }
    fn last_v_parameter(&self) -> Result<f64, Failure> {
        self.surf.last_v_parameter()
    }
    fn u_continuity(&self) -> Result<Shape, Failure> {
        self.surf.u_continuity()
    }
    fn v_continuity(&self) -> Result<Shape, Failure> {
        self.surf.v_continuity()
    }
    fn nb_u_intervals(&self, shape: Shape) -> Result<i32, Failure> {
        self.surf.nb_u_intervals(shape)
    }
    fn nb_v_intervals(&self, shape: Shape) -> Result<i32, Failure> {
        self.surf.nb_v_intervals(shape)
    }
    fn u_intervals(&self, bounds: &mut Array1<'_, f64>, shape: Shape) -> Result<(), Failure> {
        self.surf.u_intervals(bounds, shape)
    }
    fn v_intervals(&self, bounds: &mut Array1<'_, f64>, shape: Shape) -> Result<(), Failure> {
        self.surf.v_intervals(bounds, shape)
    }
    fn u_trim(
        &self,
        first: f64,
        last: f64,
        tolerance: f64,
    ) -> Result<Handle<dyn Adaptor3dSurface>, Failure> {
        self.transformed_adaptor()?.u_trim(first, last, tolerance)
    }
    fn v_trim(
        &self,
        first: f64,
        last: f64,
        tolerance: f64,
    ) -> Result<Handle<dyn Adaptor3dSurface>, Failure> {
        self.transformed_adaptor()?.v_trim(first, last, tolerance)
    }
    fn is_u_closed(&self) -> Result<bool, Failure> {
        self.surf.is_u_closed()
    }
    fn is_v_closed(&self) -> Result<bool, Failure> {
        self.surf.is_v_closed()
    }
    fn is_u_periodic(&self) -> Result<bool, Failure> {
        self.surf.is_u_periodic()
    }
    fn u_period(&self) -> Result<f64, Failure> {
        self.surf.u_period()
    }
    fn is_v_periodic(&self) -> Result<bool, Failure> {
        self.surf.is_v_periodic()
    }
    fn v_period(&self) -> Result<f64, Failure> {
        self.surf.v_period()
    }
    fn u_resolution(&self, r3d: f64) -> Result<f64, Failure> {
        self.surf.u_resolution(r3d)
    }
    fn v_resolution(&self, r3d: f64) -> Result<f64, Failure> {
        self.surf.v_resolution(r3d)
    }
    fn get_type(&self) -> Result<SurfaceType, Failure> {
        self.surf.get_type()
    }
    fn plane(&self) -> Result<Plane3, Failure> {
        self.transformed_adaptor()?.plane()
    }
    fn cylinder(&self) -> Result<Cylinder3, Failure> {
        self.transformed_adaptor()?.cylinder()
    }
    fn cone(&self) -> Result<Cone3, Failure> {
        self.transformed_adaptor()?.cone()
    }
    fn sphere(&self) -> Result<Sphere3, Failure> {
        self.transformed_adaptor()?.sphere()
    }
    fn torus(&self) -> Result<Torus3, Failure> {
        self.transformed_adaptor()?.torus()
    }
    fn u_degree(&self) -> Result<i32, Failure> {
        self.surf.u_degree()
    }
    fn nb_u_poles(&self) -> Result<i32, Failure> {
        self.surf.nb_u_poles()
    }
    fn v_degree(&self) -> Result<i32, Failure> {
        self.surf.v_degree()
    }
    fn nb_v_poles(&self) -> Result<i32, Failure> {
        self.surf.nb_v_poles()
    }
    fn nb_u_knots(&self) -> Result<i32, Failure> {
        self.surf.nb_u_knots()
    }
    fn nb_v_knots(&self) -> Result<i32, Failure> {
        self.surf.nb_v_knots()
    }
    fn is_u_rational(&self) -> Result<bool, Failure> {
        self.surf.is_u_rational()
    }
    fn is_v_rational(&self) -> Result<bool, Failure> {
        self.surf.is_v_rational()
    }
    fn bezier(&self) -> Result<Handle<GeomBezierSurface>, Failure> {
        self.transformed_adaptor()?.bezier()
    }
    fn bspline(&self) -> Result<Handle<GeomBSplineSurface>, Failure> {
        self.transformed_adaptor()?.bspline()
    }
    fn axe_of_revolution(&self) -> Result<Axis1, Failure> {
        self.transformed_adaptor()?.axe_of_revolution()
    }
    fn direction(&self) -> Result<Direction3, Failure> {
        self.transformed_adaptor()?.direction()
    }
    fn basis_curve(&self) -> Result<Handle<dyn Adaptor3dCurve>, Failure> {
        self.transformed_adaptor()?.basis_curve()
    }
    fn basis_surface(&self) -> Result<Handle<dyn Adaptor3dSurface>, Failure> {
        self.transformed_adaptor()?.basis_surface()
    }
    fn offset_value(&self) -> Result<f64, Failure> {
        self.transformed_adaptor()?.offset_value()
    }
    fn eval_d0(&self, u: f64, v: f64) -> Result<Point3, Failure> {
        Ok(self.transformed_by(self.surf.eval_d0(u, v)?, Point3::transformed))
    }
    fn eval_d1(&self, u: f64, v: f64) -> Result<GeomSurfaceD1, Failure> {
        Ok(self.transformed_by(self.surf.eval_d1(u, v)?, d1_transformed))
    }
    fn eval_d2(&self, u: f64, v: f64) -> Result<GeomSurfaceD2, Failure> {
        Ok(self.transformed_by(self.surf.eval_d2(u, v)?, d2_transformed))
    }
    fn eval_d3(&self, u: f64, v: f64) -> Result<GeomSurfaceD3, Failure> {
        Ok(self.transformed_by(self.surf.eval_d3(u, v)?, d3_transformed))
    }
    fn eval_dn(&self, u: f64, v: f64, nu: i32, nv: i32) -> Result<Vector3, Failure> {
        Ok(self.transformed_by(self.surf.eval_dn(u, v, nu, nv)?, Vector3::transformed))
    }
}
