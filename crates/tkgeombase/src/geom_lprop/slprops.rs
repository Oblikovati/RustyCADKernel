// Rust translation of ModelingData/TKGeomBase/GeomLProp/GeomLProp_SLProps.hxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomLProp_SLPropsBase`: point, derivatives, tangents, normal and curvatures
//! of a surface at one parameter pair.
use super::surface_access::{DirectAccess, SurfaceAccess};
use super::surface_utils::{
    self, SurfaceCurvatures, SurfaceJet, SurfaceProps, SurfacePropsView, SurfaceStatuses,
};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::GeomSurface;
use rustycad_tkmath::gp::{Direction3, Point3, Vector3};
use std::marker::PhantomData;

/// `GeomLProp_SLPropsBase<SurfaceType, Access>`: the local properties of a
/// surface `S` at `(U, V)`, computed lazily and cached.
///
/// The curvature getters and the tangents keep two native by-value reads (see
/// docs/tkgeombase-progress.md): `MaxCurvature()`... return the value stored
/// *before* the curvature is computed, and the first `TangentU()`/`TangentV()` of
/// a fresh object orients the second derivative.
///
/// ```
/// use rustycad_tkernel::standard::Handle;
/// use rustycad_tkg3d::geom::{GeomSphericalSurface, GeomSurface};
/// use rustycad_tkgeombase::geom_lprop::SLProps;
/// use rustycad_tkmath::gp::Axis3;
/// let sphere = GeomSphericalSurface::new(Axis3::default(), 2.0).unwrap();
/// let surface: Handle<dyn GeomSurface> = Handle::new(sphere).cast();
/// let mut props = SLProps::with_parameters(surface, 0.3, 0.4, 2, 1e-7).unwrap();
/// assert_eq!(props.max_curvature().unwrap(), 0.0); // the value before the computation
/// assert_eq!(props.max_curvature().unwrap(), -0.5);
/// ```
pub struct SLPropsBase<S, A = DirectAccess> {
    my_surf: S,
    my_uv: (f64, f64),
    my_der_order: i32,
    my_cn: i32,
    my_lin_tol: f64,
    my_jet: SurfaceJet,
    my_normal: Direction3,
    my_curvatures: SurfaceCurvatures,
    my_significant_first_derivative_order_u: i32,
    my_significant_first_derivative_order_v: i32,
    my_statuses: SurfaceStatuses,
    access: PhantomData<A>,
}

/// `GeomLProp_SLProps`: the surface local properties of a `Geom_Surface` handle.
pub type SLProps = SLPropsBase<Handle<dyn GeomSurface>>;

// The native `Standard_OutOfRange_Raise_if(N < 0 || N > 2, ...)` of the constructors.
fn check_level(n: i32) -> Result<(), Failure> {
    if !(0..=2).contains(&n) {
        return Err(Failure::out_of_range(&format!(
            "GeomLProp_SLPropsBase::GeomLProp_SLPropsBase(): N = {n}; expected in [0, 2]"
        )));
    }
    Ok(())
}

impl<S, A: SurfaceAccess<S>> SLPropsBase<S, A> {
    /// `GeomLProp_SLPropsBase(S, U, V, N, Resolution)`: properties of `surface` at
    /// `(u, v)` up to derivative `n` (0 to 2, else Standard_OutOfRange);
    /// `SLProps::with_parameters(surface, 0.5, 0.5, 2, 1e-7)?`.
    pub fn with_parameters(
        surface: S,
        u: f64,
        v: f64,
        n: i32,
        resolution: f64,
    ) -> Result<Self, Failure> {
        let mut props: Self = Self::new(surface, n, resolution)?;
        props.set_parameters(u, v)?;
        Ok(props)
    }

    /// `GeomLProp_SLPropsBase(S, N, Resolution)`: as [`Self::with_parameters`]
    /// without parameters; `SLProps::new(surface, 2, 1e-7)?`.
    pub fn new(surface: S, n: i32, resolution: f64) -> Result<Self, Failure> {
        check_level(n)?;
        Ok(Self::unchecked(surface, n, 4, resolution))
    }

    /// `GeomLProp_SLPropsBase(N, Resolution)`: properties without a surface (a
    /// default `S`); nothing is defined until [`Self::set_surface`];
    /// `SLProps::without_surface(2, 1e-7)?`.
    pub fn without_surface(n: i32, resolution: f64) -> Result<Self, Failure>
    where
        S: Default,
    {
        check_level(n)?;
        Ok(Self::unchecked(S::default(), n, 0, resolution))
    }

