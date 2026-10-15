//! The split of one section edge by state and the section accessors of the builder (TopOpeBRepBuild_Section.cxx).
use super::TopOpeBRepBuildBuilder;
use super::build::null_hds;
use crate::top_ope_brep_build::globals::{GLOBAL_HASSD, GLOBAL_ISSP, THE_IN, THE_ON, THE_OUT};
use crate::top_ope_brep_build::{
    TopOpeBRepBuildEdgeBuilder, TopOpeBRepBuildGTool, TopOpeBRepBuildGTopo,
    TopOpeBRepBuildPaveClassifier, TopOpeBRepBuildPaveSet,
};
use crate::top_ope_brep_ds::TopOpeBRepDSCurveExplorer;
use crate::top_ope_brep_ds::export::{
    fds_has_same_domain2d, fds_has_same_domain3d, fun_ds_getoov_hds,
};
use crate::top_ope_brep_tool::{TopOpeBRepToolTOOL, fun_ds_copy_edge, fun_ds_parameter};
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Edge, Shape};
use rustycad_tkbrep::toptools::ShapeMapHasher;
use rustycad_tkernel::ncollection::{DataMap, List, Map};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum, State};

impl TopOpeBRepBuildBuilder {
    /// `SplitSectionEdge(E)`: the section edge split ON, IN and OUT of the faces (a copy
    /// with its same domain reference vertices, or itself, ON when it has no geometry nor
    /// 3d same domain or all its vertex interferences are on its bounds);
    /// `b.split_section_edge(&e)?`.
    pub fn split_section_edge(&mut self, ea: &Shape) -> Result<(), Failure> {
        let hds = self.my_data_structure.clone();
        let h = hds.get().ok_or_else(null_hds)?;
        let eor = topods::edge(ea)?;
        let ef = topods::edge(&eor.oriented(Orientation::Forward))?;
        let (rank_ef, hsd3d, hsd2d) = {
            let bds = h.ds();
            (
                bds.ancestor_rank(ef.as_ref()),
                fds_has_same_domain3d(&bds, ea, None),
                fds_has_same_domain2d(&bds, ea, None),
            )
        };
        let hg = h.has_geometry(ea);
        let cond = hg || hsd3d;
        GLOBAL_HASSD.set(if hsd2d {
            2
        } else if hsd3d {
            3
        } else {
            0
        });
        if self.my_split_section_edges_done {
            return Ok(());
        }
        let all_gb1 = self.all_bound_vertices(&eor, rank_ef)?;
        if !cond || !hg || all_gb1 {
            return self.split_section_edge_on(&eor);
        }
        let (mut lesd1, mut lesd2) = (List::default(), List::default());
        self.g_find_sam_dom_of(ea, &mut lesd1, &mut lesd2)?;
        let com = TopOpeBRepBuildGTool::g_com_unsh(ShapeEnum::Face, ShapeEnum::Face);
        let mut lem = self.split_section_edge_on_state(&ef, &com, THE_ON)?;
        let mut new_lem = List::default();
        for esp in lem.iter() {
            let mut lspe = List::default();
            let ok = TopOpeBRepToolTOOL::split_e(&topods::edge(esp)?, &mut lspe)?;
            if !ok || lspe.len() < 2 {
                new_lem.append(esp.clone());
            } else {
                new_lem.append_list(&mut lspe);
            }
        }
        lem.clear();
        lem.append_list(&mut new_lem);
        self.store_section_splits(ea, State::On, &lem);
        let cut = TopOpeBRepBuildGTool::g_cut_unsh(ShapeEnum::Face, ShapeEnum::Face);
        let lemnc = self.split_section_edge_on_state(&ef, &cut.copy_permuted(), THE_IN)?;
        self.store_section_splits(ea, State::In, &lemnc);
        let lem_out = self.split_section_edge_on_state(&ef, &cut, THE_OUT)?;
        self.store_section_splits(ea, State::Out, &lem_out);
        GLOBAL_HASSD.set(0);
        Ok(())
    }

