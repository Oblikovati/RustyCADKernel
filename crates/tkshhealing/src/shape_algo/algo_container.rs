// Rust translation of ModelingAlgorithms/TKShHealing/ShapeAlgo/ShapeAlgo_AlgoContainer.hxx/.lxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `ShapeAlgo_AlgoContainer`: the Shape Healing algorithms that data exchange calls
//! through one object; most members delegate to a package function.
use super::ShapeAlgoToolContainer;
use crate::shape_analysis::{ShapeAnalysis, ShapeAnalysisWire};
use crate::shape_construct::{
    ShapeConstruct, ShapeConstructApproxSettings, ShapeConstructSurfaceBounds,
};
use crate::shape_custom::ShapeCustomSurface;
use crate::shape_extend::{ShapeExtendStatus, ShapeExtendWireData};
use crate::shape_upgrade::{
    BoundedCurve2dSequence, BoundedCurveSequence, ShapeUpgrade, ShapeUpgradeShapeDivideContinuity,
};
use rustycad_tkbrep::topods::{Face, Shape, Wire};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkg2d::geom2d::Geom2dBSplineCurve;
use rustycad_tkg3d::geom::{GeomBSplineCurve, GeomBSplineSurface, GeomCurve, GeomSurface};
use rustycad_tkmath::geom_abs::Shape as Continuity;

/// `ShapeAlgo_AlgoContainer`: the algorithms of Shape Healing used by data exchange, with
/// the tool container (`myTC`). The native members are virtual, but no OCCT class
/// overrides them.
///
/// ```
/// use rustycad_tkshhealing::shape_algo::ShapeAlgoAlgoContainer;
/// let container = ShapeAlgoAlgoContainer::new();
/// assert!(!container.tool_container().is_null());
/// ```
pub struct ShapeAlgoAlgoContainer {
    parent: StandardTransient,
    tools: SharedState<Handle<ShapeAlgoToolContainer>>,
}
impl_transient!(
    ShapeAlgoAlgoContainer,
    "ShapeAlgo_AlgoContainer",
    StandardTransient,
    parent
);

impl Default for ShapeAlgoAlgoContainer {
    fn default() -> Self {
        Self::new()
    }
}

/// The outputs of `ConnectNextWire`: its result, `distmin`, `revsewd` and `revnextsewd`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ShapeAlgoWireConnection {
    /// The native return value: the wires were connected.
    pub connected: bool,
    /// `distmin`: the gap between the joined ends.
    pub distance: f64,
    /// `revsewd`: the loaded wire was reversed.
    pub reversed_wire: bool,
    /// `revnextsewd`: the next wire was reversed.
    pub reversed_next: bool,
}

impl ShapeAlgoAlgoContainer {
    /// `ShapeAlgo_AlgoContainer()`: with a new tool container;
    /// `ShapeAlgoAlgoContainer::new()`.
    pub fn new() -> Self {
        Self {
            parent: StandardTransient::default(),
            tools: SharedState::new(Handle::new(ShapeAlgoToolContainer::new())),
        }
    }

    /// `SetToolContainer(TC)`; `container.set_tool_container(&tools)`.
    pub fn set_tool_container(&self, tools: &Handle<ShapeAlgoToolContainer>) {
        *self.tools.write() = tools.clone();
    }

    /// `ToolContainer()`; `container.tool_container()`.
    pub fn tool_container(&self) -> Handle<ShapeAlgoToolContainer> {
        self.tools.read().clone()
    }

    /// `ConnectNextWire(saw, nextsewd, maxtol, distmin, revsewd, revnextsewd)`: adds
    /// `next` to the wire of `saw` at the closest ends (CheckShapeConnect), reversing the
    /// loaded wire or `next` when needed. An empty `next` is connected; an empty loaded
    /// wire gets `next` as is. A null handle is the native null dereference
    /// (Standard_NullObject).
    /// Example: `container.connect_next_wire(&saw, &next, 1e-3)?.connected`.
    pub fn connect_next_wire(
        &self,
        saw: &Handle<ShapeAnalysisWire>,
        next: &Handle<ShapeExtendWireData>,
        max_tol: f64,
    ) -> Result<ShapeAlgoWireConnection, Failure> {
        let next_data = next.get().ok_or_else(|| null_object("nextsewd"))?;
        let analysis = saw.get().ok_or_else(|| null_object("saw"))?;
        let mut result = ShapeAlgoWireConnection {
            connected: true,
            ..Default::default()
        };
        if next_data.nb_edges() == 0 {
            return Ok(result);
        }
        let sewd = analysis.wire_data();
        let wire_data = sewd.get().ok_or_else(|| null_object("saw->WireData()"))?;
        if wire_data.nb_edges() == 0 {
            wire_data.add_wire_data(next_data, 0)?;
            return Ok(result);
        }
        let mut distances = [0.; 4];
        let next_wire = next_data.wire()?;
        analysis.check_shape_connect_into(&mut distances, &next_wire, max_tol)?;
        orient_for_connection(analysis, wire_data, next_data, distances, &mut result)?;
        result.connected = !analysis.last_check_status(ShapeExtendStatus::Fail);
        if result.connected {
            wire_data.add_wire_data(next_data, 0)?;
        }
        Ok(result)
    }

    /// `C0BSplineToSequenceOfC1BSplineCurve(BS, seqBS)` (3D):
    /// ShapeUpgrade::C0BSplineToSequenceOfC1BSplineCurve; `None` is the native false.
    /// Example: `container.c0_bspline_to_sequence_of_c1_bspline_curve(&bspline)?`.
    pub fn c0_bspline_to_sequence_of_c1_bspline_curve(
        &self,
        bspline: &Handle<GeomBSplineCurve>,
    ) -> Result<Option<BoundedCurveSequence>, Failure> {
        ShapeUpgrade::c0_bspline_to_sequence_of_c1_bspline_curve(bspline)
    }

