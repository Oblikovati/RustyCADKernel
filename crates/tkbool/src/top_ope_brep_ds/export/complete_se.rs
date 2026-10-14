//! The completions of the section edge interferences FUN_ds_completeforSE1 to SE3.
use super::config::{fds_lo_iinfsup, mkt_states};
use super::{
    fds_data, fds_parameter, fds_si_is_gi_of_i_of_sba_of_t_of_i, fds_tdata, fun_ds_getoov,
    fun_select_traint_interference, has_same_domain, href, iref,
};
use crate::top_ope_brep_ds::process_interferences_tool::{
    fun_select_gi_interference, fun_select_itrasha_interference, fun_select_sk_interference,
    fun_select_traori_interference, fun_select_trasha_interference, make_epv_interference_sk,
};
use crate::top_ope_brep_ds::{
    ListOfInterference, TopOpeBRepDSDataStructure, TopOpeBRepDSEdgeVertexInterference,
    TopOpeBRepDSHDataStructure, TopOpeBRepDSInterference, TopOpeBRepDSKind, TopOpeBRepDSTKI,
    TopOpeBRepDSTransition, fdscnx_edge_connexity_same_shape_ds, store_interference_ds,
};
use crate::top_ope_brep_tool::{
    fun_tool_maxtol, fun_tool_orient_v_in_e, fun_tool_paron_ef, fun_tool_proj_pone,
};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topods::{self, Edge, Face, Shape};
use rustycad_tkbrep::toptools::ShapeMapHasher;
use rustycad_tkernel::ncollection::IndexedMap;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::gp::Point2;

type H = Handle<TopOpeBRepDSInterference>;

/// The interferences of a list grouped by geometry in the `TopOpeBRepDS_TKI` order.
pub(crate) fn tki_groups(
    l: &ListOfInterference,
) -> Result<Vec<(TopOpeBRepDSKind, i32, ListOfInterference)>, Failure> {
    let mut tki = TopOpeBRepDSTKI::new();
    tki.fill_on_geometry(l)?;
    let mut out = Vec::new();
    tki.init();
    while let Some((k, g, loi)) = tki.value() {
        out.push((k, g, loi.clone()));
        tki.next();
    }
    Ok(out)
}

/// The interferences of a list grouped by support in the `TopOpeBRepDS_TKI` order.
pub(crate) fn tki_support_groups(
    l: &ListOfInterference,
) -> Result<Vec<(TopOpeBRepDSKind, i32, ListOfInterference)>, Failure> {
    let mut tki = TopOpeBRepDSTKI::new();
    tki.fill_on_support(l)?;
    let mut out = Vec::new();
    tki.init();
    while let Some((k, g, loi)) = tki.value() {
        out.push((k, g, loi.clone()));
        tki.next();
    }
    Ok(out)
}

/// `occ::down_cast<TopOpeBRepDS_EdgeVertexInterference>(I)->GBound()`;
/// Standard_NullObject when `i` is no edge/vertex interference.
pub(crate) fn evi_gbound(i: &H) -> Result<bool, Failure> {
    let evi: Handle<TopOpeBRepDSEdgeVertexInterference> = i.cast();
    evi.get().map(|e| e.gbound()).ok_or_else(|| {
        Failure::with_type(
            "Standard_NullObject",
            "GBound: the interference is no TopOpeBRepDS_EdgeVertexInterference",
            "TopOpeBRepDS_EdgeVertexInterference",
        )
    })
}

/// The edge/vertex GBound when `i` is one, else false (`isevi ? GBound : false`).
fn gbound_if_evi(i: &H) -> bool {
    let evi: Handle<TopOpeBRepDSEdgeVertexInterference> = i.cast();
    evi.get().is_some_and(|e| e.gbound())
}

