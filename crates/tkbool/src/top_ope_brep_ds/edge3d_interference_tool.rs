// Rust translation of
// ModelingAlgorithms/TKBool/TopOpeBRepDS/TopOpeBRepDS_Edge3dInterferenceTool.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TopOpeBRepDS_Edge3dInterferenceTool`: the 3d transition of an edge at a point
//! where it crosses several faces bounded by another edge, accumulated with a surface
//! transition around that edge.
use super::export::config::dir;
use super::export::iref;
use super::{
    TopOpeBRepDSCurvePointInterference, TopOpeBRepDSEdgeVertexInterference,
    TopOpeBRepDSInterference, TopOpeBRepDSKind,
};
use crate::top_ope_brep_tool::{
    fun_tool_nggeom_f, fun_tool_nggeom_f_on_edge, fun_tool_orient_e_in_f_forward,
    fun_tool_par_v_on_e, fun_tool_proj_pone, tggeom_e,
};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_adaptor::{BRepAdaptorCurve, BRepAdaptorCurve2d};
use rustycad_tkbrep::topods::{self, Edge, Face, Shape};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum, State};
use rustycad_tkgeomalgo::toptrans::SurfaceTransition;
use rustycad_tkmath::gp::{Direction3, Point3, Vector3};

/// `FUN_Raise()` (static): Standard_ProgramError.
fn raise() -> Failure {
    Failure::with_type(
        "Standard_ProgramError",
        "Edge3dInterferenceTool",
        "TopOpeBRepDS_Edge3dInterferenceTool",
    )
}

/// `FUN_hasparam(I, paronE)` (static): the parameter of a curve/point (POINT) or
/// edge/vertex (VERTEX) interference; `None` when of another type; 0 for other kinds.
fn hasparam(i: &Handle<TopOpeBRepDSInterference>) -> Result<Option<f64>, Failure> {
    match iref(i)?.geometry_type() {
        TopOpeBRepDSKind::Point => {
            let cpi: Handle<TopOpeBRepDSCurvePointInterference> = i.cast();
            Ok(cpi.get().map(|c| c.parameter()))
        }
        TopOpeBRepDSKind::Vertex => {
            let evi: Handle<TopOpeBRepDSEdgeVertexInterference> = i.cast();
            Ok(evi.get().map(|e| e.parameter()))
        }
        _ => Ok(Some(0.)),
    }
}

/// `FUN_paronOOE(OOE, IsVertex, VonOO, P3d, paronOOE)` (static): the parameter on
/// `ooe` of the vertex `von_oo` (status > 1) or of the projection of `p3d` (within 1000
/// times its tolerance, at most 1e-2).
fn paron_ooe(
    ooe: &Edge,
    is_vertex: i32,
    von_oo: &Shape,
    p3d: Point3,
) -> Result<Option<f64>, Failure> {
    if is_vertex > 1 {
        return fun_tool_par_v_on_e(&topods::vertex(von_oo)?, ooe);
    }
    let (mut par, mut dist) = (0., 0.);
    let ok = fun_tool_proj_pone(p3d, ooe, &mut par, &mut dist)?;
    let tol = (BRepTool::edge_tolerance(ooe)? * 1.0e3).min(1.0e-2);
    Ok((ok && dist <= tol).then_some(par))
}

/// `FUN_keepIonF(tgref, parE, E, F, tola)` (static): `e` neither tangent to the
/// reference tangent nor along it in `f`.
fn keep_i_on_f(tgref: Vector3, par_e: f64, e: &Edge, f: &Face, tola: f64) -> Result<bool, Failure> {
    let Some(tmp) = tggeom_e(par_e, e)? else {
        return Ok(false);
    };
    let tg_e = dir(tmp)?;
    if (1. - tgref.dot(Vector3::from(tg_e)).abs()).abs() < tola {
        return Ok(false);
    }
    // natively gp_Dir(dd) is built before the check: a missing normal throws
    let ng_f = dir(fun_tool_nggeom_f_on_edge(par_e, e, f)?.unwrap_or_default())?;
    let prod = tgref
        .crossed(Vector3::from(tg_e))
        .dot(Vector3::from(ng_f))
        .abs();
    Ok((1. - prod).abs() >= tola)
}

/// The edge 3d interference tool; natively the vertex status and tolerance start
/// uninitialized (0 here); `let mut t = TopOpeBRepDSEdge3dInterferenceTool::new();`.
#[derive(Clone)]
pub struct TopOpeBRepDSEdge3dInterferenceTool {
    my_face_oriented: i32,
    my_tool: SurfaceTransition,
    my_tole: f64,
    myrefdef: bool,
    my_is_vertex: i32,
    my_von_oo: Shape,
    my_p3d: Point3,
    my_tgtref: Direction3,
}

impl Default for TopOpeBRepDSEdge3dInterferenceTool {
    fn default() -> Self {
        Self::new()
    }
}

/// Whether the transition of `i` is 2d (FORWARD or REVERSED seen from IN).
fn is2d(i: &Handle<TopOpeBRepDSInterference>) -> Result<bool, Failure> {
    let o = iref(i)?
        .transition()
        .orientation(State::In, ShapeEnum::Face);
    Ok(o == Orientation::Forward || o == Orientation::Reversed)
}

