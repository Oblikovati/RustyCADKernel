// Rust translation of ChFi3d_ChBuilder::SimulKPart and the two-surface
// ChFi3d_ChBuilder::SimulSurf (ChFi3d_ChBuilder.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The simulation of the chamfers: their linear sections without the surfaces.
use super::ChFi3dChBuilder;
use super::functions::{ChamferRun, chamfer_functions, chamfer_kind, offset_guide, with_chamfer};
use crate::chfi_ds::{ChamfMethod, CircSection, SurfDataHandle, lock};
use crate::chfi3d::builder::{ComputeCall, SurfCall, SurfOut};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorSurface;
use rustycad_tkmath::elslib;
use rustycad_tkmath::geom_abs::SurfaceType;
use rustycad_tkmath::gp::{Line3, Point2};

/// A linear section.
fn line_section(line: Line3, first: f64, last: f64) -> CircSection {
    let mut s = CircSection::new();
    s.set_line(line, first, last);
    s
}

impl ChFi3dChBuilder {
    /// `SimulKPart(SD)`: the sections of a particular case (two on a plane, one every
    /// five degrees on a cone).
    pub(super) fn simul_kpart_impl(&self, sd: &SurfDataHandle) -> Result<(), Failure> {
        let (s, p1f, p1l, p2f, p2l) = {
            let g = lock(sd);
            let at = |on_s: i32, first: bool| -> Result<Point2, Failure> {
                let fi = g.interference(on_s);
                fi.pcurve_on_surf().value(fi.parameter(first))
            };
            (
                self.base.ds.surface(g.surf()).surface().clone(),
                at(1, true)?,
                at(1, false)?,
                at(2, true)?,
                at(2, false)?,
            )
        };
        let a_s = GeomAdaptorSurface::with_surface(&s)?;
        let (v1, v2) = (p1f.y(), p2f.y());
        let u1 = p1f.x().max(p2f.x());
        let u2 = p1l.x().min(p2l.x());
        let sections = match a_s.get_type()? {
            SurfaceType::Plane => {
                let pl = a_s.plane()?;
                vec![
                    line_section(elslib::plane_u_iso(pl.position(), u1), v1, v2),
                    line_section(elslib::plane_u_iso(pl.position(), u2), v1, v2),
                ]
            }
            SurfaceType::Cone => {
                let co = a_s.cone()?;
                let (rad, sang) = (co.ref_radius(), co.semi_angle());
                let n = ((36. * (u2 - u1) / std::f64::consts::PI + 1.) as i32).max(2);
                (1..=n)
                    .map(|i| {
                        let u = u1 + f64::from(i - 1) * (u2 - u1) / f64::from(n - 1);
                        line_section(elslib::cone_u_iso(co.position(), rad, sang, u), v1, v2)
                    })
                    .collect()
            }
            _ => Vec::new(),
        };
        lock(sd).set_simul(&crate::chfi3d::builder::sections_transient(sections)?);
        Ok(())
    }

    /// The largest second derivative of the guide sampled at 21 points (`radiusspine`).
    fn spine_curvature(call: &SurfCall<'_>, max_step: f64) -> Result<f64, Failure> {
        let fi = call.guide.first_parameter()?;
        let mut radiusspine: f64 = 0.;
        for i in 0..=20 {
            let w = fi + f64::from(i) * max_step;
            let (_, _, d2) = call.guide.d2(w)?;
            radiusspine = radiusspine.max(d2.square_magnitude());
        }
        Ok(radiusspine)
    }

    /// The two-surface `SimulSurf`: the walking of the simulation with the chamfer
    /// function, its linear sections stored, the ends set and the obstacles kept only
    /// when no face continues the contour there.
    pub(super) fn simul_surf_impl(
        &mut self,
        data: &mut SurfDataHandle,
        call: &SurfCall<'_>,
        out: &mut SurfOut,
    ) -> Result<bool, Failure> {
        let kind = chamfer_kind(call.spine, "SimulSurf : this is not the spine of a chamfer")?;
        let (la, fi) = (call.guide.last_parameter()?, call.guide.first_parameter()?);
        let max_step = (la - fi) * 0.05;
        let radiusspine = Self::spine_curvature(call, max_step)?;
        let pfirst = out.first;
        self.base.extend_ends_to_spine(call.spine, out)?;
        let radius = match kind.method {
            ChamfMethod::Sym => kind.d1.max(radiusspine),
            ChamfMethod::TwoDist => kind.d1.max(kind.d2).max(radiusspine),
            ChamfMethod::DistAngle => kind.d1.max(kind.d1 * kind.angle.tan()).max(radiusspine),
        };
        let s1: Handle<dyn Adaptor3dSurface> = call.s1.clone().cast();
        let s2: Handle<dyn Adaptor3dSurface> = call.s2.clone().cast();
        let offset = if kind.method == ChamfMethod::TwoDist
            && kind.mode != crate::chfi_ds::ChamfMode::ClassicChamfer
        {
            offset_guide(call.spine, call.guide)
        } else {
            None
        };
        let fns = chamfer_functions(&kind, (&s1, &s2), call.guide, offset.as_ref(), call.choix)?;
        let mut walk = call.walk;
        walk.max_step = max_step;
        walk.fleche = radius * 1.0e-2;
        let ccall = ComputeCall {
            hguide: call.guide,
            spine: None,
            s1: &s1,
            i1: call.i1,
            s2: &s2,
            i2: call.i2,
            pfirst,
            walk,
            soldep: call.soldep,
            rec: (call.rec_on_s1, call.rec_on_s2),
        };
        let mut lin = None;
        let base = &self.base;
        let mut done = false;
        let mut sections = Vec::new();
        let mut pts = [Point2::default(); 4];
        with_chamfer(fns, &mut |run: &mut dyn ChamferRun| {
            done = base.simul_data(&ccall, offset.as_ref(), run.functions()?, 4, out, &mut lin)?;
            if !done {
                return Ok(());
            }
            let l = lin.clone().ok_or_else(|| Failure::new("SimulSurf"))?;
            let l = lock(&l);
            let nbp = l.nb_points();
            for i in 1..=nbp {
                let p = l.point(i)?;
                let (u1, v1) = p.parameters_on_s1()?;
                let (u2, v2) = p.parameters_on_s2()?;
                let (p1, p2, line) = run.section([u1, v1, u2, v2])?;
                sections.push(line_section(line, p1, p2));
                if i == 1 {
                    pts[0] = Point2::new(u1, v1);
                    pts[2] = Point2::new(u2, v2);
                }
                if i == nbp {
                    pts[1] = Point2::new(u1, v1);
                    pts[3] = Point2::new(u2, v2);
                }
            }
            Ok(())
        })?;
        self.base.done = done;
        if !done {
            return Ok(false);
        }
        let lin = lin.ok_or_else(|| Failure::new("SimulSurf"))?;
        {
            let mut d = lock(data);
            d.set_simul(&crate::chfi3d::builder::sections_transient(sections)?);
            d.set_2d_points(pts);
        }
        self.base.store_line_ends(data, &lin, [1, 2])?;
        let reverse = !call.walk.forward || call.walk.inside;
        if out.intf != 0 && reverse {
            out.intf = self.base.obstacle_at_end(data, call, true, out.intf)?;
        }
        if out.intl != 0 {
            out.intl = self.base.obstacle_at_end(data, call, false, out.intl)?;
        }
        Ok(true)
    }
}
