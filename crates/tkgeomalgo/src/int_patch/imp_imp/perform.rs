// Copyright (c) 1992-1999 Matra Datavision
// Copyright (c) 1999-2024 OPEN CASCADE SAS
//
// This file is part of Open CASCADE Technology software library.
//
// This library is free software; you can redistribute it and/or modify it under
// the terms of the GNU Lesser General Public License version 2.1 as published
// by the Free Software Foundation, with special exception defined in the file
// OCCT_LGPL_EXCEPTION.txt. Consult the file LICENSE_LGPL_21.txt included in OCCT
// distribution for complete text of the license and disclaimer of any warranty.
//
// Alternatively, this file may be used under the terms of Open CASCADE
// commercial license or contractual agreement.
// Rust translation of IntPatch_ImpImpIntersection::Perform and SetQuad of
// ModelingAlgorithms/TKGeomAlgo/IntPatch/IntPatch_ImpImpIntersection.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The intersection of two quadric patches: the pair solver of the quadrics, then the
//! points on the restrictions of both domains put on the lines.
use super::super::arc_function::IntPatchArcFunction;
use super::cyl_cyl::{CyCyFlags, int_cycy};
use super::{IntPatchImpImpIntersection, IntPatchImpImpStatus, cone_cone, cone_sphere, cyl_cone, cyl_sphere, plane, plane_cone, sphere_sphere, torus};
use crate::int_start::TheTopolTool;
use crate::int_surf::IntSurfQuadric;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::native::std_min;
use rustycad_tkg3d::adaptor3d::h_surface_tool::Adaptor3dSurfaceHandle;
use rustycad_tkmath::bnd::BoundingBox2;
use rustycad_tkmath::geom_abs::SurfaceType;
use rustycad_tkmath::gp::Point2;
use std::f64::consts::PI;

/// `SetQuad(theS, theTS, theQuad)`: the quadric of the surface and its rank (1 plane,
/// 2 cylinder, 3 cone, 4 sphere, 5 torus; 0 for another surface).
fn set_quad(s: &Adaptor3dSurfaceHandle) -> Result<(SurfaceType, IntSurfQuadric, i32), Failure> {
    let ts: SurfaceType = s.get_type()?;
    let mut quad = IntSurfQuadric::new();
    let rank: i32 = match ts {
        SurfaceType::Plane => {
            quad.set_plane(&s.plane()?);
            1
        }
        SurfaceType::Cylinder => {
            quad.set_cylinder(&s.cylinder()?);
            2
        }
        SurfaceType::Cone => {
            quad.set_cone(&s.cone()?);
            3
        }
        SurfaceType::Sphere => {
            quad.set_sphere(&s.sphere()?);
            4
        }
        SurfaceType::Torus => {
            quad.set_torus(&s.torus()?);
            5
        }
        _ => 0,
    };
    Ok((ts, quad, rank))
}

/// The inputs of `Perform`.
pub(super) struct PerformInput<'a> {
    pub(super) s1: &'a Adaptor3dSurfaceHandle,
    pub(super) d1: &'a (dyn TheTopolTool + 'static),
    pub(super) s2: &'a Adaptor3dSurfaceHandle,
    pub(super) d2: &'a (dyn TheTopolTool + 'static),
    pub(super) tol_arc: f64,
    pub(super) tol_tang: f64,
    pub(super) keep_rline: bool,
}

/// The flags the pair solvers set.
#[derive(Default)]
pub(super) struct PairFlags {
    pub(super) same_surf: bool,
    pub(super) multpoint: bool,
    pub(super) b_empty: bool,
    pub(super) post_processing: bool,
}

/// `Standard_ConstructionError` of a surface that is no quadric.
fn not_quadrics(t1: SurfaceType, t2: SurfaceType) -> Failure {
    Failure::with_message(
        "Standard_ConstructionError",
        &format!("IntPatch_ImpImpIntersection::Perform() - surfaces {t1:?} and {t2:?}; expected two quadrics"),
    )
}

