// Rust translation of the PerformSurf overrides of FilletSurf_InternalBuilder
// (FilletSurf_InternalBuilder.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The virtual members of the internal builder: the two-surface `PerformSurf` walks with
//! the simulation and approximates the line; the overloads on restrictions are refused;
//! the other members are those of the fillet builder.
use super::internal_builder::FilletSurfInternalBuilder;
use crate::blend_func::{ConstRad, ConstRadInv};
use crate::brep_blend::{BRepBlendLineHandle, SharedAppFunction, WalkingDomain};
use crate::chfi_ds::{
    CommonPoint, ElSpine, SpineHandle, StripeHandle, SurfDataHandle, lock, shared,
};
use crate::chfi3d::builder::{ComputeCall, LineFunction};
use crate::chfi3d::{
    ChFi3dBuilder, ChFi3dBuilderOps, RstCall, SurfCall, SurfOut, fil_common_point,
};
use rustycad_tkbrep::brep_adaptor::BRepAdaptorSurface;
use rustycad_tkbrep::topods::{Face, Vertex};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkg3d::top_abs::State;
use rustycad_tkmath::math::Vector;

/// The refusal of the overloads of `PerformSurf` on restrictions.
fn not_implemented() -> Failure {
    Failure::with_message(
        "Standard_DomainError",
        "BlendFunc_CSConstRad::Section : Not implemented",
    )
}

/// The constant radius of the fillet spine (Standard_ConstructionError otherwise).
fn radius_of(spine: &SpineHandle) -> Result<f64, Failure> {
    let g = lock(spine);
    let fsp = g.as_fil_spine().ok_or_else(|| {
        Failure::with_message(
            "Standard_ConstructionError",
            "PerformSurf : this is not the spine of a fillet",
        )
    })?;
    if !fsp.is_constant() {
        return Err(Failure::with_message(
            "Standard_ConstructionError",
            "PerformSurf : no variable radiuses",
        ));
    }
    fsp.radius()
}

/// The common points of the ends of the line that are on restrictions.
fn line_ends_on_arcs(
    data: &SurfDataHandle,
    lin: &BRepBlendLineHandle,
    tol: f64,
) -> Result<(), Failure> {
    let l = lock(lin);
    let mut d = lock(data);
    let (t1, t2) = (l.transition_on_s1()?, l.transition_on_s2()?);
    let ends = [
        (l.start_point_on_first(), t1, true, 1),
        (l.end_point_on_first(), t1, false, 1),
        (l.start_point_on_second(), t2, true, 2),
        (l.end_point_on_second(), t2, false, 2),
    ];
    for (ext, trans, start, on_s) in ends {
        if ext.nb_point_on_rst() != 0 {
            fil_common_point(ext, trans, start, d.change_vertex(start, on_s), tol)?;
        }
    }
    Ok(())
}

impl FilletSurfInternalBuilder {
    /// Whether the end of the fillet at the common point stays an obstacle
    /// (`!SearchFace(Spine, cp, F, bid)`), the first side tried before the second.
    fn obstacle(
        &self,
        spine: &SpineHandle,
        cps: [&CommonPoint; 2],
        faces: [Face; 2],
    ) -> Result<i32, Failure> {
        let mut flag = 0;
        let mut ok = false;
        for k in 0..2 {
            if cps[k].is_on_arc() && !ok {
                let mut bid = Face::default();
                flag = i32::from(!self.fil.base.search_face(
                    Some(spine),
                    cps[k],
                    &faces[k],
                    &mut bid,
                )?);
                ok = flag != 0;
            }
        }
        Ok(flag)
    }

    /// The two-surface `PerformSurf`: the simulation walking with at least 20 sections,
    /// the common points of the ends on restrictions, the approximation (a failure raises
    /// Standard_Failure) and the obstacles at the ends.
    fn perform_surf_fillet(
        &mut self,
        seq: &mut [SurfDataHandle],
        call: &SurfCall<'_>,
        out: &mut SurfOut,
    ) -> Result<bool, Failure> {
        let data = seq
            .first()
            .cloned()
            .ok_or_else(|| Failure::out_of_range("PerformSurf: no surface data"))?;
        let radius = radius_of(call.spine)?;
        let or = call.s1.face().orientation();
        let s1: Handle<dyn Adaptor3dSurface> = call.s1.clone().cast();
        let s2: Handle<dyn Adaptor3dSurface> = call.s2.clone().cast();
        let guide: Handle<dyn Adaptor3dCurve> = call.guide.clone().cast();
        let func = shared(ConstRad::new(&s1, &s2, &guide)?);
        let mut finv = ConstRadInv::new(&s1, &s2, &guide);
        lock(&func).set_radius(radius, call.choix);
        finv.set_radius(radius, call.choix);
        lock(&func).set_shape(self.fil.section_shape);
        let ccall = ComputeCall {
            hguide: call.guide,
            spine: None,
            s1: &s1,
            i1: call.i1,
            s2: &s2,
            i2: call.i2,
            pfirst: out.first,
            walk: call.walk,
            soldep: call.soldep,
            rec: (call.rec_on_s1, call.rec_on_s2),
        };
        let mut lin = None;
        let done = {
            let mut f = lock(&func);
            self.fil
                .base
                .simul_data(&ccall, None, (&mut *f, &mut finv), 20, out, &mut lin)?
        };
        self.fil.base.done = done;
        if !done {
            return Ok(false);
        }
        let lin = lin.ok_or_else(|| Failure::new("PerformSurf"))?;
        line_ends_on_arcs(&data, &lin, self.fil.base.tolapp3d)?;
        let app: SharedAppFunction = func.clone();
        let done = self.fil.base.complete_data(
            &data,
            (LineFunction::Surfaces, &app),
            &lin,
            (&s1, &s2),
            or,
            [false; 4],
            false,
        )?;
        self.fil.base.done = done;
        if !done {
            return Err(Failure::new("PerformSurf : Failed approximation!"));
        }
        self.end_obstacles(&data, call, out)?;
        let mut d = lock(&data);
        d.set_first_spine_param(out.first);
        d.set_last_spine_param(out.last);
        Ok(true)
    }

