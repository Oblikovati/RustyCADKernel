//! Helpers of make_transition.

use crate::top_ope_brep_tool::normal::nt;
use crate::top_ope_brep_tool::project::fun_tool_bounds;
use crate::top_ope_brep_tool::tool::{TOOL_INFFIRST, TOOL_SUPLAST, TopOpeBRepToolTOOL};
use crate::top_ope_brep_tool::tool_tangent::tggeom_e;
use crate::top_ope_brep_tool::topology::fun_tool_value_on_edge;
use rustycad_tkbrep::brep_adaptor::{BRepAdaptorCurve, BRepAdaptorSurface};
use rustycad_tkbrep::topods::{Edge, Face};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkg3d::top_abs::State;
use rustycad_tkmath::gp::{Direction2, Direction3, Point2, Vector2, Vector3};

pub(super) const BEFORE: i32 = 1;
pub(super) const AFTER: i32 = 2;
pub(super) const IS_IN_IFH1: i32 = 1;
pub(super) const IS_IN_IFH2: i32 = 2;
pub(super) const IS_ON2: i32 = 21;
pub(super) const IS_ON2_IFSS: i32 = 10;
pub(super) const IS_IN2_IFSS: i32 = 11;
pub(super) const IS_OU2_IFSS: i32 = 12;

/// `FUN_tolang()` (static): 1e6 times Precision::Angular().
pub(super) fn fun_tolang() -> f64 {
    precision::angular() * 1.0e6
}

/// `FUN_nullcurv(curv)` (static): below 100 times Precision::Confusion().
pub(super) fn fun_nullcurv(curv: f64) -> bool {
    curv < precision::confusion() * 1.0e2
}

/// `FUN_mkT2dquad(curvC1, curvC2)` (static): the kind of decision from the curvatures
/// of two tangent quadric curves.
pub(super) fn fun_mk_t2dquad_kind(curv_c1: f64, curv_c2: f64) -> i32 {
    let (nullc1, nullc2) = (fun_nullcurv(curv_c1), fun_nullcurv(curv_c2));
    if nullc2 && nullc1 {
        return IS_ON2;
    }
    if nullc2 {
        return IS_IN_IFH1;
    }
    if nullc1 {
        return IS_IN_IFH2;
    }
    if (curv_c2 - curv_c1).abs() < 1.0e-2 {
        return IS_ON2_IFSS;
    }
    if curv_c1 > curv_c2 {
        IS_IN2_IFSS
    } else {
        IS_OU2_IFSS
    }
}

/// `FUN_getnearpar(e, par, f, l, factor, sta, nearpar)` (static): the parameter moved
/// by `factor` of the range before or after `par`; `None` on the bound in that sense.
pub(super) fn fun_getnearpar(
    e: &Edge,
    par: f64,
    (f, l): (f64, f64),
    factor: f64,
    sta: i32,
) -> Result<Option<f64>, Failure> {
    let bc = BRepAdaptorCurve::with_edge(e)?;
    let tol1d = bc.resolution(bc.tolerance()?)?;
    let onf = (par - f).abs() < tol1d;
    let onl = (par - l).abs() < tol1d;
    if (onf && sta == BEFORE) || (onl && sta == AFTER) {
        return Ok(None);
    }
    Ok(Some(if sta == BEFORE {
        par - factor * (l - f)
    } else {
        par + factor * (l - f)
    }))
}

/// `FUN_tg(e, par, f, l, factor, tg, st)` (static): the tangent at the near parameter
/// before `par`, else after it, with the side used.
pub(super) fn fun_tg(
    e: &Edge,
    par: f64,
    range: (f64, f64),
    factor: f64,
) -> Result<Option<(Direction3, i32)>, Failure> {
    for st in [BEFORE, AFTER] {
        let Some(pn) = fun_getnearpar(e, par, range, factor, st)? else {
            continue;
        };
        let Some(tmp) = tggeom_e(pn, e)? else {
            continue;
        };
        return Ok(Some((Direction3::from_xyz(*tmp.xyz())?, st)));
    }
    Ok(None)
}

/// The tangent leaving the point towards the near one.
pub(super) fn leaving_tangent((tgnear, st): (Direction3, i32)) -> Direction3 {
    if st == AFTER {
        tgnear
    } else {
        tgnear.reversed()
    }
}

