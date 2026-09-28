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
// Rust translation of ModelingData/TKG3d/Adaptor3d/Adaptor3d_TopolTool.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Adaptor3d_TopolTool`: the topology of an adapted surface seen as its
//! parametric rectangle: the boundary lines (restrictions) and their vertices,
//! point classification, and the sample points of the intersection algorithms.
mod bspl_samples;
mod classify;
mod restrictions;
mod samples;
use super::h_vertex::Adaptor3dHVertex;
use super::surface::Adaptor3dSurface;
use crate::geom::errors::{domain, not_implemented, null_object};
use crate::top_abs::{Orientation, State};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkg2d::adaptor2d::{Adaptor2dCurve2d, Adaptor2dLine2d};
use rustycad_tkmath::gp::{Cone3, Point2, Point3};

/// The fields of one `Adaptor3d_TopolTool` (the native protected and private data).
pub(super) struct TopolState {
    pub(super) surface: Handle<dyn Adaptor3dSurface>,
    pub(super) nb_samples_u: i32,
    pub(super) nb_samples_v: i32,
    pub(super) u_pars: Option<Array1<'static, f64>>,
    pub(super) v_pars: Option<Array1<'static, f64>>,
    pub(super) nb_restr: i32,
    pub(super) id_restr: i32,
    pub(super) uinf: f64,
    pub(super) usup: f64,
    pub(super) vinf: f64,
    pub(super) vsup: f64,
    pub(super) restr: [Handle<Adaptor2dLine2d>; 4],
    pub(super) nb_vtx: i32,
    pub(super) id_vtx: i32,
    pub(super) vtx: [Handle<Adaptor3dHVertex>; 2],
}
impl TopolState {
    // The fields as the native allocation leaves them (zero-filled), then the
    // default constructor's sample counts.
    fn zeroed(nb_samples: i32) -> Self {
        Self {
            surface: Handle::default(),
            nb_samples_u: nb_samples,
            nb_samples_v: nb_samples,
            u_pars: None,
            v_pars: None,
            nb_restr: 0,
            id_restr: 0,
            uinf: 0.0,
            usup: 0.0,
            vinf: 0.0,
            vsup: 0.0,
            restr: Default::default(),
            nb_vtx: 0,
            id_vtx: 0,
            vtx: Default::default(),
        }
    }
}

/// The parametric rectangle of an adapted surface as a topology
/// (`Adaptor3d_TopolTool`): up to four boundary lines, the two vertices of a
/// restriction, point classification and sample points.
///
/// Every member takes `&self`; the iterators and sample caches live behind one lock.
/// The native class has virtual members that BRepTopAdaptor overrides; that
/// subclass is not ported, so the members are inherent here;
/// `Adaptor3dTopolTool::with_surface(&surface)?`.
pub struct Adaptor3dTopolTool {
    parent: StandardTransient,
    state: SharedState<TopolState>,
}
impl Default for Adaptor3dTopolTool {
    fn default() -> Self {
        Self {
            parent: StandardTransient::default(),
            state: SharedState::new(TopolState::zeroed(-1)),
        }
    }
}
impl_transient!(
    Adaptor3dTopolTool,
    "Adaptor3d_TopolTool",
    StandardTransient,
    parent
);

