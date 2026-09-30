// Copyright (c) 1992-1999 Matra Datavision
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

//! Behavior of the IntPatch core beyond the numeric probe, which compares it with native
//! OCCT bit for bit: the records and their edits, the vertex computations of the lines,
//! the tools and the RTTI of the line hierarchy.
use rustycad_tkernel::standard::{Handle, RuntimeTyped};
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkg3d::adaptor3d::h_surface_tool::Adaptor3dSurfaceHandle;
use rustycad_tkg3d::geom::{GeomCylindricalSurface, GeomPlane, GeomSphericalSurface, GeomSurface};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorSurface;
use rustycad_tkgeomalgo::int_patch::*;
use rustycad_tkgeomalgo::int_start::TheArc;
use rustycad_tkgeomalgo::int_surf::{
    IntSurfLineOn2S, IntSurfPntOn2S, IntSurfQuadric, IntSurfTransition, IntSurfTypeTrans,
};
use rustycad_tkmath::gp::{Axis3, Circle3, Direction2, Direction3, Point2, Point3, Sphere3};
use std::f64::consts::PI;

fn frame(z: f64) -> Axis3 {
    Axis3::from_normal(
        Point3::new(0.0, 0.0, z),
        Direction3::new(0.0, 0.0, 1.0).unwrap(),
    )
    .unwrap()
}

fn adapted(s: Handle<dyn GeomSurface>) -> Adaptor3dSurfaceHandle {
    Handle::new(GeomAdaptorSurface::with_surface(&s).unwrap()).cast()
}

/// The vertical line `u = x` of a parametric space over `v` in `[v0, v1]`.
fn vertical_arc(x: f64, v0: f64, v1: f64) -> TheArc {
    let dir = Direction2::new(0.0, 1.0).unwrap();
    let line: Handle<dyn Geom2dCurve> =
        Handle::new(Geom2dLine::from_point(Point2::new(x, 0.0), dir)).cast();
    Handle::new(Geom2dAdaptorCurve::with_range(&line, v0, v1).unwrap()).cast()
}

fn free(x: f64, para: f64) -> IntPatchPoint {
    let mut p = IntPatchPoint::new();
    p.set_value_tangent(Point3::new(x, 0.0, 0.0), 1.0e-7, false);
    p.set_parameters(x, 0.0, x, 1.0);
    p.set_parameter(para);
    p
}

fn polyline(n: i32) -> Handle<IntSurfLineOn2S> {
    let line = IntSurfLineOn2S::new(None);
    for i in 0..n {
        let x = 0.1 * f64::from(i);
        let mut p = IntSurfPntOn2S::new();
        p.set_value_all(Point3::new(x, 0.0, 0.0), x, 0.0, x, 1.0);
        line.add(&p);
    }
    Handle::new(line)
}

#[test]
fn point_arcs_reverse_their_crossing_transitions_only() {
    let arc = vertical_arc(1.0, 0.0, 1.0);
    let mut p = IntPatchPoint::new();
    let (tin, touch) = (
        IntSurfTransition::with_type(true, IntSurfTypeTrans::In),
        IntSurfTransition::touch(
            true,
            rustycad_tkgeomalgo::int_surf::IntSurfSituation::Inside,
            false,
        ),
    );
    p.set_arc(true, &arc, 0.5, &tin, &touch);
    assert!(p.is_on_dom_s1() && !p.is_vertex_on_s1());
    assert!(p.arc_on_s1().unwrap() == arc);
    p.reverse_transition();
    let line = p.transition_line_arc1().unwrap();
    assert_eq!(line.transition_type(), IntSurfTypeTrans::Out);
    assert!(!line.is_tangent().unwrap());
    // A TOUCH transition becomes a fresh, undecided one.
    assert_eq!(
        p.transition_on_s1().unwrap().transition_type(),
        IntSurfTypeTrans::Undecided
    );
    // SetValue(Pt, Tol, Tangent) clears the domain flags but keeps the arc.
    p.set_value_tangent(Point3::new(0.0, 0.0, 0.0), 1.0e-7, true);
    assert!(!p.is_on_dom_s1() && p.arc_on_s1().is_err());
    assert_eq!(
        p.parameter_on_arc2().unwrap_err().exception_type(),
        "Standard_DomainError"
    );
}

#[test]
fn point_dump_prints_the_native_text() {
    let mut text = String::new();
    free(0.5, 2.0).dump(&mut text).unwrap();
    assert!(text.starts_with("----------- IntPatch_Point : \nP(+0.50000000000000000000,"));
    assert!(text.ends_with("(Para:+2.00000000000000000000)\n\nToler = 1e-07\n"));
}