/// `FUN_getsta(mkt, tga1, tga2, xx2, sta)` (static): the state from the decision kind
/// and the leaving tangents.
pub(super) fn fun_getsta(mkt: i32, tga1: Direction3, tga2: Direction3, xx2: Direction3) -> State {
    match mkt {
        IS_IN_IFH1 => {
            if tga1.dot(xx2) > 0. {
                State::In
            } else {
                State::Out
            }
        }
        IS_IN_IFH2 => {
            if tga2.dot(xx2) < 0. {
                State::In
            } else {
                State::Out
            }
        }
        IS_ON2_IFSS => {
            if tga1.dot(tga2) > 0. {
                State::On
            } else {
                State::In
            }
        }
        IS_IN2_IFSS => {
            if tga1.dot(xx2) < 0. {
                State::Out
            } else {
                State::In
            }
        }
        IS_OU2_IFSS => {
            if tga2.dot(xx2) < 0. {
                State::In
            } else {
                State::Out
            }
        }
        _ => State::On,
    }
}

/// Whether the decision needs the first (and the second) leaving tangent.
pub(super) fn needs_tangents(mkt: i32) -> (bool, bool) {
    let mk1 = matches!(mkt, IS_IN_IFH1 | IS_ON2_IFSS | IS_IN2_IFSS);
    let mk2 = matches!(mkt, IS_IN_IFH2 | IS_ON2_IFSS | IS_OU2_IFSS);
    (mk1, mk2)
}

/// `FUN_mkT2dquad(e1, par1, f1, l1, e2, par2, mkt, xx2, factor, sta)` (static): the
/// state of two tangent quadric edges.
pub(super) fn fun_mk_t2dquad(
    (e1, par1, range1): (&Edge, f64, (f64, f64)),
    (e2, par2): (&Edge, f64),
    mkt: i32,
    xx2: Direction3,
    factor: f64,
) -> Result<Option<State>, Failure> {
    let (mk1, mk2) = needs_tangents(mkt);
    let (mut tga1, mut tga2) = (Direction3::Z, Direction3::Z);
    if mk1 {
        let Some(found) = fun_tg(e1, par1, range1, factor)? else {
            return Ok(None);
        };
        tga1 = leaving_tangent(found);
    }
    if mk2 {
        let range2 = fun_tool_bounds(e2)?;
        let Some(found) = fun_tg(e2, par2, range2, factor)? else {
            return Ok(None);
        };
        tga2 = leaving_tangent(found);
    }
    Ok(Some(fun_getsta(mkt, tga1, tga2, xx2)))
}

/// The UV coordinate moved back into the period when it is out of the bounds.
pub(super) fn periodic_back(
    x: f64,
    on: i32,
    closed: bool,
    period: impl FnOnce() -> Result<f64, Failure>,
) -> Result<Option<f64>, Failure> {
    if on != TOOL_INFFIRST && on != TOOL_SUPLAST {
        return Ok(Some(x));
    }
    if !closed {
        return Ok(None);
    }
    let per = period()?;
    Ok(Some(if on == TOOL_INFFIRST {
        x + per
    } else {
        x - per
    }))
}

/// `FUN_getnearuv(f, uv, factor, sta, duv, nearuv)` (static): the UV point moved by
/// `factor` along `duv` (backwards before), brought back by a period when `uv` is out
/// of the bounds of a closed direction; `None` for an open one.
pub(super) fn fun_getnearuv(
    f: &Face,
    uv: Point2,
    factor: f64,
    sta: i32,
    duv: Direction2,
) -> Result<Option<Point2>, Failure> {
    let bs = BRepAdaptorSurface::with_face(f, true)?;
    let mut xuv = Vector2::from(duv) * factor;
    if sta == BEFORE {
        xuv.reverse();
    }
    let nearuv = uv.translated(xuv);
    let (onu, onv) = TopOpeBRepToolTOOL::stuv_f(uv, f)?;
    if onu == 0 && onv == 0 {
        return Ok(Some(nearuv));
    }
    let Some(nearu) = periodic_back(nearuv.x(), onu, bs.is_u_closed()?, || bs.u_period())? else {
        return Ok(None);
    };
    let Some(nearv) = periodic_back(nearuv.y(), onv, bs.is_v_closed()?, || bs.v_period())? else {
        return Ok(None);
    };
    Ok(Some(Point2::new(nearu, nearv)))
}

