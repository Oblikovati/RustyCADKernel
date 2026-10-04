//! Public value records nested in the `GeomAdaptor_Surface` declaration.
use crate::adaptor3d::Adaptor3dCurve;
use crate::geom::{GeomBSplineSurface, GeomBezierSurface, GeomOffsetSurface};
use crate::geom_eval::RepSurfaceDesc;
use rustycad_tkernel::standard::Handle;
use rustycad_tkmath::bsplslib::SharedBSplineSurfaceCache;
use rustycad_tkmath::gp::{Axis1, Cone3, Cylinder3, Plane3, Sphere3, Torus3, Xyz};

use super::GeomAdaptorSurface;

/// Extrusion evaluation state (`GeomAdaptor_Surface::ExtrusionData`).
#[derive(Clone, Default)]
pub struct ExtrusionData {
    /// Adaptor for the basis curve.
    pub basis_curve: Handle<dyn Adaptor3dCurve>,
    /// Extrusion direction in XYZ coordinates.
    pub direction: Xyz,
    /// Optional evaluation representation captured at load time.
    pub eval_rep: Handle<dyn RepSurfaceDesc>,
}

/// Revolution evaluation state (`GeomAdaptor_Surface::RevolutionData`).
#[derive(Clone, Default)]
pub struct RevolutionData {
    /// Adaptor for the basis curve.
    pub basis_curve: Handle<dyn Adaptor3dCurve>,
    /// Revolution axis.
    pub axis: Axis1,
    /// Optional evaluation representation captured at load time.
    pub eval_rep: Handle<dyn RepSurfaceDesc>,
}

/// Offset-surface evaluation state (`GeomAdaptor_Surface::OffsetData`).
#[derive(Clone, Default)]
pub struct OffsetData {
    /// Adaptor for the basis surface.
    pub basis_adaptor: Handle<GeomAdaptorSurface>,
    /// Equivalent adaptor when one is available.
    pub equivalent_adaptor: Handle<GeomAdaptorSurface>,
    /// Original offset surface for osculating queries.
    pub offset_surface: Handle<GeomOffsetSurface>,
    /// Signed offset distance.
    pub offset: f64,
    /// Optional evaluation representation captured at load time.
    pub eval_rep: Handle<dyn RepSurfaceDesc>,
}

/// Bezier surface evaluation state with a nullable shared cache handle.
#[derive(Clone, Default)]
pub struct BezierData {
    /// Bezier surface retained without downcasting again.
    pub surface: Handle<GeomBezierSurface>,
    /// Nullable shared cache with copy-preserved identity and mutation.
    pub cache: SharedBSplineSurfaceCache,
    /// Optional evaluation representation captured at load time.
    pub eval_rep: Handle<dyn RepSurfaceDesc>,
}

/// B-spline surface evaluation state with a nullable shared cache handle.
#[derive(Clone, Default)]
pub struct BSplineData {
    /// B-spline surface retained without downcasting again.
    pub surface: Handle<GeomBSplineSurface>,
    /// Nullable shared cache with copy-preserved identity and mutation.
    pub cache: SharedBSplineSurfaceCache,
    /// Optional evaluation representation captured at load time.
    pub eval_rep: Handle<dyn RepSurfaceDesc>,
}

/// One alternative from `GeomAdaptor_Surface::SurfaceDataVariant`.
#[derive(Clone, Default)]
pub enum SurfaceDataVariant {
    /// Native `std::monostate` state.
    #[default]
    Uninitialized,
    /// Elementary plane parameters.
    Plane(Plane3),
    /// Elementary cylinder parameters.
    Cylinder(Cylinder3),
    /// Elementary cone parameters.
    Cone(Cone3),
    /// Elementary sphere parameters.
    Sphere(Sphere3),
    /// Elementary torus parameters.
    Torus(Torus3),
    /// Extrusion state.
    Extrusion(ExtrusionData),
    /// Revolution state.
    Revolution(RevolutionData),
    /// Offset state.
    Offset(OffsetData),
    /// Bezier surface state.
    Bezier(BezierData),
    /// B-spline surface state.
    BSpline(BSplineData),
}
