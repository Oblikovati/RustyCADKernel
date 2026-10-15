//! The section edges along V isos of periodic faces split at the period bound
//! (SplitEvisoONperiodicF of TopOpeBRepBuild_GridSS.cxx), and GSplitFace.
use super::TopOpeBRepBuildBuilder;
use super::build::null_hds;
use crate::top_ope_brep_build::{
    TopOpeBRepBuildGTopo, TopOpeBRepBuildShapeSet, TopOpeBRepBuildWireEdgeSet,
};
use crate::top_ope_brep_ds::export::fds_data;
use crate::top_ope_brep_ds::process_interferences_tool::{
    fun_select_gk_interference, fun_select_sk_interference, fun_select_trasha_interference,
};
use crate::top_ope_brep_ds::{
    ListOfInterference, TopOpeBRepDSCurvePointInterference, TopOpeBRepDSHDataStructure,
    TopOpeBRepDSKind, TopOpeBRepDSTransition,
};
use crate::top_ope_brep_tool::{
    TopOpeBRepToolCORRISO, fc2d_curve_on_surface, fun_tool_closed_s_flag, fun_tool_pcurve_on_f,
    fun_tool_value_on_edge, uviso,
};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Edge, Face, Shape};
use rustycad_tkernel::ncollection::{IndexedMap, List};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum, State};
use rustycad_tkmath::gp::{Direction2, Point2, Point3};

/// `SPLITEDGE`: the code of EdgeOUTofBoundsUV for an edge to split.
const SPLITEDGE: i32 = 0;

/// `FUN_getG(P, LI, HDS, iEinterf)` (static): the point of a curve/point interference
/// of the list at `p` (its tolerance), with the support of the interference (0 none).
fn fun_get_g(
    p: Point3,
    li: &ListOfInterference,
    hds: &TopOpeBRepDSHDataStructure,
) -> Result<(i32, i32), Failure> {
    let mut i_einterf = 0;
    for i in li.iter() {
        let ssi: Handle<TopOpeBRepDSCurvePointInterference> = i.clone().cast();
        let Some(s) = ssi.get() else {
            continue;
        };
        let gi = s.geometry();
        i_einterf = s.support();
        let dsp = hds.point(gi)?;
        if dsp.point().is_equal(&p, dsp.tolerance()) {
            return Ok((gi, i_einterf));
        }
    }
    Ok((0, i_einterf))
}

/// `FUN_EPIforEvisoONperiodicF(E, F, EPIlist, HDS, loCPI)` (static): the curve/point
/// interference at the split parameter of an edge crossing the period bound of the
/// face, at a point of the face edge interferences; false when none.
fn fun_epi_for_eviso_on_periodic_f(
    e: &Edge,
    f: &Face,
    epilist: &ListOfInterference,
    hds: &TopOpeBRepDSHDataStructure,
    locpi: &mut ListOfInterference,
) -> Result<bool, Failure> {
    let mut corriso = TopOpeBRepToolCORRISO::with_face(f)?;
    corriso.init(f.as_ref())?;
    let on_u = corriso.refclosed(1).is_some_and(|(c, _)| c);
    let tolu = corriso.tol(1, BRepTool::face_tolerance(f)?)?;
    let (recadre, parone) = corriso.edge_out_of_bounds_uv(e, on_u, tolu)?;
    if recadre != SPLITEDGE {
        return Ok(false);
    }
    let Some(p3d) = fun_tool_value_on_edge(parone, e)? else {
        return Ok(false);
    };
    let (ig, mut i_einterf) = fun_get_g(p3d, epilist, hds)?;
    if ig == 0 {
        return Ok(false);
    }
    if hds.shape(i_einterf, true).shape_type()? != ShapeEnum::Edge {
        i_einterf = 0;
    }
    let is = hds.shape_index(e.as_ref(), true);
    let mut t = TopOpeBRepDSTransition::new(State::In, State::In, ShapeEnum::Edge, ShapeEnum::Edge);
    t.set_index(is);
    let cpi = TopOpeBRepDSCurvePointInterference::new(
        &t,
        TopOpeBRepDSKind::Edge,
        i_einterf,
        TopOpeBRepDSKind::Point,
        ig,
        parone,
    );
    locpi.append(Handle::new(cpi).cast());
    Ok(true)
}