impl IntPatchImpImpIntersection {
    /// `Perform(S1, D1, S2, D2, TolArc, TolTang, theIsReqToKeepRLine)` (the native
    /// default of the flag is false, which removes the restriction lines along a conic;
    /// TopOpeBRep keeps them): Standard_ConstructionError unless both surfaces are
    /// quadrics. `inter.perform(&s1, &*d1, &s2, &*d2, 1e-7, 1e-6, false)?`.
    #[allow(clippy::too_many_arguments)]
    pub fn perform(
        &mut self,
        s1: &Adaptor3dSurfaceHandle,
        d1: &(dyn TheTopolTool + 'static),
        s2: &Adaptor3dSurfaceHandle,
        d2: &(dyn TheTopolTool + 'static),
        tol_arc: f64,
        tol_tang: f64,
        is_req_to_keep_rline: bool,
    ) -> Result<(), Failure> {
        self.my_done = IntPatchImpImpStatus::Fail;
        self.spnt.clear();
        self.slin.clear();
        (self.empt, self.tgte, self.oppo) = (true, false, false);
        let input = PerformInput { s1, d1, s2, d2, tol_arc, tol_tang, keep_rline: is_req_to_keep_rline };
        let (typs1, quad1, it1) = set_quad(s1)?;
        let (typs2, quad2, it2) = set_quad(s2)?;
        if it1 == 0 || it2 == 0 {
            return Err(not_quadrics(typs1, typs2));
        }
        let mut flags = PairFlags { post_processing: true, ..PairFlags::default() };
        if !self.pair(&input, (&quad1, &quad2), (it1, it2), &mut flags)? {
            return Ok(());
        }
        if flags.b_empty {
            if self.my_done == IntPatchImpImpStatus::Fail {
                self.my_done = IntPatchImpImpStatus::Ok;
            }
            return Ok(());
        }
        if flags.post_processing && !self.post_process(&input, (&quad1, &quad2), (typs1, typs2), &flags)? {
            return Ok(());
        }
        self.finish(&input, (&quad1, &quad2))
    }

    /// The pair solver of the quadrics (`iT1 * 10 + iT2`, reversed when the first rank
    /// is the greater); false when it fails.
    fn pair(
        &mut self,
        input: &PerformInput<'_>,
        quads: (&IntSurfQuadric, &IntSurfQuadric),
        (it1, it2): (i32, i32),
        flags: &mut PairFlags,
    ) -> Result<bool, Failure> {
        let reverse: bool = it1 > it2;
        let tolang: f64 = 1.0e-8;
        let tt: f64 = input.tol_tang;
        let (slin, spnt) = (&mut self.slin, &mut self.spnt);
        let empt: &mut bool = &mut self.empt;
        let ok: bool = match it1 * 10 + it2 {
            11 => plane::int_pp(quads, (tolang, tt), &mut flags.same_surf, slin)?,
            12 | 21 => {
                let cyl = if reverse { input.s1 } else { input.s2 };
                let (vmin, vmax) = (cyl.first_v_parameter()?, cyl.last_v_parameter()?);
                let h: f64 = if precision::is_negative_infinite(vmin) || precision::is_positive_infinite(vmax) { 0.0 } else { vmax - vmin };
                plane::int_pcy(quads, (tolang, tt), reverse, empt, slin, h)?
            }
            13 | 31 => plane_cone::int_pco(quads, (tolang, tt), reverse, empt, &mut flags.multpoint, slin, spnt)?,
            14 | 41 => plane::int_psp(quads, (tolang, tt), reverse, empt, slin, spnt)?,
            15 | 51 => torus::int_pto(quads, tt, reverse, empt, slin)?,
            22 => return self.pair_cycy(input, quads, flags),
            23 | 32 => cyl_cone::int_cyco(quads, tt, reverse, empt, &mut flags.multpoint, slin, spnt)?,
            24 | 42 => cyl_sphere::int_cysp(quads, tt, reverse, empt, &mut flags.multpoint, slin, spnt)?,
            25 | 52 | 45 | 54 => torus::int_quadric_torus(quads, tt, reverse, empt, slin)?,
            33 => cone_cone::int_coco(quads, tt, empt, &mut flags.same_surf, &mut flags.multpoint, slin, spnt)?,
            34 | 43 => cone_sphere::int_cosp(quads, tt, reverse, empt, &mut flags.multpoint, slin, spnt)?,
            35 | 53 => {
                // The cone and torus case does not update the empty flag, as natively.
                return torus::int_quadric_torus(quads, tt, reverse, empt, slin);
            }
            44 => sphere_sphere::int_spsp(quads, tt, (empt, &mut flags.same_surf), slin, spnt)?,
            55 => torus::int_toto(quads, tt, &mut flags.same_surf, empt, slin)?,
            _ => return Err(Failure::with_message("Standard_ConstructionError", "IntPatch_ImpImpIntersection::Perform() - unknown pair of quadrics")),
        };
        // Plane/plane does not update the empty flag either.
        flags.b_empty = ok && it1 * 10 + it2 != 11 && self.empt;
        Ok(ok)
    }

    /// Two cylinders: `IntCyCy` over the UV boxes of the surfaces (U limited to one
    /// period, the 2d tolerance bounded by 1e-4 for small radii); its walking lines need
    /// no post-processing.
    fn pair_cycy(
        &mut self,
        input: &PerformInput<'_>,
        quads: (&IntSurfQuadric, &IntSurfQuadric),
        flags: &mut PairFlags,
    ) -> Result<bool, Failure> {
        let (s1, s2) = (input.s1, input.s2);
        let uperiod: f64 = 2.0 * PI;
        let (u1f, mut u1l) = (s1.first_u_parameter()?, s1.last_u_parameter()?);
        let (u2f, mut u2l) = (s2.first_u_parameter()?, s2.last_u_parameter()?);
        if u1l - u1f > uperiod {
            u1l = u1f + uperiod;
        }
        if u2l - u2f > uperiod {
            u2l = u2f + uperiod;
        }
        let mut box1 = BoundingBox2::default();
        let mut box2 = BoundingBox2::default();
        box1.add_point(Point2::new(u1f, s1.first_v_parameter()?));
        box1.add_point(Point2::new(u1l, s1.last_v_parameter()?));
        box2.add_point(Point2::new(u2f, s2.first_v_parameter()?));
        box2.add_point(Point2::new(u2l, s2.last_v_parameter()?));
        // The resolution is too big for a small radius: bound it by a constant.
        let tol_2d: f64 = std_min(1.0e-4, std_min(s1.u_resolution(input.tol_tang)?, s2.u_resolution(input.tol_tang)?));
        let cyflags = CyCyFlags { empty: &mut self.empt, same: &mut flags.same_surf, multpoint: &mut flags.multpoint };
        self.my_done = int_cycy(quads, input.tol_tang, tol_2d, (&box1, &box2), cyflags, &mut self.slin, &mut self.spnt)?;
        if self.my_done == IntPatchImpImpStatus::Fail {
            return Ok(false);
        }
        flags.b_empty = self.empt;
        if let Some(first) = self.slin.first() {
            let w: rustycad_tkernel::standard::Handle<super::super::wline::IntPatchWLine> = first.cast();
            if !w.is_null() {
                // No geometric solution.
                flags.post_processing = false;
            }
        }
        Ok(true)
    }
}

/// The arc function of the quadric `quad` on the surface `s`.
pub(super) fn arc_function(quad: &IntSurfQuadric, s: &Adaptor3dSurfaceHandle) -> IntPatchArcFunction {
    let mut f = IntPatchArcFunction::new();
    f.set_quadric(quad);
    f.set_surface(s);
    f
}
