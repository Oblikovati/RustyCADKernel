// Rust translation of TopOpeBRepDS_InterferenceTool.hxx/.cxx at
// OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
use super::{
    TopOpeBRepDSConfig, TopOpeBRepDSCurvePointInterference, TopOpeBRepDSEdgeVertexInterference,
    TopOpeBRepDSFaceEdgeInterference, TopOpeBRepDSInterference, TopOpeBRepDSKind,
    TopOpeBRepDSSolidSurfaceInterference, TopOpeBRepDSSurfaceCurveInterference,
    TopOpeBRepDSTransition,
};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dCurve;

/// Constructs typed interference handles and accesses curve-point parameters.
/// Example: `TopOpeBRepDSInterferenceTool::parameter(&interference)?`.
pub struct TopOpeBRepDSInterferenceTool;

impl TopOpeBRepDSInterferenceTool {
    /// Creates a curve-point interference on the specified support; `Self::make_edge_interference(&t, sk, si, gk, gi, p)`.
    pub fn make_edge_interference(
        transition: &TopOpeBRepDSTransition,
        support_kind: TopOpeBRepDSKind,
        support: i32,
        geometry_kind: TopOpeBRepDSKind,
        geometry: i32,
        parameter: f64,
    ) -> Handle<TopOpeBRepDSInterference> {
        Self::make_curve_interference(
            transition,
            support_kind,
            support,
            geometry_kind,
            geometry,
            parameter,
        )
    }

    /// Preserves both caller-supplied kinds and indices; `Self::make_curve_interference(&t, sk, si, gk, gi, p)`.
    pub fn make_curve_interference(
        transition: &TopOpeBRepDSTransition,
        support_kind: TopOpeBRepDSKind,
        support: i32,
        geometry_kind: TopOpeBRepDSKind,
        geometry: i32,
        parameter: f64,
    ) -> Handle<TopOpeBRepDSInterference> {
        Handle::new(TopOpeBRepDSCurvePointInterference::new(
            transition,
            support_kind,
            support,
            geometry_kind,
            geometry,
            parameter,
        ))
        .cast()
    }

    /// Copies into a new object with the complemented transition; `Self::duplicate_curve_point_interference(&i)?`.
    pub fn duplicate_curve_point_interference(
        interference: &Handle<TopOpeBRepDSInterference>,
    ) -> Result<Handle<TopOpeBRepDSInterference>, Failure> {
        with_curve_point(interference, |point| {
            Self::make_curve_interference(
                &point.transition().complement(),
                point.support_type(),
                point.support(),
                point.geometry_type(),
                point.geometry(),
                point.parameter(),
            )
        })
    }

    /// Creates a face/curve interference retaining the p-curve handle; `Self::make_face_curve_interference(&t, 1, 2, &pc)`.
    pub fn make_face_curve_interference(
        transition: &TopOpeBRepDSTransition,
        face: i32,
        curve: i32,
        pcurve: &Handle<dyn Geom2dCurve>,
    ) -> Handle<TopOpeBRepDSInterference> {
        Handle::new(TopOpeBRepDSSurfaceCurveInterference::new(
            transition,
            TopOpeBRepDSKind::Face,
            face,
            TopOpeBRepDSKind::Curve,
            curve,
            pcurve,
        ))
        .cast()
    }

    /// Creates the solid/surface specialization; `Self::make_solid_surface_interference(&t, 1, 2)`.
    pub fn make_solid_surface_interference(
        transition: &TopOpeBRepDSTransition,
        solid: i32,
        surface: i32,
    ) -> Handle<TopOpeBRepDSInterference> {
        Handle::new(TopOpeBRepDSSolidSurfaceInterference::new(
            transition,
            TopOpeBRepDSKind::Solid,
            solid,
            TopOpeBRepDSKind::Surface,
            surface,
        ))
        .cast()
    }

    /// Preserves the vertex's bound flag, configuration and parameter; `Self::make_edge_vertex_interference(&t, 1, 2, true, config, 0.5)`.
    pub fn make_edge_vertex_interference(
        transition: &TopOpeBRepDSTransition,
        edge: i32,
        vertex: i32,
        vertex_is_bound: bool,
        config: TopOpeBRepDSConfig,
        parameter: f64,
    ) -> Handle<TopOpeBRepDSInterference> {
        Handle::new(TopOpeBRepDSEdgeVertexInterference::on_edge(
            transition,
            edge,
            vertex,
            vertex_is_bound,
            config,
            parameter,
        ))
        .cast()
    }

    /// Creates a face/edge interference; `Self::make_face_edge_interference(&t, 1, 2, false, config)`.
    pub fn make_face_edge_interference(
        transition: &TopOpeBRepDSTransition,
        face: i32,
        edge: i32,
        edge_is_bound: bool,
        config: TopOpeBRepDSConfig,
    ) -> Handle<TopOpeBRepDSInterference> {
        Handle::new(TopOpeBRepDSFaceEdgeInterference::new(
            transition,
            face,
            edge,
            edge_is_bound,
            config,
        ))
        .cast()
    }

    /// Reads the curve-point specialization's parameter; `Self::parameter(&i)?`.
    pub fn parameter(interference: &Handle<TopOpeBRepDSInterference>) -> Result<f64, Failure> {
        with_curve_point(interference, TopOpeBRepDSCurvePointInterference::parameter)
    }

    /// Updates the shared curve-point object through its handle; `Self::set_parameter(&i, 0.5)?`.
    pub fn set_parameter(
        interference: &Handle<TopOpeBRepDSInterference>,
        parameter: f64,
    ) -> Result<(), Failure> {
        with_curve_point(interference, |point| point.set_parameter(parameter))
    }
}

fn with_curve_point<T>(
    interference: &Handle<TopOpeBRepDSInterference>,
    operation: impl FnOnce(&TopOpeBRepDSCurvePointInterference) -> T,
) -> Result<T, Failure> {
    // Native Parameter downcasts then dereferences; invalid handles have no defined
    // native result. Keep that misuse explicit instead of turning it into a panic.
    let point: Handle<TopOpeBRepDSCurvePointInterference> = interference.clone().cast();
    let point = point.get().ok_or_else(|| {
        Failure::with_type(
            "Standard_NullObject",
            &format!(
                "interference handle {:?}; expected a non-null TopOpeBRepDS_CurvePointInterference",
                interference.object_address()
            ),
            "TopOpeBRepDS_InterferenceTool",
        )
    })?;
    Ok(operation(point))
}
