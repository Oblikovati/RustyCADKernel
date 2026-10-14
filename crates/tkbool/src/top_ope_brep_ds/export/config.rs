//! The geometric queries on edges and faces of the structure (FDS_Config3d,
//! FDS_getupperlower, FDS_LOIinfsup, FDS_parbefaft, FDS_stateEwithF2d, FUN_ds_shareG,
//! FUN_ds_mkTonFsdm, FUN_ds_oriEinF, FUN_ds_ONesd).
use super::{fds_idata, fds_parameter, has_same_domain, href};
use crate::top_ope_brep_ds::{
    ListOfInterference, TopOpeBRepDSConfig, TopOpeBRepDSDataStructure, TopOpeBRepDSHDataStructure,
    TopOpeBRepDSKind, TopOpeBRepDSPointIterator, TopOpeBRepDSTKI, TopOpeBRepDSTransition,
};
use crate::top_ope_brep_tool::{
    TopOpeBRepToolMakeTransition, TopOpeBRepToolTOOL, fsc_get_psc_with_reference,
    fsc_state_ponface, fun_tool_bounds, fun_tool_find_pin_e, fun_tool_getxx_ng,
    fun_tool_nggeom_f_on_edge, fun_tool_orient_e_in_f_forward, fun_tool_proj_pone,
    fun_tool_value_on_edge, tggeom_e,
};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{self, Edge, Face, Shape};
use rustycad_tkbrep::toptools::ShapeMapHasher;
use rustycad_tkernel::ncollection::IndexedMap;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::{self, Orientation, ShapeEnum, State};
use rustycad_tkmath::gp::{Direction3, Point2, Point3, Vector3};

/// `gp_Dir(V)`: the unit direction of a vector; Standard_ConstructionError for a null one.
pub(crate) fn dir(v: Vector3) -> Result<Direction3, Failure> {
    Direction3::from_xyz(*v.xyz())
}

/// The transition states of `TopOpeBRepTool_makeTransition` (`Initialize`, then
/// `SetRest` with a restriction edge, then `MkTonE`): `None` when a step fails.
pub(crate) fn mkt_states(
    e: &Edge,
    range: (f64, f64, f64),
    f: &Face,
    uv: Point2,
    factor: f64,
    rest: Option<(&Edge, f64)>,
) -> Result<Option<(State, State)>, Failure> {
    let mut mkt = TopOpeBRepToolMakeTransition::new();
    if !mkt.initialize(e, range, f, uv, factor)? {
        return Ok(None);
    }
    if let Some((es, pes)) = rest
        && !mkt.set_rest(es, pes)?
    {
        return Ok(None);
    }
    let (ok, stb, sta) = mkt.mk_ton_e()?;
    Ok(ok.then_some((stb, sta)))
}

/// `FDS_Config3d(E1, E2, c)`: SAMEORIENTED when the tangents of `e1` at an inner point
/// and of `e2` at its projection agree, else DIFFORIENTED; `None` when a tangent is
/// missing; `fds_config3d(&e1, &e2)?`.
pub fn fds_config3d(e1: &Shape, e2: &Shape) -> Result<Option<TopOpeBRepDSConfig>, Failure> {
    let (e1, e2) = (topods::edge(e1)?, topods::edge(e2)?);
    let (pe1_pnt, pe1) = fun_tool_find_pin_e(&e1)?;
    let ve1 = tggeom_e(pe1, &e1)?;
    let (mut pe2, mut de2) = (0., 0.);
    let ve2 = if fun_tool_proj_pone(pe1_pnt, &e2, &mut pe2, &mut de2)? {
        tggeom_e(pe2, &e2)?
    } else {
        None
    };
    let (Some(ve1), Some(ve2)) = (ve1, ve2) else {
        return Ok(None);
    };
    let same = dir(ve1)?.dot(dir(ve2)?) > 0.;
    Ok(Some(if same {
        TopOpeBRepDSConfig::SameOriented
    } else {
        TopOpeBRepDSConfig::DiffOriented
    }))
}