    /// The splits of the section edge on a state appended (the edge marked split).
    fn store_section_splits(&mut self, ea: &Shape, sta: State, l: &List<Shape>) {
        self.mark_split(ea, sta, true);
        Self::g_copy_list(l, self.change_split(ea, sta));
    }

    /// The edges of the paves of the forward section edge filled with `GLOBAL_issp` set.
    fn split_section_edge_on_state(
        &mut self,
        ef: &Edge,
        g: &TopOpeBRepBuildGTopo,
        issp: i32,
    ) -> Result<List<Shape>, Failure> {
        self.my_edge_reference = ef.clone();
        let mut pvs = TopOpeBRepBuildPaveSet::new(ef.as_ref())?;
        GLOBAL_ISSP.set(issp);
        let empty = self.my_empty_shape_list.clone();
        let r = self.g_fill_edge_pvs(ef.as_ref(), &empty, g, &mut pvs);
        GLOBAL_ISSP.set(0);
        r?;
        let mut vcl = TopOpeBRepBuildPaveClassifier::new(ef.as_ref())?;
        if pvs.has_equal_parameters()? {
            vcl.set_first_parameter(pvs.equal_parameters());
        }
        let mut edbu = TopOpeBRepBuildEdgeBuilder::new();
        edbu.init_edge_builder(&mut pvs, &mut vcl, false)?;
        let mut lem = List::default();
        self.gedbu_make_edges(ef.as_ref(), &mut edbu, &mut lem)?;
        Ok(lem)
    }

    /// Whether the vertex interferences of the edge are all at its bounds (their vertex,
    /// or same domain vertex for another rank); false at a POINT or when none.
    fn all_bound_vertices(&self, eor: &Edge, rank_ef: i32) -> Result<bool, Failure> {
        let h = self.hds()?;
        let (vf, vl) = topexp::vertices(eor, false);
        let loi = h.ds().shape_interferences(eor.as_ref(), true).clone();
        let mut all_gb1 = false;
        for i in loi.iter() {
            let (gt, g, _, _) = crate::top_ope_brep_ds::export::fds_data(i);
            if gt == crate::top_ope_brep_ds::TopOpeBRepDSKind::Point {
                return Ok(false);
            }
            let (rk_g, v) = {
                let bds = h.ds();
                (bds.ancestor_rank_index(g), bds.shape(g, true).clone())
            };
            all_gb1 = if rk_g == rank_ef {
                v.is_same(vf.as_ref()) || v.is_same(vl.as_ref())
            } else {
                let oov = fun_ds_getoov_hds(&v, &self.my_data_structure)?.unwrap_or_default();
                oov.is_same(vf.as_ref()) || oov.is_same(vl.as_ref())
            };
            if !all_gb1 {
                break;
            }
        }
        Ok(all_gb1)
    }

    /// The split ON of a section edge as one edge: a copy with the same domain reference
    /// vertices when one differs, else the edge.
    fn split_section_edge_on(&mut self, eor: &Edge) -> Result<(), Failure> {
        let h = self.my_data_structure.clone();
        let h = h.get().ok_or_else(null_hds)?;
        self.mark_split(eor.as_ref(), State::On, true);
        let mut new_eor = Shape::default();
        fun_ds_copy_edge(eor.as_ref(), &mut new_eor)?;
        let (bb, mut hasnew) = (BRepBuilder::default(), false);
        for v in Explorer::new(eor.as_ref(), ShapeEnum::Vertex, ShapeEnum::Shape)? {
            let parv = BRepTool::parameter(&topods::vertex(&v)?, eor)?;
            let iv = h.ds().shape_index(&v, true);
            let hsd = iv != 0 && h.has_same_domain(&v, true);
            let ivref = if hsd { h.same_domain_reference(&v) } else { 0 };
            if !(hsd && iv != ivref) {
                bb.add(&mut new_eor, &v)?;
                fun_ds_parameter(&new_eor, &v, parv)?;
                continue;
            }
            hasnew = true;
            let vref = h.ds().shape(ivref, true).oriented(v.orientation());
            bb.add(&mut new_eor, &vref)?;
            fun_ds_parameter(&new_eor, &vref, parv)?;
        }
        let lon = self.change_split(eor.as_ref(), State::On);
        lon.append(if hasnew {
            new_eor
        } else {
            eor.clone().into_shape()
        });
        Ok(())
    }

