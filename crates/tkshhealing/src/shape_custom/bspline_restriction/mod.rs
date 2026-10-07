// Rust translation of ModelingAlgorithms/TKShHealing/ShapeCustom/ShapeCustom_BSplineRestriction.hxx/.lxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `ShapeCustom_BSplineRestriction`: approximation of surfaces, 3D curves and p-curves by
//! B-splines within a maximal degree and number of segments (docs/shhealing-design.md
//! section 7).
mod accessors;
mod curve2d;
mod curve3d;
mod curve_approx;
mod predicates;
mod surface;
mod surface_approx;

pub use surface::ShapeCustomRestrictionBox;

use super::modification::{ShapeCustomModification, ShapeCustomModificationFields};
use super::restriction_parameters::ShapeCustomRestrictionParameters;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_tools::{BRepToolsModification, NewCurve, NewSurface};
use rustycad_tkbrep::topods::{Edge, Face, Vertex};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::message::{Gravity, Msg};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkernel::tcollection::AsciiString;
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkmath::geom_abs::Shape as Continuity;
use rustycad_tkmath::gp::Point3;

/// The private fields of `ShapeCustom_BSplineRestriction`.
#[derive(Clone)]
struct RestrictionState {
    continuity_3d: Continuity,
    continuity_2d: Continuity,
    max_degree: i32,
    nb_max_seg: i32,
    tol_3d: f64,
    tol_2d: f64,
    surface_error: f64,
    curve_3d_error: f64,
    curve_2d_error: f64,
    nb_of_span: i32,
    approx_surface: bool,
    approx_curve_3d: bool,
    approx_curve_2d: bool,
    degree_priority: bool,
    converted: bool,
    rational: bool,
    parameters: Handle<ShapeCustomRestrictionParameters>,
}

/// The settings of the native full constructors, in their order.
#[derive(Clone, Copy)]
pub struct ShapeCustomBSplineRestrictionSettings {
    /// anApproxSurfaceFlag.
    pub approx_surface: bool,
    /// anApproxCurve3dFlag.
    pub approx_curve_3d: bool,
    /// anApproxCurve2dFlag.
    pub approx_curve_2d: bool,
    /// aTol3d.
    pub tol_3d: f64,
    /// aTol2d.
    pub tol_2d: f64,
    /// aContinuity3d.
    pub continuity_3d: Continuity,
    /// aContinuity2d.
    pub continuity_2d: Continuity,
    /// aMaxDegree.
    pub max_degree: i32,
    /// aNbMaxSeg.
    pub nb_max_seg: i32,
    /// Degree: priority to the degree (true) or to the number of segments.
    pub degree_priority: bool,
    /// Rational: convert rational curves and surfaces too.
    pub rational: bool,
}

impl Default for ShapeCustomBSplineRestrictionSettings {
    /// The values of the native default constructor.
    fn default() -> Self {
        Self {
            approx_surface: true,
            approx_curve_3d: true,
            approx_curve_2d: true,
            tol_3d: 0.01,
            tol_2d: 1e-6,
            continuity_3d: Continuity::C1,
            continuity_2d: Continuity::C2,
            max_degree: 9,
            nb_max_seg: 10000,
            degree_priority: true,
            rational: false,
        }
    }
}

/// `ShapeCustom_BSplineRestriction`: a modification that approximates the surfaces, 3D
/// curves and p-curves selected by its ShapeCustom_RestrictionParameters (and those whose
/// degree, number of segments or rationality exceed the limits) by B-splines.
///
/// ```
/// use rustycad_tkshhealing::shape_custom::ShapeCustomBSplineRestriction;
/// let restriction = ShapeCustomBSplineRestriction::new();
/// let (c3d, c2d) = (restriction.curve_3d_error(), restriction.curve_2d_error());
/// assert_eq!(restriction.max_errors(), (1e-7, c3d, c2d));
/// assert_eq!(restriction.nb_of_span(), 0);
/// ```
pub struct ShapeCustomBSplineRestriction {
    parent: StandardTransient,
    messages: ShapeCustomModificationFields,
    state: SharedState<RestrictionState>,
}
impl_transient!(
    ShapeCustomBSplineRestriction,
    "ShapeCustom_BSplineRestriction",
    dyn ShapeCustomModification,
    parent,
    interfaces: [dyn BRepToolsModification, dyn ShapeCustomModification]
);

impl Default for ShapeCustomBSplineRestriction {
    fn default() -> Self {
        Self::with_settings(ShapeCustomBSplineRestrictionSettings::default())
    }
}

impl ShapeCustomModification for ShapeCustomBSplineRestriction {
    fn modification_messages(&self) -> &ShapeCustomModificationFields {
        &self.messages
    }
}

