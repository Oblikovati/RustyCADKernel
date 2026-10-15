// Rust translation of ModelingAlgorithms/TKBool/TopOpeBRepDS/TopOpeBRepDS_redu.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The reductions of the transitions of a section edge (FUN_scanloi, FUN_ds_redu2d1d,
//! FUN_ds_GetTr).
use super::export::config::mkt_states;
use super::export::{fds_idata, fds_parameter, fun_ds_has_i2d, iref};
use super::process_interferences_tool::{
    fun_select_sk_interference, fun_select_traori_interference, fun_select_trasha_interference,
};
use super::{
    ListOfInterference, TopOpeBRepDSDataStructure, TopOpeBRepDSInterference, TopOpeBRepDSKind,
    TopOpeBRepDSTransition,
};
use crate::top_ope_brep_tool::{
    fun_tool_in_s, fun_tool_is_closing_e, fun_tool_par_e_tol, fun_tool_paron_ef_tol,
};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topods;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum, State};
use rustycad_tkmath::gp::Point2;

/// `FUN_scanloi(lI, lFOR, FOR, lREV, REV, lINT, INT, lEXT, EXT)`: the interferences
/// split by their orientation from IN (the others in `lext`); the four counts;
/// `let (nf, nr, ni, ne) = fun_scanloi(&l, &mut lf, &mut lr, &mut li, &mut le)?;`.
pub fn fun_scanloi(
    li: &ListOfInterference,
    lfor: &mut ListOfInterference,
    lrev: &mut ListOfInterference,
    lint: &mut ListOfInterference,
    lext: &mut ListOfInterference,
) -> Result<(i32, i32, i32, i32), Failure> {
    lfor.clear();
    lrev.clear();
    lint.clear();
    *lext = li.clone();
    let nfor = fun_select_traori_interference(lext, Orientation::Forward, lfor)?;
    let nrev = fun_select_traori_interference(lext, Orientation::Reversed, lrev)?;
    let nint = fun_select_traori_interference(lext, Orientation::Internal, lint)?;
    Ok((nfor, nrev, nint, lext.extent()))
}

/// The 2d transition of a closing edge of the face from the projection of the
/// section edge parameter (`FUN_ds_redu2d1d` closing branch).
fn redu_closing(
    bds: &TopOpeBRepDSDataStructure,
    ise: i32,
    i2d: &Handle<TopOpeBRepDSInterference>,
    e: &topods::Edge,
    f: &topods::Face,
    new_t2d: &mut TopOpeBRepDSTransition,
) -> Result<bool, Failure> {
    let t2d = iref(i2d)?.transition();
    if t2d.before() == t2d.after() {
        return Ok(false);
    }
    let se = topods::edge(bds.shape(ise, true))?;
    let tol_f = BRepTool::face_tolerance(f)? * 1.0e2;
    let tol_e = BRepTool::edge_tolerance(e)? * 1.0e2;
    let par_se = fds_parameter(i2d)?;
    let mut par_e = 0.;
    if !fun_tool_par_e_tol(&se, par_se, e, &mut par_e, tol_e)? {
        return Ok(false);
    }
    let mut uv = Point2::default();
    if !fun_tool_paron_ef_tol(e, par_e, f, &mut uv, tol_f)? {
        return Ok(false);
    }
    let Some((stb1, sta1)) = mkt_states(&se, (0., 0., par_se), f, uv, 1.0e-4, Some((e, par_e)))?
    else {
        return Ok(false);
    };
    new_t2d.set_before(stb1, ShapeEnum::Face);
    new_t2d.set_after(sta1, ShapeEnum::Face);
    Ok(true)
}

