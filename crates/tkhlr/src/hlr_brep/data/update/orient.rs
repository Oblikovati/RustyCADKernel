//! HLRBRep_Data.cxx: OrientOutLine and OrientOthEdge, the orientations of the edges
//! of a face with outlines.
use super::super::Body;
use super::{null_wire, sl_props, view_side};
use crate::hlr_algo::EdgesBlock;
use crate::hlr_brep::EdgeFaceTool;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::{self, Orientation};
use rustycad_tkmath::gp::{Direction3, Point2, Vector3};

// The edge blocks of the wires of a face (shared handles).
fn wire_blocks(body: &Body<'_>, face: i32) -> Result<Vec<Handle<EdgesBlock>>, Failure> {
    let wb = body.face(face)?.wires().get().ok_or_else(null_wire)?;
    (1..=wb.nb_wires()).map(|iw| wb.wire(iw)).collect()
}

fn block(eb: &Handle<EdgesBlock>) -> Result<&EdgesBlock, Failure> {
    eb.get().ok_or_else(null_wire)
}

impl Body<'_> {
    /// OrientOutLine(I, FD): orients the outlines and internal edges of the face
    /// along the visible side of the surface; returns whether the first orientation
    /// change of a closed uncut face inverts it.
    pub(super) fn orient_out_line(&mut self, face: i32) -> Result<bool, Failure> {
        let (mut inverted, mut first_inversion) = (false, true);
        for eb in wire_blocks(self, face)? {
            let eb = block(&eb)?;
            for ie in 1..=eb.nb_edges() {
                self.w.fe = eb.edge(ie)?;
                let fe = self.w.fe;
                let vertical = self.edge(fe)?.vertical();
                let used = eb.double(ie)? || eb.iso_line(ie)? || vertical;
                self.edge_mut(fe)?.set_used(used);
                if (eb.out_line(ie)? || eb.internal(ie)?) && !vertical {
                    self.w.fe_geom = fe;
                    if let Some(ori) = self.outline_orientation(fe)? {
                        self.w.fe_ori = ori;
                        let fd = self.face(face)?;
                        let forward = fd.orientation() == Orientation::Forward;
                        if !fd.cut()
                            && fd.closed()
                            && first_inversion
                            && (eb.orientation(ie)? == ori) != forward
                        {
                            first_inversion = false;
                            inverted = true;
                        }
                        eb.set_orientation(ie, ori)?;
                    }
                    self.edge_mut(fe)?.set_used(true);
                }
            }
        }
        Ok(inverted)
    }

    // The parameter OrientOutLine samples: the middle, or the end at a vertex.
    fn outline_parameter(&self, fe: i32) -> Result<f64, Failure> {
        let ed = self.edge(fe)?;
        let ec = ed.geometry();
        Ok(match (ed.v_sta(), ed.v_end()) {
            (0, 0) => 0.,
            (0, _) => ec.parameter_3d(ec.last_parameter()?)?,
            (_, 0) => ec.parameter_3d(ec.first_parameter()?)?,
            _ => ec.parameter_3d((ec.last_parameter()? + ec.first_parameter()?) / 2.)?,
        })
    }

    // FORWARD when the visible side of the surface is on the left of the outline.
    fn outline_orientation(&self, fe: i32) -> Result<Option<Orientation>, Failure> {
        let p = self.outline_parameter(fe)?;
        let ec = self.edge(fe)?.geometry();
        let s = self.face_geom()?;
        let Some((pu, pv)) = EdgeFaceTool::uv_point(p, ec, s)? else {
            return Ok(None);
        };
        let mut props = sl_props(s)?;
        props.set_parameters(pu, pv)?;
        let (mut pt, mut tg) = ec.d1_3d(p)?;
        let proj = &*self.w.proj;
        let mut v = if proj.perspective() {
            let mut p2d = Point2::default();
            proj.project(pt, &mut p2d);
            Direction3::new(p2d.x(), p2d.y(), -proj.focus()?)?
        } else {
            Direction3::new(0., 0., -1.)?
        };
        v.transform(proj.inverted_transformation());
        if !props.is_normal_defined()? {
            return Ok(None);
        }
        let curv = EdgeFaceTool::curvature_value(s, pu, pv, v)?;
        let mut nm = Vector3::from(props.normal()?);
        if curv > 0. {
            nm.reverse();
        }
        let t = proj.transformation();
        tg.transform(t);
        pt.transform(t);
        nm.transform(t);
        nm.cross(tg);
        let r = view_side(proj, nm, pt)?;
        Ok(Some(if r > 0. {
            Orientation::Forward
        } else {
            Orientation::Reversed
        }))
    }

    /// OrientOthEdge(I, FD): reverses the other unused edges of the face where the
    /// surface normal at their middle looks away.
    pub(super) fn orient_oth_edge(&mut self, face: i32) -> Result<(), Failure> {
        for eb in wire_blocks(self, face)? {
            let eb = block(&eb)?;
            for ie in 1..=eb.nb_edges() {
                self.w.fe = eb.edge(ie)?;
                self.w.fe_ori = eb.orientation(ie)?;
                let fe = self.w.fe;
                if self.edge(fe)?.used() {
                    continue;
                }
                self.edge_mut(fe)?.set_used(true);
                self.w.fe_geom = fe;
                if self.normal_looks_away(fe)? {
                    self.w.fe_ori = top_abs::reverse(self.w.fe_ori);
                    eb.set_orientation(ie, self.w.fe_ori)?;
                }
            }
        }
        Ok(())
    }

    fn normal_looks_away(&self, fe: i32) -> Result<bool, Failure> {
        let ec = self.edge(fe)?.geometry();
        let s = self.face_geom()?;
        let p = ec.parameter_3d((ec.last_parameter()? + ec.first_parameter()?) / 2.)?;
        let Some((pu, pv)) = EdgeFaceTool::uv_point(p, ec, s)? else {
            return Ok(false);
        };
        let mut pt = ec.value_3d(p)?;
        let mut props = sl_props(s)?;
        props.set_parameters(pu, pv)?;
        if !props.is_normal_defined()? {
            return Ok(false);
        }
        let mut nm = Vector3::from(props.normal()?);
        let t = self.w.proj.transformation();
        pt.transform(t);
        nm.transform(t);
        Ok(view_side(&self.w.proj, nm, pt)? < 0.)
    }
}
