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
// Rust translation of PerformBounds, InternalPerform*, AppendIntAna, AppendPoint and
// AppendSegment (IntCurveSurface_Inter.pxx, IntCurveSurface_HInter.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The protected steps of the algorithm: the dispatch on the curve kind, the
//! polygon-polyhedron search refined by the exact intersector, the quadric path and
//! the appending of points.
use super::{CurveOf, InterGen, InterTypes, SamplePolygon, SamplePolyhedron, SurfaceOf};
use crate::int_curve_surface::inter_utils::{
    self as utils, SortedStartPoints, THE_TOLTANGENCY, UvLimits, clamp_uv_parameters,
    sort_start_points,
};
use crate::int_curve_surface::tools::{InterCurveTool, InterSurfaceTool};
use crate::int_imp::TheCurveTool;
use crate::int_imp::{IntImpIntCS, IntImpZerCSParFunc};
use crate::intf::{Interference, InterferencePolygonPolyhedron};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::Adaptor3dTopolTool;
use rustycad_tkgeombase::int_ana::IntConicQuad;
use rustycad_tkmath::bnd::BoundSortBox;
use rustycad_tkmath::geom_abs::{CurveType, Shape, SurfaceType};
use rustycad_tkmath::math::FunctionSetRoot;

/// The curve tool of an instance.
type C<T> = <T as InterTypes>::CurveTool;
/// The surface tool of an instance.
type S<T> = <T as InterTypes>::SurfaceTool;

/// Whether a surface takes the quadric path (plane, cylinder, cone, sphere).
fn is_quadric(kind: SurfaceType) -> bool {
    matches!(
        kind,
        SurfaceType::Plane | SurfaceType::Cylinder | SurfaceType::Cone | SurfaceType::Sphere
    )
}

