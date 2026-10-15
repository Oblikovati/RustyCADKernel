// Rust translation of ModelingAlgorithms/TKBool/TopOpeBRepDS/TopOpeBRepDS_TOOL.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TopOpeBRepDS_TOOL`: the same domain edges sharing geometry with an edge, and their
//! relative configuration. The `_ds` twins take the locked structure.
use super::export::{fun_ds_getoov, has_same_domain, href, iref};
use super::process_interferences_tool::fun_select_trasha_interference;
use super::{
    ListOfInterference, TopOpeBRepDSConfig, TopOpeBRepDSDataStructure, TopOpeBRepDSHDataStructure,
    TopOpeBRepDSInterference, TopOpeBRepDSListOfShapeOn1State, TopOpeBRepDSShapeShapeInterference,
};
use crate::top_ope_brep_tool::{
    TopOpeBRepToolTOOL, fun_tool_bounds, fun_tool_curves_so_at_both, fun_tool_par_e_tol,
};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Edge, Shape};
use rustycad_tkbrep::toptools::ShapeMapHasher;
use rustycad_tkernel::ncollection::{DataMap, List, Map};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};

/// The split edges ON of edges (`MEspON`).
pub type MapOfShapeOn1State = DataMap<Shape, TopOpeBRepDSListOfShapeOn1State, ShapeMapHasher>;

/// `GetConfig` result: same oriented.
const SAMEORIENTED: i32 = 1;
/// `GetConfig` result: opposite oriented.
const DIFFORIENTED: i32 = 2;

/// `TopOpeBRepDS_TOOL` (static members only).
pub struct TopOpeBRepDSTOOL;

/// The 1d interferences split as `FDS_sortGb`: (GBound false, GBound true, geometry
/// with same domain).
fn sort_gb(
    bds: &TopOpeBRepDSDataStructure,
    li: &ListOfInterference,
) -> Result<[Vec<Handle<TopOpeBRepDSInterference>>; 3], Failure> {
    let mut out: [Vec<_>; 3] = Default::default();
    for i in li.iter() {
        let ssi: Handle<TopOpeBRepDSShapeShapeInterference> = i.cast();
        let Some(ssi) = ssi.get() else {
            out[0].push(i.clone());
            continue;
        };
        let gb1 = ssi.gbound();
        if has_same_domain(bds, bds.shape(iref(i)?.geometry(), true)) {
            out[2].push(i.clone());
        } else {
            out[usize::from(gb1)].push(i.clone());
        }
    }
    Ok(out)
}

/// Whether the same domain edge `esd` of a 1d interference `i` at a vertex with a same
/// domain vertex shares the geometry of `e` (inside tangents agreeing, or a vertex
/// inside or closing).
fn shares_at_sd_vertex(
    bds: &TopOpeBRepDSDataStructure,
    e: &Edge,
    esd: &Edge,
    i: &Handle<TopOpeBRepDSInterference>,
) -> Result<bool, Failure> {
    let vg = topods::vertex(bds.shape(iref(i)?.geometry(), true))?;
    let Some(vsd) = fun_ds_getoov(&vg, bds) else {
        return Ok(false);
    };
    let vsd = topods::vertex(&vsd)?;
    let ssi: Handle<TopOpeBRepDSShapeShapeInterference> = i.cast();
    let gb1 = ssi.get().is_some_and(|s| s.gbound());
    let (v_e, v_esd) = if gb1 { (&vg, &vsd) } else { (&vsd, &vg) };
    let (mut ov_e, mut ov_esd) = (0, 0);
    let Some(tg_e) = TopOpeBRepToolTOOL::tg_inside(v_e, e, &mut ov_e)? else {
        return Ok(false);
    };
    let Some(tg_esd) = TopOpeBRepToolTOOL::tg_inside(v_esd, esd, &mut ov_esd)? else {
        return Ok(false);
    };
    let inside = |o: i32| o == TOOL_CLOSING || o == TOOL_INTERNAL;
    if inside(ov_e) || inside(ov_esd) {
        return Ok(true);
    }
    let d = |v: rustycad_tkmath::gp::Vector3| rustycad_tkmath::gp::Direction3::from_xyz(*v.xyz());
    Ok(d(tg_e)?.dot(d(tg_esd)?) > 0.)
}

use crate::top_ope_brep_tool::{TOOL_CLOSING, TOOL_INTERNAL};