impl TopOpeBRepDSEdge3dInterferenceTool {
    /// `TopOpeBRepDS_Edge3dInterferenceTool()`; `TopOpeBRepDSEdge3dInterferenceTool::new()`.
    pub fn new() -> Self {
        Self {
            my_face_oriented: 0,
            my_tool: SurfaceTransition::new(),
            my_tole: 0.,
            myrefdef: false,
            my_is_vertex: 0,
            my_von_oo: Shape::default(),
            my_p3d: Point3::default(),
            my_tgtref: Direction3::default(),
        }
    }

    /// `InitPointVertex(IsVertex, VonOO)`: the geometry status (0 point, 1 vertex on the
    /// reference edge, 2 on the other edge, 3 on both) and the vertex on the other edge;
    /// `t.init_point_vertex(0, &Shape::default())`.
    pub fn init_point_vertex(&mut self, is_vertex: i32, von_oo: &Shape) {
        self.my_is_vertex = is_vertex;
        if is_vertex > 1 {
            self.my_von_oo = von_oo.clone();
        }
    }

    /// `Init(Eref, E, F, I)`: the reference tangent at the point of `i` on `eref` and the
    /// frame around `e`, unless `e` is tangent to `eref` or along it in `f` with a 2d
    /// transition; Standard_ProgramError when a parameter or tangent is missing;
    /// `t.init(&eref, &e, &f, &i)?`.
    pub fn init(
        &mut self,
        eref: &Shape,
        e: &Shape,
        f: &Shape,
        i: &Handle<TopOpeBRepDSInterference>,
    ) -> Result<(), Failure> {
        let (eeref, ee, ff) = (topods::edge(eref)?, topods::edge(e)?, topods::face(f)?);
        self.myrefdef = false;
        self.my_tole = precision::angular();
        let pref = hasparam(i)?.ok_or_else(raise)?;
        self.my_p3d = BRepAdaptorCurve::with_edge(&eeref)?.value(pref)?;
        let tgref = dir(tggeom_e(pref, &eeref)?.ok_or_else(raise)?)?;
        let p_oo =
            paron_ooe(&ee, self.my_is_vertex, &self.my_von_oo, self.my_p3d)?.ok_or_else(raise)?;
        let tg_oo = dir(tggeom_e(p_oo, &ee)?.ok_or_else(raise)?)?;
        let dot = 1. - tgref.dot(tg_oo).abs();
        if dot.abs() < precision::confusion() {
            return Ok(());
        }
        if !keep_i_on_f(Vector3::from(tgref), p_oo, &ee, &ff, self.my_tole)? && is2d(i)? {
            return Ok(());
        }
        self.myrefdef = true;
        self.my_face_oriented = iref(i)?.transition().index()?;
        self.my_tgtref = tgref;
        let norm = tg_oo.crossed(tgref)?;
        self.my_tool.reset_line(tg_oo, norm)
    }

    /// `Add(Eref, E, F, I)`: the face `f` around `e` compared (after `Init` when the
    /// reference is undefined); `t.add(&eref, &e, &f, &i)?`.
    pub fn add(
        &mut self,
        eref: &Shape,
        e: &Shape,
        f: &Shape,
        i: &Handle<TopOpeBRepDSInterference>,
    ) -> Result<(), Failure> {
        if !self.myrefdef {
            self.init(eref, e, f, i)?;
        }
        if !self.myrefdef {
            return Ok(());
        }
        let (ee, ff) = (topods::edge(e)?, topods::face(f)?);
        let Some(p_oo) = paron_ooe(&ee, self.my_is_vertex, &self.my_von_oo, self.my_p3d)? else {
            return Ok(());
        };
        let uv = BRepAdaptorCurve2d::with_edge_on_face(&ee, &ff)?.value(p_oo)?;
        if !keep_i_on_f(Vector3::from(self.my_tgtref), p_oo, &ee, &ff, self.my_tole)? && is2d(i)? {
            return Ok(());
        }
        let oriloc = iref(i)?
            .transition()
            .orientation(State::In, ShapeEnum::Face);
        let (ok, oritan) = fun_tool_orient_e_in_f_forward(&ee, &ff)?;
        if !ok {
            return Ok(());
        }
        let norm = dir(fun_tool_nggeom_f(uv, &ff)?)?;
        self.my_tool
            .compare_plane(self.my_tole, norm, oriloc, oritan)
    }

    /// `Transition(I)`: the face of reference as support of `i` and the accumulated
    /// states as its transition (on FACE shapes); `t.transition(&i)?`.
    pub fn transition(&self, i: &Handle<TopOpeBRepDSInterference>) -> Result<(), Failure> {
        let ii = iref(i)?;
        ii.set_support(self.my_face_oriented);
        let (stb, sta) = (self.my_tool.state_before(), self.my_tool.state_after());
        ii.change_transition(|t| t.set(stb, sta, ShapeEnum::Face, ShapeEnum::Face));
        Ok(())
    }
}
