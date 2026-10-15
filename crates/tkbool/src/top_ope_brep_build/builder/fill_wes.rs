//! The wire edge sets of the same domain faces (TopOpeBRepBuild_ffwesk.cxx) and of the
//! curves of a face (TopOpeBRepBuild_fctwes.cxx).
use super::TopOpeBRepBuildBuilder;
use super::grid_ee::fun_aresamegeom;
use crate::top_ope_brep_build::{
    TopOpeBRepBuildGTopo, TopOpeBRepBuildShapeSet, TopOpeBRepBuildWireEdgeSet,
};
use crate::top_ope_brep_ds::{TopOpeBRepDSConfig, TopOpeBRepDSCurveIterator};
use crate::top_ope_brep_tool::{fc2d_curve_on_surface, fc2d_has_curve_on_surface};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Shape};
use rustycad_tkbrep::toptools::ShapeMapHasher;
use rustycad_tkernel::ncollection::{IndexedMap, List};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{self, ShapeEnum, State};

impl TopOpeBRepBuildBuilder {
    /// The face of a same domain list filled (Kfill 1 faces, 2 curves, 3 ON parts) with
    /// the table reversed for a face not oriented as the reference.
    #[allow(clippy::too_many_arguments)]
    fn fill_face_k(
        &mut self,
        s: &Shape,
        lsclass: &List<Shape>,
        g: &mut TopOpeBRepBuildGTopo,
        gistoreverse1: bool,
        oref: rustycad_tkg3d::top_abs::Orientation,
        wes: &mut TopOpeBRepBuildWireEdgeSet,
        kfill: i32,
    ) -> Result<(), Failure> {
        let (os, conf) = {
            let hds = self.hds()?;
            let bds = hds.ds();
            (
                bds.shape(hds.shape_index(s, true), true).orientation(),
                bds.same_domain_ori(s),
            )
        };
        let b1 = conf == TopOpeBRepDSConfig::DiffOriented && os == oref;
        let b2 = conf == TopOpeBRepDSConfig::SameOriented && os != oref;
        if b1 || b2 {
            g.set_reverse(!gistoreverse1);
        }
        match kfill {
            1 => self.g_fill_face_wes(s, lsclass, g, wes),
            2 => self.g_fill_curve_topology_wes(s, g, wes),
            3 => self.g_fill_on_parts_wes(s, g, lsclass, wes),
            _ => Ok(()),
        }
    }

    /// `GFillFacesWESK(LS1, LS2, G1, WES, Kfill)`: the faces of each list not merged
    /// yet filled into the wire edge set (Kfill 1 faces, 2 curves, 3 ON parts), the
    /// table permuted for the second list; `b.g_fill_faces_wesk(&l1, &l2, &g, &mut wes, 1)?`.
    pub fn g_fill_faces_wesk(
        &mut self,
        ls1: &List<Shape>,
        ls2: &List<Shape>,
        g1: &TopOpeBRepBuildGTopo,
        wes: &mut TopOpeBRepBuildWireEdgeSet,
        kfill: i32,
    ) -> Result<(), Failure> {
        let Some(f1) = ls1.first().cloned() else {
            return Ok(());
        };
        let (tb1, tb2) = g1.states_on()?;
        self.my_face_reference = topods::face(&f1)?;
        let oref = {
            let bds = self.hds()?.ds();
            bds.shape(bds.same_domain_ref(&f1), true).orientation()
        };
        for (g0, ls, lo, tb) in [(*g1, ls1, ls2, tb1), (g1.copy_permuted(), ls2, ls1, tb2)] {
            let mut g = g0;
            let gistoreverse1 = g.is_to_reverse1()?;
            for s in ls.iter() {
                if !self.is_merged(s, tb) {
                    self.fill_face_k(s, lo, &mut g, gistoreverse1, oref, wes, kfill)?;
                }
            }
        }
        Ok(())
    }