    /// `Intf` (only walking backward) and `Intl`: the ends on restrictions that stay
    /// obstacles.
    fn end_obstacles(
        &self,
        data: &SurfDataHandle,
        call: &SurfCall<'_>,
        out: &mut SurfOut,
    ) -> Result<(), Failure> {
        let faces = [call.s1.face().clone(), call.s2.face().clone()];
        let (f1, f2, l1, l2) = {
            let d = lock(data);
            (
                d.vertex_first_on_s1().clone(),
                d.vertex_first_on_s2().clone(),
                d.vertex_last_on_s1().clone(),
                d.vertex_last_on_s2().clone(),
            )
        };
        if !call.walk.forward {
            out.intf = self.obstacle(call.spine, [&f1, &f2], faces.clone())?;
        }
        out.intl = self.obstacle(call.spine, [&l1, &l2], faces)?;
        Ok(())
    }
}

impl ChFi3dBuilderOps for FilletSurfInternalBuilder {
    fn base(&self) -> &ChFi3dBuilder {
        &self.fil.base
    }
    fn base_mut(&mut self) -> &mut ChFi3dBuilder {
        &mut self.fil.base
    }
    fn simul_kpart(&self, sd: &SurfDataHandle) -> Result<(), Failure> {
        self.fil.simul_kpart(sd)
    }
    fn simul_surf(
        &mut self,
        data: &mut SurfDataHandle,
        call: &SurfCall<'_>,
        out: &mut SurfOut,
    ) -> Result<bool, Failure> {
        self.fil.simul_surf(data, call, out)
    }
    fn simul_surf_rst1(
        &mut self,
        data: &mut SurfDataHandle,
        call: &RstCall<'_>,
        out: &mut SurfOut,
        decroch: &mut [bool; 2],
    ) -> Result<(), Failure> {
        self.fil.simul_surf_rst1(data, call, out, decroch)
    }
    fn simul_surf_rst2(
        &mut self,
        data: &mut SurfDataHandle,
        call: &RstCall<'_>,
        out: &mut SurfOut,
        decroch: &mut [bool; 2],
    ) -> Result<(), Failure> {
        self.fil.simul_surf_rst2(data, call, out, decroch)
    }
    fn simul_surf_rst_rst(
        &mut self,
        data: &mut SurfDataHandle,
        call: &RstCall<'_>,
        out: &mut SurfOut,
        decroch: &mut [bool; 2],
    ) -> Result<(), Failure> {
        self.fil.simul_surf_rst_rst(data, call, out, decroch)
    }
    fn perform_surf(
        &mut self,
        data: &mut Vec<SurfDataHandle>,
        call: &SurfCall<'_>,
        out: &mut SurfOut,
    ) -> Result<bool, Failure> {
        self.perform_surf_fillet(data, call, out)
    }
    fn perform_surf_rst1(
        &mut self,
        _: &mut Vec<SurfDataHandle>,
        _: &RstCall<'_>,
        _: &mut SurfOut,
        _: &mut [bool; 2],
    ) -> Result<(), Failure> {
        Err(not_implemented())
    }
    fn perform_surf_rst2(
        &mut self,
        _: &mut Vec<SurfDataHandle>,
        _: &RstCall<'_>,
        _: &mut SurfOut,
        _: &mut [bool; 2],
    ) -> Result<(), Failure> {
        Err(not_implemented())
    }
    fn perform_surf_rst_rst(
        &mut self,
        _: &mut Vec<SurfDataHandle>,
        _: &RstCall<'_>,
        _: &mut SurfOut,
        _: &mut [bool; 2],
    ) -> Result<(), Failure> {
        Err(not_implemented())
    }
    fn perform_two_corner(&mut self, index: i32) -> Result<(), Failure> {
        self.fil.perform_two_corner(index)
    }
    fn perform_three_corner(&mut self, index: i32) -> Result<(), Failure> {
        self.fil.perform_three_corner(index)
    }
    fn extent_one_corner(&mut self, v: &Vertex, s: &StripeHandle) -> Result<(), Failure> {
        self.fil.extent_one_corner(v, s)
    }
    fn extent_two_corner(&mut self, v: &Vertex, ls: &[StripeHandle]) -> Result<(), Failure> {
        self.fil.extent_two_corner(v, ls)
    }
    fn extent_three_corner(&mut self, v: &Vertex, ls: &[StripeHandle]) -> Result<(), Failure> {
        self.fil.extent_three_corner(v, ls)
    }
    fn perform_first_section(
        &self,
        s: &SpineHandle,
        hguide: &Handle<ElSpine>,
        choix: i32,
        surfaces: (&Handle<BRepAdaptorSurface>, &Handle<BRepAdaptorSurface>),
        domains: (&WalkingDomain, &WalkingDomain),
        par: f64,
        soldep: &mut Vector<'static>,
    ) -> Result<(bool, State, State), Failure> {
        self.fil
            .perform_first_section(s, hguide, choix, surfaces, domains, par, soldep)
    }
    fn set_regul(&mut self) -> Result<(), Failure> {
        self.fil.set_regul()
    }
}