/// `FDS_getupperlower(HDS, edgeIndex, paredge, p1, p2)`: the nearest point parameters
/// of the edge below and above `paredge` (its bounds without any);
/// `let (p1, p2) = fds_getupperlower(&hds, 3, 0.5)?;`.
pub fn fds_getupperlower(
    hds: &Handle<TopOpeBRepDSHDataStructure>,
    edge_index: i32,
    paredge: f64,
) -> Result<(f64, f64), Failure> {
    let bds = href(hds)?.ds();
    let e = topods::edge(bds.shape(edge_index, true))?;
    let (mut p1, mut p2) = fun_tool_bounds(&e)?;
    let mut pite = TopOpeBRepDSPointIterator::new(bds.shape_interferences(&e, true))?;
    while pite.more() {
        let par = pite.parameter()?;
        if par > p1 && par < paredge {
            p1 = par;
        }
        if par < p2 && par > paredge {
            p2 = par;
        }
        pite.next()?;
    }
    Ok((p1, p2))
}

/// The first point/vertex interference of each other geometry with FACE transitions
/// (`FDS_LOIinfsup` selection).
fn loi_sans_gds(
    loi: &ListOfInterference,
    kds: TopOpeBRepDSKind,
    gds: i32,
) -> Result<Vec<Handle<crate::top_ope_brep_ds::TopOpeBRepDSInterference>>, Failure> {
    let mut out = Vec::new();
    let mut tki = TopOpeBRepDSTKI::new();
    tki.fill_on_geometry(loi)?;
    tki.init();
    while let Some((k, g, l)) = tki.value() {
        let pv = k == TopOpeBRepDSKind::Point || k == TopOpeBRepDSKind::Vertex;
        if pv && !(k == kds && g == gds) {
            for i in l.iter() {
                let ((tsb1, _, tsa1, _), _) = fds_idata(i);
                let pointlike = i.is_kind_name("TopOpeBRepDS_CurvePointInterference")
                    || i.is_kind_name("TopOpeBRepDS_EdgeVertexInterference");
                if tsb1 == ShapeEnum::Face && tsa1 == ShapeEnum::Face && pointlike {
                    out.push(i.clone());
                    break;
                }
            }
        }
        tki.next();
    }
    Ok(out)
}

/// `FDS_LOIinfsup(BDS, E, pE, KDS, GDS, LOI, pbef, paft, isonboundper)`: the
/// parameters of the nearest other points (one per geometry, with FACE transitions)
/// before and after `pe` on the edge (its bounds without any); on the closing vertex
/// of a closed edge, the greatest and smallest of them, with `isonboundper` true;
/// `let (pbef, paft, onper) = fds_lo_iinfsup(&ds, &e, 0.5, TopOpeBRepDSKind::Point, 1, &loi)?;`.
pub fn fds_lo_iinfsup(
    _bds: &TopOpeBRepDSDataStructure,
    e: &Edge,
    pe: f64,
    kds: TopOpeBRepDSKind,
    gds: i32,
    loi: &ListOfInterference,
) -> Result<(f64, f64, bool), Failure> {
    let (f, l) = fun_tool_bounds(e)?;
    let (mut pbef, mut paft) = (f, l);
    if loi.is_empty() {
        return Ok((pbef, paft, false));
    }
    let sans = loi_sans_gds(loi, kds, gds)?;
    if sans.is_empty() {
        return Ok((pbef, paft, false));
    }
    let mut tol = precision::parametric(BRepTool::edge_tolerance(e)?);
    let mut isonboundper = false;
    if let Some(v) = TopOpeBRepToolTOOL::closed_e(e)? {
        tol = tol.max(precision::parametric(BRepTool::vertex_tolerance(&v)?));
        isonboundper = (pe - f).abs() <= tol || (pe - l).abs() <= tol;
    }
    for i in &sans {
        let p = fds_parameter(i)?;
        if isonboundper {
            pbef = if p > pbef { p } else { pbef };
            paft = if p < paft { p } else { paft };
        } else {
            pbef = if p > pbef && p < pe { p } else { pbef };
            paft = if p < paft && p > pe { p } else { paft };
        }
    }
    Ok((pbef, paft, isonboundper))
}

/// `FDS_parbefaft(BDS, E, pE, pbef, paft, isonboundper, p1, p2)`: the parameters
/// between the neighbours and `pe` (or the bounds of a periodic edge);
/// `let (p1, p2) = fds_parbefaft(&ds, &e, 0.5, 0., 1., false)?;`.
pub fn fds_parbefaft(
    _bds: &TopOpeBRepDSDataStructure,
    e: &Edge,
    pe: f64,
    pbef: f64,
    paft: f64,
    isonboundper: bool,
) -> Result<(f64, f64), Failure> {
    let t = 0.3178947713;
    let (f, l) = fun_tool_bounds(e)?;
    Ok(if isonboundper {
        ((1. - t) * pbef + t * l, (1. - t) * f + t * paft)
    } else {
        ((1. - t) * pbef + t * pe, (1. - t) * pe + t * paft)
    })
}

