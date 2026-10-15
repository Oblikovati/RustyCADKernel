// Rust translation of the restriction overloads of ChFi3d_FilBuilder::SimulSurf
// (ChFi3d_FilBuilder.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The simulation of the fillets on obstacles: one side or both on a restriction.
use super::functions::{
    FunctionSettings, RstRstFillet, RstRstParts, RstRstSides, RstSideRefs, SurfRstFillet,
    SurfRstParts, rst_rst_const, rst_rst_evol, surf_rst_const, surf_rst_evol,
};
use super::simul::{SimulSections, law_of, simul_params};
use super::{ChFi3dFilBuilder, constant_radius};
use crate::brep_blend::BRepBlendLineHandle;
use crate::chfi_ds::{CircSection, SurfDataHandle, lock};
use crate::chfi3d::builder::{BlendSide, RstCall, RstSide, RstWalk, SurfOut};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkmath::gp::Point2;

const NOT_FILLET: &str = "PerformSurf : this is not the spine of the fillet";

/// The surface and the null restriction of the side of a walking on a surface.
pub(super) struct WalkSide {
    pub s: Handle<dyn Adaptor3dSurface>,
    pub rst: Handle<dyn Adaptor2dCurve2d>,
}

impl WalkSide {
    /// The side `side` of a call: its surface, its restriction (null on a surface).
    pub(super) fn of(side: &BlendSide<'_>) -> Self {
        Self {
            s: side.s.clone().cast(),
            rst: side
                .rst
                .as_ref()
                .map(|r| r.pc.clone().cast())
                .unwrap_or_default(),
        }
    }
}

/// The side on a restriction of a call, Standard_ConstructionError without one.
pub(super) fn rst_refs<'a>(side: &'a BlendSide<'a>) -> Result<RstSideRefs<'a>, Failure> {
    let rst = side.rst.as_ref().ok_or_else(|| {
        Failure::with_message(
            "Standard_ConstructionError",
            "SimulSurf : no restriction on the side",
        )
    })?;
    Ok(RstSideRefs { surf: side.s, rst })
}

/// The sections of a surface-restriction line and the 2d points of its ends on the
/// surface (`pf`, `pl`) and on the surface of the restriction (`ppcf`, `ppcl`).
fn surf_rst_sections<F: SurfRstFillet>(
    lin: &BRepBlendLineHandle,
    func: &mut F,
) -> Result<(Vec<CircSection>, [Point2; 4]), Failure> {
    let l = lock(lin);
    let nbp = l.nb_points();
    let mut sections = Vec::new();
    let mut pts = [Point2::default(); 4];
    for i in 1..=nbp {
        let p = l.point(i)?;
        let (u, v) = p.parameters_on_s()?;
        let (p1, p2, ci) = func.section_at(p.parameter(), (u, v, p.parameter_on_c()?))?;
        let mut isec = CircSection::new();
        isec.set_circle(ci, p1, p2);
        sections.push(isec);
        let (u2, v2) = p.parameters_on_s2()?;
        if i == 1 {
            pts[0] = Point2::new(u, v);
            pts[2] = Point2::new(u2, v2);
        }
        if i == nbp {
            pts[1] = Point2::new(u, v);
            pts[3] = Point2::new(u2, v2);
        }
    }
    Ok((sections, pts))
}