/// `FUN_tgef(f, uv, duv, factor, tge, tg0, tgef, st)` (static): `tg0 ^ nt` at the near
/// UV point before `uv`, else after it, with the side used; `None` when the normal
/// fails.
pub(super) fn fun_tgef(
    f: &Face,
    uv: Point2,
    duv: Direction2,
    factor: f64,
    tg0: Direction3,
) -> Result<Option<(Direction3, i32)>, Failure> {
    for st in [BEFORE, AFTER] {
        let Some(nearuv) = fun_getnearuv(f, uv, factor, st, duv)? else {
            continue;
        };
        let mut ntd = Direction3::Z;
        if !nt(nearuv, f, &mut ntd)? {
            return Ok(None);
        }
        return Ok(Some((tg0.crossed(ntd)?, st)));
    }
    Ok(None)
}

/// `FUN_mkT3dquad(e, pf, pl, par, f, uv, tge, ntf, mkt, factor, sta)` (static): the
/// state of a quadric edge tangent to a quadric face.
pub(super) fn fun_mk_t3dquad(
    (e, range, par): (&Edge, (f64, f64), f64),
    (f, uv): (&Face, Point2),
    (tge, ntf): (Direction3, Direction3),
    mkt: i32,
    factor: f64,
) -> Result<Option<State>, Failure> {
    let xxef = ntf.reversed();
    let tg0 = ntf.crossed(tge)?;
    let (mke, mkef) = needs_tangents(mkt);
    let (mut tgae, mut tgaef) = (Direction3::Z, Direction3::Z);
    if mke {
        let Some(found) = fun_tg(e, par, range, factor)? else {
            return Ok(None);
        };
        tgae = leaving_tangent(found);
    }
    if mkef {
        let fac3d: f64 = 0.01;
        let Some(duv) = TopOpeBRepToolTOOL::getduv(f, uv, Vector3::from(tge), fac3d)? else {
            return Ok(None);
        };
        let Some(found) = fun_tgef(f, uv, duv, factor, tg0)? else {
            return Ok(None);
        };
        tgaef = leaving_tangent(found);
    }
    Ok(Some(fun_getsta(mkt, tgae, tgaef, xxef)))
}

/// `FUN_stawithES(tgE, xxES, st)` (static): the state on the side of the restriction
/// edge (UNKNOWN when the edge is tangent to it).
pub(super) fn fun_sta_with_es(tg_e: Direction3, xx_es: Direction3, st: i32) -> State {
    let prod = tg_e.dot(xx_es);
    if prod.abs() < fun_tolang() {
        return State::Unknown;
    }
    match (prod > 0., st == BEFORE) {
        (true, true) | (false, false) => State::Out,
        _ => State::In,
    }
}

/// `FUN_stawithES(tgE, xxES, st, stt)` (static): an IN or ON state kept when on the
/// inner side of the restriction, OUT otherwise; UNKNOWN for other states.
pub(super) fn fun_sta_with_es_state(
    tg_e: Direction3,
    xx_es: Direction3,
    st: i32,
    stt: State,
) -> State {
    if stt == State::Unknown {
        return State::Unknown;
    }
    let st_es = fun_sta_with_es(tg_e, xx_es, st);
    if stt == State::In || stt == State::On {
        return if st_es == State::In { stt } else { State::Out };
    }
    State::Unknown
}

/// `FUN_staproj(e, pf, pl, pe, factor, st, f, sta)` (static): the state of the near
/// point of the edge projected on the face; `sta` is set to UNKNOWN once the point is
/// found and kept when the near parameter or point is missing.
pub(super) fn fun_staproj(
    (e, range, pe): (&Edge, (f64, f64), f64),
    factor: f64,
    st: i32,
    f: &Face,
    sta: &mut State,
) -> Result<bool, Failure> {
    let Some(par) = fun_getnearpar(e, pe, range, factor, st)? else {
        return Ok(false);
    };
    let Some(pt) = fun_tool_value_on_edge(par, e)? else {
        return Ok(false);
    };
    let mut uv = Point2::default();
    let (ok, s) = TopOpeBRepToolTOOL::getstp3d_f(pt, f, &mut uv)?;
    *sta = s;
    Ok(ok)
}
