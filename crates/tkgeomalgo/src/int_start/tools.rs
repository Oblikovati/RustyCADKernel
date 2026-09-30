// Copyright (c) 1995-1999 Matra Datavision
// Copyright (c) 1999-2014 OPEN CASCADE SAS
//
// This file is part of Open CASCADE Technology software library.
//
// This library is free software; you can redistribute it and/or modify it under
// the terms of the GNU Lesser General Public License version 2.1 as published
// by the Free Software Foundation, with special exception defined in the file
// OCCT_LGPL_EXCEPTION.txt. Consult the file LICENSE_LGPL_21.txt included in OCCT
// distribution for complete text of the license and disclaimer of any warranty.
//
// Alternatively, this file may be used under the terms of Open CASCADE
// commercial license or contractual agreement.
// Rust translation of the macro types of ModelingAlgorithms/TKGeomAlgo/IntStart/
// IntStart_SearchOnBoundaries.gxx and IntStart_SearchInside.gxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The type parameters of the two IntStart templates (docs/geomalgo-design.md section
//! 12): the members the bodies call on `TheTopolTool`, `TheSOBTool`, `TheFunction`,
//! `IntStart_ThePathPoint` and `IntStart_TheSegment`, under their native names, and the
//! bundles an instance names.
use crate::int_imp::{IntImpZerImpFunc, TheISurfaceTool, ThePSurfaceTool};
use crate::int_surf::IntSurfQuadric;
use rustycad_tkbrep::topods::Edge;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg3d::adaptor3d::{Adaptor3dHVertex, Adaptor3dSurface, Adaptor3dTopolTool};
use rustycad_tkg3d::top_abs::{Orientation, State};
use rustycad_tkmath::gp::{Direction2, Point2, Point3, Vector3};
use rustycad_tkmath::math::{FunctionSetWithDerivatives, FunctionWithDerivative, Vector};

/// `TheArc`: a restriction of the domain, `occ::handle<Adaptor2d_Curve2d>` in every
/// instance.
pub type TheArc = Handle<dyn Adaptor2dCurve2d>;
/// `TheVertex`: a vertex of the domain, `occ::handle<Adaptor3d_HVertex>` in every
/// instance.
pub type TheVertex = Handle<Adaptor3dHVertex>;

/// `TheTopolTool`: the domain of the surface, whose native class is the virtual
/// `Adaptor3d_TopolTool`. Every member takes `&self`, as the domain is shared through a
/// handle and keeps its iterators inside.
///
/// `Adaptor3dTopolTool` implements it; `domain.nb_samples()?`.
pub trait TheTopolTool {
    /// `NbSamplesU()`.
    fn nb_samples_u(&self) -> Result<i32, Failure>;
    /// `NbSamplesV()`.
    fn nb_samples_v(&self) -> Result<i32, Failure>;
    /// `NbSamples()`.
    fn nb_samples(&self) -> Result<i32, Failure>;
    /// `SamplePoint(Index, P2d, P3d)`.
    fn sample_point(&self, index: i32) -> Result<(Point2, Point3), Failure>;
    /// `Classify(P, Tol, ReacdreOnPeriodic)`.
    fn classify(&self, point: Point2, tol: f64, reframe: bool) -> Result<State, Failure>;
    /// `Init()`: restarts the arc iterator.
    fn init(&self);
    /// `More()`.
    fn more(&self) -> bool;
    /// `Value()`: the current arc.
    fn value(&self) -> Result<TheArc, Failure>;
    /// `Next()`.
    fn next(&self);
    /// `Initialize(A)`: the vertex iterator runs on the arc `arc`.
    fn initialize_arc(&self, arc: &TheArc) -> Result<(), Failure>;
    /// `InitVertexIterator()`.
    fn init_vertex_iterator(&self);
    /// `MoreVertex()`.
    fn more_vertex(&self) -> bool;
    /// `Vertex()`: the current vertex.
    fn vertex(&self) -> Result<TheVertex, Failure>;
    /// `NextVertex()`.
    fn next_vertex(&self);
    /// `Identical(V1, V2)`.
    fn identical(&self, first: &TheVertex, second: &TheVertex) -> Result<bool, Failure>;
    /// `Edge()`: the edge of the current arc, where the native member returns a `void*`
    /// to a `TopoDS_Edge` (or null).
    fn edge(&self) -> Option<Edge>;
    /// `Has3d()`: whether the restrictions carry 3d data (false for the base
    /// `Adaptor3d_TopolTool`). IntPatch_RstInt reads it.
    fn has_3d(&self) -> bool {
        false
    }
    /// `Tol3d(C)`: the 3d tolerance of an arc; the base domain raises
    /// Standard_DomainError (no 3d representation).
    fn tol_3d_arc(&self, _arc: &TheArc) -> Result<f64, Failure> {
        Err(no_3d_representation())
    }
    /// `Tol3d(V)`: the 3d tolerance of a vertex; the base domain raises
    /// Standard_DomainError.
    fn tol_3d_vertex(&self, _vertex: &TheVertex) -> Result<f64, Failure> {
        Err(no_3d_representation())
    }
    /// `Orientation(C)`: the orientation of an arc (FORWARD for the base domain).
    /// IntPatch_ImpPrmIntersection reads it.
    fn arc_orientation(&self, _arc: &TheArc) -> Orientation {
        Orientation::Forward
    }
    /// `Orientation(V)`: the orientation of a vertex (its own one for the base domain).
    fn vertex_orientation(&self, vertex: &TheVertex) -> Result<Orientation, Failure> {
        vertex.get().map(|v| v.orientation()).ok_or_else(|| {
            Failure::with_message(
                "Standard_NullObject",
                "Adaptor3d_TopolTool::Orientation() - the vertex handle is null; expected a vertex",
            )
        })
    }
}

