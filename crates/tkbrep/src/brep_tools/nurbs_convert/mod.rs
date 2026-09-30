// Rust translation of ModelingData/TKBRep/BRepTools/BRepTools_NurbsConvertModification.hxx
// and the NewSurface, NewCurve, NewPoint, NewParameter, Continuity and GetUpdatedEdges
// members of BRepTools_NurbsConvertModification.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepTools_NurbsConvertModification`: the modification that converts every curve and
//! surface into a B-spline.
mod curve2d;
mod helpers;
mod mesh;
mod surface;

use super::copy_modification::CopyModification;
use super::modification::{BRepToolsModification, NewCurve, NewSurface};
use crate::bintools::handle_table::{HandleKey, IdentityHasher};
use crate::brep::BRepTool;
use crate::topods::{Edge, Face, Shape, Vertex};
use helpers::{is_convert, new_curve_of, new_parameter_on_curve};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::ncollection::{Array1, IndexedDataMap};
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::geom::{GeomBSplineCurve, GeomCurve, GeomTrimmedCurve};
use rustycad_tkgeombase::geom_convert::curve_to_bspline_curve;
use rustycad_tkmath::bsplclib;
use rustycad_tkmath::convert::ParameterisationType;
use rustycad_tkmath::geom_abs::Shape as Continuity;
use rustycad_tkmath::gp::Point3;
use rustycad_tkmath::poly::{Polygon3D, PolygonOnTriangulation, Triangulation};
use rustycad_tkmath::toploc::Location;

/// `NCollection_IndexedDataMap<occ::handle<Standard_Transient>, occ::handle<Standard_Transient>>`:
/// each original curve or surface with its conversion, by object identity.
pub(super) type ConversionMap =
    IndexedDataMap<HandleKey<StandardTransient>, Handle<StandardTransient>, IdentityHasher>;

/// The mutable state of the modification.
#[derive(Default)]
struct NurbsState {
    /// `myled`: the seam edges already processed.
    led: Vec<Shape>,
    /// `mylcu`: the p-curves kept for the second pass of a seam (not aligned with
    /// `led`, as natively).
    lcu: Vec<Handle<StandardTransient>>,
    /// `myMap`.
    map: ConversionMap,
    /// `myUpdatedEdges`: the edges whose tolerance grew.
    updated_edges: Vec<Shape>,
}

/// `BRepTools_NurbsConvertModification`: surfaces become B-spline surfaces (trimmed to
/// the face bounds), curves B-spline curves on the edge range and p-curves are
/// projected on the new surfaces; the mesh is copied with its parameters remapped.
///
/// The native class derives from `BRepTools_CopyModification`, which the Rust type
/// embeds as its parent field.
///
/// ```
/// use rustycad_tkbrep::brep_tools::NurbsConvertModification;
/// let modification = NurbsConvertModification::new();
/// assert!(modification.updated_edges().is_empty());
/// ```
pub struct NurbsConvertModification {
    parent: CopyModification,
    state: SharedState<NurbsState>,
}
impl_transient!(
    NurbsConvertModification,
    "BRepTools_NurbsConvertModification",
    CopyModification,
    parent,
    interfaces: [dyn BRepToolsModification]
);

impl Default for NurbsConvertModification {
    fn default() -> Self {
        Self::new()
    }
}

impl NurbsConvertModification {
    /// `BRepTools_NurbsConvertModification()`; `NurbsConvertModification::new()`.
    pub fn new() -> Self {
        Self {
            parent: CopyModification::default(),
            state: SharedState::new(NurbsState::default()),
        }
    }

    /// `GetUpdatedEdges()`: the new edges whose tolerance was raised by
    /// `BRepTools::EvalAndUpdateTol`; `modification.updated_edges()`.
    pub fn updated_edges(&self) -> Vec<Shape> {
        self.state.read().updated_edges.clone()
    }

    /// `myMap.Add(original, converted)` (an existing key keeps its conversion).
    fn record(&self, original: Handle<StandardTransient>, converted: Handle<StandardTransient>) {
        self.state.write().map.add(HandleKey(original), converted);
    }

    /// `myMap.Contains(original)`.
    fn is_recorded(&self, original: &Handle<StandardTransient>) -> bool {
        self.state.read().map.contains(&HandleKey(original.clone()))
    }

    /// A snapshot of `myMap` for the lookups of the mesh members.
    fn conversions(&self) -> ConversionMap {
        let state = self.state.read();
        let mut copy = ConversionMap::default();
        for index in 1..=state.map.extent() {
            if let (Ok(key), Ok(value)) =
                (state.map.find_key(index), state.map.find_from_index(index))
            {
                copy.add(HandleKey(key.0.clone()), value.clone());
            }
        }
        copy
    }

    /// `myUpdatedEdges.Append(newE)` when `new_tolerance` exceeds `tolerance`.
    fn raise_tolerance(&self, new_edge: &Edge, new_tolerance: f64, tolerance: &mut f64) {
        if new_tolerance > *tolerance {
            *tolerance = new_tolerance;
            self.state
                .write()
                .updated_edges
                .push(new_edge.clone().into_shape());
        }
    }
}

impl BRepToolsModification for NurbsConvertModification {
    fn new_surface(&self, face: &Face, result: &mut NewSurface) -> Result<bool, Failure> {
        self.converted_surface(face, result)
    }

    fn new_triangulation(
        &self,
        face: &Face,
        triangulation: &mut Handle<Triangulation>,
    ) -> Result<bool, Failure> {
        self.converted_triangulation(face, triangulation)
    }