    fn unchecked(surface: S, n: i32, cn: i32, resolution: f64) -> Self {
        Self {
            my_surf: surface,
            my_uv: (f64::MAX, f64::MAX),
            my_der_order: n,
            my_cn: cn,
            my_lin_tol: resolution,
            my_jet: SurfaceJet::default(),
            my_normal: Direction3::default(),
            my_curvatures: SurfaceCurvatures::default(),
            my_significant_first_derivative_order_u: 0,
            my_significant_first_derivative_order_v: 0,
            my_statuses: SurfaceStatuses::default(),
            access: PhantomData,
        }
    }

    /// `SetSurface(S)`: replace the surface; `props.set_surface(surface)`.
    pub fn set_surface(&mut self, surface: S) {
        self.my_surf = surface;
        self.my_cn = 4;
    }

    /// `SetParameters(U, V)`: evaluate at `(u, v)` and reset every status;
    /// `props.set_parameters(0.5, 0.5)?`.
    pub fn set_parameters(&mut self, u: f64, v: f64) -> Result<(), Failure> {
        surface_utils::set_parameters::<A, S>(
            &self.my_surf,
            (u, v),
            &mut self.my_uv,
            self.my_der_order,
            &mut self.my_jet,
            &mut self.my_statuses,
        )
    }

    /// `Value()`: the point; `props.value()`.
    pub fn value(&self) -> Point3 {
        self.my_jet.point
    }

    /// `D1U()`: the first U derivative, computed if not yet; `props.d1u()?`.
    pub fn d1u(&mut self) -> Result<Vector3, Failure> {
        self.ensure(1, |jet| jet.d1u)
    }

    /// `D1V()`: the first V derivative, computed if not yet; `props.d1v()?`.
    pub fn d1v(&mut self) -> Result<Vector3, Failure> {
        self.ensure(1, |jet| jet.d1v)
    }

    /// `D2U()`: the second U derivative, computed if not yet; `props.d2u()?`.
    pub fn d2u(&mut self) -> Result<Vector3, Failure> {
        self.ensure(2, |jet| jet.d2u)
    }

    /// `D2V()`: the second V derivative, computed if not yet; `props.d2v()?`.
    pub fn d2v(&mut self) -> Result<Vector3, Failure> {
        self.ensure(2, |jet| jet.d2v)
    }

    /// `DUV()`: the mixed derivative, computed if not yet; `props.duv()?`.
    pub fn duv(&mut self) -> Result<Vector3, Failure> {
        self.ensure(2, |jet| jet.duv)
    }

    fn ensure(
        &mut self,
        required: i32,
        field: fn(&SurfaceJet) -> Vector3,
    ) -> Result<Vector3, Failure> {
        surface_utils::ensure_surf_deriv::<A, S>(
            &self.my_surf,
            self.my_uv,
            &mut self.my_der_order,
            required,
            &mut self.my_jet,
            field,
        )
    }

    /// `IsTangentUDefined()`: whether D1U or D2U is longer than the resolution;
    /// `props.is_tangent_u_defined()?`.
    pub fn is_tangent_u_defined(&mut self) -> Result<bool, Failure> {
        let (cn, lin_tol): (i32, f64) = (self.my_cn, self.my_lin_tol);
        let mut order: i32 = self.my_significant_first_derivative_order_u;
        let mut status = self.my_statuses.u_tangent;
        let defined =
            surface_utils::is_tangent_u_defined(self, cn, lin_tol, &mut order, &mut status);
        self.my_significant_first_derivative_order_u = order;
        self.my_statuses.u_tangent = status;
        defined
    }

    /// `TangentU(D)`: the tangent of the iso-V curve, LProp_NotDefined when it is
    /// not defined; `props.tangent_u()?`.
    pub fn tangent_u(&mut self) -> Result<Direction3, Failure> {
        let sig_order: i32 = self.my_significant_first_derivative_order_u;
        surface_utils::tangent_u::<A, Self>(self, sig_order)
    }

    /// `IsTangentVDefined()`: whether D1V or D2V is longer than the resolution;
    /// `props.is_tangent_v_defined()?`.
    pub fn is_tangent_v_defined(&mut self) -> Result<bool, Failure> {
        let (cn, lin_tol): (i32, f64) = (self.my_cn, self.my_lin_tol);
        let mut order: i32 = self.my_significant_first_derivative_order_v;
        let mut status = self.my_statuses.v_tangent;
        let defined =
            surface_utils::is_tangent_v_defined(self, cn, lin_tol, &mut order, &mut status);
        self.my_significant_first_derivative_order_v = order;
        self.my_statuses.v_tangent = status;
        defined
    }

    /// `TangentV(D)`: the tangent of the iso-U curve, LProp_NotDefined when it is
    /// not defined; `props.tangent_v()?`.
    pub fn tangent_v(&mut self) -> Result<Direction3, Failure> {
        let sig_order: i32 = self.my_significant_first_derivative_order_v;
        surface_utils::tangent_v::<A, Self>(self, sig_order)
    }

