//! The first fillet and the fillets between two others of FilDS.
use super::ds_fil::{FilDs, contains};
use super::ds_tools::{fil_curve_in_ds, fil_point_in_ds, null};
use super::pcurves::{compute_pcurv, compute_pcurv_geom};
use crate::chfi_ds::{CommonPoint, Regul, StripeHandle, SurfDataHandle, lock};
use rustycad_tkbool::top_ope_brep_ds::TopOpeBRepDSDataStructure;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::{self, Orientation};
use rustycad_tkmath::gp::Point2;

impl FilDs {
    /// The first points of the stripe on the arcs (or the singular end vertex on the
    /// spine edge) and the first curve of the stripe on the fillet surface.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn first_fillet(
        &mut self,
        fd: &SurfDataHandle,
        (v1, v2): (&CommonPoint, &CommonPoint),
        cor_dat: &StripeHandle,
        dstr: &mut TopOpeBRepDSDataStructure,
        et1: &mut Orientation,
        in_ds1: bool,
    ) -> Result<(), Failure> {
        self.is_vertex = (v1.is_vertex(), v2.is_vertex());
        self.singulier_en_bout = v1.point().is_equal(&v2.point(), 0.);
        if self.singulier_en_bout {
            if v1.is_vertex() && v2.is_vertex() {
                self.is_vertex = (true, true);
                let arcspine = self.spine_edge(1)?;
                self.bout_de_vtx = v1.vertex()?.clone();
                let ivtx = lock(cor_dat).index_first_point_on_s1();
                self.spine_vertex_interference(dstr, &arcspine, ivtx, None)?;
            }
        } else {
            for (k, v) in [(0, v1), (1, v2)] {
                if !v.is_on_arc() {
                    continue;
                }
                let iarc = dstr.add_shape(v.arc()?.as_ref());
                let ipoin = if k == 0 { self.ipoin.0 } else { self.ipoin.1 };
                if k == 0 {
                    self.iarc.0 = iarc
                } else {
                    self.iarc.1 = iarc
                }
                if !contains(
                    dstr.shape_interferences_index(iarc, true)?,
                    iarc,
                    ipoin,
                    false,
                    false,
                ) {
                    let isv = if k == 0 {
                        self.is_vertex.0
                    } else {
                        self.is_vertex.1
                    };
                    let i = fil_point_in_ds(
                        v.transition_on_arc()?,
                        iarc,
                        ipoin,
                        v.parameter_on_arc()?,
                        isv,
                    );
                    dstr.change_shape_interferences(v.arc()?.as_ref()).append(i);
                }
            }
        }
        if in_ds1 {
            return Ok(());
        }
        let isurf = lock(fd).surf();
        let (pc_or, icurv, pcurv, par) = {
            let g = lock(cor_dat);
            (
                g.first_pcurve_orientation(),
                g.first_curve(),
                g.first_pcurve().clone(),
                g.first_parameters(),
            )
        };
        *et1 = top_abs::compose(*et1, pc_or);
        self.icurv = icurv;
        if self.closed && !self.singulier_en_bout {
            self.regcout.set_curve(icurv);
            self.regcout.set_s1(isurf, false);
        }
        (self.pcurv, self.par) = (pcurv, par);
        self.end_curve(dstr, isurf, pc_or, *et1)
    }

    /// The end points of the stripe curve (when it has none) and its curve on the fillet
    /// surface, the curve nullified when its ends are one point.
    pub(super) fn end_curve(
        &self,
        dstr: &mut TopOpeBRepDSDataStructure,
        isurf: i32,
        pc_or: Orientation,
        et1: Orientation,
    ) -> Result<(), Failure> {
        let (pardeb, parfin) = self.par;
        let li = dstr.change_curve_interferences(self.icurv);
        if li.is_empty() {
            let (a, b) = if pc_or == Orientation::Reversed {
                (
                    fil_point_in_ds(
                        Orientation::Reversed,
                        self.icurv,
                        self.ipoin.0,
                        parfin,
                        self.is_vertex.0,
                    ),
                    fil_point_in_ds(
                        Orientation::Forward,
                        self.icurv,
                        self.ipoin.1,
                        pardeb,
                        self.is_vertex.1,
                    ),
                )
            } else {
                (
                    fil_point_in_ds(
                        Orientation::Forward,
                        self.icurv,
                        self.ipoin.0,
                        pardeb,
                        self.is_vertex.0,
                    ),
                    fil_point_in_ds(
                        Orientation::Reversed,
                        self.icurv,
                        self.ipoin.1,
                        parfin,
                        self.is_vertex.1,
                    ),
                )
            };
            li.append(a);
            li.append(b);
        }
        let interfc1 = fil_curve_in_ds(self.icurv, isurf, &self.pcurv, et1);
        dstr.change_surface_interferences(isurf)
            .append(interfc1.clone());
        if self.ipoin.0 == self.ipoin.1 {
            let tc = dstr.change_curve(self.icurv);
            *tc.change_curve() = Handle::default();
            tc.set_sci(&interfc1, &Handle::default());
        }
        Ok(())
    }

    /// The curve between this fillet and the previous one on this fillet surface (a
    /// degenerated one on the spine vertex).
    pub(super) fn between_fillets(
        &mut self,
        fd: &SurfDataHandle,
        dstr: &mut TopOpeBRepDSDataStructure,
        reglist: &mut Vec<Regul>,
        et1: Orientation,
    ) -> Result<(), Failure> {
        if self.degene && self.is_vertex.0 {
            self.num_edge += 1;
            let arcspine = self.spine_edge(self.num_edge)?;
            let ivtx = dstr.add_shape(self.bout_de_vtx.as_ref());
            self.spine_vertex_interference(dstr, &arcspine, ivtx, None)?;
        }
        let (isurf, uv1, uv2) = {
            let g = lock(fd);
            let uv = |fi: &crate::chfi_ds::FaceInterference| -> Result<Point2, Failure> {
                fi.pcurve_on_surf()
                    .get()
                    .ok_or_else(|| null("ChFi3d_FilDS: null pcurve"))?
                    .value(fi.first_parameter())
            };
            (
                g.surf(),
                uv(g.interference_on_s1())?,
                uv(g.interference_on_s2())?,
            )
        };
        if self.degene {
            self.pcurv = compute_pcurv(uv1, uv2, self.par, false)?;
            let i = fil_curve_in_ds(self.icurv, isurf, &self.pcurv, et1);
            dstr.change_surface_interferences(isurf).append(i.clone());
            let tc = dstr.change_curve(self.icurv);
            *tc.change_curve() = Handle::default();
            tc.set_sci(&i, &Handle::default());
            return Ok(());
        }
        self.regfilfil.set_s2(isurf, false);
        reglist.push(self.regfilfil);
        let c = dstr.curve(self.icurv).curve().clone();
        let s = dstr.surface(isurf).surface().clone();
        let mut tolreached = 0.;
        self.pcurv = compute_pcurv_geom(
            &c,
            (uv1, uv2),
            &s,
            self.par,
            self.tol.0,
            &mut tolreached,
            false,
        )?;
        let tc = dstr.change_curve(self.icurv);
        let t = tc.tolerance().max(tolreached);
        tc.set_tolerance(t);
        let i = fil_curve_in_ds(self.icurv, isurf, &self.pcurv, et1);
        dstr.change_surface_interferences(isurf).append(i);
        Ok(())
    }
}