/// `FDS_stateEwithF2d(BDS, E, pE, KDS, GDS, F1, TrmemeS)`: the states on `f1` (ON
/// taken as IN) of the edge points between `pe` and its neighbours, set before and
/// after on FACE in `trmemes`; `fds_state_e_with_f2d(&ds, &e, 0.5, k, 1, &f, &mut t)?`.
pub fn fds_state_e_with_f2d(
    bds: &TopOpeBRepDSDataStructure,
    e: &Edge,
    pe: f64,
    kds: TopOpeBRepDSKind,
    gds: i32,
    f1: &Face,
    trmemes: &mut TopOpeBRepDSTransition,
) -> Result<bool, Failure> {
    let loi = bds.shape_interferences(e, true);
    let (pbef, paft, isonper) = fds_lo_iinfsup(bds, e, pe, kds, gds, loi)?;
    let (t1, t2) = fds_parbefaft(bds, e, pe, pbef, paft, isonper)?;
    let (Some(p1), Some(p2)) = (
        fun_tool_value_on_edge(t1, e)?,
        fun_tool_value_on_edge(t2, e)?,
    ) else {
        return Ok(false);
    };
    let f1s: &Shape = f1;
    let (sta1, sta2) = fsc_get_psc_with_reference(f1s, |psc| -> Result<_, Failure> {
        Ok((
            fsc_state_ponface(p1, f1s, psc)?,
            fsc_state_ponface(p2, f1s, psc)?,
        ))
    })??;
    let in_on = |s: State| if s == State::On { State::In } else { s };
    trmemes.set_before(in_on(sta1), ShapeEnum::Face);
    trmemes.set_after(in_on(sta2), ShapeEnum::Face);
    Ok(true)
}

/// The normal crossed with the tangent of `e` at `par` on `f` (`FUN_tool_getxx` with
/// `FUN_tool_nggeomF`): `None` when either is missing.
fn xx_on(f: &Face, e: &Edge, par: f64) -> Result<Option<Direction3>, Failure> {
    let Some(ng) = fun_tool_nggeom_f_on_edge(par, e, f)? else {
        return Ok(None);
    };
    fun_tool_getxx_ng(f, e, par, dir(ng)?)
}

/// The projection parameter of `p` on `e` within `tol`.
fn proj_within(p: Point3, e: &Edge, tol: f64) -> Result<Option<f64>, Failure> {
    let (mut par, mut d) = (0., 0.);
    if !fun_tool_proj_pone(p, e, &mut par, &mut d)? || d > tol {
        return Ok(None);
    }
    Ok(Some(par))
}

/// `FUN_ds_shareG(HDS, iF1, iF2, iE2, Esp, shareG)`: at an inner point of `esp` on the
/// same domain edges `e2` (of `f2`) and `e1` (of `f1`), whether the faces lie on the
/// same side (their normals crossed with the tangents agree); `None` when undecided;
/// `fun_ds_share_g(&hds, 1, 2, 3, &esp)?`.
pub fn fun_ds_share_g(
    hds: &Handle<TopOpeBRepDSHDataStructure>,
    i_f1: i32,
    i_f2: i32,
    i_e2: i32,
    esp: &Edge,
) -> Result<Option<bool>, Failure> {
    let bds = href(hds)?.ds();
    if !has_same_domain(&bds, bds.shape(i_e2, true)) {
        return Ok(None);
    }
    let f1 = topods::face(bds.shape(i_f1, true))?;
    let f2 = topods::face(bds.shape(i_f2, true))?;
    let e2 = topods::edge(bds.shape(i_e2, true))?;
    let tol = precision::confusion() * 1.0e3;
    let (f, l) = fun_tool_bounds(esp)?;
    let x = 0.45678;
    let Some(p) = fun_tool_value_on_edge((1. - x) * f + x * l, esp)? else {
        return Ok(None);
    };
    let Some(par2) = proj_within(p, &e2, tol)? else {
        return Ok(None);
    };
    let Some(nxx2) = xx_on(&f2, &e2, par2)? else {
        return Ok(None);
    };
    let mut me1: IndexedMap<Shape, ShapeMapHasher> = IndexedMap::default();
    topexp::map_shapes_of_type(&f1, ShapeEnum::Edge, &mut me1)?;
    for e1 in bds.shape_same_domain_index(i_e2).iter() {
        let e1 = topods::edge(e1)?;
        if !me1.contains(e1.as_ref()) {
            continue;
        }
        let Some(par1) = proj_within(p, &e1, tol)? else {
            continue;
        };
        let Some(nxx1) = xx_on(&f1, &e1, par1)? else {
            return Ok(None);
        };
        return Ok(Some(nxx1.dot(nxx2) > 0.));
    }
    Ok(None)
}