/// `FUN_getEPIonEds(FOR, HDS, EPI)` (static, with FUN_getEPI): the curve/point
/// interferences of POINT geometry on a FACE of the edges of the face.
fn fun_get_epi_on_eds(
    for_: &Shape,
    hds: &TopOpeBRepDSHDataStructure,
) -> Result<ListOfInterference, Failure> {
    let mut epi = ListOfInterference::default();
    for e in Explorer::new(for_, ShapeEnum::Edge, ShapeEnum::Shape)? {
        if !hds.has_shape(&e, true) {
            continue;
        }
        for i in hds.ds().shape_interferences(&e, true).iter() {
            let cpi: Handle<TopOpeBRepDSCurvePointInterference> = i.clone().cast();
            if cpi.get().is_none() {
                continue;
            }
            let (gt, _, st, _) = fds_data(i);
            if gt == TopOpeBRepDSKind::Point && st == TopOpeBRepDSKind::Face {
                epi.append(i.clone());
            }
        }
    }
    Ok(epi)
}

/// The EDGE interferences on a FACE of FACE transitions of the face.
fn face_edge_face_interferences(
    hds: &TopOpeBRepDSHDataStructure,
    ff: &Shape,
) -> Result<ListOfInterference, Failure> {
    let mut li = hds.ds().shape_interferences(ff, true).clone();
    let (mut li1, mut li2, mut li3) = (
        ListOfInterference::default(),
        ListOfInterference::default(),
        ListOfInterference::default(),
    );
    if fun_select_gk_interference(&mut li, TopOpeBRepDSKind::Edge, &mut li1)? < 1
        || fun_select_sk_interference(&mut li1, TopOpeBRepDSKind::Face, &mut li2)? < 1
        || fun_select_trasha_interference(&mut li2, ShapeEnum::Face, &mut li3)? < 1
    {
        return Ok(ListOfInterference::default());
    }
    Ok(li3)
}

/// `FUN_SplitEvisoONperiodicF(HDS, FF)` (static): the section edges of the face along a
/// V iso of a closed face given the curve/point interference of their split at the
/// period bound; Standard_ProgramError when no p-curve can be made.
fn fun_split_eviso_on_periodic_f(
    hds: &Handle<TopOpeBRepDSHDataStructure>,
    ff: &Shape,
) -> Result<(), Failure> {
    let h = hds.get().ok_or_else(null_hds)?;
    for i in face_edge_face_interferences(h, ff)?.iter() {
        let (_, gi, _, si) = fds_data(i);
        let fs = topods::face(&h.shape(si, true))?;
        if !fun_tool_closed_s_flag(fs.as_ref())? {
            continue;
        }
        let eg = topods::edge(&h.shape(gi, true))?;
        if !h.ds().is_section_edge(&eg, true) {
            continue;
        }
        let (mut pc, _, _, _) = fc2d_curve_on_surface(&eg, &fs, false)?;
        if pc.is_null() {
            if !fun_tool_pcurve_on_f(&fs, &eg)? {
                return Err(Failure::with_type(
                    "Standard_ProgramError",
                    "_Builder::SplitONVisolineonCyl",
                    "TopOpeBRepBuild_Builder",
                ));
            }
            pc = fc2d_curve_on_surface(&eg, &fs, false)?.0;
        }
        let (mut d2d, mut o2d) = (Direction2::default(), Point2::default());
        let (_, _, viso) = uviso(&pc, &mut d2d, &mut o2d)?;
        if !viso {
            continue;
        }
        let epilist = fun_get_epi_on_eds(fs.as_ref(), h)?;
        let mut locpi = ListOfInterference::default();
        fun_epi_for_eviso_on_periodic_f(&eg, &fs, &epilist, h, &mut locpi)?;
        h.change_ds()
            .change_shape_interferences(eg.as_ref())
            .append_list(&mut locpi);
    }
    Ok(())
}