impl ChFi3dFilBuilder {
    /// The settings of a simulation walking on restrictions: the step and the deflection
    /// of `SimulParams`, 4 sections at least.
    fn simul_rst_walk<'a>(
        &self,
        call: &'a RstCall<'a>,
        out: &SurfOut,
    ) -> Result<RstWalk<'a>, Failure> {
        let (max_step, locfleche) = simul_params(call.guide, call.spine)?;
        let mut walk = call.walk;
        walk.max_step = max_step;
        walk.fleche = locfleche;
        Ok(RstWalk {
            hguide: call.guide,
            pfirst: out.first,
            walk,
            soldep: call.soldep,
            nb_sec_min: 4,
            rec: call.rec,
        })
    }

    /// The surface-restriction simulation with the functions `parts`: the walking, the
    /// sections and the decrochement at the end walked to (Standard_Failure on failure).
    fn simul_with_surf_rst<F: SurfRstFillet>(
        &mut self,
        mut parts: SurfRstParts<F>,
        sides: (
            &WalkSide,
            &crate::brep_blend::WalkingDomain,
            &WalkSide,
            &crate::brep_blend::WalkingDomain,
        ),
        w: &RstWalk<'_>,
        out: &mut SurfOut,
    ) -> Result<(BRepBlendLineHandle, bool, Vec<CircSection>, [Point2; 4]), Failure> {
        let side1: RstSide<'_> = (&sides.0.s, &sides.0.rst, sides.1);
        let side2: RstSide<'_> = (&sides.2.s, &sides.2.rst, sides.3);
        let mut lin = None;
        let (done, decroch) = {
            let mut f = lock(&parts.func);
            self.base.walk_surf_rst(
                None,
                (side1, side2),
                &mut *f,
                (&mut *parts.finv, &mut *parts.finvp, &mut *parts.finvc),
                w,
                out,
                &mut lin,
            )?
        };
        self.base.done = done;
        if !done {
            return Err(Failure::new("SimulSurf : Failed process!"));
        }
        let lin = lin.ok_or_else(|| Failure::new("SimulSurf : Failed process!"))?;
        let (sections, pts) = surf_rst_sections(&lin, &mut *lock(&parts.func))?;
        Ok((lin, decroch, sections, pts))
    }

    /// The `SimulSurf` overloads with one side on a restriction (`rst_on_first`: the first
    /// side, else the second): the surface-restriction function from the surface side to
    /// the restriction, its sections and the ends of the line stored on the sides.
    pub(super) fn simul_surf_on_rst(
        &mut self,
        data: &mut SurfDataHandle,
        call: &RstCall<'_>,
        out: &mut SurfOut,
        decroch: &mut [bool; 2],
        rst_on_first: bool,
    ) -> Result<(), Failure> {
        let radius = constant_radius(call.spine, NOT_FILLET)?;
        let (surf_side, rst_side) = if rst_on_first {
            (&call.side2, &call.side1)
        } else {
            (&call.side1, &call.side2)
        };
        let refs = rst_refs(rst_side)?;
        let settings = FunctionSettings {
            guide: call.guide,
            choix: call.choix,
            shape: self.section_shape,
        };
        let w = self.simul_rst_walk(call, out)?;
        let (ws, wr) = (WalkSide::of(surf_side), WalkSide::of(rst_side));
        let sides = (&ws, surf_side.i, &wr, rst_side.i);
        let (lin, dec, sections, pts) = if let Some(rad) = radius {
            let parts = surf_rst_const(surf_side.s, surf_side.orientation, &refs, rad, &settings)?;
            self.simul_with_surf_rst(parts, sides, &w, out)?
        } else {
            let law = law_of(call.spine, call.guide)?;
            let parts = surf_rst_evol(surf_side.s, surf_side.orientation, &refs, &law, &settings)?;
            self.simul_with_surf_rst(parts, sides, &w, out)?
        };
        let [pf, pl, ppcf, ppcl] = pts;
        let (pf2d, line_sides, k) = if rst_on_first {
            ([ppcf, ppcl, pf, pl], [2, 1], 0)
        } else {
            ([pf, pl, ppcf, ppcl], [1, 2], 1)
        };
        decroch[k] = dec;
        let secs = SimulSections { sections, pf: pf2d };
        self.store_simulation(data, secs, &lin, line_sides)
    }

    /// The `SimulSurf` overload with both sides on restrictions.
    pub(super) fn simul_surf_rst_rst_impl(
        &mut self,
        data: &mut SurfDataHandle,
        call: &RstCall<'_>,
        out: &mut SurfOut,
        decroch: &mut [bool; 2],
    ) -> Result<(), Failure> {
        let radius = constant_radius(call.spine, NOT_FILLET)?;
        let sides = RstRstSides {
            side1: rst_refs(&call.side1)?,
            or1: call.side1.orientation,
            side2: rst_refs(&call.side2)?,
            or2: call.side2.orientation,
        };
        let settings = FunctionSettings {
            guide: call.guide,
            choix: call.choix,
            shape: self.section_shape,
        };
        let w = self.simul_rst_walk(call, out)?;
        let (lin, dec, sections) = if let Some(rad) = radius {
            let parts = rst_rst_const(&sides, rad, &settings)?;
            self.simul_with_rst_rst(parts, call, &w, out)?
        } else {
            let law = law_of(call.spine, call.guide)?;
            let parts = rst_rst_evol(&sides, &law, &settings)?;
            self.simul_with_rst_rst(parts, call, &w, out)?
        };
        *decroch = dec;
        let simul = super::simul::sections_transient(sections)?;
        lock(data).set_simul(&simul);
        self.store_line_ends(data, &lin, [1, 2])
    }

    /// The restriction-restriction simulation with the functions `parts`.
    fn simul_with_rst_rst<F: RstRstFillet>(
        &mut self,
        mut parts: RstRstParts<F>,
        call: &RstCall<'_>,
        w: &RstWalk<'_>,
        out: &mut SurfOut,
    ) -> Result<(BRepBlendLineHandle, [bool; 2], Vec<CircSection>), Failure> {
        let (w1, w2) = (WalkSide::of(&call.side1), WalkSide::of(&call.side2));
        let mut lin = None;
        let (done, decroch) = {
            let mut f = lock(&parts.func);
            let [f1, f2] = &mut parts.finv;
            let [p1, p2] = &mut parts.finvp;
            self.base.walk_rst_rst(
                None,
                (
                    (&w1.s, &w1.rst, call.side1.i),
                    (&w2.s, &w2.rst, call.side2.i),
                ),
                &mut *f,
                ([&mut **f1, &mut **f2], [&mut **p1, &mut **p2]),
                w,
                out,
                &mut lin,
            )?
        };
        self.base.done = done;
        if !done {
            return Err(Failure::new("SimulSurf : Failed processing!"));
        }
        let lin = lin.ok_or_else(|| Failure::new("SimulSurf : Failed processing!"))?;
        let mut sections = Vec::new();
        {
            let l = lock(&lin);
            let mut f = lock(&parts.func);
            for i in 1..=l.nb_points() {
                let p = l.point(i)?;
                let uv = (p.parameter_on_c1()?, p.parameter_on_c2()?);
                let (p1, p2, ci) = f.section_at(p.parameter(), uv)?;
                let mut isec = CircSection::new();
                isec.set_circle(ci, p1, p2);
                sections.push(isec);
            }
        }
        Ok((lin, decroch, sections))
    }
}