#[test]
fn wline_inserts_the_vertices_and_adds_its_end_points() {
    let line = IntPatchWLine::new_undecided(&polyline(4), false);
    // One vertex on the point 2, one between the points 3 and 4.
    line.add_vertex(&free(0.1, 2.0), false);
    line.add_vertex(&free(0.25, 3.0), false);
    line.compute_vertex_parameters(1.0e-7).unwrap();
    assert_eq!(line.nb_pnts().unwrap(), 5);
    assert_eq!(line.nb_vertex(), 4);
    let params: Vec<f64> = (1..=4)
        .map(|i| line.vertex(i).unwrap().parameter_on_line())
        .collect();
    assert_eq!(params, [1.0, 2.0, 4.0, 5.0]);
    assert_eq!(line.first_point_index().unwrap().1, 1);
    assert_eq!(line.last_point_index().unwrap().1, 4);
    assert_eq!(line.point(4).unwrap().value(), Point3::new(0.25, 0.0, 0.0));
}

#[test]
fn wline_vertex_edits_check_their_indices() {
    let line = IntPatchWLine::new_undecided(&polyline(2), false);
    line.add_vertex(&free(0.0, 1.0), false);
    line.add_vertex(&free(0.1, 2.0), true);
    assert_eq!(line.vertex(1).unwrap().parameter_on_line(), 2.0);
    assert_eq!(
        line.remove_vertex(3).unwrap_err().exception_type(),
        "Standard_OutOfRange"
    );
    line.change_vertex(2, &mut |p| p.set_parameter(7.0))
        .unwrap();
    assert_eq!(line.vertex(2).unwrap().parameter_on_line(), 7.0);
    // Past the vertices, InsertVertexBefore appends; below 1 it raises.
    line.insert_vertex_before(9, &free(0.2, 3.0)).unwrap();
    assert_eq!(line.nb_vertex(), 3);
    assert!(line.insert_vertex_before(0, &free(0.2, 3.0)).is_err());
    let empty = IntPatchWLine::new_undecided(&Handle::default(), false);
    assert_eq!(
        empty.nb_pnts().unwrap_err().exception_type(),
        "Standard_NullObject"
    );
}

#[test]
fn rline_keeps_one_vertex_per_arc_parameter() {
    let arc = vertical_arc(0.0, 0.0, 1.0);
    let line = IntPatchRLine::new_undecided(false);
    let tr = IntSurfTransition::new();
    for para in [3.0, 1.0, 3.0] {
        let mut p = free(0.0, para);
        p.set_arc(true, &arc, 0.5, &tr, &tr);
        line.add_vertex(&p, false);
    }
    line.set_arc_on_s1(&arc);
    line.compute_vertex_parameters(0.0).unwrap();
    assert_eq!(line.nb_vertex(), 2);
    assert_eq!(line.param_on_s1(), (1.0, 3.0));
    assert_eq!(line.param_on_s2(), (0.0, 0.0));
    assert_eq!(line.last_point().unwrap().parameter_on_line(), 3.0);
    assert_eq!(
        line.nb_pnts().unwrap_err().exception_type(),
        "Standard_DomainError"
    );
}

#[test]
fn gline_wraps_circle_vertices_and_replace_appends() {
    let circle = Circle3::new(frame(0.0).ax2().unwrap(), 1.0).unwrap();
    let line = IntPatchGLine::from_circle(&circle, false, IntPatchLineTransitions::Undecided);
    line.add_vertex(&free(1.0, 1.0)).unwrap();
    line.add_vertex(&free(1.0, 2.0)).unwrap();
    line.set_first_point(1);
    line.set_last_point(2);
    // 1.5 + 2 Pi is brought back to 1.5; 0.5 stays before the first point: dropped.
    line.add_vertex(&free(1.0, 1.5 + 2.0 * PI)).unwrap();
    line.add_vertex(&free(1.0, 0.5)).unwrap();
    assert_eq!(line.nb_vertex(), 3);
    assert_eq!(line.vertex(3).unwrap().parameter_on_line(), 1.5);
    line.replace(1, &free(1.0, 9.0));
    assert_eq!(line.nb_vertex(), 4);
    assert_eq!(line.circle().unwrap().radius(), 1.0);
    assert_eq!(
        line.line().unwrap_err().exception_type(),
        "Standard_DomainError"
    );
}

#[test]
fn aline_reads_its_analytic_curve() {
    use rustycad_tkgeombase::int_ana::{IntQuadQuad, Quadric};
    use rustycad_tkmath::gp::Cylinder3;
    let cyl = Cylinder3::new(frame(0.0), 1.0).unwrap();
    let sphere = Sphere3::new(
        Axis3::from_normal(
            Point3::new(0.5, 0.0, 0.0),
            Direction3::new(0.0, 0.0, 1.0).unwrap(),
        )
        .unwrap(),
        1.2,
    )
    .unwrap();
    let inter = IntQuadQuad::cylinder(cyl, &Quadric::from_sphere(sphere), 1.0e-8).unwrap();
    let line = IntPatchALine::new(
        inter.curve(1).unwrap(),
        false,
        IntPatchLineTransitions::Undecided,
    );
    let (first, _) = line.first_parameter().unwrap();
    let p = line.value(first).unwrap();
    // The point is on the cylinder and on the sphere.
    assert!((p.x().hypot(p.y()) - 1.0).abs() < 1.0e-7);
    assert!((p.distance(&Point3::new(0.5, 0.0, 0.0)) - 1.2).abs() < 1.0e-7);
    line.add_vertex(&free(0.0, first));
    line.compute_vertex_parameters(1.0e-7).unwrap();
    assert_eq!(line.nb_vertex(), 1);
}

