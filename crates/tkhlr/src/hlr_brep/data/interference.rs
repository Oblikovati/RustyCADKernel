//! HLRBRep_Data.cxx: InitInterference, NextInterference, RejectedInterference and
//! SameVertex, the walk on the intersections of the current edge with the boundary
//! of the hiding face.
use super::{Body, Data, box_rejected, data_array};
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::standard::real::real_last;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::geom_abs::CurveType;

// The ends moved away from a shared vertex (`THE_VERTEX_PARAMETER_CUT`).
const VERTEX_PARAMETER_CUT: f64 = 2.0e-1;

// The ends of the two edges at a shared vertex: (h1, e1, h2, e2).
type SharedEnds = (bool, bool, bool, bool);

impl Data {
    /// InitInterference(): starts the walk on the intersections of the current
    /// edge with the edges of the hiding face; `ds.init_interference()?`.
    pub fn init_interference(&self) -> Result<(), Failure> {
        self.body(|b| {
            b.w.ll_curve = b.w.le;
            let fd =
                b.f.value(b.w.i_face)
                    .map_err(data_array("HLRBRep_Data::InitInterference"))?;
            b.w.face_itr1.init_edge(fd)?;
            b.w.nb_points = 0;
            b.w.nb_segments = 0;
            b.w.i_interf = 0;
            b.next_interference()
        })
    }

    /// NextInterference(): the next intersection point or segment end, intersecting
    /// the current edge with the next face edges whose wire and edge boxes meet its
    /// box (the intersection cache answers the pairs already computed);
    /// `ds.next_interference()?`.
    pub fn next_interference(&self) -> Result<(), Failure> {
        self.body(|b| b.next_interference())
    }

    /// RejectedInterference(): whether the current intersection is rejected (above
    /// the face, or a doubtful transition); else the interference is built;
    /// `ds.rejected_interference()?`.
    pub fn rejected_interference(&self) -> Result<bool, Failure> {
        self.body(|b| b.rejected_interference())
    }
}