/// `FUN_ds_hasSDMancestorfaces(HDS, E1, E2, Fsd1, Fsd2)` (static): a connex face of
/// `e1` and a same domain face of it connex to `e2`.
fn has_sdm_ancestor_faces(
    bds: &TopOpeBRepDSDataStructure,
    e1: &Shape,
    e2: &Shape,
) -> Result<Option<(Face, Face)>, Failure> {
    let lfc_e1 = fdscnx_edge_connexity_same_shape_ds(e1, bds)?;
    let lfc_e2 = fdscnx_edge_connexity_same_shape_ds(e2, bds)?;
    let mut mapfc_e2: IndexedMap<Shape, ShapeMapHasher> = IndexedMap::default();
    for f in lfc_e2.iter() {
        mapfc_e2.add(f.clone());
    }
    for fc_e1 in lfc_e1.iter() {
        if !has_same_domain(bds, fc_e1) {
            continue;
        }
        for f2 in bds.shape_same_domain(fc_e1).iter() {
            if mapfc_e2.contains(f2) {
                return Ok(Some((topods::face(fc_e1)?, topods::face(f2)?)));
            }
        }
    }
    Ok(None)
}

/// One geometry of `FUN_ds_completeforSE1`.
fn se1_geometry(
    bds: &mut TopOpeBRepDSDataStructure,
    se: &Edge,
    k: TopOpeBRepDSKind,
    loi: &ListOfInterference,
) -> Result<(), Failure> {
    let ise = bds.shape_index(se, true);
    let mut loicopy = loi.clone();
    let mut l_i1 = ListOfInterference::default();
    if !fun_select_traint_interference(&loicopy, &mut l_i1)? {
        return Ok(());
    }
    let (mut l_i2, mut l_i3) = (ListOfInterference::default(), ListOfInterference::default());
    if fun_select_trasha_interference(&mut l_i1, ShapeEnum::Face, &mut l_i2)? < 1
        || fun_select_sk_interference(&mut l_i2, TopOpeBRepDSKind::Edge, &mut l_i3)? < 1
    {
        return Ok(());
    }
    for i in l_i3.iter() {
        if fds_si_is_gi_of_i_of_sba_of_t_of_i(bds, ise, i)? {
            return Ok(());
        }
    }
    for i in l_i3.iter() {
        let (_, g1, _, s) = fds_data(i);
        let es = topods::edge(bds.shape(s, true))?;
        let Some((_fse, fes)) = has_sdm_ancestor_faces(bds, se, &es)? else {
            continue;
        };
        let ifes = bds.shape_index(&fes, true);
        let mut lifcopy = bds.shape_interferences(&fes, true).clone();
        let mut lifound = ListOfInterference::default();
        if fun_select_gi_interference(&mut lifcopy, ise, &mut lifound)? < 1 {
            continue;
        }
        super::fds_copy(loi, &mut loicopy);
        let mut l_i4 = ListOfInterference::default();
        fun_select_itrasha_interference(&mut loicopy, ifes, &mut l_i4)?;
        let (mut lfor, mut lrev) = (ListOfInterference::default(), ListOfInterference::default());
        let nfor = fun_select_traori_interference(&mut l_i4, Orientation::Forward, &mut lfor)?;
        let nrev = fun_select_traori_interference(&mut l_i4, Orientation::Reversed, &mut lrev)?;
        if nfor > 0 || nrev > 0 {
            break;
        }
        let mut new_t = TopOpeBRepDSTransition::from_orientation(Orientation::Internal);
        new_t.set_index(ifes);
        let par = fds_parameter(i)?;
        let b = gbound_if_evi(i);
        let new_i = make_epv_interference_sk(&new_t, s, g1, par, k, TopOpeBRepDSKind::Edge, b)?;
        store_interference_ds(bds, &new_i, se)?;
        break;
    }
    Ok(())
}

/// `FUN_ds_completeforSE1(HDS)`: on each section edge, at a geometry with INTERNAL
/// FACE/EDGE interferences not seen by the faces before/after, an INTERNAL interference
/// on the edge of a same domain ancestor face is added; `fun_ds_complete_for_se1(&hds)?`.
pub fn fun_ds_complete_for_se1(hds: &Handle<TopOpeBRepDSHDataStructure>) -> Result<(), Failure> {
    let mut bds = href(hds)?.change_ds();
    for i in 1..=bds.nb_section_edges() {
        let se = bds.section_edge(i, true)?;
        let li = bds.shape_interferences(&se, true).clone();
        for (k, _, loi) in tki_groups(&li)? {
            se1_geometry(&mut bds, &se, k, &loi)?;
        }
    }
    Ok(())
}