impl TopOpeBRepBuildBuilder {
    /// `SplitEvisoONperiodicF()`: the faces of the structure on a periodic surface have
    /// their V iso section edges split at the period bound; `b.split_eviso_on_periodic_f()?`.
    pub fn split_eviso_on_periodic_f(&mut self) -> Result<(), Failure> {
        let hds = self.my_data_structure.clone();
        let h = hds.get().ok_or_else(null_hds)?;
        for i in 1..=h.nb_shapes() {
            let for_ = h.shape(i, true);
            if for_.shape_type()? != ShapeEnum::Face {
                continue;
            }
            let su = BRepTool::surface(&topods::face(&for_)?)?;
            let s = su.get().ok_or_else(null_hds)?;
            if !(s.is_u_periodic() || s.is_v_periodic()) {
                continue;
            }
            fun_split_eviso_on_periodic_f(&hds, &for_.oriented(Orientation::Forward))?;
        }
        Ok(())
    }

    /// `GSplitFace(FOR, GG1, LSclass)`: the faces of the wire edge set of the face (its
    /// ON parts, its kept wires and edges, its curves), the ON patches made, the faces
    /// kept on the state as its splits; `b.g_split_face(&f, &g, &lc)?`.
    pub fn g_split_face(
        &mut self,
        for_: &Shape,
        gg1: &TopOpeBRepBuildGTopo,
        lsclass: &List<Shape>,
    ) -> Result<(), Failure> {
        let mut g1 = *gg1;
        g1.set_reverse(false);
        let (tb1, _) = g1.states_on()?;
        let ff = for_.oriented(Orientation::Forward);
        let mut wes = TopOpeBRepBuildWireEdgeSet::new(&ff)?;
        self.g_fill_on_parts_wes(for_, &g1, lsclass, &mut wes)?;
        let mut an_edges_on = List::default();
        if self.my_process_on {
            let to_rev = self.opefus();
            for e in wes.start_elements().iter() {
                an_edges_on.append(if to_rev { e.reversed() } else { e.clone() });
            }
            self.my_on_elem_map.clear(true);
        }
        self.g_fill_face_wes(&ff, lsclass, &g1, &mut wes)?;
        let n1 = wes.start_elements().len() as i32;
        self.g_fill_curve_topology_wes(&ff, &g1, &mut wes)?;
        let n2 = wes.start_elements().len() as i32;
        self.my_edge_avoid.clear();
        Self::g_copy_list_range(
            &wes.start_elements().clone(),
            n1 + 1,
            n2,
            &mut self.my_edge_avoid,
        );
        self.mark_split(&ff, tb1, true);
        let mut lof = List::default();
        self.gwes_make_faces(&ff, &mut wes, &mut lof)?;
        if self.my_process_on && (!an_edges_on.is_empty() || !self.my_on_elem_map.is_empty()) {
            let mut a_map_oe: IndexedMap<Shape> = IndexedMap::default();
            for f in lof.iter() {
                for e in Explorer::new(f, ShapeEnum::Edge, ShapeEnum::Shape)? {
                    a_map_oe.add(e);
                }
            }
            self.fill_on_patches(&an_edges_on, for_, &a_map_oe)?;
            self.my_on_elem_map.clear(true);
        }
        self.change_split(&ff, tb1).clear();
        let mut lofs = List::default();
        let empty = self.my_empty_shape_list.clone();
        self.g_keep_shapes(&ff, &empty, tb1, &lof, &mut lofs)?;
        *self.change_split(&ff, tb1) = lofs;
        Ok(())
    }
}