    /// `C0BSplineToSequenceOfC1BSplineCurve(BS, seqBS)` (2D), as the 3D overload.
    /// Example: `container.c0_bspline2d_to_sequence_of_c1_bspline_curve(&bspline)?`.
    pub fn c0_bspline2d_to_sequence_of_c1_bspline_curve(
        &self,
        bspline: &Handle<Geom2dBSplineCurve>,
    ) -> Result<Option<BoundedCurve2dSequence>, Failure> {
        ShapeUpgrade::c0_bspline2d_to_sequence_of_c1_bspline_curve(bspline)
    }

    /// `C0ShapeToC1Shape(shape, tol)`: the result of ShapeUpgrade_ShapeDivideContinuity
    /// with the tolerance and the C1 boundary and surface criteria.
    /// Example: `container.c0_shape_to_c1_shape(&shape, 1e-3)?`.
    pub fn c0_shape_to_c1_shape(&self, shape: &Shape, tol: f64) -> Result<Shape, Failure> {
        let divider = ShapeUpgradeShapeDivideContinuity::new(shape.clone());
        divider.set_tolerance(tol);
        divider.set_boundary_criterion(Continuity::C1);
        divider.set_surface_criterion(Continuity::C1);
        divider.perform(true)?;
        Ok(divider.result())
    }

    /// `ConvertSurfaceToBSpline(surf, UF, UL, VF, VL)`: ShapeConstruct with
    /// Precision::Confusion, C1, 100 segments and the maximal B-spline degree.
    /// Example: `container.convert_surface_to_bspline(&surface, bounds)?`.
    pub fn convert_surface_to_bspline(
        &self,
        surface: &Handle<dyn GeomSurface>,
        bounds: ShapeConstructSurfaceBounds,
    ) -> Result<Handle<GeomBSplineSurface>, Failure> {
        let settings = ShapeConstructApproxSettings {
            tol3d: precision::confusion(),
            continuity: Continuity::C1,
            max_segments: 100,
            max_degree: GeomBSplineSurface::max_degree(),
        };
        ShapeConstruct::convert_surface_to_bspline(surface, bounds, settings)
    }

    /// `OuterWire(face)`: ShapeAnalysis::OuterWire; `container.outer_wire(&face)?`.
    pub fn outer_wire(&self, face: &Face) -> Result<Wire, Failure> {
        ShapeAnalysis::outer_wire(face)
    }

    /// `ConvertToPeriodic(surf)`: ShapeCustom_Surface(surf).ConvertToPeriodic(false)
    /// (default precision); `container.convert_to_periodic(&surface)?`.
    pub fn convert_to_periodic(
        &self,
        surface: &Handle<dyn GeomSurface>,
    ) -> Result<Handle<dyn GeomSurface>, Failure> {
        ShapeCustomSurface::with_surface(surface).convert_to_periodic(false, -1.)
    }

    /// `GetFaceUVBounds(F, Umin, Umax, Vmin, Vmax)`: ShapeAnalysis::GetFaceUVBounds;
    /// `let (umin, umax, vmin, vmax) = container.get_face_uv_bounds(&face)?;`.
    pub fn get_face_uv_bounds(&self, face: &Face) -> Result<(f64, f64, f64, f64), Failure> {
        ShapeAnalysis::get_face_uv_bounds(face)
    }

    /// `ConvertCurveToBSpline(C3D, First, Last, Tol3d, Continuity, MaxSegments,
    /// MaxDegree)`: ShapeConstruct::ConvertCurveToBSpline.
    /// Example: `container.convert_curve_to_bspline(&curve, 0., 1., settings)?`.
    pub fn convert_curve_to_bspline(
        &self,
        curve: &Handle<dyn GeomCurve>,
        first: f64,
        last: f64,
        settings: ShapeConstructApproxSettings,
    ) -> Result<Handle<GeomBSplineCurve>, Failure> {
        ShapeConstruct::convert_curve_to_bspline(
            curve,
            first,
            last,
            settings.tol3d,
            settings.continuity,
            settings.max_segments,
            settings.max_degree,
        )
    }
}

/// The reversals of `ConnectNextWire` after CheckShapeConnect gave `distances` (tail-head,
/// tail-tail, head-tail, head-head).
fn orient_for_connection(
    analysis: &ShapeAnalysisWire,
    wire: &ShapeExtendWireData,
    next: &ShapeExtendWireData,
    [tail_head, tail_tail, head_tail, head_head]: [f64; 4],
    result: &mut ShapeAlgoWireConnection,
) -> Result<(), Failure> {
    let status = |s| analysis.last_check_status(s);
    let precision = analysis.precision();
    result.distance = tail_head;
    let head_side = status(ShapeExtendStatus::Done4) || status(ShapeExtendStatus::Done3);
    if tail_head > precision && tail_tail > precision && head_side {
        wire.reverse()?;
        result.distance = head_head;
        result.reversed_wire = true;
        if status(ShapeExtendStatus::Done3) {
            next.reverse()?;
            result.reversed_next = true;
            result.distance = head_tail;
        }
    } else if !status(ShapeExtendStatus::Fail) && !status(ShapeExtendStatus::Done5) {
        next.reverse()?;
        result.reversed_next = true;
        result.distance = tail_tail;
    }
    Ok(())
}

pub(super) fn null_object(what: &str) -> Failure {
    Failure::with_type(
        "Standard_NullObject",
        &format!("ShapeAlgo_AlgoContainer: null {what}; expected a non-null handle"),
        "",
    )
}