impl TopOpeBRepDSTOOL {
    /// `EShareG(HDS, E, lEsd)` on the locked structure.
    pub fn e_share_g_ds(
        bds: &TopOpeBRepDSDataStructure,
        e: &Edge,
        l_esd: &mut List<Shape>,
    ) -> Result<i32, Failure> {
        l_esd.clear();
        if BRepTool::degenerated(e)? {
            if !has_same_domain(bds, e) {
                return Ok(0);
            }
            for s in bds.shape_same_domain(e).iter() {
                l_esd.append(s.clone());
            }
            return Ok(l_esd.extent());
        }
        let mut lii = bds.shape_interferences(e, true).clone();
        let mut l1d = ListOfInterference::default();
        if fun_select_trasha_interference(&mut lii, ShapeEnum::Edge, &mut l1d)? == 0 {
            return Ok(0);
        }
        let mut mapesd: Map<Shape, ShapeMapHasher> = Map::default();
        let [gb0, gb1, gbsd] = sort_gb(bds, &l1d)?;
        for i in gb0.iter().chain(gb1.iter()) {
            mapesd.add(bds.shape(iref(i)?.support(), true).clone());
        }
        for i in &gbsd {
            let esd = topods::edge(bds.shape(iref(i)?.support(), true))?;
            if mapesd.contains(esd.as_ref()) {
                continue;
            }
            if shares_at_sd_vertex(bds, e, &esd, i)? {
                mapesd.add(esd.into_shape());
            }
        }
        for s in mapesd.iter() {
            l_esd.append(s.clone());
        }
        Ok(l_esd.extent())
    }

    /// `EShareG(HDS, E, lEsd)`: the same domain edges sharing the geometry of `e` (all
    /// of them for a degenerated edge) in `l_esd`; their count;
    /// `TopOpeBRepDSTOOL::e_share_g(&hds, &e, &mut l)?`.
    pub fn e_share_g(
        hds: &Handle<TopOpeBRepDSHDataStructure>,
        e: &Edge,
        l_esd: &mut List<Shape>,
    ) -> Result<i32, Failure> {
        Self::e_share_g_ds(&href(hds)?.ds(), e, l_esd)
    }

    /// `ShareG(HDS, i1, i2)` on the locked structure.
    pub fn share_g_ds(bds: &TopOpeBRepDSDataStructure, i1: i32, i2: i32) -> bool {
        let (s1, s2) = (bds.shape(i1, true), bds.shape(i2, true));
        has_same_domain(bds, s1) && bds.shape_same_domain(s1).iter().any(|s| s.is_same(s2))
    }

    /// `ShareG(HDS, i1, i2)`: the shape `i2` same domain as `i1`;
    /// `TopOpeBRepDSTOOL::share_g(&hds, 1, 2)?`.
    pub fn share_g(
        hds: &Handle<TopOpeBRepDSHDataStructure>,
        i1: i32,
        i2: i32,
    ) -> Result<bool, Failure> {
        Ok(Self::share_g_ds(&href(hds)?.ds(), i1, i2))
    }

    /// `GetEsd(HDS, S, ie, iesd)` on the locked structure.
    pub fn get_esd_ds(
        bds: &TopOpeBRepDSDataStructure,
        s: &Shape,
        ie: i32,
    ) -> Result<Option<i32>, Failure> {
        let mut mesd_s: Map<Shape, ShapeMapHasher> = Map::default();
        for e in Explorer::new(s, ShapeEnum::Edge, ShapeEnum::Shape)? {
            if bds.has_shape(&e, true) && has_same_domain(bds, &e) {
                mesd_s.add(e);
            }
        }
        for esd in bds.shape_same_domain(bds.shape(ie, true)).iter() {
            if mesd_s.contains(esd) {
                return Ok(Some(bds.shape_index(esd, true)));
            }
        }
        Ok(None)
    }

    /// `GetEsd(HDS, S, ie, iesd)`: the index of an edge of `s` same domain as the edge
    /// `ie`; `TopOpeBRepDSTOOL::get_esd(&hds, &face, 3)?`.
    pub fn get_esd(
        hds: &Handle<TopOpeBRepDSHDataStructure>,
        s: &Shape,
        ie: i32,
    ) -> Result<Option<i32>, Failure> {
        Self::get_esd_ds(&href(hds)?.ds(), s, ie)
    }