/// `FUN_ds_redu2d1d(BDS, ISE, I2d, l1d, newT2d)`: the transition of the 2d vertex
/// interference `i2d` of the section edge `ise` reduced with the 1d interferences of
/// the edges of its face (IN before/after where they are); false when not reducible;
/// `fun_ds_redu2d1d(&ds, 3, &i2d, &l1d, &mut t)?`.
pub fn fun_ds_redu2d1d(
    bds: &TopOpeBRepDSDataStructure,
    ise: i32,
    i2d: &Handle<TopOpeBRepDSInterference>,
    l1d: &ListOfInterference,
    new_t2d: &mut TopOpeBRepDSTransition,
) -> Result<bool, Failure> {
    let ((sb2, ib2, _, ia2), (gt2, _, _, s2)) = fds_idata(i2d);
    let o2 = iref(i2d)?
        .transition()
        .orientation(State::In, ShapeEnum::Face);
    new_t2d.set_index(ib2);
    new_t2d.set_orientation(o2);
    if !(ib2 == ia2 && sb2 == ShapeEnum::Face && gt2 == TopOpeBRepDSKind::Vertex) {
        return Ok(false);
    }
    topods::edge(bds.shape(ise, true))?;
    let f = topods::face(bds.shape(ib2, true))?;
    let e = topods::edge(bds.shape(s2, true))?;
    if fun_tool_is_closing_e(&e, &f, &f)? {
        return redu_closing(bds, ise, i2d, &e, &f, new_t2d);
    }
    let (mut before_in1d, mut after_in1d) = (false, false);
    for i1d in l1d.iter() {
        let ((_, ib1, _, ia1), _) = fds_idata(i1d);
        if ib1 != ia1 {
            continue;
        }
        let o1 = iref(i1d)?
            .transition()
            .orientation(State::In, ShapeEnum::Face);
        let esd = topods::edge(bds.shape(ib1, true))?;
        if !fun_tool_in_s(&esd, &f)? {
            continue;
        }
        let b_in = o1 == Orientation::Internal || o1 == Orientation::Reversed;
        let a_in = o1 == Orientation::Internal || o1 == Orientation::Forward;
        if b_in && a_in {
            return Ok(false);
        }
        before_in1d |= b_in;
        after_in1d |= a_in;
    }
    if before_in1d {
        new_t2d.set_before(State::In, ShapeEnum::Face);
    }
    if after_in1d {
        new_t2d.set_after(State::In, ShapeEnum::Face);
    }
    Ok(true)
}

/// The state on one side, the index of the shape deciding it and its dimension.
pub type SideState = (State, i32, i32);

/// The interferences of one dimension split by orientation: (FOR, REV, INT, EXT).
type Scan = [ListOfInterference; 4];

/// The scan of a list.
fn scan(l: &ListOfInterference) -> Result<Scan, Failure> {
    let mut s: Scan = Default::default();
    let [f, r, i, e] = &mut s;
    fun_scanloi(l, f, r, i, e)?;
    Ok(s)
}

/// The index before (or after) of the first interference of `a` then `b`.
fn first_index(
    a: &ListOfInterference,
    b: &ListOfInterference,
    after: bool,
) -> Result<i32, Failure> {
    let first = a.iter().chain(b.iter()).next().ok_or_else(|| {
        Failure::with_type(
            "Standard_NoSuchObject",
            "NCollection_List::First",
            "NCollection_List",
        )
    })?;
    let t = iref(first)?.transition();
    Ok(if after {
        t.index_after()
    } else {
        t.index_before()
    })
}

/// The state, shape index and dimension on one side (`FUN_ds_GetTr` cascades: IN in
/// 1d, 2d, 3d, then OUT in 3d, 2d, 1d).
fn side(scans: &[Scan; 3], after: bool) -> Result<SideState, Failure> {
    // the IN lists before are REV+INT, after FOR+INT; the OUT lists before FOR+EXT,
    // after REV+EXT
    let (in_a, out_a) = if after { (0, 1) } else { (1, 0) };
    for (d, s) in scans.iter().enumerate() {
        if s[in_a].extent() + s[2].extent() > 0 {
            return Ok((
                State::In,
                first_index(&s[in_a], &s[2], after)?,
                d as i32 + 1,
            ));
        }
    }
    for (d, s) in scans.iter().enumerate().rev() {
        if s[out_a].extent() + s[3].extent() != 0 {
            return Ok((
                State::Out,
                first_index(&s[out_a], &s[3], after)?,
                d as i32 + 1,
            ));
        }
    }
    Ok((State::Unknown, 0, 0))
}

/// `FUN_ds_GetTr(BDS, ISE, G, LIG, stb, isb, bdim, sta, isa, adim)`: the states before
/// and after the geometry on the section edge from its 1d, 2d and 3d interferences,
/// with the index of the shape and the dimension deciding them;
/// `let ((stb, isb, bdim), (sta, isa, adim)) = fun_ds_get_tr(&ds, 3, 1, &lig)?;`.
pub fn fun_ds_get_tr(
    _bds: &TopOpeBRepDSDataStructure,
    ise: i32,
    _g: i32,
    lig: &ListOfInterference,
) -> Result<(SideState, SideState), Failure> {
    let mut lig_copy = lig.clone();
    let mut l3d = ListOfInterference::default();
    fun_select_sk_interference(&mut lig_copy, TopOpeBRepDSKind::Face, &mut l3d)?;
    let mut l2d = ListOfInterference::default();
    fun_ds_has_i2d(ise, lig, &mut l2d)?;
    let mut lig_copy = lig.clone();
    let mut l1d = ListOfInterference::default();
    fun_select_trasha_interference(&mut lig_copy, ShapeEnum::Edge, &mut l1d)?;
    let scans = [scan(&l1d)?, scan(&l2d)?, scan(&l3d)?];
    Ok((side(&scans, false)?, side(&scans, true)?))
}
