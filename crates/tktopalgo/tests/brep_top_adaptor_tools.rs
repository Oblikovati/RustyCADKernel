// Copyright (c) 2026 OPEN CASCADE SAS
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

//! Behavior tests of BRepTopAdaptor_TopolTool, BRepTopAdaptor_HVertex,
//! BRepTopAdaptor_Tool and the FClass2d members the upstream suite does not reach. The
//! numeric probes (bta_* requests) compare them bit for bit with OCCT; these tests pin
//! the documented behavior on faces built by the probe's face reader.
mod adaptor_records;
#[allow(dead_code)]
#[path = "../examples/tp_probe/face.rs"]
mod face;
#[allow(dead_code)]
#[path = "../../tkg2d/examples/g2_probe/print.rs"]
mod print;

use adaptor_records::{CYLINDER, DISCONNECTED, HOLED};
use face::{TpFace, tp_face};
use print::Reader;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorSurface;
use rustycad_tkbrep::topods::Vertex;
use rustycad_tkernel::standard::Handle;
use rustycad_tkernel::standard::real::real_first;
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkg3d::adaptor3d::{Adaptor3dHVertex, Adaptor3dSurface};
use rustycad_tkg3d::top_abs::{Orientation, State};
use rustycad_tkmath::geom_abs::SurfaceType;
use rustycad_tkmath::gp::Point2;
use rustycad_tktopalgo::brep_top_adaptor::{
    BRepTopAdaptorFClass2d, BRepTopAdaptorHVertex, BRepTopAdaptorTool, BRepTopAdaptorTopolTool,
};

fn face(record: &[f64]) -> TpFace {
    tp_face(&mut Reader::new(record)).expect("face record")
}

fn tool(record: &[f64], restriction: bool) -> (TpFace, BRepTopAdaptorTopolTool) {
    let f = face(record);
    let surface = BRepAdaptorSurface::with_face(&f.face, restriction).unwrap();
    let handle: Handle<dyn Adaptor3dSurface> = Handle::new(surface).cast();
    let tool = BRepTopAdaptorTopolTool::with_surface(&handle).unwrap();
    (f, tool)
}

#[test]
fn topol_tool_walks_one_arc_per_edge_occurrence() {
    let (f, tool) = tool(&HOLED, true);
    assert!(!tool.more());
    tool.init();
    let mut count = 0;
    while tool.more() {
        let arc = tool.value().unwrap();
        assert!(
            f.edges
                .iter()
                .any(|edge| edge.is_same(&tool.edge().unwrap()))
        );
        // The REVERSED hole wire composes into the orientation of its edges.
        let expected = if count < 4 {
            Orientation::Forward
        } else {
            Orientation::Reversed
        };
        assert_eq!(tool.curve_orientation(&arc).unwrap(), expected);
        assert_eq!(tool.curve_tol_3d(&arc).unwrap(), 1e-7);
        tool.next();
        count += 1;
    }
    assert_eq!(count, 8);
    assert_eq!(
        tool.value().err().unwrap().exception_type(),
        "Standard_NoSuchObject"
    );
}

#[test]
fn topol_tool_vertices_are_brep_hvertices_of_the_arc() {
    let (_, tool) = tool(&HOLED, true);
    tool.init();
    let arc = tool.value().unwrap();
    tool.initialize_curve(&arc).unwrap();
    tool.init_vertex_iterator().unwrap();
    let first = tool.vertex().unwrap();
    assert_eq!(
        tool.vertex_orientation(&first).unwrap(),
        Orientation::Forward
    );
    assert_eq!(tool.vertex_tol_3d(&first).unwrap(), 1e-7);
    let brep: Handle<BRepTopAdaptorHVertex> = first.cast();
    let hvertex = brep.get().unwrap();
    assert_eq!(hvertex.value(), Point2::new(real_first(), real_first()));
    assert_eq!(hvertex.parameter(&arc).unwrap(), 0.0);
    assert!(hvertex.is_same(&first).unwrap());
    tool.next_vertex().unwrap();
    let last = tool.vertex().unwrap();
    assert_eq!(
        tool.vertex_orientation(&last).unwrap(),
        Orientation::Reversed
    );
    let last_brep: Handle<BRepTopAdaptorHVertex> = last.cast();
    assert_eq!(last_brep.get().unwrap().parameter(&arc).unwrap(), 12.0);
    tool.next_vertex().unwrap();
    assert!(!tool.more_vertex());
    // Past the last vertex the explorer's current shape is the edge.
    let error = tool.vertex().err().unwrap();
    assert_eq!(error.exception_type(), "Standard_TypeMismatch");
}