impl<T: InterTypes> InterGen<T> {
    /// `Perform(Curve, Surface, U1, V1, U2, V2)` (protected): the patch bounds
    /// clamped to 1e50; a conic goes to its analytic path, another curve against a
    /// non-quadric to the polygon path (one polygon per C2 interval of the curve,
    /// sampled with deflection 0.1 and at least 10 points), against a quadric to
    /// the quadric path; `inter.perform_bounds(&c, &s, u1, v1, u2, v2)?`.
    pub fn perform_bounds(
        &mut self,
        curve: &CurveOf<T>,
        surface: &SurfaceOf<T>,
        u1: f64,
        v1: f64,
        u2: f64,
        v2: f64,
    ) -> Result<(), Failure> {
        let mut l = UvLimits { u1, u2, v1, v2 };
        clamp_uv_parameters(&mut l);
        let (u1, v1, u2, v2) = (l.u1, l.v1, l.u2, l.v2);
        match C::<T>::get_type(curve)? {
            CurveType::Line => {
                self.perform_conic_surf_line(C::<T>::line(curve)?, curve, surface, u1, v1, u2, v2)
            }
            CurveType::Circle => self.perform_conic_surf_circle(
                C::<T>::circle(curve)?,
                curve,
                surface,
                u1,
                v1,
                u2,
                v2,
            ),
            CurveType::Ellipse => self.perform_conic_surf_ellipse(
                C::<T>::ellipse(curve)?,
                curve,
                surface,
                u1,
                v1,
                u2,
                v2,
            ),
            CurveType::Parabola => self.perform_conic_surf_parabola(
                C::<T>::parabola(curve)?,
                curve,
                surface,
                u1,
                v1,
                u2,
                v2,
            ),
            CurveType::Hyperbola => self.perform_conic_surf_hyperbola(
                C::<T>::hyperbola(curve)?,
                curve,
                surface,
                u1,
                v1,
                u2,
                v2,
            ),
            _ => self.perform_other_curve(curve, surface, [u1, v1, u2, v2]),
        }
    }
    /// The `default` case of PerformBounds.
    fn perform_other_curve(
        &mut self,
        curve: &CurveOf<T>,
        surface: &SurfaceOf<T>,
        [u1, v1, u2, v2]: [f64; 4],
    ) -> Result<(), Failure> {
        let nb_intervals: i32 = C::<T>::nb_intervals(curve, Shape::C2)?;
        if is_quadric(S::<T>::get_type(surface)?) {
            return self.internal_perform_curve_quadric(curve, surface);
        }
        let ranges: Vec<(f64, f64)> = if nb_intervals > 1 {
            let mut tab: Array1<'static, f64> = Array1::with_bounds(1, nb_intervals + 1)
                .map_err(|e| e.into_failure("NCollection_Array1"))?;
            C::<T>::intervals(curve, &mut tab, Shape::C2)?;
            (1..=nb_intervals).map(|i| (tab[i], tab[i + 1])).collect()
        } else {
            vec![(
                C::<T>::first_parameter(curve)?,
                C::<T>::last_parameter(curve)?,
            )]
        };
        for (w1, w2) in ranges {
            let pars = C::<T>::sample_pars(curve, w1, w2, 0.1, 10)?;
            let polygon = T::Polygon::with_parameters(curve, &pars)?;
            self.internal_perform_polygon(curve, &polygon, surface, u1, v1, u2, v2)?;
        }
        Ok(())
    }
    /// `InternalPerformCurveQuadric(Curve, Surface)` (protected): appends the points
    /// at the roots of the curve-quadric distance; `inter.internal_perform_curve_quadric(&c, &s)?`.
    pub fn internal_perform_curve_quadric(
        &mut self,
        curve: &CurveOf<T>,
        surface: &SurfaceOf<T>,
    ) -> Result<(), Failure> {
        for point in utils::perform_curve_quadric::<C<T>, S<T>>(curve, surface)? {
            self.append_point(point);
        }
        Ok(())
    }
    /// `InternalPerform(Curve, Polygon, Surface, Polyhedron, U1, V1, U2, V2)`
    /// (protected): the section points of the polygon-polyhedron interference,
    /// sorted and refined by the exact intersector within the patch and the polygon
    /// range; `inter.internal_perform_polyhedron(&c, &pg, &s, &ph, u1, v1, u2, v2)?`.
    #[allow(clippy::too_many_arguments)]
    pub fn internal_perform_polyhedron(
        &mut self,
        curve: &CurveOf<T>,
        polygon: &T::Polygon,
        surface: &SurfaceOf<T>,
        polyhedron: &T::Polyhedron,
        u0: f64,
        v0: f64,
        u1: f64,
        v1: f64,
    ) -> Result<(), Failure> {
        let interference =
            InterferencePolygonPolyhedron::<T::PolygonTool, T::PolyhedronTool>::with_polygon(
                polygon, polyhedron,
            )?;
        self.refine_interference(
            &interference,
            curve,
            polygon,
            surface,
            polyhedron,
            [u0, v0, u1, v1],
        )
    }
    /// `InternalPerform(..., BSB)` (protected): the same with a prepared grid of the
    /// triangles; `inter.internal_perform_polyhedron_grid(&c, &pg, &s, &ph, [u1, v1, u2, v2], &mut bsb)?`.
    pub fn internal_perform_polyhedron_grid(
        &mut self,
        curve: &CurveOf<T>,
        polygon: &T::Polygon,
        surface: &SurfaceOf<T>,
        polyhedron: &T::Polyhedron,
        bounds: [f64; 4],
        grid: &mut BoundSortBox,
    ) -> Result<(), Failure> {
        let interference =
            InterferencePolygonPolyhedron::<T::PolygonTool, T::PolyhedronTool>::with_polygon_grid(
                polygon, polyhedron, grid,
            )?;
        self.refine_interference(&interference, curve, polygon, surface, polyhedron, bounds)
    }
    /// The common body of both InternalPerform: the exact intersector on the CS
    /// function (tolerance `THE_TOLTANGENCY`) with a solver of 100 iterations,
    /// then the sorted start points refined and appended.
    fn refine_interference(
        &mut self,
        interference: &Interference,
        curve: &CurveOf<T>,
        polygon: &T::Polygon,
        surface: &SurfaceOf<T>,
        polyhedron: &T::Polyhedron,
        [u0, v0, u1, v1]: [f64; 4],
    ) -> Result<(), Failure> {
        let function = IntImpZerCSParFunc::<S<T>, C<T>>::new(surface, curve);
        let mut exact = IntImpIntCS::from_function(&function, THE_TOLTANGENCY);
        let mut rsnld = FunctionSetRoot::without_tolerance(exact.function(), 100)?;
        let (winf, wsup) = (polygon.inf_parameter(), polygon.sup_parameter());
        let mut points: SortedStartPoints =
            utils::collect_interference_points(interference, polyhedron, polygon)?;
        sort_start_points(&mut points);
        let bounds = [u0, u1, v0, v1, winf, wsup];
        let found = utils::process_sorted_points::<C<T>, S<T>>(
            &mut exact, &mut rsnld, &points, bounds, curve, surface,
        )?;
        for point in found {
            self.append_point(point);
        }
        Ok(())
    }
    /// `InternalPerform(Curve, Polygon, Surface, U1, V1, U2, V2)` (protected): a
    /// quadric takes the quadric path; a B-spline surface is sampled by an
    /// Adaptor3d_TopolTool on the patch (deflection 0.1, at least 10 x 10), any other
    /// surface uniformly (at most 40 x 40); then the polygon-polyhedron search;
    /// `inter.internal_perform_polygon(&c, &pg, &s, u1, v1, u2, v2)?`.
    #[allow(clippy::too_many_arguments)]
    pub fn internal_perform_polygon(
        &mut self,
        curve: &CurveOf<T>,
        polygon: &T::Polygon,
        surface: &SurfaceOf<T>,
        u1: f64,
        v1: f64,
        u2: f64,
        v2: f64,
    ) -> Result<(), Failure> {
        let kind: SurfaceType = S::<T>::get_type(surface)?;
        if is_quadric(kind) {
            return self.internal_perform_curve_quadric(curve, surface);
        }
        let polyhedron = if kind != SurfaceType::BSplineSurface {
            let nbsu: i32 = S::<T>::nb_samples_u(surface, u1, u2)?.min(40);
            let nbsv: i32 = S::<T>::nb_samples_v(surface, v1, v2)?.min(40);
            T::Polyhedron::uniform(surface, nbsu, nbsv, [u1, v1, u2, v2])?
        } else {
            Self::bspline_polyhedron(surface, [u1, v1, u2, v2])?
        };
        self.internal_perform_polyhedron(curve, polygon, surface, &polyhedron, u1, v1, u2, v2)
    }
    /// The polyhedron of a B-spline patch at the sample parameters of a topological
    /// tool on the trimmed surface.
    fn bspline_polyhedron(
        surface: &SurfaceOf<T>,
        [u1, v1, u2, v2]: [f64; 4],
    ) -> Result<T::Polyhedron, Failure> {
        let trimmed = S::<T>::u_trim(surface, u1, u2, 1.0e-9)?;
        let trimmed = trimmed.v_trim(v1, v2, 1.0e-9)?;
        let top_tool = Adaptor3dTopolTool::with_surface(&trimmed)?;
        top_tool.sample_pnts(0.1, 10, 10)?;
        let (upars, vpars) = (top_tool.u_parameters()?, top_tool.v_parameters()?);
        T::Polyhedron::with_parameters(surface, &upars, &vpars)
    }
    /// `AppendIntAna(Curve, Surface, InterAna)` (protected): appends the points of an
    /// analytic intersection, or marks the curve parallel to the surface;
    /// `inter.append_int_ana(&c, &s, &ana)?`.
    pub fn append_int_ana(
        &mut self,
        curve: &CurveOf<T>,
        surface: &SurfaceOf<T>,
        int_ana: &IntConicQuad,
    ) -> Result<(), Failure> {
        match utils::process_int_ana::<C<T>, S<T>>(curve, surface, int_ana)? {
            Some(Some(points)) => points.into_iter().for_each(|p| self.append_point(p)),
            Some(None) => self.base.my_is_parallel = true,
            None => {}
        }
        Ok(())
    }
    /// `AppendPoint(Curve, w, Surface, u, v)` (protected): appends the point at
    /// `(w, u, v)` when it lies in the curve and surface ranges;
    /// `inter.append_curve_point(&c, 0.5, &s, 0.1, 0.2)?`.
    pub fn append_curve_point(
        &mut self,
        curve: &CurveOf<T>,
        w: f64,
        surface: &SurfaceOf<T>,
        u: f64,
        v: f64,
    ) -> Result<(), Failure> {
        if let Some(point) = utils::compute_append_point::<C<T>, S<T>>(curve, w, surface, u, v)? {
            self.append_point(point);
        }
        Ok(())
    }
    /// `AppendSegment(Curve, u0, u1, Surface)` (protected): not implemented natively,
    /// does nothing; `inter.append_curve_segment(&c, 0.0, 1.0, &s)`.
    pub fn append_curve_segment(
        &mut self,
        _curve: &CurveOf<T>,
        _u0: f64,
        _u1: f64,
        _surface: &SurfaceOf<T>,
    ) {
    }
}