/// The parameter on `es` of the vertex `g` for `FUN_ds_completeforSE2` (`None` to skip).
fn se2_vertex_par(
    bds: &TopOpeBRepDSDataStructure,
    g: i32,
    rk_se: i32,
    es: &Edge,
) -> Result<Option<f64>, Failure> {
    let vg = topods::vertex(bds.shape(g, true))?;
    if bds.ancestor_rank_index(g) == rk_se {
        if fun_tool_orient_v_in_e(&vg, es) == 0 {
            return Ok(None);
        }
        return BRepTool::parameter(&vg, es).map(Some);
    }
    if let Some(oov) = fun_ds_getoov(&vg, bds) {
        return BRepTool::parameter(&topods::vertex(&oov)?, es).map(Some);
    }
    let t = BRepTool::vertex_tolerance(&vg)?.max_native(fun_tool_maxtol(es)?);
    let (mut par_es, mut dd) = (0., 1.0e1);
    if !fun_tool_proj_pone(BRepTool::pnt(&vg)?, es, &mut par_es, &mut dd)? || dd > t {
        return Ok(None);
    }
    Ok(Some(par_es))
}

/// `(t1 > t2) ? t1 : t2`.
trait MaxNative {
    fn max_native(self, t2: f64) -> f64;
}

impl MaxNative for f64 {
    fn max_native(self, t2: f64) -> f64 {
        if self > t2 { self } else { t2 }
    }
}

/// The parameter on `es` of the point `g` for `FUN_ds_completeforSE2` (`None` to skip).
fn se2_point_par(
    bds: &TopOpeBRepDSDataStructure,
    g: i32,
    es: &Edge,
) -> Result<Option<f64>, Failure> {
    let p = bds.point(g)?;
    let t = p.tolerance().max_native(fun_tool_maxtol(es)?);
    let (mut oopar, mut d) = (0., 1.0e1);
    if !fun_tool_proj_pone(p.point(), es, &mut oopar, &mut d)? || d > t {
        return Ok(None);
    }
    Ok(Some(oopar))
}

/// The interference of one connex face `fcx` of `FUN_ds_completeforSE2` (none when
/// the transition cannot be computed).
#[allow(clippy::too_many_arguments)] // the native locals
fn se2_face(
    bds: &TopOpeBRepDSDataStructure,
    se: &Edge,
    (k, g, par): (TopOpeBRepDSKind, i32, f64),
    i: &H,
    es: &Edge,
    fcx: &Face,
    oopar: f64,
) -> Result<Option<H>, Failure> {
    let ifcx = bds.shape_index(fcx, true);
    let mut oouv = Point2::default();
    if !fun_tool_paron_ef(es, oopar, fcx, &mut oouv)? {
        return Ok(None);
    }
    let lse = bds.shape_interferences(se, true);
    let (par1, par2, _) = fds_lo_iinfsup(bds, se, par, k, g, lse)?;
    let Some((stb, sta)) = mkt_states(se, (par1, par2, par), fcx, oouv, 1.0e-4, Some((es, oopar)))?
    else {
        return Ok(None);
    };
    let mut new_t = TopOpeBRepDSTransition::default();
    new_t.set_before(stb, ShapeEnum::Face);
    new_t.set_after(sta, ShapeEnum::Face);
    new_t.set_index(ifcx);
    let b = if k == TopOpeBRepDSKind::Vertex {
        evi_gbound(i)?
    } else {
        false
    };
    make_epv_interference_sk(&new_t, ifcx, g, par, k, TopOpeBRepDSKind::Face, b).map(Some)
}

/// Whether the vertex `g` (or its same domain vertex) bounds the section edge.
fn vertex_on_se(bds: &TopOpeBRepDSDataStructure, g: i32, se: &Edge) -> Result<bool, Failure> {
    let vg = topods::vertex(bds.shape(g, true))?;
    let oov = if has_same_domain(bds, &vg) {
        fun_ds_getoov(&vg, bds)
    } else {
        None
    };
    let mut ov_se = fun_tool_orient_v_in_e(&vg, se);
    if ov_se == 0
        && let Some(oov) = &oov
    {
        ov_se = fun_tool_orient_v_in_e(&topods::vertex(oov)?, se);
    }
    Ok(ov_se != 0)
}