#[test]
fn topol_tool_samples_by_surface_kind() {
    let (_, plane) = tool(&HOLED, true);
    assert_eq!(
        (plane.nb_samples_u().unwrap(), plane.nb_samples_v().unwrap()),
        (10, 10)
    );
    assert_eq!(plane.nb_samples().unwrap(), 100);
    let (p2d, p3d) = plane.sample_point(0).unwrap();
    assert_eq!(p2d, Point2::new(12.0 / 11.0, 8.0 / 11.0));
    assert_eq!((p3d.x(), p3d.y(), p3d.z()), (12.0 / 11.0, 8.0 / 11.0, 0.0));
    assert!(!plane.domain_is_infinite().unwrap());
    let (_, cylinder) = tool(&CYLINDER, true);
    // (int)(8 * 2 pi) = 50 capped at 30, (int)(7 * 5) = 35 capped at 15.
    assert_eq!(cylinder.nb_samples_u().unwrap(), 30);
    assert_eq!(cylinder.nb_samples_v().unwrap(), 15);
}

#[test]
fn topol_tool_classifies_with_its_first_tolerance() {
    let (_, tool) = tool(&HOLED, true);
    assert_eq!(
        tool.classify(Point2::new(1.0, 1.0), 1e-7, true).unwrap(),
        State::In
    );
    assert_eq!(
        tool.classify(Point2::new(5.0, 4.0), 1e-7, true).unwrap(),
        State::Out
    );
    assert!(
        tool.is_the_point_on(Point2::new(0.0, 3.0), 1e-7, true)
            .unwrap()
    );
    assert!(
        !tool
            .is_the_point_on(Point2::new(1.0, 1.0), 1e-7, true)
            .unwrap()
    );
    tool.destroy();
    assert_eq!(
        tool.classify(Point2::new(14.0, 4.0), 1e-7, true).unwrap(),
        State::Out
    );
}

#[test]
fn topol_tool_errors_follow_the_native_exceptions() {
    let tool = BRepTopAdaptorTopolTool::new();
    assert!(tool.has_3d());
    assert_eq!(
        tool.classify(Point2::new(0.0, 0.0), 1e-7, true).unwrap(),
        State::Unknown
    );
    assert_eq!(
        tool.initialize().err().unwrap().exception_type(),
        "Standard_NotImplemented"
    );
    let curve: Handle<dyn Adaptor2dCurve2d> = Handle::new(Geom2dAdaptorCurve::default()).cast();
    let error = tool.initialize_curve(&curve).err().unwrap();
    assert_eq!(error.exception_type(), "Standard_ConstructionError");
    assert_eq!(
        tool.curve_tol_3d(&curve).err().unwrap().exception_type(),
        "Standard_DomainError"
    );
    assert_eq!(
        tool.init_vertex_iterator().err().unwrap().exception_type(),
        "Standard_NullObject"
    );
    let base = Handle::new(Adaptor3dHVertex::new(
        Point2::new(1.0, 2.0),
        Orientation::Reversed,
        0.5,
    ));
    assert_eq!(
        tool.vertex_orientation(&base).unwrap(),
        Orientation::Reversed
    );
    assert_eq!(
        tool.pnt(&base).err().unwrap().exception_type(),
        "Standard_DomainError"
    );
    let null_vertex = Handle::new(BRepTopAdaptorHVertex::new(
        &Vertex::default(),
        &Handle::default(),
    ));
    let error = tool.vertex_tol_3d(&null_vertex.cast()).err().unwrap();
    assert_eq!(error.exception_type(), "Standard_DomainError");
    assert_eq!(
        tool.sample_point(0).err().unwrap().exception_type(),
        "Standard_NullObject"
    );
}

#[test]
fn tool_pairs_a_restricted_surface_with_its_topology_tool() {
    let f = face(&HOLED);
    let tool = BRepTopAdaptorTool::with_face(&f.face, 1e-9).unwrap();
    assert!(tool.is_loaded());
    assert_eq!(tool.get_surface().get_type().unwrap(), SurfaceType::Plane);
    assert_eq!(tool.get_surface().last_u_parameter().unwrap(), 12.0);
    assert_eq!(tool.get_topol_tool().nb_samples().unwrap(), 100);
    let mut other = BRepTopAdaptorTool::new();
    assert!(!other.is_loaded());
    other.set_topol_tool(&tool.get_topol_tool());
    let same = other.get_topol_tool().object_address() == tool.get_topol_tool().object_address();
    assert!(same);
}

#[test]
fn fclass2d_answers_in_after_destroy_and_out_at_infinity() {
    let f = face(&HOLED);
    let mut classifier = BRepTopAdaptorFClass2d::new(&f.face, 1e-7).unwrap();
    assert_eq!(classifier.perform_infinite_point().unwrap(), State::Out);
    classifier.destroy();
    assert_eq!(
        classifier.perform(Point2::new(5.0, 4.0), true).unwrap(),
        State::In
    );
}

#[test]
fn fclass2d_classifies_exactly_on_a_bad_wire() {
    let f = face(&DISCONNECTED);
    let classifier = BRepTopAdaptorFClass2d::new(&f.face, 1e-7).unwrap();
    // The exact classifier sees the square: its boundary is ON with Perform too.
    assert_eq!(
        classifier.perform(Point2::new(0.0, 5.0), true).unwrap(),
        State::On
    );
    assert_eq!(
        classifier.perform(Point2::new(11.0, 5.0), true).unwrap(),
        State::Out
    );
}
