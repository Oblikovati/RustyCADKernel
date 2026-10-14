// Rust translation of
// ModelingAlgorithms/TKBool/TopOpeBRepDS/TopOpeBRepDS_ProcessFaceInterferences.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The filters of the face interferences used by `TopOpeBRepDS_Filter` (FUN_mkTonF,
//! FUN_edgeofface, FUN_keepFinterference, FUN_unkeepFdoubleGBoundinterferences,
//! FUN_resolveFUNKNOWN).
use super::export::config::dir;
use super::export::{fds_data, fds_idata, iref};
use super::funk::fun_unkf_stasta;
use super::process_interferences_tool::fun_unkeep_unknown;
use super::process_interferences_tool::select::{to_list, to_vec};
use super::tool::MapOfShapeOn1State;
use super::{
    ListOfInterference, TopOpeBRepDSDataStructure, TopOpeBRepDSFaceEdgeInterference,
    TopOpeBRepDSInterference, TopOpeBRepDSKind, TopOpeBRepDSShapeShapeInterference,
    TopOpeBRepDSTransition,
};
use crate::top_ope_brep_tool::{
    TopOpeBRepToolShapeClassifier, fun_tool_bounds, fun_tool_curves_so, fun_tool_nggeom_f,
    fun_tool_par_f, nt, tggeom_e,
};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Edge, Face, Shape};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum, State};
use rustycad_tkmath::gp::{Direction3, Point2};

type H = Handle<TopOpeBRepDSInterference>;

/// `FUN_mkTonF(F, FS, E, T)`: the transition FORWARD/REVERSED on `fs` of the face `f`
/// along its edge `e` (from the normals at an inner point); false (the transition
/// UNKNOWN) when tangent or undecided; `fun_mk_ton_f(&f, &fs, &e, &mut t)?`.
pub fn fun_mk_ton_f(
    f: &Face,
    fs: &Face,
    e: &Edge,
    t: &mut TopOpeBRepDSTransition,
) -> Result<bool, Failure> {
    if BRepTool::degenerated(e)? {
        return Ok(false);
    }
    t.set(
        State::Unknown,
        State::Unknown,
        ShapeEnum::Face,
        ShapeEnum::Face,
    );
    let tola = 1.0e-6;
    let (fp, lp) = fun_tool_bounds(e)?;
    let par_t = 0.456789;
    let pmil = (1. - par_t) * fp + par_t * lp;
    let Some(tg_e) = tggeom_e(pmil, e)? else {
        return Ok(false);
    };
    let (mut uv_f, mut uv_fs) = (Point2::default(), Point2::default());
    if !fun_tool_par_f(e, pmil, f, &mut uv_f)? || !fun_tool_par_f(e, pmil, fs, &mut uv_fs)? {
        return Ok(false);
    }
    let ng_f = dir(fun_tool_nggeom_f(uv_f, f)?)?;
    let tg_e = dir(tg_e)?;
    if (1. - ng_f.dot(tg_e).abs()).abs() < tola {
        return Ok(false);
    }
    let mut nt_fs = Direction3::default();
    if !nt(uv_fs, fs, &mut nt_fs)? {
        return Ok(false);
    }
    let yy = ng_f.crossed(tg_e)?.dot(nt_fs);
    if yy.abs() < tola {
        return Ok(false);
    }
    t.set_orientation(if yy < 0. {
        Orientation::Forward
    } else {
        Orientation::Reversed
    });
    Ok(true)
}

/// `FUN_edgeofface(E, F)`: `e` is an edge of `f`; `fun_edgeofface(&e, &f)?`.
pub fn fun_edgeofface(e: &Shape, f: &Shape) -> Result<bool, Failure> {
    for x in Explorer::new(f, ShapeEnum::Edge, ShapeEnum::Shape)? {
        if x.is_same(e) {
            return Ok(true);
        }
    }
    Ok(false)
}

/// `FUN_keepFinterference(DS, I, F)`: a face/edge interference whose edge is not an
/// edge of `f` (any other interference is kept); `fun_keep_f_interference(&ds, &i, &f)?`.
pub fn fun_keep_f_interference(
    ds: &TopOpeBRepDSDataStructure,
    i: &H,
    f: &Shape,
) -> Result<bool, Failure> {
    if i.is_kind_name("TopOpeBRepDS_FaceEdgeInterference") {
        let eg = ds.shape(iref(i)?.geometry(), true);
        return Ok(!fun_edgeofface(eg, f)?);
    }
    Ok(true)
}

/// The GBound of a known shape/shape interference (`None` for an unknown transition or
/// another type).
fn known_gbound(i: &H) -> Result<Option<bool>, Failure> {
    if iref(i)?.transition().is_unknown() {
        return Ok(None);
    }
    let ssi: Handle<TopOpeBRepDSShapeShapeInterference> = i.cast();
    Ok(ssi.get().map(|s| s.gbound()))
}