#[test]
fn lines_are_viewed_through_their_abstract_handles() {
    let w: Handle<dyn IntPatchLine> =
        Handle::new(IntPatchWLine::new_undecided(&polyline(2), true)).cast();
    assert_eq!(w.dynamic_type().unwrap().name(), "IntPatch_WLine");
    assert!(w.is_kind_name("IntPatch_PointLine") && w.is_kind_name("IntPatch_Line"));
    assert!(w.is_tangent() && w.arc_type() == IntPatchIType::Walking);
    let pl: Handle<dyn IntPatchPointLine> = w.cast();
    assert_eq!(pl.nb_pnts().unwrap(), 2);
    let g: Handle<dyn IntPatchLine> = Handle::new(IntPatchRLine::new_undecided(false)).cast();
    assert!(g.cast::<IntPatchWLine>().is_null());
    assert_eq!(
        <dyn IntPatchLine as RuntimeTyped>::static_type().name(),
        "IntPatch_Line"
    );
}

#[test]
fn tools_sample_and_intersect() {
    let plane = adapted(Handle::new(GeomPlane::new(frame(0.5))).cast());
    assert_eq!(
        IntPatchHInterTool::nb_samples_u(&plane, 0.0, 1.0).unwrap(),
        2
    );
    let geom: Handle<dyn GeomSurface> = Handle::new(GeomPlane::new(frame(0.5))).cast();
    let square: Adaptor3dSurfaceHandle =
        Handle::new(GeomAdaptorSurface::with_bounds(&geom, (0.0, 1.0, 0.0, 1.0)).unwrap()).cast();
    let mut tool = IntPatchHInterTool::new();
    // A 2 x 2 sampled plane has one grid cell: the fixed points on the square.
    assert_eq!(tool.nb_sample_points(&square).unwrap(), 4);
    assert_eq!(tool.sample_point(&square, 1).unwrap(), (0.24, 0.26));
    let arc = vertical_arc(0.0, -2.0, 2.0);
    assert_eq!(IntPatchHCurve2dTool::nb_samples(&arc, 0.0, 1.0).unwrap(), 2);
    assert_eq!(IntPatchHInterTool::bounds(&arc).unwrap(), (-2.0, 2.0));
    // The vertical line u = 0 of the unit cylinder meets the plane z = 0.5 at (1, 0, 0.5).
    let cylinder =
        adapted(Handle::new(GeomCylindricalSurface::new(frame(0.0), 1.0).unwrap()).cast());
    let f = IntPatchCSFunction::new(&plane, &arc, &cylinder);
    let mut inter = IntPatchCurvIntSurf::new([0.9, 0.1, 0.3], &f, 1.0e-7, 0.0).unwrap();
    assert!(inter.is_done() && !inter.is_empty().unwrap());
    assert!(inter.point().unwrap().distance(&Point3::new(1.0, 0.0, 0.5)) < 1.0e-6);
    assert!((inter.parameter_on_curve().unwrap() - 0.5).abs() < 1.0e-6);
    assert!(inter.function().root() < 1.0e-12);
}

#[test]
fn arc_function_is_the_quadric_distance_along_the_arc() {
    let plane = adapted(Handle::new(GeomPlane::new(frame(0.0))).cast());
    let sphere = IntSurfQuadric::from_sphere(&Sphere3::new(frame(0.0), 1.0).unwrap());
    let mut f = IntPatchArcFunction::new();
    f.set_quadric(&sphere);
    f.set_surface(&plane);
    f.set_arc(&vertical_arc(0.0, -2.0, 2.0));
    assert!(f.value_at(2.0).unwrap().abs() - 1.0 < 1.0e-12);
    assert!((f.derivative_at(2.0).unwrap() - 1.0).abs() < 1.0e-12);
    assert_eq!(f.get_state_number(), 1);
    assert_eq!(f.valpoint(1).unwrap(), Point3::new(0.0, 2.0, 0.0));
    assert!(f.valpoint(2).is_err());
    // The plane and the straight arc sample 2 points each.
    assert_eq!(f.nb_samples().unwrap(), 2);
}

#[test]
fn the_curvature_of_a_plane_section_of_a_sphere_is_its_radius() {
    let plane = adapted(Handle::new(GeomPlane::new(frame(0.0))).cast());
    let sphere = adapted(Handle::new(GeomSphericalSurface::new(frame(0.0), 2.0).unwrap()).cast());
    let mut p = IntSurfPntOn2S::new();
    p.set_value_all(Point3::new(2.0, 0.0, 0.0), 2.0, 0.0, 0.0, 0.0);
    let radius = curvature_radius_of_inters_line(&plane, &sphere, &p).unwrap();
    assert!((radius - 2.0).abs() < 1.0e-9, "{radius}");
    // Parallel planes: no tangent, -1.
    let other = adapted(Handle::new(GeomPlane::new(frame(1.0))).cast());
    assert_eq!(
        curvature_radius_of_inters_line(&plane, &other, &p).unwrap(),
        -1.0
    );
}