impl ShapeCustomBSplineRestriction {
    /// `ShapeCustom_BSplineRestriction()`: tolerances 0.01 / 1e-6, C1 / C2, degree 9,
    /// 10000 segments, degree priority, no rational conversion and default restriction
    /// parameters; `ShapeCustomBSplineRestriction::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// The constructor with the flags, tolerances, continuities and limits (and new default
    /// restriction parameters); `ShapeCustomBSplineRestriction::with_settings(settings)`.
    pub fn with_settings(settings: ShapeCustomBSplineRestrictionSettings) -> Self {
        let parameters = Handle::new(ShapeCustomRestrictionParameters::new());
        Self::with_settings_and_modes(settings, &parameters)
    }

    /// The constructor with the settings and the restriction parameters `modes` (shared);
    /// `ShapeCustomBSplineRestriction::with_settings_and_modes(settings, &modes)`.
    pub fn with_settings_and_modes(
        settings: ShapeCustomBSplineRestrictionSettings,
        modes: &Handle<ShapeCustomRestrictionParameters>,
    ) -> Self {
        let state = RestrictionState {
            continuity_3d: settings.continuity_3d,
            continuity_2d: settings.continuity_2d,
            max_degree: settings.max_degree,
            nb_max_seg: settings.nb_max_seg,
            tol_3d: settings.tol_3d,
            tol_2d: settings.tol_2d,
            surface_error: precision::confusion(),
            curve_3d_error: precision::confusion(),
            curve_2d_error: precision::p_confusion(),
            nb_of_span: 0,
            approx_surface: settings.approx_surface,
            approx_curve_3d: settings.approx_curve_3d,
            approx_curve_2d: settings.approx_curve_2d,
            degree_priority: settings.degree_priority,
            converted: false,
            rational: settings.rational,
            parameters: modes.clone(),
        };
        Self {
            parent: StandardTransient::default(),
            messages: ShapeCustomModificationFields::default(),
            state: SharedState::new(state),
        }
    }

    fn read(&self) -> RestrictionState {
        self.state.read().clone()
    }

    /// The restriction parameters; native dereferences them unchecked, so a null handle is
    /// a Standard_NullObject failure.
    fn parameters(&self) -> Result<Handle<ShapeCustomRestrictionParameters>, Failure> {
        let parameters = self.state.read().parameters.clone();
        if parameters.is_null() {
            return Err(Failure::with_type(
                "Standard_NullObject",
                "ShapeCustom_BSplineRestriction: null ShapeCustom_RestrictionParameters; expected the restriction modes",
                "",
            ));
        }
        Ok(parameters)
    }

    fn send(&self, shape: &rustycad_tkbrep::topods::Shape, key: &[u8]) {
        let key = AsciiString::from_bytes(key);
        self.send_msg(shape, &Msg::from_key(&key), Gravity::Info);
    }
}

impl BRepToolsModification for ShapeCustomBSplineRestriction {
    /// The face surface approximated within the limits (MSG1 for a B-spline origin, MSG0
    /// otherwise); false without the surface flag or when nothing needs a conversion.
    fn new_surface(&self, face: &Face, result: &mut NewSurface) -> Result<bool, Failure> {
        self.restricted_surface(face, result)
    }

    /// The 3D curve approximated within the limits, or copied when a surface or p-curve of
    /// the edge is converted.
    fn new_curve(&self, edge: &Edge, result: &mut NewCurve) -> Result<bool, Failure> {
        self.restricted_curve(edge, result)
    }

    /// The vertex point and tolerance once a surface or curve was approximated (the native
    /// `myConvert`); the tolerance is written in both cases.
    fn new_point(
        &self,
        vertex: &Vertex,
        point: &mut Point3,
        tolerance: &mut f64,
    ) -> Result<bool, Failure> {
        *tolerance = BRepTool::vertex_tolerance(vertex)?;
        if !self.read().converted {
            return Ok(false);
        }
        *point = BRepTool::pnt(vertex)?;
        Ok(true)
    }

    /// The p-curve approximated within the limits, or copied when the edge is converted.
    fn new_curve2d(
        &self,
        edge: &Edge,
        face: &Face,
        new_edge: &Edge,
        _new_face: &Face,
        curve: &mut Handle<dyn Geom2dCurve>,
        tolerance: &mut f64,
    ) -> Result<bool, Failure> {
        self.restricted_pcurve(
            curve2d::PCurveRequest {
                edge,
                face,
                new_edge,
            },
            curve,
            tolerance,
        )
    }

    fn new_parameter(
        &self,
        _vertex: &Vertex,
        _edge: &Edge,
        _parameter: &mut f64,
        _tolerance: &mut f64,
    ) -> Result<bool, Failure> {
        Ok(false)
    }

    /// The continuity of the edge between the original faces.
    fn continuity(
        &self,
        edge: &Edge,
        face1: &Face,
        face2: &Face,
        _new_edge: &Edge,
        _new_face1: &Face,
        _new_face2: &Face,
    ) -> Result<Continuity, Failure> {
        BRepTool::continuity_on_faces(edge, face1, face2)
    }
}