/// `FUN_unkeepFdoubleGBoundinterferences(LI, BDS, SIX)`: two face/edge interferences of
/// the same bounding edge on different faces both removed;
/// `fun_unkeep_fdouble_gbound_interferences(&mut l)?`.
pub fn fun_unkeep_fdouble_gbound_interferences(li: &mut ListOfInterference) -> Result<(), Failure> {
    let mut v = to_vec(li);
    let mut a = 0;
    while a < v.len() {
        let Some(is_b1) = known_gbound(&v[a])? else {
            a += 1;
            continue;
        };
        let (gt1, g1, st1, s1) = fds_data(&v[a]);
        let mut cond1 = false;
        let mut b = a + 1;
        while b < v.len() {
            let Some(is_b2) = known_gbound(&v[b])? else {
                b += 1;
                continue;
            };
            let (gt2, g2, st2, s2) = fds_data(&v[b]);
            let cond2 = gt2 == gt1
                && gt1 == TopOpeBRepDSKind::Edge
                && g2 == g1
                && st2 == st1
                && st1 == TopOpeBRepDSKind::Face
                && s2 != s1
                && is_b1
                && is_b2;
            if cond2 {
                cond1 = true;
                v.remove(b);
            } else {
                b += 1;
            }
        }
        if cond1 {
            v.remove(a);
        } else {
            a += 1;
        }
    }
    *li = to_list(v);
    Ok(())
}

/// The split of `ee` (ON) used for a face/edge interference of `FUN_resolveFUNKNOWN`:
/// (the edge, its GBound).
fn edge_split(
    mesp: &MapOfShapeOn1State,
    ee: &Edge,
    (ff, fs): (&Face, &Face),
    is_eegb: bool,
) -> Result<(Edge, bool), Failure> {
    let Some(los1) = mesp.seek(ee.as_ref()) else {
        return Ok((ee.clone(), is_eegb));
    };
    let los = los1.list_on_state();
    if !los1.is_split() || los.is_empty() {
        return Ok((ee.clone(), is_eegb));
    }
    let mut eesp = topods::edge(los.first().unwrap_or(&Shape::default()))?;
    let gb = is_eegb && eesp.is_same(ee);
    if los.extent() > 1 {
        for ae in los.iter() {
            let ae = topods::edge(ae)?;
            let (f, l) = fun_tool_bounds(&ae)?;
            let par_t = 0.456789;
            let pmil = (1. - par_t) * f + par_t * l;
            let mut uv_f = Point2::default();
            if fun_tool_par_f(&ae, pmil, ff, &mut uv_f)?
                && fun_tool_par_f(&ae, pmil, fs, &mut uv_f)?
            {
                eesp = ae;
                break;
            }
        }
    }
    Ok((eesp, gb))
}

/// The states of the face along the split edge (`FUN_mkTonF`, else `FUN_UNKFstasta`
/// with the classifier on the shape of index the rank of `fs` when it has a solid).
fn face_states(
    bds: &TopOpeBRepDSDataStructure,
    (ff, fs, s1): (&Face, &Face, i32),
    eesp: &Edge,
    is_eegb: bool,
    p_classif: Option<&mut TopOpeBRepToolShapeClassifier>,
) -> Result<(State, State), Failure> {
    let mut t = TopOpeBRepDSTransition::default();
    if fun_mk_ton_f(ff, fs, eesp, &mut t)? {
        return Ok((t.before(), t.after()));
    }
    let mut p_class = None;
    if let Some(pc) = p_classif {
        let rank_fs = bds.ancestor_rank_index(s1);
        let a_sref = bds.shape(rank_fs, true).clone();
        if Explorer::new(&a_sref, ShapeEnum::Solid, ShapeEnum::Shape)?.more() {
            pc.set_reference(&a_sref)?;
            p_class = Some(pc);
        }
    }
    fun_unkf_stasta(ff, fs, eesp, is_eegb, p_class)
}

/// `FUN_resolveFUNKNOWN(LI, BDS, SIX, MEsp, pClassif)`: the unknown transitions of a
/// face tangent to another along an edge resolved from the states of the face on both
/// sides of the edge (or of its split ON), then the still unknown ones removed;
/// `fun_resolve_f_unknown(&mut l, &mut ds, 3, &mesp, None)?`.
pub fn fun_resolve_f_unknown(
    li: &mut ListOfInterference,
    bds: &mut TopOpeBRepDSDataStructure,
    six: i32,
    mesp: &MapOfShapeOn1State,
    mut p_classif: Option<&mut TopOpeBRepToolShapeClassifier>,
) -> Result<(), Failure> {
    let ff = topods::face(bds.shape(six, true))?;
    for i1 in li.iter() {
        if !iref(i1)?.transition().is_unknown() {
            continue;
        }
        let ((tsb1, isb1, tsa1, isa1), (gt1, g1, st1, s1)) = fds_idata(i1);
        let idt = tsb1 == ShapeEnum::Face
            && tsa1 == ShapeEnum::Face
            && gt1 == TopOpeBRepDSKind::Edge
            && st1 == TopOpeBRepDSKind::Face;
        if !(idt && isb1 == s1 && isa1 == s1) {
            continue;
        }
        let ee = topods::edge(bds.shape(g1, true))?;
        BRepTool::range(&ee)?;
        let fei: Handle<TopOpeBRepDSFaceEdgeInterference> = i1.cast();
        let Some(fei) = fei.get() else {
            continue;
        };
        let fs = topods::face(bds.shape(s1, true))?;
        let (eesp, is_eegb) = edge_split(mesp, &ee, (&ff, &fs), fei.gbound())?;
        let mut is_so = true;
        if !eesp.is_same(&ee) && !fun_tool_curves_so(&eesp, &ee, &mut is_so)? {
            continue;
        }
        let classif = p_classif.as_deref_mut();
        let (mut stateb, mut statea) = face_states(bds, (&ff, &fs, s1), &eesp, is_eegb, classif)?;
        if stateb == State::Unknown || statea == State::Unknown {
            continue;
        }
        if !is_so {
            std::mem::swap(&mut stateb, &mut statea);
        }
        iref(i1)?.change_transition(|t| t.set(stateb, statea, tsb1, tsa1));
    }
    fun_unkeep_unknown(li, bds, six)
}