/// `FUN_ds_mkTonFsdm(HDS, iF1, iF2, iE2, iEG, paronEG, Esp, pardef, T)`: the transition
/// OUT/IN or IN/OUT on `f2` of the edge `eg` of `f1` (same domain as `e2` of `f2`) at
/// `paronEG` (without `pardef`, natively at the projection of the origin); `None` when
/// undecided; `fun_ds_mk_ton_fsdm(&hds, 1, 2, 3, 3, 0.5, &esp, true)?`.
#[allow(clippy::too_many_arguments)] // the native signature
pub fn fun_ds_mk_ton_fsdm(
    hds: &Handle<TopOpeBRepDSHDataStructure>,
    i_f1: i32,
    i_f2: i32,
    i_e2: i32,
    i_eg: i32,
    paron_eg: f64,
    esp: &Edge,
    pardef: bool,
) -> Result<Option<TopOpeBRepDSTransition>, Failure> {
    let bds = href(hds)?.ds();
    if !has_same_domain(&bds, bds.shape(i_e2, true)) {
        return Ok(None);
    }
    let f1 = topods::face(bds.shape(i_f1, true))?;
    let f2 = topods::face(bds.shape(i_f2, true))?;
    let e2 = topods::edge(bds.shape(i_e2, true))?;
    let eg = topods::edge(bds.shape(i_eg, true))?;
    let tol = precision::confusion() * 1.0e3;
    let par_eg = if pardef {
        paron_eg
    } else {
        let _bounds = fun_tool_bounds(esp)?;
        match proj_within(Point3::default(), &eg, tol)? {
            Some(p) => p,
            None => return Ok(None),
        }
    };
    let Some(p) = fun_tool_value_on_edge(par_eg, &eg)? else {
        return Ok(None);
    };
    let Some(tgt_eg) = tggeom_e(par_eg, &eg)? else {
        return Ok(None);
    };
    let Some(ng_f1) = fun_tool_nggeom_f_on_edge(par_eg, &eg, &f1)? else {
        return Ok(None);
    };
    let beafter = ng_f1.crossed(tgt_eg);
    let par2 = if i_eg == i_e2 {
        par_eg
    } else {
        match proj_within(p, &e2, tol)? {
            Some(p2) => p2,
            None => return Ok(None),
        }
    };
    let Some(nxx2) = xx_on(&f2, &e2, par2)? else {
        return Ok(None);
    };
    if i_eg != i_e2 && !super::fun_ds_sdm(&bds, &eg, &e2) {
        return Ok(None);
    }
    let prod = beafter.dot(Vector3::from(nxx2));
    if (1. - prod.abs()).abs() >= precision::angular() * 1.0e3 {
        return Ok(None);
    }
    let (b, a) = if prod > 0. {
        (State::Out, State::In)
    } else {
        (State::In, State::Out)
    };
    Ok(Some(TopOpeBRepDSTransition::new(
        b,
        a,
        ShapeEnum::Face,
        ShapeEnum::Face,
    )))
}

/// `FUN_ds_oriEinF` result: no orientation found.
pub const ORIEINF_UNKNOWN: i32 = 0;
/// `FUN_ds_oriEinF` result: an edge of the face itself.
pub const ORIEINF_ONSAMESHA: i32 = 1;
/// `FUN_ds_oriEinF` result: a closing edge of the face itself.
pub const ORIEINF_CLOSESAME: i32 = 11;
/// `FUN_ds_oriEinF` result: an edge of a same domain face of the other rank.
pub const ORIEINF_ONOPPOSHA: i32 = 2;
/// `FUN_ds_oriEinF` result: a closing edge of a same domain face of the other rank.
pub const ORIEINF_CLOSEOPPO: i32 = 22;
/// `FUN_ds_oriEinF` result: FORWARD and REVERSED in same domain faces of the other rank.
pub const ORIEINF_FORREVOPPO: i32 = 222;

