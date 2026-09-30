// Created on: 1993-04-07
// Created by: Laurent BUCHARD
// Copyright (c) 1993-1999 Matra Datavision
// Copyright (c) 1999-2014 OPEN CASCADE SAS
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
// Rust translation of PerformConicSurf* (IntCurveSurface_Inter.pxx, IntCurveSurface_HInter.cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The paths of a conic: analytic against a quadric (and a line against a torus or
//! a non-degenerate cone), sampled otherwise.
use super::{CurveOf, InterGen, InterTypes, SamplePolygon, SamplePolyhedron, SurfaceOf};
use crate::int_curve_surface::inter_utils::{
    self as utils, InfFlags, THE_TOLERANCE_ANGULAIRE, UvLimits, est_lim_for_inf_extr,
    est_lim_for_inf_offs, est_lim_for_inf_revl, est_lim_for_inf_surf,
};
use crate::int_curve_surface::tools::InterSurfaceTool;
use crate::intf::Tool;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkgeombase::int_ana::{IntConicQuad, Quadric};
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tkmath::geom_abs::SurfaceType;
use rustycad_tkmath::gp::{Circle3, Ellipse3, Hyperbola3, Line3, Parabola3};
use std::f64::consts::PI;

/// `THE_TOLERANCE`: the linear tolerance of the circle and ellipse plane cases.
const THE_TOLERANCE: f64 = 0.00000001;
/// `THE_NBSAMPLESONCIRCLE`.
const THE_NBSAMPLESONCIRCLE: i32 = 32;
/// `THE_NBSAMPLESONELLIPSE`.
const THE_NBSAMPLESONELLIPSE: i32 = 32;
/// `THE_NBSAMPLESONPARAB`.
const THE_NBSAMPLESONPARAB: i32 = 16;
/// `THE_NBSAMPLESONHYPR`.
const THE_NBSAMPLESONHYPR: i32 = 32;

/// The surface tool of an instance.
type S<T> = <T as InterTypes>::SurfaceTool;
/// The curve tool of an instance.
type C<T> = <T as InterTypes>::CurveTool;

/// The quadric of a cylinder, cone or sphere patch, `None` for another surface.
fn solid_quadric<T: InterTypes>(
    kind: SurfaceType,
    surface: &SurfaceOf<T>,
) -> Result<Option<Quadric>, Failure> {
    Ok(match kind {
        SurfaceType::Cylinder => Some(Quadric::from_cylinder(S::<T>::cylinder(surface)?)),
        SurfaceType::Cone => Some(Quadric::from_cone(S::<T>::cone(surface)?)),
        SurfaceType::Sphere => Some(Quadric::from_sphere(S::<T>::sphere(surface)?)),
        _ => None,
    })
}