impl Body<'_> {
    pub(super) fn next_interference(&mut self) -> Result<(), Failure> {
        self.w.i_interf += 1;
        while !self.w.more_interference() && self.w.face_itr1.more_edge() {
            if self.w.face_itr1.beginning_of_wire() {
                let wire = self.w.face_itr1.wire()?;
                let mm = wire.get().ok_or_else(super::update::null_wire)?.min_max();
                if box_rejected(&mm, self.edge(self.w.le_min_max)?.min_max()) {
                    self.w.face_itr1.skip_wire()?;
                    continue;
                }
            }
            self.load_fe()?;
            if self.fe_may_intersect()? {
                self.intersect_le_fe()?;
            }
            self.w.face_itr1.next_edge()?;
        }
        Ok(())
    }

    // `myFE` and its cached data from the edge iterator of the face.
    fn load_fe(&mut self) -> Result<(), Failure> {
        let itr = &self.w.face_itr1;
        self.w.fe = itr.edge()?;
        self.w.fe_ori = itr.orientation()?;
        self.w.fe_out_line = itr.out_line()?;
        self.w.fe_internal = itr.internal()?;
        self.w.fe_double = itr.double()?;
        self.w.fe_data = self.w.fe;
        self.w.fe_geom = self.w.fe;
        let ed = self
            .e
            .value(self.w.fe)
            .map_err(data_array("HLRBRep_Data::NextInterference"))?;
        self.w.fe_tol = ed.tolerance();
        self.w.fe_type = ed.geometry().get_type();
        Ok(())
    }

    // A boundary edge, not vertical, not double unless an outline, not known to
    // miss the current edge, whose box meets it, and not a useless self
    // intersection.
    fn fe_may_intersect(&mut self) -> Result<bool, Failure> {
        let (le, fe) = (self.w.le, self.w.fe);
        if !matches!(self.w.fe_ori, Orientation::Forward | Orientation::Reversed) {
            return Ok(false);
        }
        let ed = self.edge(fe)?;
        if ed.vertical() || (self.w.fe_double && !self.w.fe_out_line) {
            return Ok(false);
        }
        if self.w.cache()?.no_intersection(le as usize, fe as usize) {
            return Ok(false);
        }
        if box_rejected(
            self.edge(fe)?.min_max(),
            self.edge(self.w.le_min_max)?.min_max(),
        ) {
            return Ok(false);
        }
        if le == fe && self.edge(le)?.auto_intersection_done() {
            let led = self.edge_mut(le)?;
            led.set_auto_intersection_done(true);
            if led.simple() {
                return Ok(false);
            }
        }
        Ok(true)
    }

    // The ends of the two edges at shared vertices (SameVertex on the four pairs).
    fn shared_ends(&mut self) -> Result<SharedEnds, Failure> {
        let mut ends = (false, false, false, false);
        self.w.same_vertex = false;
        self.w.intersected = true;
        if self.w.le == self.w.fe {
            return Ok(ends);
        }
        for (h1, h2) in [(true, true), (true, false), (false, true), (false, false)] {
            if self.same_vertex(h1, h2)? {
                self.w.same_vertex = true;
                if h1 {
                    ends.0 = true
                } else {
                    ends.1 = true
                }
                if h2 { ends.2 = true } else { ends.3 = true }
            }
        }
        Ok(ends)
    }

    fn intersect_le_fe(&mut self) -> Result<(), Failure> {
        let (h1, e1, h2, e2) = self.shared_ends()?;
        self.w.nb_points = 0;
        self.w.nb_segments = 0;
        self.w.i_interf = 1;
        if !self.w.intersected {
            return Ok(());
        }
        let (le, fe) = (self.w.le, self.w.fe);
        let cut = |b: bool| if b { VERTEX_PARAMETER_CUT } else { 0. };
        let moved = self.w.same_vertex || le == fe;
        let (da1, db1, da2, db2) = if moved {
            (cut(h1), cut(e1), cut(h2), cut(e2))
        } else {
            (0., 0., 0., 0.)
        };
        let (led, fed) = (
            self.e
                .value(le)
                .map_err(data_array("HLRBRep_Data::NextInterference"))?,
            self.e
                .value(fe)
                .map_err(data_array("HLRBRep_Data::NextInterference"))?,
        );
        let w = &mut *self.w;
        if le == fe {
            w.intersector.perform_self(led, da1, db1)?;
        } else {
            let (su, sv) = w.cache()?.single_intersection(le as usize, fe as usize);
            if su != real_last() {
                w.intersector.simulate_one_point(led, su, fed, sv)?;
            } else {
                w.intersector
                    .perform_edges(le, led, da1, db1, fe, fed, da2, db2, w.same_vertex)?;
                if w.intersector.is_done()?
                    && w.intersector.nb_points()? == 1
                    && w.intersector.nb_segments()? == 0
                {
                    let p = w.intersector.point(1)?;
                    w.cache()?.set_intersection(le as usize, fe as usize, &p);
                }
            }
        }
        if w.intersector.is_done()? {
            w.nb_points = w.intersector.nb_points()?;
            w.nb_segments = w.intersector.nb_segments()?;
            if w.nb_segments + w.nb_points == 0 {
                w.cache()?.set_no_intersection(le as usize, fe as usize);
            }
        } else {
            w.nb_points = 0;
            w.nb_segments = 0;
        }
        Ok(())
    }

    pub(super) fn rejected_interference(&mut self) -> Result<bool, Failure> {
        let w = &self.w;
        if w.i_interf <= w.nb_points {
            let p = w.intersector.point(w.i_interf)?;
            return self.rejected_point(&p, Orientation::External, 0);
        }
        let n = w.i_interf - w.nb_points;
        let mut first_point = (n & 1) != 0;
        let mut nseg = n >> 1;
        if first_point {
            nseg += 1;
        }
        let seg = w.intersector.segment(nseg)?;
        let lg = self.edge(w.le)?.geometry();
        let pf = lg.parameter_3d(seg.first_point()?.param_on_first())?;
        let pl = lg.parameter_3d(seg.last_point()?.param_on_first())?;
        if pf > pl {
            first_point = !first_point;
        }
        if first_point {
            self.rejected_point(&seg.first_point()?, Orientation::Forward, nseg)
        } else {
            self.rejected_point(&seg.last_point()?, Orientation::Reversed, -nseg)
        }
    }

    /// SameVertex(h1, h2): whether the chosen ends of the current edge and the face
    /// edge are one vertex; decides whether they must still be intersected.
    fn same_vertex(&mut self, h1: bool, h2: bool) -> Result<bool, Failure> {
        let (led, fed) = (self.edge(self.w.le)?, self.edge(self.w.fe_data)?);
        let v1 = if h1 { led.v_sta() } else { led.v_end() };
        let v2 = if h2 { fed.v_sta() } else { fed.v_end() };
        if v1 != v2 {
            return Ok(false);
        }
        let conic =
            |t: CurveType| matches!(t, CurveType::Line | CurveType::Circle | CurveType::Ellipse);
        let mut intersected = !(conic(self.w.le_type) && conic(self.w.fe_type));
        let out_lv = if h1 {
            led.out_lv_sta()
        } else {
            led.out_lv_end()
        };
        let other_case = if out_lv {
            !(self.w.i_face_test || self.w.le_internal)
        } else {
            !self.w.i_face_test
        };
        if other_case
            && (if h1 {
                led.cut_at_sta()
            } else {
                led.cut_at_end()
            })
        {
            // Two connected outlines do not intersect themselves.
            intersected = false;
        }
        self.w.intersected = intersected;
        Ok(true)
    }
}
