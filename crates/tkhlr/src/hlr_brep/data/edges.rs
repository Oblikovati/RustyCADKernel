//! HLRBRep_Data.cxx: InitBoundSort, InitEdge, MoreEdge, NextEdge and Edge, the walk
//! on the edges a face can hide.
use super::{Body, Data, box_rejected, data_array};
use crate::hlr_algo::MinMaxIndices;
use crate::hlr_topo_brep::MapOfShapeTool;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tktopalgo::brep_top_adaptor::BRepTopAdaptorTool;

impl Data {
    /// InitBoundSort(MinMaxTot, e1, e2): the edges `e1` to `e2` that are not all
    /// hidden and whose box meets `min_max_tot`, in order;
    /// `ds.init_bound_sort(&total, 1, ds.nb_edges())?`.
    pub fn init_bound_sort(
        &self,
        min_max_tot: &MinMaxIndices,
        e1: i32,
        e2: i32,
    ) -> Result<(), Failure> {
        self.body(|b| {
            b.w.nbr_sort_ed = 0;
            for e in e1..=e2 {
                let ed = b.edge(e)?;
                if ed.status().all_hidden() {
                    continue;
                }
                let rejected = box_rejected(min_max_tot, ed.min_max());
                b.w.le_min_max = e;
                if !rejected {
                    b.w.nbr_sort_ed += 1;
                    let n = b.w.nbr_sort_ed as usize;
                    *b.w.edge_indices.get_mut(n).ok_or_else(|| out_of_range(n))? = e;
                }
            }
            Ok(())
        })
    }

    /// InitEdge(FI, MST): starts the walk on the edges the face `fi` can hide: its own
    /// edges first when it is not simple, then the sorted edges; loads the face in the
    /// intersector and its classifier (made and bound in `mst` when missing);
    /// `ds.init_edge(1, &mut tools)?`.
    pub fn init_edge(&self, fi: i32, mst: &mut MapOfShapeTool) -> Result<(), Failure> {
        self.body(|b| {
            b.w.hide_count += 2;
            b.w.i_face = fi;
            b.w.i_face_geom = Some(fi);
            let fd =
                b.f.value(fi)
                    .map_err(data_array("HLRBRep_Data::InitEdge"))?;
            b.w.i_face_back = fd.back();
            b.w.i_face_simp = fd.simple();
            b.w.i_face_type = fd.geometry().get_type();
            b.w.i_face_test = !b.w.i_face_simp;
            b.w.intersector.load();
            b.w.loaded_face = Some(fi);
            let face = Shape::from(fd.geometry().surface().face());
            b.w.classifier = if mst.is_bound(&face) {
                mst.find(&face)?.get_topol_tool()
            } else {
                let brt = BRepTopAdaptorTool::with_face(
                    &fd.geometry().surface().face(),
                    precision::p_confusion(),
                )?;
                mst.bind(face, brt.clone());
                brt.get_topol_tool()
            };
            if b.w.i_face_test {
                b.w.i_face_smpl = !fd.cut();
                let fd =
                    b.f.value(fi)
                        .map_err(data_array("HLRBRep_Data::InitEdge"))?;
                b.w.face_itr2.init_edge(fd)?;
            } else {
                b.mark_face_edges(fi)?;
                b.w.cur_sort_ed = 1;
            }
            b.next_edge(false)
        })
    }

    /// MoreEdge(): whether there is a current edge to hide (the end of the own edges
    /// of the face switches to the sorted edges); `ds.more_edge()?`.
    pub fn more_edge(&self) -> Result<bool, Failure> {
        self.body(|b| b.more_edge())
    }

    /// NextEdge(skip): the next edge that the face may hide (not vertical, not
    /// already tested, not all hidden, meeting the face box, not above the face);
    /// `ds.next_edge(true)?`.
    pub fn next_edge(&self, skip: bool) -> Result<(), Failure> {
        self.body(|b| b.next_edge(skip))
    }

    /// Edge(): the index of the current edge; `ds.edge()?`.
    pub fn edge(&self) -> Result<i32, Failure> {
        self.body(|b| b.current_edge())
    }
}

fn out_of_range(n: usize) -> Failure {
    Failure::with_message(
        "Standard_OutOfRange",
        &format!(
            "HLRBRep_Data: sorted edge index {n} is out of the edge index array; expected 0..=NbEdges"
        ),
    )
}