/// The orientation of `e` in the same domain faces of `ff` of the other rank
/// (`FUN_ds_oriEinF` other-rank branch).
fn ori_e_in_sdm(
    bds: &TopOpeBRepDSDataStructure,
    e: &Edge,
    ff: &Face,
    rk_f: i32,
    o: &mut Orientation,
) -> Result<i32, Failure> {
    let o_f = bds.shape(bds.shape_index(ff, true), true).orientation();
    let c = bds.same_domain_ori(ff);
    let (mut has_for, mut has_rev) = (false, false);
    for fsdm in bds.shape_same_domain(ff).iter() {
        let fsdm = topods::face(fsdm)?;
        if bds.ancestor_rank(&fsdm) == rk_f {
            continue;
        }
        let (samsha, oo) = fun_tool_orient_e_in_f_forward(e, &fsdm)?;
        *o = oo;
        if !samsha {
            continue;
        }
        if BRepTool::is_closed_on_face(e, &fsdm)? {
            return Ok(ORIEINF_CLOSEOPPO);
        }
        let inex = |o: Orientation| o == Orientation::Internal || o == Orientation::External;
        if bds.same_domain_ori(&fsdm) != c && !inex(*o) {
            *o = top_abs::complement(*o);
        }
        let o_fsdm = bds.shape(bds.shape_index(&fsdm, true), true).orientation();
        if o_f != o_fsdm && !inex(o_fsdm) {
            *o = top_abs::complement(*o);
        }
        has_for = has_for || *o == Orientation::Forward;
        has_rev = has_rev || *o == Orientation::Reversed;
    }
    if has_for && has_rev {
        return Ok(ORIEINF_FORREVOPPO);
    }
    if has_for || has_rev {
        *o = if has_for {
            Orientation::Forward
        } else {
            Orientation::Reversed
        };
        return Ok(ORIEINF_ONOPPOSHA);
    }
    Ok(ORIEINF_UNKNOWN)
}

/// `FUN_ds_oriEinF(BDS, E, F, O)`: how the edge lies in the face (ORIEINF_*) with its
/// orientation (EXTERNAL when unknown); `let (k, o) = fun_ds_ori_e_in_f(&ds, &e, &f)?;`.
pub fn fun_ds_ori_e_in_f(
    bds: &TopOpeBRepDSDataStructure,
    e: &Edge,
    f: &Shape,
) -> Result<(i32, Orientation), Failure> {
    let mut o = Orientation::External;
    let (rk_f, rk_e) = (bds.ancestor_rank(f), bds.ancestor_rank(e));
    let ff = topods::face(f)?;
    if rk_f == rk_e {
        let (samsha, oo) = fun_tool_orient_e_in_f_forward(e, &ff)?;
        o = oo;
        if !samsha {
            return Ok((ORIEINF_UNKNOWN, o));
        }
        let closed = BRepTool::is_closed_on_face(e, &ff)?;
        return Ok((
            if closed {
                ORIEINF_CLOSESAME
            } else {
                ORIEINF_ONSAMESHA
            },
            o,
        ));
    }
    if bds.shape_same_domain(&ff).is_empty() {
        return Ok((ORIEINF_UNKNOWN, o));
    }
    let k = ori_e_in_sdm(bds, e, &ff, rk_f, &mut o)?;
    Ok((k, o))
}

/// `FUN_ds_ONesd(BDS, IE, EspON, IEsd)`: the index of the first same domain edge of
/// the edge `ie` passing within 1000 times its tolerance of an inner point of
/// `esp_on`; `fun_ds_one_sd(&ds, 3, &esp)?`.
pub fn fun_ds_one_sd(
    bds: &TopOpeBRepDSDataStructure,
    ie: i32,
    esp_on: &Shape,
) -> Result<Option<i32>, Failure> {
    let e = bds.shape(ie, true);
    let esp = topods::edge(esp_on)?;
    let (f, l) = fun_tool_bounds(&esp)?;
    let x = 0.456789;
    let Some(p3d) = fun_tool_value_on_edge((1. - x) * f + x * l, &esp)? else {
        return Ok(None);
    };
    for esd in bds.shape_same_domain(e).iter() {
        let esd = topods::edge(esd)?;
        let (mut parp, mut d) = (0., 0.);
        if !fun_tool_proj_pone(p3d, &esd, &mut parp, &mut d)? {
            continue;
        }
        if d < BRepTool::edge_tolerance(&esd)? * 1.0e3 {
            return Ok(Some(bds.shape_index(&esd, true)));
        }
    }
    Ok(None)
}