    /// `Section()`: the section edges (made once): the new edges of the curves and the
    /// splits of the section edges; `b.section()?`.
    pub fn section(&mut self) -> Result<&List<Shape>, Failure> {
        if !self.my_section_done {
            self.my_section_done = true;
            let mut s = std::mem::take(&mut self.my_section);
            let r = self
                .section_curves(&mut s)
                .and_then(|()| self.section_edges(&mut s));
            self.my_section = s;
            r?;
        }
        Ok(&self.my_section)
    }

    /// `Section(L)`: `l` set to the section; `b.section_into(&mut l)?`.
    pub fn section_into(&mut self, l: &mut List<Shape>) -> Result<(), Failure> {
        *l = self.section()?.clone();
        Ok(())
    }

    /// `SectionCurves(LSE)`: the new edges of the kept curves appended;
    /// `b.section_curves(&mut l)?`.
    pub fn section_curves(&self, lse: &mut List<Shape>) -> Result<(), Failure> {
        let indices: Vec<i32> = {
            let ds = self.hds()?.ds();
            let mut cex = TopOpeBRepDSCurveExplorer::new(&ds, true);
            let mut v = Vec::new();
            while cex.more() {
                v.push(cex.index());
                cex.next();
            }
            v
        };
        for ic in indices {
            for e in self.new_edges(ic).iter() {
                lse.append(e.clone());
            }
        }
        Ok(())
    }

    /// `SectionEdges(LSE)`: the section edges (in the particular case 1), else their
    /// splits ON (or IN), or the edges without geometry nor same domain, once each;
    /// `b.section_edges(&mut l)?`.
    pub fn section_edges(&mut self, lse: &mut List<Shape>) -> Result<(), Failure> {
        let edges = self.section_edges_of()?;
        if self.is_k_part() == 1 {
            for e in edges {
                lse.append(e.into_shape());
            }
            return Ok(());
        }
        self.split_section_edges()?;
        let mut mos: Map<Shape, ShapeMapHasher> = Map::default();
        for e in edges {
            let es = e.as_ref();
            let staspl = if self.is_split(es, State::On) {
                State::On
            } else if self.is_split(es, State::In) {
                State::In
            } else {
                State::Unknown
            };
            if staspl != State::Unknown {
                for s in self.splits(es, staspl).iter() {
                    if mos.add(s.clone()) {
                        lse.append(s.clone());
                    }
                }
            } else {
                let h = self.hds()?;
                if !(h.has_geometry(es) || h.has_same_domain(es, true)) && mos.add(es.clone()) {
                    lse.append(es.clone());
                }
            }
        }
        Ok(())
    }

    /// `FillSecEdgeAncestorMap(aShapeRank, aMapON, anAncMap)`: the section edges of the
    /// rank in `a_map_on`, and their splits in it, bound to the section edge;
    /// `b.fill_sec_edge_ancestor_map(1, &mon, &mut anc)?`.
    pub fn fill_sec_edge_ancestor_map(
        &self,
        a_shape_rank: i32,
        a_map_on: &Map<Shape, ShapeMapHasher>,
        an_anc_map: &mut DataMap<Shape, Shape, ShapeMapHasher>,
    ) -> Result<(), Failure> {
        for e in self.section_edges_of()? {
            let es = e.as_ref();
            if self.shape_rank(es) != a_shape_rank {
                continue;
            }
            if a_map_on.contains(es) {
                an_anc_map.bind(es.clone(), es.clone());
                continue;
            }
            for sta in [State::In, State::On, State::Out] {
                if self.is_split(es, sta) {
                    for s in self.splits(es, sta).iter() {
                        if a_map_on.contains(s) {
                            an_anc_map.bind(s.clone(), es.clone());
                        }
                    }
                }
            }
        }
        Ok(())
    }
}