impl Adaptor3dTopolTool {
    /// A tool on `surface` (`Adaptor3d_TopolTool(S)`); the fields the constructor does
    /// not set start zeroed (so NbSamplesV reads 0 until the samples are computed);
    /// `Adaptor3dTopolTool::with_surface(&surface)?`.
    pub fn with_surface(surface: &Handle<dyn Adaptor3dSurface>) -> Result<Self, Failure> {
        let tool = Self {
            parent: StandardTransient::default(),
            state: SharedState::new(TopolState::zeroed(0)),
        };
        tool.initialize_surface(surface)?;
        Ok(tool)
    }
    /// `Initialize()`: Standard_NotImplemented; `tool.initialize()`.
    pub fn initialize(&self) -> Result<(), Failure> {
        Err(not_implemented("Adaptor3d_TopolTool::Initialize ()"))
    }
    /// Restart the restriction iterator (`Init`); `tool.init()`.
    pub fn init(&self) {
        self.state.write().id_restr = 0;
    }
    /// Whether a restriction remains (`More`); `tool.more()`.
    pub fn more(&self) -> bool {
        let state = self.state.read();
        state.id_restr < state.nb_restr
    }
    /// The current restriction; Standard_DomainError past the last one (`Value`);
    /// `tool.value()?`.
    pub fn value(&self) -> Result<Handle<dyn Adaptor2dCurve2d>, Failure> {
        let state = self.state.read();
        if state.id_restr >= state.nb_restr {
            return Err(domain(&format!(
                "Adaptor3d_TopolTool::Value: restriction {} of {}; expected More()",
                state.id_restr, state.nb_restr
            )));
        }
        Ok(state.restr[state.id_restr as usize].clone().cast())
    }
    /// Move to the next restriction (`Next`); `tool.next()`.
    pub fn next(&self) {
        self.state.write().id_restr += 1;
    }
    /// Restart the vertex iterator (`InitVertexIterator`); `tool.init_vertex_iterator()`.
    pub fn init_vertex_iterator(&self) {
        self.state.write().id_vtx = 0;
    }
    /// Whether a vertex remains (`MoreVertex`); `tool.more_vertex()`.
    pub fn more_vertex(&self) -> bool {
        let state = self.state.read();
        state.id_vtx < state.nb_vtx
    }
    /// The current vertex; Standard_DomainError past the last one (`Vertex`);
    /// `tool.vertex()?`.
    pub fn vertex(&self) -> Result<Handle<Adaptor3dHVertex>, Failure> {
        let state = self.state.read();
        if state.id_vtx >= state.nb_vtx {
            return Err(domain(&format!(
                "Adaptor3d_TopolTool::Vertex: vertex {} of {}; expected MoreVertex()",
                state.id_vtx, state.nb_vtx
            )));
        }
        Ok(state.vtx[state.id_vtx as usize].clone())
    }
    /// Move to the next vertex (`NextVertex`); `tool.next_vertex()`.
    pub fn next_vertex(&self) {
        self.state.write().id_vtx += 1;
    }
    /// FORWARD for every restriction (`Orientation(C)`); `tool.curve_orientation(&c)`.
    pub fn curve_orientation(&self, _curve: &Handle<dyn Adaptor2dCurve2d>) -> Orientation {
        Orientation::Forward
    }
    /// The orientation of a vertex (`Orientation(V)`); `tool.vertex_orientation(&v)?`.
    pub fn vertex_orientation(
        &self,
        vertex: &Handle<Adaptor3dHVertex>,
    ) -> Result<Orientation, Failure> {
        Ok(live_vertex(vertex)?.orientation())
    }
    /// Whether two vertices are the same (`Identical`); `tool.identical(&v1, &v2)?`.
    pub fn identical(
        &self,
        first: &Handle<Adaptor3dHVertex>,
        second: &Handle<Adaptor3dHVertex>,
    ) -> Result<bool, Failure> {
        live_vertex(first)?.is_same(second)
    }
    /// Whether the restrictions carry 3D data: never (`Has3d`); `tool.has_3d()`.
    pub fn has_3d(&self) -> bool {
        false
    }
    /// Standard_DomainError: no 3D representation (`Tol3d(C)`); `tool.curve_tol_3d(&c)`.
    pub fn curve_tol_3d(&self, _curve: &Handle<dyn Adaptor2dCurve2d>) -> Result<f64, Failure> {
        Err(no_3d())
    }
    /// Standard_DomainError: no 3D representation (`Tol3d(V)`); `tool.vertex_tol_3d(&v)`.
    pub fn vertex_tol_3d(&self, _vertex: &Handle<Adaptor3dHVertex>) -> Result<f64, Failure> {
        Err(no_3d())
    }
    /// Standard_DomainError: no 3D representation (`Pnt`); `tool.pnt(&v)`.
    pub fn pnt(&self, _vertex: &Handle<Adaptor3dHVertex>) -> Result<Point3, Failure> {
        Err(no_3d())
    }
    /// Whether a bound of the rectangle is infinite (`DomainIsInfinite`);
    /// `tool.domain_is_infinite()`.
    pub fn domain_is_infinite(&self) -> bool {
        use rustycad_tkernel::precision::{is_negative_infinite, is_positive_infinite};
        let state = self.state.read();
        is_negative_infinite(state.uinf)
            || is_positive_infinite(state.usup)
            || is_negative_infinite(state.vinf)
            || is_positive_infinite(state.vsup)
    }
    /// The edge behind the tool: none for a bare surface (`Edge`, a null `void*`);
    /// `tool.edge()`.
    pub fn edge(&self) -> Option<std::ptr::NonNull<std::ffi::c_void>> {
        None
    }
    /// Classify `point` against the rectangle (`Classify`; the periodic flag is
    /// unused natively); `tool.classify(p, 1e-7, true)`.
    pub fn classify(&self, point: Point2, tolerance: f64, _reframe_on_periodic: bool) -> State {
        classify::classify(&self.state.read(), point, tolerance)
    }
    /// Whether `point` is on the boundary of the rectangle (`IsThePointOn`);
    /// `tool.is_the_point_on(p, 1e-7, true)`.
    pub fn is_the_point_on(
        &self,
        point: Point2,
        tolerance: f64,
        _reframe_on_periodic: bool,
    ) -> bool {
        classify::is_the_point_on(&self.state.read(), point, tolerance)
    }
    /// The parameters of the apex of `cone` (`GetConeApexParam`);
    /// `let (u, v) = Adaptor3dTopolTool::get_cone_apex_param(&cone);`.
    pub fn get_cone_apex_param(cone: &Cone3) -> (f64, f64) {
        samples::cone_apex_param(cone)
    }
}

impl Adaptor3dTopolTool {
    // The surface, read without keeping the lock (the evaluations call other
    // objects); natively a null one is dereferenced.
    pub(super) fn live_surface(&self) -> Result<Handle<dyn Adaptor3dSurface>, Failure> {
        let surface: Handle<dyn Adaptor3dSurface> = self.state.read().surface.clone();
        if surface.is_null() {
            return Err(null_object(
                "Adaptor3d_TopolTool: no surface; expected Initialize(S) first",
            ));
        }
        Ok(surface)
    }
}

fn no_3d() -> Failure {
    domain("Adaptor3d_TopolTool: has no 3d representation")
}

// The vertex behind a handle; natively a null one is dereferenced.
fn live_vertex(vertex: &Handle<Adaptor3dHVertex>) -> Result<&Adaptor3dHVertex, Failure> {
    vertex
        .get()
        .ok_or_else(|| null_object("Adaptor3d_TopolTool: null vertex; expected a vertex"))
}