/// The Standard_DomainError of `Tol3d` on a domain without 3d data.
fn no_3d_representation() -> Failure {
    Failure::with_message(
        "Standard_DomainError",
        "Adaptor3d_TopolTool: has no 3d representation",
    )
}

/// `TheSOBTool`: the static members the boundary search calls on the arcs and vertices.
///
/// `IntPatch_HInterTool` implements it when IntPatch is ported; `T::bounds(&arc)?`.
pub trait TheSOBTool {
    /// `Parameter(V, A)`: the parameter of the vertex on the arc.
    fn parameter(vertex: &TheVertex, arc: &TheArc) -> Result<f64, Failure>;
    /// `Tolerance(V, A)`: the parametric tolerance of the vertex on the arc.
    fn tolerance(vertex: &TheVertex, arc: &TheArc) -> Result<f64, Failure>;
    /// `HasBeenSeen(A)`: whether the arc was already processed.
    fn has_been_seen(arc: &TheArc) -> Result<bool, Failure>;
    /// `Bounds(A, Pdeb, Pfin)`.
    fn bounds(arc: &TheArc) -> Result<(f64, f64), Failure>;
    /// `NbPoints(A)`: the known points of a seen arc.
    fn nb_points(arc: &TheArc) -> Result<i32, Failure>;
    /// `Value(A, i, Pt, Tol, U)`: a known point, its tolerance and its parameter.
    fn value(arc: &TheArc, index: i32) -> Result<(Point3, f64, f64), Failure>;
    /// `IsVertex(A, i)`.
    fn is_vertex(arc: &TheArc, index: i32) -> Result<bool, Failure>;
    /// `Vertex(A, i, V)`.
    fn vertex(arc: &TheArc, index: i32) -> Result<TheVertex, Failure>;
    /// `NbSegments(A)`: the known segments of a seen arc.
    fn nb_segments(arc: &TheArc) -> Result<i32, Failure>;
    /// `HasFirstPoint(A, i, Index)`: the index of the first point, when there is one.
    fn has_first_point(arc: &TheArc, index: i32) -> Result<Option<i32>, Failure>;
    /// `HasLastPoint(A, i, Index)`: the index of the last point, when there is one.
    fn has_last_point(arc: &TheArc, index: i32) -> Result<Option<i32>, Failure>;
    /// `IsAllSolution(A)`.
    fn is_all_solution(arc: &TheArc) -> Result<bool, Failure>;
}

/// `TheFunction` of the boundary search: the signed distance of the quadric along an
/// arc, a `math_FunctionWithDerivative`. The direct calls are fallible where the native
/// ones can raise; a failure caught by a solver callback is kept and raised by
/// [`TheArcFunction::take_failure`].
///
/// `IntPatch_ArcFunction` implements it when IntPatch is ported;
/// `function.checked_value(0.5)?`.
pub trait TheArcFunction: FunctionWithDerivative {
    /// `Set(A)`: the arc to follow; clears the stored points.
    fn set_arc(&mut self, arc: &TheArc);
    /// `Value(X, F)`: `None` when the native member returns false (F unchanged).
    fn checked_value(&mut self, x: f64) -> Result<Option<f64>, Failure>;
    /// `Values(X, F, D)`: `None` when the native member returns false.
    fn checked_values(&mut self, x: f64) -> Result<Option<(f64, f64)>, Failure>;
    /// `NbSamples()`.
    fn nb_samples(&self) -> Result<i32, Failure>;
    /// `GetStateNumber()`: stores the last point and returns its index.
    fn get_state_number(&mut self) -> i32;
    /// `Valpoint(Index)`: a stored point; Standard_OutOfRange outside the stored ones.
    fn valpoint(&self, index: i32) -> Result<Point3, Failure>;
    /// `Quadric()`.
    fn quadric(&self) -> &IntSurfQuadric;
    /// `Surface()`.
    fn surface(&self) -> Handle<dyn Adaptor3dSurface>;
    /// `LastComputedPoint()`.
    fn last_computed_point(&self) -> Point3;
    /// The first failure a solver callback caught since the last call.
    fn take_failure(&mut self) -> Result<(), Failure>;
}