/// One geometry of `FUN_ds_completeforSE2`.
fn se2_geometry(
    bds: &mut TopOpeBRepDSDataStructure,
    se: &Edge,
    (k, g): (TopOpeBRepDSKind, i32),
    loi: &ListOfInterference,
) -> Result<(), Failure> {
    let rk_se = bds.ancestor_rank(se);
    if k == TopOpeBRepDSKind::Vertex && vertex_on_se(bds, g, se)? {
        return Ok(());
    }
    let mut loicopy = loi.clone();
    let (mut l1, mut lf, mut lfe): (ListOfInterference, ListOfInterference, ListOfInterference) =
        Default::default();
    fun_select_trasha_interference(&mut loicopy, ShapeEnum::Face, &mut l1)?;
    if fun_select_sk_interference(&mut l1, TopOpeBRepDSKind::Face, &mut lf)? > 1
        || fun_select_sk_interference(&mut l1, TopOpeBRepDSKind::Edge, &mut lfe)? == 0
    {
        return Ok(());
    }
    let lfe: &mut ListOfInterference = &mut lfe;
    let i = lfe.first().cloned().unwrap_or_default();
    let par = fds_parameter(&i)?;
    let (k, g, _, s) = fds_data(&i);
    let (_, isb, _, _) = fds_tdata(&i);
    let ftra = topods::face(bds.shape(isb, true))?;
    let es = topods::edge(bds.shape(s, true))?;
    for fcx in fdscnx_edge_connexity_same_shape_ds(&es, bds)?.iter() {
        let fcx = topods::face(fcx)?;
        if fcx.is_same(&ftra) {
            continue;
        }
        let mut l2 = ListOfInterference::default();
        if fun_select_itrasha_interference(lfe, bds.shape_index(&fcx, true), &mut l2)? > 0 {
            continue;
        }
        let oopar = match k {
            TopOpeBRepDSKind::Point => se2_point_par(bds, g, &es)?,
            TopOpeBRepDSKind::Vertex => se2_vertex_par(bds, g, rk_se, &es)?,
            _ => Some(0.),
        };
        let Some(oopar) = oopar else {
            continue;
        };
        if let Some(new_i) = se2_face(bds, se, (k, g, par), &i, &es, &fcx, oopar)? {
            store_interference_ds(bds, &new_i, se)?;
        }
    }
    Ok(())
}

/// `FUN_ds_completeforSE2(HDS)`: on each section edge, at a geometry with one FACE/EDGE
/// interference, the transitions on the other connex faces of its support edge are
/// computed and added; `fun_ds_complete_for_se2(&hds)?`.
pub fn fun_ds_complete_for_se2(hds: &Handle<TopOpeBRepDSHDataStructure>) -> Result<(), Failure> {
    let mut bds = href(hds)?.change_ds();
    for i in 1..=bds.nb_section_edges() {
        let se = bds.section_edge(i, true)?;
        let li = bds.shape_interferences(&se, true).clone();
        for (k, g, loi) in tki_groups(&li)? {
            se2_geometry(&mut bds, &se, (k, g), &loi)?;
        }
    }
    Ok(())
}

/// `FUN_ds_completeforSE3(BDS, SE, K, G, loi, parE, IES, ITRASHA, Tr)` (static): the
/// transition on the face of a 3d interference of an INTERNAL/EXTERNAL point (with its
/// edge, face and parameter); `None` when not applicable.
fn se3_transition(
    bds: &TopOpeBRepDSDataStructure,
    se: &Edge,
    k: TopOpeBRepDSKind,
    loi: &ListOfInterference,
) -> Result<Option<(f64, i32, TopOpeBRepDSTransition)>, Failure> {
    if k == TopOpeBRepDSKind::Vertex {
        return Ok(None);
    }
    let ise = bds.shape_index(se, true);
    let mut loicopy = loi.clone();
    let (mut lint, mut lext) = (ListOfInterference::default(), ListOfInterference::default());
    let nint = fun_select_traori_interference(&mut loicopy, Orientation::Internal, &mut lint)?;
    let next = fun_select_traori_interference(&mut loicopy, Orientation::External, &mut lext)?;
    if nint + next < 1 {
        return Ok(None);
    }
    let mut l1 = lint;
    super::fds_copy(&lext, &mut l1);
    let mut l2 = ListOfInterference::default();
    if fun_select_trasha_interference(&mut l1, ShapeEnum::Face, &mut l2)? < 1 {
        return Ok(None);
    }
    let mut okb = false;
    for i2 in l2.iter() {
        okb = fds_si_is_gi_of_i_of_sba_of_t_of_i(bds, ise, i2)?;
        if okb {
            break;
        }
    }
    if !okb {
        return Ok(None);
    }
    let (mut l2e, mut l2f) = (ListOfInterference::default(), ListOfInterference::default());
    let n2e = fun_select_sk_interference(&mut l2, TopOpeBRepDSKind::Edge, &mut l2e)?;
    let n2f = fun_select_sk_interference(&mut l2, TopOpeBRepDSKind::Face, &mut l2f)?;
    if n2e == 0 || n2f == 0 {
        return Ok(None);
    }
    let s_i3d = iref(l2f.first().unwrap_or(&H::default()))?.support();
    let mut l3d = ListOfInterference::default();
    if fun_select_itrasha_interference(&mut l2e, s_i3d, &mut l3d)? < 1 {
        return Ok(None);
    }
    let i3 = l3d.first().cloned().unwrap_or_default();
    se3_on_face(bds, se, &i3)
}