impl<T: InterTypes> InterGen<T> {
    /// `PerformConicSurf(gp_Lin, ...)` (protected): analytic against a plane, a
    /// cylinder, a sphere, a torus (when the line-torus solver is done) and a cone
    /// of semi-angle below `pi/2 - 1e-7`; otherwise a polyhedron of at least 20 x 20
    /// samples on the patch (made finite by the EstLim functions) and one two-point
    /// polygon per segment of the line inside its box;
    /// `inter.perform_conic_surf_line(line, &c, &s, u1, v1, u2, v2)?`.
    #[allow(clippy::too_many_arguments)]
    pub fn perform_conic_surf_line(
        &mut self,
        line: Line3,
        curve: &CurveOf<T>,
        surface: &SurfaceOf<T>,
        u1: f64,
        v1: f64,
        u2: f64,
        v2: f64,
    ) -> Result<(), Failure> {
        let kind: SurfaceType = S::<T>::get_type(surface)?;
        let int_ana: Option<IntConicQuad> = match kind {
            SurfaceType::Plane => Some(IntConicQuad::line_plane(
                line,
                S::<T>::plane(surface)?,
                THE_TOLERANCE_ANGULAIRE,
                0.0,
                0.0,
            )?),
            SurfaceType::Cylinder => Some(IntConicQuad::line_quadric(
                line,
                &Quadric::from_cylinder(S::<T>::cylinder(surface)?),
            )?),
            SurfaceType::Sphere => Some(IntConicQuad::line_quadric(
                line,
                &Quadric::from_sphere(S::<T>::sphere(surface)?),
            )?),
            SurfaceType::Torus => {
                if let Some(points) = utils::process_lin_torus::<C<T>, S<T>>(line, curve, surface)?
                {
                    points.into_iter().for_each(|p| self.append_point(p));
                    return Ok(());
                }
                None
            }
            SurfaceType::Cone => {
                let correction: f64 = 1.0e5 * precision::angular();
                let cn = S::<T>::cone(surface)?;
                if cn.semi_angle().abs() < PI / 2.0 - correction {
                    Some(IntConicQuad::line_quadric(line, &Quadric::from_cone(cn))?)
                } else {
                    None
                }
            }
            _ => None,
        };
        match int_ana {
            Some(int_ana) => self.append_int_ana(curve, surface, &int_ana),
            None => self.sample_line(line, curve, surface, kind, [u1, v1, u2, v2]),
        }
    }
    /// The sampled path of a line (`!isAnaProcessed`).
    fn sample_line(
        &mut self,
        line: Line3,
        curve: &CurveOf<T>,
        surface: &SurfaceOf<T>,
        kind: SurfaceType,
        [u1, v1, u2, v2]: [f64; 4],
    ) -> Result<(), Failure> {
        let nbsu: i32 = S::<T>::nb_samples_u(surface, u1, u2)?.max(1);
        let nbsv: i32 = S::<T>::nb_samples_v(surface, v1, v2)?.max(1);
        let inf = InfFlags {
            u1: precision::is_infinite(u1),
            u2: precision::is_infinite(u2),
            v1: precision::is_infinite(v1),
            v2: precision::is_infinite(v2),
        };
        let mut l = UvLimits { u1, u2, v1, v2 };
        if inf.u1 || inf.u2 || inf.v1 || inf.v2 {
            let no_intersection: bool = match kind {
                SurfaceType::SurfaceOfExtrusion => {
                    est_lim_for_inf_extr::<S<T>>(line, surface, false, nbsu, inf, &mut l)?
                }
                SurfaceType::SurfaceOfRevolution => {
                    est_lim_for_inf_revl::<S<T>>(line, surface, inf, &mut l)?
                }
                SurfaceType::OffsetSurface => {
                    est_lim_for_inf_offs::<S<T>>(line, surface, nbsu, inf, &mut l)?
                }
                _ => {
                    est_lim_for_inf_surf(&mut l);
                    false
                }
            };
            if no_intersection {
                return Ok(());
            }
        }
        let bounds = [l.u1, l.v1, l.u2, l.v2];
        let polyhedron = T::Polyhedron::uniform(surface, nbsu.max(20), nbsv.max(20), bounds)?;
        let mut bnd_tool = Tool::new();
        let mut box_line = BoundingBox3::default();
        bnd_tool.lin_box(line, polyhedron.bounding(), &mut box_line);
        for nbseg in 1..=bnd_tool.nb_segments() {
            let (mut pinf, mut psup) = (bnd_tool.begin_param(nbseg)?, bnd_tool.end_param(nbseg)?);
            if (psup - pinf) < 1e-10 {
                pinf -= 1e-10;
                psup += 1e-10;
            }
            let polygon = T::Polygon::with_range(curve, pinf, psup, 2)?;
            self.internal_perform_polyhedron(
                curve,
                &polygon,
                surface,
                &polyhedron,
                l.u1,
                l.v1,
                l.u2,
                l.v2,
            )?;
        }
        Ok(())
    }
    /// The analytic path of a circle or ellipse against a plane or a solid quadric,
    /// else a polygon of `nb_samples` samples through the polygon path.
    fn perform_closed_conic(
        &mut self,
        plane_case: impl FnOnce(rustycad_tkmath::gp::Plane3) -> Result<IntConicQuad, Failure>,
        quadric_case: impl FnOnce(&Quadric) -> Result<IntConicQuad, Failure>,
        nb_samples: i32,
        curve: &CurveOf<T>,
        surface: &SurfaceOf<T>,
        [u1, v1, u2, v2]: [f64; 4],
    ) -> Result<(), Failure> {
        let kind: SurfaceType = S::<T>::get_type(surface)?;
        let int_ana: IntConicQuad = if kind == SurfaceType::Plane {
            plane_case(S::<T>::plane(surface)?)?
        } else if let Some(quadric) = solid_quadric::<T>(kind, surface)? {
            quadric_case(&quadric)?
        } else {
            let polygon = T::Polygon::with_samples(curve, nb_samples)?;
            return self.internal_perform_polygon(curve, &polygon, surface, u1, v1, u2, v2);
        };
        self.append_int_ana(curve, surface, &int_ana)
    }
    /// `PerformConicSurf(gp_Circ, ...)` (protected): analytic against a plane (1e-12
    /// angular, 1e-8 linear tolerance), a cylinder, a cone and a sphere, else a
    /// polygon of 32 samples; `inter.perform_conic_surf_circle(circle, &c, &s, u1, v1, u2, v2)?`.
    #[allow(clippy::too_many_arguments)]
    pub fn perform_conic_surf_circle(
        &mut self,
        circle: Circle3,
        curve: &CurveOf<T>,
        surface: &SurfaceOf<T>,
        u1: f64,
        v1: f64,
        u2: f64,
        v2: f64,
    ) -> Result<(), Failure> {
        self.perform_closed_conic(
            |pln| IntConicQuad::circle_plane(circle, pln, THE_TOLERANCE_ANGULAIRE, THE_TOLERANCE),
            |q| IntConicQuad::circle_quadric(circle, q),
            THE_NBSAMPLESONCIRCLE,
            curve,
            surface,
            [u1, v1, u2, v2],
        )
    }
    /// `PerformConicSurf(gp_Elips, ...)` (protected): as for a circle;
    /// `inter.perform_conic_surf_ellipse(ellipse, &c, &s, u1, v1, u2, v2)?`.
    #[allow(clippy::too_many_arguments)]
    pub fn perform_conic_surf_ellipse(
        &mut self,
        ellipse: Ellipse3,
        curve: &CurveOf<T>,
        surface: &SurfaceOf<T>,
        u1: f64,
        v1: f64,
        u2: f64,
        v2: f64,
    ) -> Result<(), Failure> {
        self.perform_closed_conic(
            |pln| IntConicQuad::ellipse_plane(ellipse, pln, THE_TOLERANCE_ANGULAIRE, THE_TOLERANCE),
            |q| IntConicQuad::ellipse_quadric(ellipse, q),
            THE_NBSAMPLESONELLIPSE,
            curve,
            surface,
            [u1, v1, u2, v2],
        )
    }
    /// The analytic path of a parabola or hyperbola against a plane or a solid
    /// quadric, else a polyhedron of at most 40 x 40 samples and, per segment of the
    /// conic inside its box (from `conic_box`), a polygon of `nb_samples` points.
    fn perform_open_conic(
        &mut self,
        analytic: &dyn Fn(&OpenConicArgument) -> Result<IntConicQuad, Failure>,
        conic_box: impl FnOnce(&mut Tool, &BoundingBox3, &mut BoundingBox3) -> Result<(), Failure>,
        nb_samples: i32,
        curve: &CurveOf<T>,
        surface: &SurfaceOf<T>,
        [u1, v1, u2, v2]: [f64; 4],
    ) -> Result<(), Failure> {
        let kind: SurfaceType = S::<T>::get_type(surface)?;
        let int_ana: IntConicQuad = if kind == SurfaceType::Plane {
            analytic(&OpenConicArgument::Plane(S::<T>::plane(surface)?))?
        } else if let Some(quadric) = solid_quadric::<T>(kind, surface)? {
            analytic(&OpenConicArgument::Quadric(quadric))?
        } else {
            let nbsu: i32 = S::<T>::nb_samples_u(surface, u1, u2)?.min(40);
            let nbsv: i32 = S::<T>::nb_samples_v(surface, v1, v2)?.min(40);
            let polyhedron = T::Polyhedron::uniform(surface, nbsu, nbsv, [u1, v1, u2, v2])?;
            let mut bnd_tool = Tool::new();
            let mut conic_bnd = BoundingBox3::default();
            conic_box(&mut bnd_tool, polyhedron.bounding(), &mut conic_bnd)?;
            for nbseg in 1..=bnd_tool.nb_segments() {
                let (first, last) = (bnd_tool.begin_param(nbseg)?, bnd_tool.end_param(nbseg)?);
                let polygon = T::Polygon::with_range(curve, first, last, nb_samples)?;
                self.internal_perform_polyhedron(
                    curve,
                    &polygon,
                    surface,
                    &polyhedron,
                    u1,
                    v1,
                    u2,
                    v2,
                )?;
            }
            return Ok(());
        };
        self.append_int_ana(curve, surface, &int_ana)
    }
    /// `PerformConicSurf(gp_Parab, ...)` (protected): analytic against a plane
    /// (1e-12 angular tolerance), a cylinder, a cone and a sphere, else polygons of
    /// 16 samples on the segments of Intf_Tool::ParabBox;
    /// `inter.perform_conic_surf_parabola(parab, &c, &s, u1, v1, u2, v2)?`.
    #[allow(clippy::too_many_arguments)]
    pub fn perform_conic_surf_parabola(
        &mut self,
        parab: Parabola3,
        curve: &CurveOf<T>,
        surface: &SurfaceOf<T>,
        u1: f64,
        v1: f64,
        u2: f64,
        v2: f64,
    ) -> Result<(), Failure> {
        let analytic = |argument: &OpenConicArgument| match argument {
            OpenConicArgument::Plane(pln) => {
                IntConicQuad::parabola_plane(parab, *pln, THE_TOLERANCE_ANGULAIRE)
            }
            OpenConicArgument::Quadric(q) => IntConicQuad::parabola_quadric(parab, q),
        };
        self.perform_open_conic(
            &analytic,
            |tool, domain, bnd| tool.parab_box(parab, domain, bnd),
            THE_NBSAMPLESONPARAB,
            curve,
            surface,
            [u1, v1, u2, v2],
        )
    }
    /// `PerformConicSurf(gp_Hypr, ...)` (protected): as for a parabola, with polygons
    /// of 32 samples on the segments of Intf_Tool::HyprBox;
    /// `inter.perform_conic_surf_hyperbola(hypr, &c, &s, u1, v1, u2, v2)?`.
    #[allow(clippy::too_many_arguments)]
    pub fn perform_conic_surf_hyperbola(
        &mut self,
        hypr: Hyperbola3,
        curve: &CurveOf<T>,
        surface: &SurfaceOf<T>,
        u1: f64,
        v1: f64,
        u2: f64,
        v2: f64,
    ) -> Result<(), Failure> {
        let analytic = |argument: &OpenConicArgument| match argument {
            OpenConicArgument::Plane(pln) => {
                IntConicQuad::hyperbola_plane(hypr, *pln, THE_TOLERANCE_ANGULAIRE)
            }
            OpenConicArgument::Quadric(q) => IntConicQuad::hyperbola_quadric(hypr, q),
        };
        self.perform_open_conic(
            &analytic,
            |tool, domain, bnd| tool.hypr_box(hypr, domain, bnd),
            THE_NBSAMPLESONHYPR,
            curve,
            surface,
            [u1, v1, u2, v2],
        )
    }
}

/// The second argument of an analytic parabola or hyperbola intersection.
enum OpenConicArgument {
    Plane(rustycad_tkmath::gp::Plane3),
    Quadric(Quadric),
}