    /// `IsNormalDefined()`: whether the CSLib normal of the current D1U and D1V is
    /// defined; `props.is_normal_defined()?`.
    pub fn is_normal_defined(&mut self) -> Result<bool, Failure> {
        surface_utils::is_normal_defined(
            self.my_jet.d1u,
            self.my_jet.d1v,
            self.my_lin_tol,
            &mut self.my_normal,
            &mut self.my_statuses.normal,
        )
    }

    /// `Normal()`: the unit normal, LProp_NotDefined when it is not defined;
    /// `props.normal()?`.
    pub fn normal(&mut self) -> Result<Direction3, Failure> {
        surface_utils::normal(self)
    }

    /// `IsCurvatureDefined()`: whether the principal curvatures are defined;
    /// `props.is_curvature_defined()?`.
    pub fn is_curvature_defined(&mut self) -> Result<bool, Failure> {
        let cn: i32 = self.my_cn;
        let mut curvatures: SurfaceCurvatures = self.my_curvatures;
        let mut status = self.my_statuses.curvature;
        let defined = surface_utils::is_curvature_defined(self, cn, &mut curvatures, &mut status);
        self.my_curvatures = curvatures;
        self.my_statuses.curvature = status;
        defined
    }

    /// `IsUmbilic()`: whether the stored extreme curvatures (read before the
    /// computation) are equal; `props.is_umbilic()?`.
    pub fn is_umbilic(&mut self) -> Result<bool, Failure> {
        let (max, min): (f64, f64) = (self.my_curvatures.max_curv, self.my_curvatures.min_curv);
        surface_utils::is_umbilic(self, max, min)
    }

    /// `MaxCurvature()`: the stored maximum curvature read before the computation;
    /// `props.max_curvature()?`.
    pub fn max_curvature(&mut self) -> Result<f64, Failure> {
        let stored: f64 = self.my_curvatures.max_curv;
        surface_utils::require_curvature(self, stored)
    }

    /// `MinCurvature()`: the stored minimum curvature read before the computation;
    /// `props.min_curvature()?`.
    pub fn min_curvature(&mut self) -> Result<f64, Failure> {
        let stored: f64 = self.my_curvatures.min_curv;
        surface_utils::require_curvature(self, stored)
    }

    /// `CurvatureDirections(MaxD, MinD)`: the directions of the maximum and minimum
    /// curvatures; `let (max_d, min_d) = props.curvature_directions()?;`.
    pub fn curvature_directions(&mut self) -> Result<(Direction3, Direction3), Failure> {
        surface_utils::curvature_directions(self)
    }

    /// `MeanCurvature()`: the stored mean curvature read before the computation;
    /// `props.mean_curvature()?`.
    pub fn mean_curvature(&mut self) -> Result<f64, Failure> {
        let stored: f64 = self.my_curvatures.mean_curv;
        surface_utils::require_curvature(self, stored)
    }

    /// `GaussianCurvature()`: the stored Gaussian curvature read before the
    /// computation; `props.gaussian_curvature()?`.
    pub fn gaussian_curvature(&mut self) -> Result<f64, Failure> {
        let stored: f64 = self.my_curvatures.gaus_curv;
        surface_utils::require_curvature(self, stored)
    }
}

impl<S, A: SurfaceAccess<S>> SurfaceProps for SLPropsBase<S, A> {
    type Surface = S;
    fn props_surface(&self) -> &S {
        &self.my_surf
    }
    fn view(&self) -> SurfacePropsView {
        SurfacePropsView {
            u: self.my_uv.0,
            v: self.my_uv.1,
            der_order: self.my_der_order,
            jet: self.my_jet,
            normal: self.my_normal,
            curvatures: self.my_curvatures,
        }
    }
    fn d1u(&mut self) -> Result<Vector3, Failure> {
        SLPropsBase::d1u(self)
    }
    fn d1v(&mut self) -> Result<Vector3, Failure> {
        SLPropsBase::d1v(self)
    }
    fn d2u(&mut self) -> Result<Vector3, Failure> {
        SLPropsBase::d2u(self)
    }
    fn d2v(&mut self) -> Result<Vector3, Failure> {
        SLPropsBase::d2v(self)
    }
    fn is_tangent_u_defined(&mut self) -> Result<bool, Failure> {
        SLPropsBase::is_tangent_u_defined(self)
    }
    fn is_tangent_v_defined(&mut self) -> Result<bool, Failure> {
        SLPropsBase::is_tangent_v_defined(self)
    }
    fn is_normal_defined(&mut self) -> Result<bool, Failure> {
        SLPropsBase::is_normal_defined(self)
    }
    fn is_curvature_defined(&mut self) -> Result<bool, Failure> {
        SLPropsBase::is_curvature_defined(self)
    }
}