    /// `GFillCurveTopologyWES(F1, G1, WES)`: the new edges of the curves of the face
    /// (on transitions of the first type of the table, not on a face of its same
    /// domain) as start elements; `b.g_fill_curve_topology_wes(&f, &g, &mut wes)?`.
    pub fn g_fill_curve_topology_wes(
        &mut self,
        f1: &Shape,
        g1: &TopOpeBRepBuildGTopo,
        wes: &mut TopOpeBRepBuildWireEdgeSet,
    ) -> Result<(), Failure> {
        g1.states_on()?;
        let (shape_interf, _) = g1.type_();
        let hds = self.my_data_structure.clone();
        let h = hds.get().ok_or_else(super::build::null_hds)?;
        let mut fcit = h.face_curves(f1)?;
        self.my_face_reference = topods::face(f1)?;
        self.my_face_to_fill = topods::face(f1)?;
        let mut a_sd_map: IndexedMap<Shape, ShapeMapHasher> = IndexedMap::default();
        if h.has_same_domain(f1, true) {
            for sdf in h.same_domain(f1).iter() {
                a_sd_map.add(sdf.clone());
            }
        }
        while fcit.more() {
            let mut take = true;
            if shape_interf != ShapeEnum::Shape {
                let i = fcit.value()?.clone();
                let ii = i.get().ok_or_else(super::build::null_hds)?;
                let t = ii.transition();
                take = t.shape_after() == shape_interf
                    && t.shape_before() == shape_interf
                    && !a_sd_map.contains(&h.shape(ii.support(), true));
            }
            if take {
                self.g_fill_curve_topology_wes_at(&fcit, g1, wes)?;
            }
            fcit.next()?;
        }
        Ok(())
    }

    /// Whether the edge is already a start element of the set.
    fn is_start_element(wes: &TopOpeBRepBuildWireEdgeSet, e: &Shape) -> bool {
        wes.start_elements().iter().any(|s| e.is_same(s))
    }

    /// `GFillCurveTopologyWES(FCit, G1, WES)`: the new edges of the current curve (but
    /// the degenerated ones and those of an equivalent curve already there), oriented
    /// on the state, with p-curves on the face to fill and the face of the set, as start
    /// elements; nothing in the common of a cut or for a restriction of the face of the
    /// set; `b.g_fill_curve_topology_wes_at(&it, &g, &mut wes)?`.
    pub fn g_fill_curve_topology_wes_at(
        &self,
        fcit: &TopOpeBRepDSCurveIterator,
        g1: &TopOpeBRepBuildGTopo,
        wes: &mut TopOpeBRepBuildWireEdgeSet,
    ) -> Result<(), Failure> {
        if !fcit.more() {
            return Ok(());
        }
        let (tb1, tb2) = g1.states_on()?;
        let mut wesf = wes.face().clone().into_shape();
        let mut ftf = self.my_face_to_fill.clone().into_shape();
        let com_of_cut = (self.opec12() || self.opec21()) && tb1 == tb2 && tb1 == State::In;
        let t = fcit
            .value()?
            .get()
            .ok_or_else(super::build::null_hds)?
            .transition();
        let mut neworie = t.orientation(tb1, ShapeEnum::Face);
        if !fun_aresamegeom(&ftf, &wesf)? {
            neworie = top_abs::complement(neworie);
        }
        if com_of_cut {
            return Ok(());
        }
        let ig = fcit.current()?;
        let c = self.hds()?.curve(ig);
        if c.is_equivalent_curve_reversed() {
            neworie = top_abs::reverse(neworie);
        }
        let restriction = c.existing_edge();
        if !restriction.is_null()
            && Explorer::new(&wesf, ShapeEnum::Edge, ShapeEnum::Shape)?
                .any(|e| restriction.as_ref().is_same(&e))
        {
            return Ok(());
        }
        for ee in self.new_edges(ig).clone().iter() {
            if c.equivalent_curve() > 0 && Self::is_start_element(wes, ee) {
                continue;
            }
            let e = topods::edge(ee)?;
            if BRepTool::degenerated(&e)? {
                continue;
            }
            let mut e = e.oriented(neworie);
            let pc = fcit.pcurve()?;
            if !fc2d_has_curve_on_surface(&topods::edge(&e)?, &topods::face(&ftf)?)? {
                self.my_build_tool
                    .pcurve_of_curve(&mut ftf, &mut e, &c, &pc)?;
            }
            if !fc2d_has_curve_on_surface(&topods::edge(&e)?, &topods::face(&wesf)?)? {
                let (c2d, _, _, _) =
                    fc2d_curve_on_surface(&topods::edge(&e)?, &topods::face(&wesf)?, false)?;
                if c2d.is_null() {
                    return Err(Failure::with_type(
                        "Standard_ProgramError",
                        "GFillCurveTopologyWES",
                        "TopOpeBRepBuild_Builder",
                    ));
                }
                self.my_build_tool.pcurve(&mut wesf, &mut e, &c2d)?;
            }
            wes.add_start_element(&e)?;
        }
        Ok(())
    }
}