    /// A degenerated edge gets a null curve; a B-spline or Bezier curve is copied when
    /// a p-curve of the edge needs a conversion (else unchanged, false); any other
    /// curve is trimmed to the edge range (a periodic curve over one period is kept
    /// whole) and converted, its knots reparametrised back onto the range when the
    /// conversion moved it by more than the curve resolution. False for an edge
    /// without curve.
    fn new_curve(&self, edge: &Edge, result: &mut NewCurve) -> Result<bool, Failure> {
        result.tolerance = BRepTool::edge_tolerance(edge)?;
        if BRepTool::degenerated(edge)? {
            result.curve = Handle::default();
            result.location = Location::default();
            return Ok(true);
        }
        let found = BRepTool::curve_with_location(edge)?;
        result.location = found.location;
        let Some(source) = found.curve.get() else {
            result.location = Location::default();
            return Ok(false);
        };
        let exact = source.dynamic_type().name();
        if exact == "Geom_BSplineCurve" || exact == "Geom_BezierCurve" {
            if !is_convert(edge)? {
                return Ok(false);
            }
            result.curve = source.copy()?.cast();
            return Ok(true);
        }
        let converted = convert_curve(&found.curve, (found.first, found.last), result.tolerance)?;
        result.curve = converted.clone().cast();
        if !self.is_recorded(&found.curve.cast()) {
            self.record(found.curve.cast(), converted.cast());
        }
        Ok(true)
    }

    fn new_polygon(&self, edge: &Edge, polygon: &mut Handle<Polygon3D>) -> Result<bool, Failure> {
        self.converted_polygon(edge, polygon)
    }

    fn new_polygon_on_triangulation(
        &self,
        edge: &Edge,
        face: &Face,
        polygon: &mut Handle<PolygonOnTriangulation>,
    ) -> Result<bool, Failure> {
        self.converted_polygon_on_triangulation(edge, face, polygon)
    }

    /// Always false: the points are kept.
    fn new_point(
        &self,
        _vertex: &Vertex,
        _point: &mut Point3,
        _tolerance: &mut f64,
    ) -> Result<bool, Failure> {
        Ok(false)
    }

    fn new_curve2d(
        &self,
        edge: &Edge,
        face: &Face,
        new_edge: &Edge,
        new_face: &Face,
        curve: &mut Handle<dyn Geom2dCurve>,
        tolerance: &mut f64,
    ) -> Result<bool, Failure> {
        self.converted_curve2d((edge, face), (new_edge, new_face), curve, tolerance)
    }

    /// The vertex tolerance and, off a degenerated edge, the parameter of the vertex
    /// point projected on the converted curve (the old parameter as the start); false
    /// when the curve was not converted or the projection is farther than the
    /// tolerance.
    fn new_parameter(
        &self,
        vertex: &Vertex,
        edge: &Edge,
        parameter: &mut f64,
        tolerance: &mut f64,
    ) -> Result<bool, Failure> {
        *tolerance = BRepTool::vertex_tolerance(vertex)?;
        if BRepTool::degenerated(edge)? {
            return Ok(false);
        }
        let point = BRepTool::pnt(vertex)?;
        *parameter = BRepTool::parameter(vertex, edge)?;
        let conversions = self.conversions();
        let (new_curve, first, last) = new_curve_of(&conversions, edge)?;
        if new_curve.is_null() {
            return Ok(false);
        }
        new_parameter_on_curve(point, &new_curve, (first, last), *tolerance, parameter)
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

/// The curve trimmed to `[first, last]` (unless it is periodic and the range is one
/// period within `0.1 * tolerance`, ending at most at the period), converted to a
/// B-spline (TgtThetaOver2), whose non-periodic knots are brought back onto the range
/// when the conversion moved its bounds by more than its resolution.
fn convert_curve(
    curve: &Handle<dyn GeomCurve>,
    (first, last): (f64, f64),
    tolerance: f64,
) -> Result<Handle<GeomBSplineCurve>, Failure> {
    let Some(source) = curve.get() else {
        return Err(crate::brep::errors::null_object(
            "BRepTools_NurbsConvertModification::NewCurve",
        ));
    };
    let tol_par = tolerance * 0.1;
    let mut trimmed = curve.clone();
    let whole_period = source.is_periodic() && {
        let period = source.period()?;
        ((last - first).abs() - period).abs() <= tol_par && last <= period
    };
    if !whole_period {
        trimmed = Handle::new(GeomTrimmedCurve::new(curve, first, last, true, true)?).cast();
    }
    let converted = curve_to_bspline_curve(&trimmed, ParameterisationType::TgtThetaOver2)?;
    let Some(bspline) = converted.get() else {
        return Ok(converted);
    };
    let (new_first, new_last) = (bspline.first_parameter(), bspline.last_parameter());
    if !bspline.is_periodic() {
        let u_tol = bspline.resolution(tolerance)?;
        if (first - new_first).abs() > u_tol || (last - new_last).abs() > u_tol {
            let mut knots: Array1<'static, f64> = bspline.knots();
            bsplclib::reparametrize(first, last, &mut knots)?;
            bspline.set_knots(&knots)?;
        }
    }
    Ok(converted)
}

impl NurbsConvertModification {
    /// The embedded `BRepTools_CopyModification`, whose members the mesh members call.
    fn copy_modification(&self) -> &CopyModification {
        &self.parent
    }
}