    /// `ShareSplitON(HDS, MEspON, i1, i2, spON)` on the locked structure.
    pub fn share_split_on_ds(
        bds: &TopOpeBRepDSDataStructure,
        mesp_on: &MapOfShapeOn1State,
        i1: i32,
        i2: i32,
    ) -> Result<Option<Shape>, Failure> {
        if !Self::share_g_ds(bds, i1, i2) {
            return Ok(None);
        }
        let los1 = mesp_on.find(bds.shape(i1, true))?;
        if !los1.is_split() || los1.list_on_state().is_empty() {
            return Ok(None);
        }
        let mut mesp1: Map<Shape, ShapeMapHasher> = Map::default();
        for s in los1.list_on_state().iter() {
            mesp1.add(s.clone());
        }
        let los2 = mesp_on.find(bds.shape(i2, true))?;
        if !los2.is_split() {
            return Ok(None);
        }
        Ok(los2
            .list_on_state()
            .iter()
            .find(|esp| mesp1.contains(esp))
            .cloned())
    }

    /// `ShareSplitON(HDS, MEspON, i1, i2, spON)`: a split ON common to the same domain
    /// shapes `i1` and `i2` (Standard_NoSuchObject when one is not in `mesp_on`);
    /// `TopOpeBRepDSTOOL::share_split_on(&hds, &m, 1, 2)?`.
    pub fn share_split_on(
        hds: &Handle<TopOpeBRepDSHDataStructure>,
        mesp_on: &MapOfShapeOn1State,
        i1: i32,
        i2: i32,
    ) -> Result<Option<Shape>, Failure> {
        Self::share_split_on_ds(&href(hds)?.ds(), mesp_on, i1, i2)
    }

    /// `HDS->SameDomainOrientation(S)` on the structure.
    fn sd_ori(bds: &TopOpeBRepDSDataStructure, s: &Shape) -> TopOpeBRepDSConfig {
        if bds.has_shape(s, true) {
            bds.same_domain_ori(s)
        } else {
            TopOpeBRepDSConfig::UnshGeometry
        }
    }

    /// `GetConfig(HDS, MEspON, ie, iesd, config)` on the locked structure.
    pub fn get_config_ds(
        bds: &TopOpeBRepDSDataStructure,
        mesp_on: &MapOfShapeOn1State,
        ie: i32,
        iesd: i32,
    ) -> Result<Option<i32>, Failure> {
        if !Self::share_g_ds(bds, ie, iesd) {
            return Ok(None);
        }
        let e = topods::edge(bds.shape(ie, true))?;
        let esd = topods::edge(bds.shape(iesd, true))?;
        let (conf, confsd) = (Self::sd_ori(bds, &e), Self::sd_ori(bds, &esd));
        let unsh = |c| c == TopOpeBRepDSConfig::UnshGeometry;
        if !unsh(conf) && !unsh(confsd) {
            let mut sameori = conf == confsd;
            sameori ^= e.orientation() == Orientation::Reversed;
            sameori ^= esd.orientation() == Orientation::Reversed;
            return Ok(Some(if sameori { SAMEORIENTED } else { DIFFORIENTED }));
        }
        let Some(e_on) = Self::share_split_on_ds(bds, mesp_on, ie, iesd)? else {
            return Ok(None);
        };
        let e_on = topods::edge(&e_on)?;
        let (f, l) = fun_tool_bounds(&e_on)?;
        let x = 0.45678;
        let par_on = (1. - x) * f + x * l;
        let (mut pare, mut paresd, mut so) = (0., 0., false);
        if !fun_tool_par_e_tol(&e_on, par_on, &e, &mut pare, BRepTool::edge_tolerance(&e)?)? {
            return Ok(None);
        }
        let tolesd = BRepTool::edge_tolerance(&esd)?;
        if !fun_tool_par_e_tol(&e_on, par_on, &esd, &mut paresd, tolesd)? {
            return Ok(None);
        }
        if !fun_tool_curves_so_at_both(&e, pare, &esd, paresd, &mut so)? {
            return Ok(None);
        }
        Ok(Some(if so { SAMEORIENTED } else { DIFFORIENTED }))
    }

    /// `GetConfig(HDS, MEspON, ie, iesd, config)`: 1 (same oriented) or 2 (opposite)
    /// for the same domain edges `ie` and `iesd`, from their configurations or along a
    /// common split ON; `TopOpeBRepDSTOOL::get_config(&hds, &m, 1, 2)?`.
    pub fn get_config(
        hds: &Handle<TopOpeBRepDSHDataStructure>,
        mesp_on: &MapOfShapeOn1State,
        ie: i32,
        iesd: i32,
    ) -> Result<Option<i32>, Failure> {
        Self::get_config_ds(&href(hds)?.ds(), mesp_on, ie, iesd)
    }
}
