//! Direct translation of Geom2dHatch_Elements_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
#[path = "../../tkg2d/tests/common/mod.rs"]
mod common;
use common::EXPECT_NEAR;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::{Geom2dCircle, Geom2dCurve};
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkgeomalgo::geom2d_hatch::{Element, Elements};
use rustycad_tkmath::gp::{Axis2d, Direction2, Point2};

fn makeCircleElement() -> Element {
    let anAxis = Axis2d::new(Point2::new(0.0, 0.0), Direction2::new(1.0, 0.0).unwrap());
    let aCircle: Handle<dyn Geom2dCurve> =
        Handle::new(Geom2dCircle::from_axis(anAxis, 1.0, true).unwrap()).cast();
    let aCurve = Geom2dAdaptorCurve::with_curve(&aCircle).unwrap();
    Element::new(aCurve, Orientation::Forward)
}

// Regression test for bug #38: CurrentEdge in const context must work
// without void* const-cast hack. Verifies that edge traversal returns
// valid curve and orientation after Bind + InitWires/InitEdges.
#[test]
fn Geom2dHatch_Elements__CurrentEdge_ReturnsValidData() {
    let mut anElements = Elements::new();
    anElements.bind(1, makeCircleElement());

    anElements.init_wires();
    assert!(anElements.more_wires());

    anElements.init_edges();
    assert!(anElements.more_edges());

    let (aEdge, anOr) = anElements.current_edge().unwrap();

    assert_eq!(anOr, Orientation::Forward);
    // Circle has parameter range [0, 2*PI].
    EXPECT_NEAR(aEdge.first_parameter().unwrap(), 0.0, 1e-10);
    assert!(aEdge.last_parameter().unwrap() > 6.0);
}

// Bind/Find/IsBound work correctly.
#[test]
fn Geom2dHatch_Elements__BindAndFind() {
    let mut anElements = Elements::new();
    assert!(!anElements.is_bound(1));

    anElements.bind(1, makeCircleElement());
    assert!(anElements.is_bound(1));

    let anElem = anElements.find(1).unwrap();
    assert_eq!(anElem.orientation(), Orientation::Forward);
}

// Clear removes all elements.
#[test]
fn Geom2dHatch_Elements__Clear_RemovesAll() {
    let mut anElements = Elements::new();
    anElements.bind(1, makeCircleElement());
    anElements.bind(2, makeCircleElement());
    assert!(anElements.is_bound(1));

    anElements.clear();
    assert!(!anElements.is_bound(1));
    assert!(!anElements.is_bound(2));
}
