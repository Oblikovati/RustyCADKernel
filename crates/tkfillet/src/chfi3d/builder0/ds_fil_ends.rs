//! `ChFi3d_FilDS` (continued): the traces of a fillet on its faces with their end points,
//! the curves at the end of the stripe and toward the next fillet, and the end points on
//! the arcs and the spine vertices.
use super::arete::{Arete, compute_arete};
use super::ds_fil::{FilDs, contains, find_index_point};
use super::ds_tools::{fil_curve_in_ds, fil_point_in_ds, index_point_in_ds, null};
use crate::chfi_ds::{FaceInterference, Regul, StripeHandle, SurfDataHandle, lock};
use rustycad_tkbool::top_ope_brep_ds::{TopOpeBRepDSCurve, TopOpeBRepDSDataStructure};
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::Edge;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::{self, Orientation, ShapeEnum};

impl FilDs {
    /// The trace of the fillet on its face `on_s`: its curve on the fillet surface, on the
    /// face (a regularity) or kept on the fillet surface, and its end points.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn face_trace(
        &mut self,
        fd: &SurfDataHandle,
        on_s: i32,
        j: usize,
        cor_dat: &StripeHandle,
        dstr: &mut TopOpeBRepDSDataStructure,
        reglist: &mut Vec<Regul>,
        in_ds2: bool,
    ) -> Result<(), Failure> {
        let (fi, isurf, ishape) = {
            let g = lock(fd);
            (g.interference(on_s).clone(), g.surf(), g.index(on_s))
        };
        let icfil = fi.line_index();
        if icfil == 0 {
            return Ok(());
        }
        let trafil = if on_s == 1 {
            self.trafil.0
        } else {
            self.trafil.1
        };
        let interfc = fil_curve_in_ds(icfil, isurf, fi.pcurve_on_surf(), trafil);
        dstr.change_surface_interferences(isurf)
            .append(interfc.clone());
        if dstr.curve(icfil).curve().is_null() {
            dstr.change_curve(icfil)
                .set_sci(&interfc, &Handle::default());
        } else {
            reglist.push(trace_on_face(dstr, &fi, (icfil, isurf, ishape)));
        }
        let (ipoin, is_vertex) = self.trace_end_point(fd, on_s, j, cor_dat, dstr, in_ds2)?;
        let (ip_prev, isv_prev) = if on_s == 1 {
            (self.ipoin.0, self.is_vertex.0)
        } else {
            (self.ipoin.1, self.is_vertex.1)
        };
        if !contains(
            dstr.change_curve_interferences(icfil),
            icfil,
            ip_prev,
            false,
            false,
        ) {
            let i = fil_point_in_ds(
                Orientation::Forward,
                icfil,
                ip_prev,
                fi.first_parameter(),
                isv_prev,
            );
            dstr.change_curve_interferences(icfil).append(i);
        }
        if ipoin == ip_prev
            || !contains(
                dstr.change_curve_interferences(icfil),
                icfil,
                ipoin,
                false,
                false,
            )
        {
            let i = fil_point_in_ds(
                Orientation::Reversed,
                icfil,
                ipoin,
                fi.last_parameter(),
                is_vertex,
            );
            dstr.change_curve_interferences(icfil).append(i);
        }
        if on_s == 1 {
            (self.ipoin.0, self.is_vertex.0) = (ipoin, is_vertex);
        } else {
            (self.ipoin.1, self.is_vertex.1) = (ipoin, is_vertex);
        }
        Ok(())
    }

    /// The point of the structure at the end of the trace on the face `on_s`: the stripe
    /// end, the stripe start of a closed stripe, the previous point on a rotation (and the
    /// other trace end on a pinch), the stripe end at the end of the stripe, a point
    /// found on the next fillet, else a new point.
    #[allow(clippy::too_many_arguments)]
    fn trace_end_point(
        &self,
        fd: &SurfDataHandle,
        on_s: i32,
        j: usize,
        cor_dat: &StripeHandle,
        dstr: &mut TopOpeBRepDSDataStructure,
        in_ds2: bool,
    ) -> Result<(i32, bool), Failure> {
        let n = self.seq.len();
        let last_on = |cd: &StripeHandle| {
            let g = lock(cd);
            if on_s == 1 {
                g.index_last_point_on_s1()
            } else {
                g.index_last_point_on_s2()
            }
        };
        let first_on = |cd: &StripeHandle| {
            let g = lock(cd);
            if on_s == 1 {
                g.index_first_point_on_s1()
            } else {
                g.index_first_point_on_s2()
            }
        };
        let (last_v, icfil) = {
            let g = lock(fd);
            (
                g.vertex(false, on_s).clone(),
                g.interference(on_s).line_index(),
            )
        };
        let is_vertex = last_v.is_vertex();
        if j == n {
            return Ok((last_on(cor_dat), is_vertex));
        }
        let seq_last_line = lock(&self.seq[n - 1]).interference(on_s).line_index();
        if j == n - 1 && dstr.curve(seq_last_line).curve().is_null() {
            if self.closed {
                return Ok((
                    first_on(cor_dat),
                    lock(&self.seq[0]).vertex(true, on_s).is_vertex(),
                ));
            }
            return Ok((
                last_on(cor_dat),
                lock(&self.seq[n - 1]).vertex(false, on_s).is_vertex(),
            ));
        }
        let (ip_prev, isv_prev) = if on_s == 1 {
            (self.ipoin.0, self.is_vertex.0)
        } else {
            (self.ipoin.1, self.is_vertex.1)
        };
        if dstr.curve(icfil).curve().is_null() {
            return Ok((ip_prev, isv_prev));
        }
        if on_s == 2
            && last_v
                .point()
                .is_equal(&lock(fd).vertex(false, 1).point(), 0.)
        {
            return Ok((self.ipoin.0, self.is_vertex.0));
        }
        let first_p = lock(&self.seq[0]).vertex(true, on_s).point();
        let last_p = lock(&self.seq[n - 1]).vertex(false, on_s).point();
        if (j == 1 || j == n - 1)
            && (last_v.point().is_equal(&first_p, 1.0e-7)
                || last_v.point().is_equal(&last_p, 1.0e-7))
        {
            return Ok((last_on(cor_dat), is_vertex));
        }
        if in_ds2 && let Some(ip) = find_index_point(dstr, fd, on_s)? {
            return Ok((ip, is_vertex));
        }
        Ok((index_point_in_ds(&last_v, dstr)?, is_vertex))
    }

    /// The last curve of the stripe on the fillet surface (with the closing regularity of
    /// a closed stripe).
    pub(super) fn last_fillet(
        &mut self,
        isurf: i32,
        cor_dat: &StripeHandle,
        dstr: &mut TopOpeBRepDSDataStructure,
        reglist: &mut Vec<Regul>,
        et1: Orientation,
    ) -> Result<(), Failure> {
        let (icurv, pcurv, pc_or, par) = {
            let g = lock(cor_dat);
            (
                g.last_curve(),
                g.last_pcurve().clone(),
                g.last_pcurve_orientation(),
                g.last_parameters(),
            )
        };
        self.icurv = icurv;
        if self.closed && !self.singulier_en_bout && self.ipoin.0 != self.ipoin.1 {
            self.regcout.set_s2(isurf, false);
            reglist.push(self.regcout);
        }
        (self.pcurv, self.par) = (pcurv, par);
        self.end_curve(dstr, isurf, pc_or, top_abs::compose(et1, pc_or))
    }

    /// The curve toward the next fillet on this fillet surface, between the end points.
    pub(super) fn arete_to_next(
        &mut self,
        fd: &SurfDataHandle,
        dstr: &mut TopOpeBRepDSDataStructure,
        et1: Orientation,
    ) -> Result<(), Failure> {
        let (cp1, cp2, uv1, uv2, isurf) = {
            let g = lock(fd);
            let uv = |fi: &FaceInterference| -> Result<rustycad_tkmath::gp::Point2, Failure> {
                fi.pcurve_on_surf()
                    .get()
                    .ok_or_else(|| null("ChFi3d_FilDS: null pcurve"))?
                    .value(fi.last_parameter())
            };
            (
                g.vertex_last_on_s1().clone(),
                g.vertex_last_on_s2().clone(),
                uv(g.interference_on_s1())?,
                uv(g.interference_on_s2())?,
                g.surf(),
            )
        };
        let s = dstr.surface(isurf).surface().clone();
        let mut ar = Arete::default();
        compute_arete((&cp1, uv1), (&cp2, uv2), &s, &mut ar, self.tol, 0)?;
        (self.pcurv, self.par) = (ar.pcurv.clone(), (ar.pardeb, ar.parfin));
        self.icurv = dstr.add_curve(&mut TopOpeBRepDSCurve::new(&ar.c3d, ar.tolreached, false));
        self.regfilfil.set_curve(self.icurv);
        self.regfilfil.set_s1(isurf, false);
        let a = fil_point_in_ds(
            Orientation::Forward,
            self.icurv,
            self.ipoin.0,
            ar.pardeb,
            self.is_vertex.0,
        );
        dstr.change_curve_interferences(self.icurv).append(a);
        let b = fil_point_in_ds(
            Orientation::Reversed,
            self.icurv,
            self.ipoin.1,
            ar.parfin,
            self.is_vertex.1,
        );
        dstr.change_curve_interferences(self.icurv).append(b);
        let i = fil_curve_in_ds(self.icurv, isurf, &self.pcurv, et1);
        dstr.change_surface_interferences(isurf).append(i);
        Ok(())
    }

    /// The end of the fillet: on a degenerated end the vertex on its spine edge, else the
    /// last points on their arcs.
    pub(super) fn end_points(
        &mut self,
        fd: &SurfDataHandle,
        j: usize,
        cor_dat: &StripeHandle,
        dstr: &mut TopOpeBRepDSDataStructure,
    ) -> Result<(), Failure> {
        self.degene = self.v3.point().is_equal(&self.v4.point(), 0.);
        if self.degene {
            if self.v3.is_vertex() && self.v4.is_vertex() {
                self.degenerated_end(j, cor_dat, dstr)?;
            }
            return Ok(());
        }
        if self.closed && j == self.seq.len() {
            return Ok(());
        }
        let (c1, c2) = {
            let g = lock(fd);
            (g.is_on_curve1(), g.is_on_curve2())
        };
        for (k, on_curve) in [(0, c1), (1, c2)] {
            let v = if k == 0 {
                self.v3.clone()
            } else {
                self.v4.clone()
            };
            if !v.is_on_arc() || (v.is_vertex() && on_curve) {
                continue;
            }
            let iarc = dstr.add_shape(v.arc()?.as_ref());
            let ipoin = if k == 0 {
                self.iarc.0 = iarc;
                self.ipoin.0
            } else {
                self.iarc.1 = iarc;
                self.ipoin.1
            };
            if !contains(
                dstr.shape_interferences_index(iarc, true)?,
                iarc,
                ipoin,
                v.is_vertex(),
                true,
            ) {
                let i = fil_point_in_ds(
                    v.transition_on_arc()?,
                    iarc,
                    ipoin,
                    v.parameter_on_arc()?,
                    v.is_vertex(),
                );
                dstr.change_shape_interferences(v.arc()?.as_ref()).append(i);
            }
        }
        Ok(())
    }

    /// The degenerated end: the spine edge of the end vertex found from the current
    /// one, the vertex added to its interferences.
    fn degenerated_end(
        &mut self,
        j: usize,
        cor_dat: &StripeHandle,
        dstr: &mut TopOpeBRepDSDataStructure,
    ) -> Result<(), Failure> {
        self.bout_de_vtx = self.v3.vertex()?.clone();
        let nb = self.spine.as_ref().map_or(0, |s| lock(s).nb_edges());
        let mut arcspine = Edge::default();
        let (mut trouve, mut ovtx) = (false, Orientation::Forward);
        while self.num_edge <= nb && !trouve {
            arcspine = self.spine_edge(self.num_edge)?;
            for v in Explorer::new(
                &arcspine.oriented(Orientation::Forward),
                ShapeEnum::Vertex,
                ShapeEnum::Shape,
            )? {
                if self.bout_de_vtx.is_same(&v) {
                    ovtx = v.orientation();
                    trouve = if self.closed && self.num_edge == 1 {
                        nb == 1
                    } else {
                        true
                    };
                    if trouve {
                        break;
                    }
                }
            }
            if !trouve {
                self.num_edge += 1;
            }
        }
        let ivtx = if j == self.seq.len() {
            lock(cor_dat).index_last_point_on_s1()
        } else {
            dstr.add_shape(self.bout_de_vtx.as_ref())
        };
        self.spine_vertex_interference(dstr, &arcspine, ivtx, Some(ovtx))
    }
}

/// The trace of a fillet on a face (a curve of the structure): its interference on the
/// face (shape) or on the fillet surface (negative index), and the regularity along it.
fn trace_on_face(
    dstr: &mut TopOpeBRepDSDataStructure,
    fi: &FaceInterference,
    (icfil, isurf, ishape): (i32, i32, i32),
) -> Regul {
    let mut reg = Regul::new();
    reg.set_curve(icfil);
    reg.set_s1(isurf, false);
    if ishape < 0 {
        reg.set_s2(-ishape, false);
        let i = fil_curve_in_ds(icfil, -ishape, fi.pcurve_on_face(), fi.transition());
        dstr.change_surface_interferences(-ishape).append(i);
    } else if ishape > 0 {
        reg.set_s2(ishape, true);
        let i = fil_curve_in_ds(icfil, ishape, fi.pcurve_on_face(), fi.transition());
        if let Ok(li) = dstr.change_shape_interferences_index(ishape) {
            li.append(i);
        }
    }
    reg
}