impl Body<'_> {
    // The edges of a simple hiding face are jumped (their hide count is set).
    fn mark_face_edges(&mut self, fi: i32) -> Result<(), Failure> {
        let fd = self
            .f
            .value(fi)
            .map_err(data_array("HLRBRep_Data::InitEdge"))?;
        self.w.face_itr1.init_edge(fd)?;
        while self.w.face_itr1.more_edge() {
            self.w.fe = self.w.face_itr1.edge()?;
            let hc = self.w.hide_count - 1;
            self.edge_mut(self.w.fe)?.set_hide_count(hc);
            self.w.face_itr1.next_edge()?;
        }
        Ok(())
    }

    pub(super) fn current_edge(&self) -> Result<i32, Failure> {
        if self.w.i_face_test {
            return self.w.face_itr2.edge();
        }
        let n = self.w.cur_sort_ed;
        let i = usize::try_from(n).map_err(|_| out_of_range(0))?;
        self.w
            .edge_indices
            .get(i)
            .copied()
            .ok_or_else(|| out_of_range(i))
    }

    // `myLE` and its cached data from the own edge iterator of the face.
    fn load_le_from_face(&mut self) -> Result<(), Failure> {
        let itr = &self.w.face_itr2;
        self.w.le = itr.edge()?;
        self.w.le_out_line = itr.out_line()?;
        self.w.le_internal = itr.internal()?;
        self.w.le_double = itr.double()?;
        self.w.le_iso_line = itr.iso_line()?;
        self.load_le_data()
    }

    // `myLEData`, `myLEGeom`, `myLEMinMax`, `myLETol` and `myLEType` of `myLE`.
    fn load_le_data(&mut self) -> Result<(), Failure> {
        let ed = self
            .e
            .value(self.w.le)
            .map_err(data_array("HLRBRep_Data::NextEdge"))?;
        self.w.le_tol = ed.tolerance();
        self.w.le_type = ed.geometry().get_type();
        self.w.le_min_max = self.w.le;
        Ok(())
    }

    pub(super) fn more_edge(&mut self) -> Result<bool, Failure> {
        if self.w.i_face_test {
            if self.w.face_itr2.more_edge() {
                self.load_le_from_face()?;
                if !self.w.le_double {
                    let hc = self.w.hide_count - 1;
                    self.edge_mut(self.w.le)?.set_hide_count(hc);
                }
                return Ok(true);
            }
            self.w.i_face_test = false;
            self.w.i_face_simp = self.w.i_face_smpl;
            let (fi, simp) = (self.w.i_face, self.w.i_face_simp);
            self.face_mut(fi)?.set_simple(simp);
            self.w.cur_sort_ed = 1;
            self.next_edge(false)?;
        }
        Ok(self.w.cur_sort_ed <= self.w.nbr_sort_ed)
    }

    // The native NextEdge recurses once per skipped edge, and a call made from the
    // own edge walk sets the hide count of the final current edge on return; the
    // loop keeps that order.
    pub(super) fn next_edge(&mut self, mut skip: bool) -> Result<(), Failure> {
        let mut pending_hide = false;
        loop {
            if skip {
                if self.w.i_face_test {
                    self.w.face_itr2.next_edge()?;
                } else {
                    self.w.cur_sort_ed += 1;
                }
            }
            skip = true;
            if !self.more_edge()? {
                break;
            }
            if self.w.i_face_test {
                self.load_le_from_face()?;
                let ed = self.edge(self.w.le)?;
                let hc = self.w.hide_count;
                if ed.vertical() || (self.w.le_double && ed.hide_count() == hc - 1) {
                    pending_hide = true;
                    continue;
                }
                self.edge_mut(self.w.le)?.set_hide_count(hc - 1);
                break;
            }
            self.w.le = self.current_edge()?;
            self.w.le_out_line = false;
            self.w.le_internal = false;
            self.w.le_double = false;
            self.w.le_iso_line = false;
            self.load_le_data()?;
            if !self.sorted_edge_rejected()? {
                break;
            }
        }
        if pending_hide {
            let hc = self.w.hide_count - 1;
            self.edge_mut(self.w.le)?.set_hide_count(hc);
        }
        Ok(())
    }

    // The tests of a sorted edge: vertical, already tested, all hidden, box, above.
    fn sorted_edge_rejected(&self) -> Result<bool, Failure> {
        let ed = self.edge(self.w.le)?;
        if ed.vertical() || ed.hide_count() > self.w.hide_count - 2 || ed.status().all_hidden() {
            return Ok(true);
        }
        let face_box = self.face_min_max()?;
        if box_rejected(&face_box, self.edge(self.w.le_min_max)?.min_max()) {
            return Ok(true);
        }
        let s = self.face_geom()?;
        s.is_above(self.w.i_face_back, ed.geometry(), f64::from(self.w.le_tol))
    }

    // `iFaceMinMax`: the box of the wires of the hiding face.
    pub(super) fn face_min_max(&self) -> Result<MinMaxIndices, Failure> {
        let fd = self.face(self.w.i_face)?;
        Ok(fd
            .wires()
            .get()
            .ok_or_else(super::update::null_wire)?
            .min_max())
    }
}