/// `IntStart_ThePathPoint`: a solution point on an arc, on a vertex or new.
///
/// `IntPatch_ThePathPointOfTheSOnBounds` implements it when IntPatch is ported;
/// `P::on_arc(p, tol, &arc, 0.5)`.
pub trait ThePathPoint: Clone + Default {
    /// The constructor on a vertex (`IsNew()` false).
    fn on_vertex(p: Point3, tol: f64, vertex: &TheVertex, arc: &TheArc, param: f64) -> Self;
    /// The constructor of a new point (`IsNew()` true).
    fn on_arc(p: Point3, tol: f64, arc: &TheArc, param: f64) -> Self;
    /// `SetValue(P, Tol, V, A, Parameter)`.
    fn set_value_on_vertex(
        &mut self,
        p: Point3,
        tol: f64,
        vertex: &TheVertex,
        arc: &TheArc,
        param: f64,
    );
    /// `SetValue(P, Tol, A, Parameter)`: a new point; the vertex is kept, as natively.
    fn set_value_on_arc(&mut self, p: Point3, tol: f64, arc: &TheArc, param: f64);
    /// `IsNew()`.
    fn is_new(&self) -> bool;
    /// `Vertex()`: Standard_DomainError on a new point.
    fn vertex(&self) -> Result<TheVertex, Failure>;
    /// `Arc()`.
    fn arc(&self) -> TheArc;
    /// `Parameter()`.
    fn parameter(&self) -> f64;
}

/// `IntStart_TheSegment`: a part of an arc that is solution.
///
/// `IntPatch_TheSegmentOfTheSOnBounds` implements it when IntPatch is ported;
/// `segment.set_limit_point(&point, true)`.
pub trait TheSegment: Clone + Default {
    /// The path point type of the segment ends.
    type PathPoint;
    /// `SetValue(A)`.
    fn set_value(&mut self, arc: &TheArc);
    /// `SetLimitPoint(V, First)`.
    fn set_limit_point(&mut self, point: &Self::PathPoint, first: bool);
}

/// The macro types of one instance of `IntStart_SearchOnBoundaries.gxx`.
///
/// `impl SearchOnBoundariesTypes for TheSOnBoundsTypes { type TopolTool = Adaptor3dTopolTool; ... }`.
pub trait SearchOnBoundariesTypes {
    /// `TheTopolTool`.
    type TopolTool: TheTopolTool + ?Sized;
    /// `TheSOBTool`.
    type SOBTool: TheSOBTool;
    /// `TheFunction`.
    type Function: TheArcFunction;
    /// `IntStart_ThePathPoint`.
    type PathPoint: ThePathPoint;
    /// `IntStart_TheSegment`.
    type Segment: TheSegment<PathPoint = Self::PathPoint>;
}

/// `TheFunction` of the inside search: the implicit function of `(u, v)` on the
/// parametric surface, a `math_FunctionSetWithDerivatives`, with the members the body
/// calls directly.
///
/// `IntPatch_TheSurfFunction` (`IntImpZerImpFunc`) implements it;
/// `function.checked_value(&x, &mut f)?`.
pub trait TheInsideFunction: FunctionSetWithDerivatives {
    /// `ThePSurface`.
    type Surface;
    /// `Set(PS)`.
    fn set_surface(&mut self, surface: &Self::Surface);
    /// `Tolerance()`.
    fn tolerance(&self) -> f64;
    /// `Value(X, F)`.
    fn checked_value(&mut self, x: &Vector<'_>, f: &mut Vector<'_>) -> Result<bool, Failure>;
    /// `Root()`: the value of the last evaluation.
    fn root(&self) -> f64;
    /// `IsTangent()` at the last evaluation.
    fn is_tangent(&mut self) -> Result<bool, Failure>;
    /// `Point()`: the surface point of the last evaluation.
    fn point(&self) -> Point3;
    /// `Direction3d()`.
    fn direction3d(&mut self) -> Result<Vector3, Failure>;
    /// `Direction2d()`.
    fn direction2d(&mut self) -> Result<Direction2, Failure>;
    /// The first failure a solver callback caught since the last call.
    fn take_failure(&mut self) -> Result<(), Failure>;
}