/// The transition of `FUN_ds_completeforSE3` on the face of `i3`.
fn se3_on_face(
    bds: &TopOpeBRepDSDataStructure,
    se: &Edge,
    i3: &H,
) -> Result<Option<(f64, i32, TopOpeBRepDSTransition)>, Failure> {
    let (_, _, _, s3) = fds_data(i3);
    let (_, isb3, _, _) = fds_tdata(i3);
    let (ies, itrasha) = (s3, isb3);
    let eline = topods::edge(bds.shape(ies, true))?;
    let f = topods::face(bds.shape(itrasha, true))?;
    let par_e = fds_parameter(i3)?;
    let mut parline = 0.;
    if !crate::top_ope_brep_tool::fun_tool_par_e(se, par_e, &eline, &mut parline)? {
        return Ok(None);
    }
    let mut uv = Point2::default();
    if !fun_tool_paron_ef(&eline, parline, &f, &mut uv)? {
        return Ok(None);
    }
    let (par1, par2) = crate::top_ope_brep_tool::fun_tool_bounds(se)?;
    let range = (par1, par2, par_e);
    let Some((stb, sta)) = mkt_states(se, range, &f, uv, 1.0e-4, Some((&eline, parline)))? else {
        return Ok(None);
    };
    let mut tr = TopOpeBRepDSTransition::default();
    tr.set_before(stb, ShapeEnum::Face);
    tr.set_after(sta, ShapeEnum::Face);
    tr.set_index(itrasha);
    let _ = ies;
    Ok(Some((par_e, itrasha, tr)))
}

/// `FUN_ds_completeforSE3(HDS)`: on each section edge, the INTERNAL/EXTERNAL point
/// interferences on a face crossed by the edge are replaced by FACE and EDGE
/// interferences of the computed transition; `fun_ds_complete_for_se3(&hds)?`.
pub fn fun_ds_complete_for_se3(hds: &Handle<TopOpeBRepDSHDataStructure>) -> Result<(), Failure> {
    let mut bds = href(hds)?.change_ds();
    for i in 1..=bds.nb_section_edges() {
        let se = bds.section_edge(i, true)?;
        let ise = bds.shape_index(&se, true);
        let li = bds.shape_interferences(&se, true).clone();
        let (mut new_li, mut hasnew_li) = (ListOfInterference::default(), false);
        for (k, g, loi) in tki_groups(&li)? {
            let tr = se3_transition(&bds, &se, k, &loi)?;
            let mut loicopy = loi.clone();
            let Some((par_e, itrasha, tr)) = tr else {
                super::fds_copy(&loicopy, &mut new_li);
                continue;
            };
            let mut l_idel = ListOfInterference::default();
            fun_select_itrasha_interference(&mut loicopy, itrasha, &mut l_idel)?;
            hasnew_li = true;
            let edge_kind = TopOpeBRepDSKind::Edge;
            let tr_fe = make_epv_interference_sk(&tr, ise, g, par_e, k, edge_kind, false)?;
            let face_kind = TopOpeBRepDSKind::Face;
            let tr_ff = make_epv_interference_sk(&tr, itrasha, g, par_e, k, face_kind, false)?;
            new_li.append(tr_ff);
            new_li.append(tr_fe);
            super::fds_copy(&loicopy, &mut new_li);
        }
        if hasnew_li {
            *bds.change_shape_interferences(&se) = new_li;
        }
    }
    Ok(())
}