/// The macro types of one instance of `IntStart_SearchInside.gxx`.
///
/// `impl SearchInsideTypes for TheSearchInsideTypes { type SurfaceTool = HSurfaceTool; ... }`.
pub trait SearchInsideTypes {
    /// `ThePSurfaceTool`.
    type SurfaceTool: ThePSurfaceTool;
    /// `TheTopolTool`.
    type TopolTool: TheTopolTool + ?Sized;
    /// `TheFunction`, over the surfaces of the tool.
    type Function: TheInsideFunction<Surface = <Self::SurfaceTool as ThePSurfaceTool>::Surface>;
}

impl TheTopolTool for Adaptor3dTopolTool {
    fn nb_samples_u(&self) -> Result<i32, Failure> {
        Adaptor3dTopolTool::nb_samples_u(self)
    }
    fn nb_samples_v(&self) -> Result<i32, Failure> {
        Adaptor3dTopolTool::nb_samples_v(self)
    }
    fn nb_samples(&self) -> Result<i32, Failure> {
        Adaptor3dTopolTool::nb_samples(self)
    }
    fn sample_point(&self, index: i32) -> Result<(Point2, Point3), Failure> {
        Adaptor3dTopolTool::sample_point(self, index)
    }
    fn classify(&self, point: Point2, tol: f64, reframe: bool) -> Result<State, Failure> {
        Ok(Adaptor3dTopolTool::classify(self, point, tol, reframe))
    }
    fn init(&self) {
        Adaptor3dTopolTool::init(self);
    }
    fn more(&self) -> bool {
        Adaptor3dTopolTool::more(self)
    }
    fn value(&self) -> Result<TheArc, Failure> {
        Adaptor3dTopolTool::value(self)
    }
    fn next(&self) {
        Adaptor3dTopolTool::next(self);
    }
    fn initialize_arc(&self, arc: &TheArc) -> Result<(), Failure> {
        self.initialize_curve(arc)
    }
    fn init_vertex_iterator(&self) {
        Adaptor3dTopolTool::init_vertex_iterator(self);
    }
    fn more_vertex(&self) -> bool {
        Adaptor3dTopolTool::more_vertex(self)
    }
    fn vertex(&self) -> Result<TheVertex, Failure> {
        Adaptor3dTopolTool::vertex(self)
    }
    fn next_vertex(&self) {
        Adaptor3dTopolTool::next_vertex(self);
    }
    fn identical(&self, first: &TheVertex, second: &TheVertex) -> Result<bool, Failure> {
        Adaptor3dTopolTool::identical(self, first, second)
    }
    // The base class has no edge: its `Edge()` returns a null pointer.
    fn edge(&self) -> Option<Edge> {
        None
    }
    fn has_3d(&self) -> bool {
        Adaptor3dTopolTool::has_3d(self)
    }
    fn tol_3d_arc(&self, arc: &TheArc) -> Result<f64, Failure> {
        self.curve_tol_3d(arc)
    }
    fn tol_3d_vertex(&self, vertex: &TheVertex) -> Result<f64, Failure> {
        Adaptor3dTopolTool::vertex_tol_3d(self, vertex)
    }
    fn arc_orientation(&self, arc: &TheArc) -> Orientation {
        self.curve_orientation(arc)
    }
    fn vertex_orientation(&self, vertex: &TheVertex) -> Result<Orientation, Failure> {
        Adaptor3dTopolTool::vertex_orientation(self, vertex)
    }
}

impl<P: ThePSurfaceTool, I: TheISurfaceTool> TheInsideFunction for IntImpZerImpFunc<P, I> {
    type Surface = P::Surface;
    fn set_surface(&mut self, surface: &P::Surface) {
        IntImpZerImpFunc::set_surface(self, surface);
    }
    fn tolerance(&self) -> f64 {
        IntImpZerImpFunc::tolerance(self)
    }
    fn checked_value(&mut self, x: &Vector<'_>, f: &mut Vector<'_>) -> Result<bool, Failure> {
        IntImpZerImpFunc::value(self, x, f)
    }
    fn root(&self) -> f64 {
        IntImpZerImpFunc::root(self)
    }
    fn is_tangent(&mut self) -> Result<bool, Failure> {
        IntImpZerImpFunc::is_tangent(self)
    }
    fn point(&self) -> Point3 {
        IntImpZerImpFunc::point(self)
    }
    fn direction3d(&mut self) -> Result<Vector3, Failure> {
        IntImpZerImpFunc::direction3d(self)
    }
    fn direction2d(&mut self) -> Result<Direction2, Failure> {
        IntImpZerImpFunc::direction2d(self)
    }
    fn take_failure(&mut self) -> Result<(), Failure> {
        IntImpZerImpFunc::take_failure(self)
    }
}
