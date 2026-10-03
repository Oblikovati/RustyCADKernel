//! Pinned OCCT `ShapeFix_WireVertex` state transitions.
use rustycad_tkbrep::{
    brep::{BRepBuilder, BRepTool},
    topods::{Edge, Vertex},
};
use rustycad_tkernel::standard::Handle;
use rustycad_tkmath::gp::{Point3, Xyz};
use rustycad_tkshhealing::{
    shape_analysis::{ShapeAnalysisEdge, ShapeAnalysisWireVertex},
    shape_extend::ShapeExtendWireData,
    shape_fix::ShapeFixWireVertex,
};
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeEdge;

#[test]
fn fix_reconnects_projected_junctions_and_preserves_start_slot_quirk()
-> Result<(), rustycad_tkernel::standard::Failure> {
    for (status, x) in [(3, 1.1), (4, 0.9), (5, 1.2)] {
        let (mut analysis, data) = analyzed_pair()?;
        set_status(&mut analysis, status)?;
        let mut fixer = ShapeFixWireVertex::new();
        fixer.init_from_analyzer(&analysis);
        assert_eq!(fixer.fix()?, 2);
        assert_eq!(analysis.status(1)?, 0);
        assert_shared_endpoints(&data, x)?;
    }
    mixed_positive_and_nonpositive_statuses_keep_native_vertex_orientations();
    Ok(())
}

#[test]
fn fix_same_keeps_native_old_endpoint_and_unloaded_calls_are_noops() {
    for close in [false, true] {
        let (mut analysis, data) = analyzed_pair().unwrap();
        set_same_status(&mut analysis, close);
        let mut fixer = ShapeFixWireVertex::new();
        fixer.init_from_analyzer(&analysis);
        assert_eq!(fixer.fix_same().unwrap(), 1);
        assert_eq!(analysis.status(1).unwrap(), 0);
        assert_not_reconnected(&data, close);
    }
    let mut unloaded = ShapeFixWireVertex::new();
    assert_eq!(unloaded.fix().unwrap(), 0);
    assert_eq!(unloaded.fix_same().unwrap(), 0);
}

#[test]
fn fix_same_compares_vertex_orientation_as_native_shape_equality_does() {
    let data = shared_endpoint_pair().unwrap();
    let mut analysis = ShapeAnalysisWireVertex::new();
    analysis.init_wire_data(&data, 1e-7);
    analysis.set_precision(1e-7);
    analysis.analyze().unwrap();
    analysis.set_same_coords(1).unwrap();
    let mut fixer = ShapeFixWireVertex::new();
    fixer.init_from_analyzer(&analysis);
    assert_eq!(fixer.fix_same().unwrap(), 1);
    assert_eq!(analysis.status(1).unwrap(), 0);
}

#[test]
fn close_fix_same_uses_oriented_curve_ranges_on_reversed_edges() {
    for (reverse_first, reverse_second, x1, x2, u1, u2) in [
        (true, false, 0.0, 1.0001, 0.0, 0.0),
        (false, true, 1.0, 2.0, 1.0, 0.9999),
    ] {
        let (mut analysis, data) = reversed_analyzed_pair(reverse_first, reverse_second).unwrap();
        analysis.set_close(1).unwrap();
        let mut fixer = ShapeFixWireVertex::new();
        fixer.init_from_analyzer(&analysis);
        assert_eq!(fixer.fix_same().unwrap(), 1);
        assert_reversed_join(&data, (x1, x2), (u1, u2), 1e-3);
    }
}

#[test]
fn fix_uses_oriented_curve_ranges_on_reversed_edges() {
    let (mut analysis, data) = reversed_analyzed_pair(true, true).unwrap();
    analysis.set_end(1, Xyz::new(1.1, 0.0, 0.0), 0.25).unwrap();
    let mut fixer = ShapeFixWireVertex::new();
    fixer.init_from_analyzer(&analysis);
    assert_eq!(fixer.fix().unwrap(), 2);
    assert_shared_join(&data, 1.1, (0.0, 0.9999));
}

fn set_same_status(analysis: &mut ShapeAnalysisWireVertex, close: bool) {
    if close {
        analysis.set_close(1).unwrap();
    } else {
        analysis.set_same_coords(1).unwrap();
    }
}

fn mixed_positive_and_nonpositive_statuses_keep_native_vertex_orientations() {
    let (mut analysis, data) = analyzed_pair().unwrap();
    analysis.set_end(1, Xyz::new(1.1, 0.0, 0.0), 0.25).unwrap();
    analysis.set_same_vertex(2).unwrap();
    let mut fixer = ShapeFixWireVertex::new();
    fixer.init_from_analyzer(&analysis);
    assert_eq!(fixer.fix().unwrap(), 1);
    assert_eq!(analysis.status(1).unwrap(), 0);
    assert_eq!(analysis.status(2).unwrap(), 0);
    let edges = ShapeAnalysisEdge::new();
    assert_eq!(
        edges.last_vertex(&data.edge(2).unwrap()).orientation(),
        rustycad_tkg3d::top_abs::Orientation::Reversed
    );
    assert_eq!(
        edges.first_vertex(&data.edge(1).unwrap()).orientation(),
        rustycad_tkg3d::top_abs::Orientation::External
    );
    assert_eq!(data.nb_edges(), 2);
}

#[test]
fn disjoined_fixer_run_leaves_the_wire_unchanged() {
    disjoined_junction_is_reset_when_another_record_triggers_fix();
    disjoined_no_curve_wire_returns_before_edge_capture();
    let (mut analysis, data) = analyzed_pair().unwrap();
    analysis.set_disjoined(1).unwrap();
    analysis.set_disjoined(2).unwrap();
    let mut fixer = ShapeFixWireVertex::new();
    fixer.init_from_analyzer(&analysis);
    assert_eq!(fixer.fix().unwrap(), 0);
    assert_eq!(analysis.status(1).unwrap(), -1);
    assert_eq!(analysis.status(2).unwrap(), -1);
    assert_not_reconnected(&data, false);
}

fn disjoined_no_curve_wire_returns_before_edge_capture() {
    let (mut analysis, data) = analyzed_pair().unwrap();
    analysis.set_disjoined(1).unwrap();
    analysis.set_disjoined(2).unwrap();
    rustycad_tkbrep::brep::BRepBuilder::default()
        .update_edge_curve(&data.edge(1).unwrap(), &Handle::default(), 0.0)
        .unwrap();
    let mut fixer = ShapeFixWireVertex::new();
    fixer.init_from_analyzer(&analysis);
    assert_eq!(fixer.fix().unwrap(), 0);
    assert_eq!(analysis.status(1).unwrap(), -1);
}

fn disjoined_junction_is_reset_when_another_record_triggers_fix() {
    let (mut analysis, _) = analyzed_pair().unwrap();
    analysis.set_disjoined(1).unwrap();
    let mut fixer = ShapeFixWireVertex::new();
    fixer.init_from_analyzer(&analysis);
    assert_eq!(fixer.fix().unwrap(), 1);
    assert_eq!(analysis.status(1).unwrap(), 0);
}

fn analyzed_pair() -> Result<
    (ShapeAnalysisWireVertex, Handle<ShapeExtendWireData>),
    rustycad_tkernel::standard::Failure,
> {
    let data = Handle::new(ShapeExtendWireData::new());
    data.add_edge(&line(0.0, 1.0)?, 0)?;
    data.add_edge(&line(1.0001, 2.0)?, 0)?;
    let mut analysis = ShapeAnalysisWireVertex::new();
    analysis.init_wire_data(&data, 0.75);
    analysis.set_precision(1e-3);
    analysis.analyze()?;
    Ok((analysis, data))
}

fn shared_endpoint_pair() -> Result<Handle<ShapeExtendWireData>, rustycad_tkernel::standard::Failure>
{
    let builder = BRepBuilder::default();
    let mut start = Vertex::default();
    let mut shared = Vertex::default();
    let mut finish = Vertex::default();
    builder.make_vertex_at(&mut start, Point3::new(0.0, 0.0, 0.0), 1e-7)?;
    builder.make_vertex_at(&mut shared, Point3::new(1.0, 0.0, 0.0), 1e-7)?;
    builder.make_vertex_at(&mut finish, Point3::new(2.0, 0.0, 0.0), 1e-7)?;
    let data = Handle::new(ShapeExtendWireData::new());
    let first = BRepBuilderAPIMakeEdge::from_vertices(&start, &shared)?.edge()?;
    let second = BRepBuilderAPIMakeEdge::from_vertices(&shared, &finish)?.edge()?;
    data.add_edge(&first, 0)?;
    data.add_edge(&second, 0)?;
    Ok(data)
}

fn reversed_analyzed_pair(
    reverse_first: bool,
    reverse_second: bool,
) -> Result<
    (ShapeAnalysisWireVertex, Handle<ShapeExtendWireData>),
    rustycad_tkernel::standard::Failure,
> {
    let mut first = line(0.0, 1.0)?;
    let mut second = line(1.0001, 2.0)?;
    if reverse_first {
        first.reverse();
    }
    if reverse_second {
        second.reverse();
    }
    let data = Handle::new(ShapeExtendWireData::new());
    data.add_edge(&first, 0)?;
    data.add_edge(&second, 0)?;
    let mut analysis = ShapeAnalysisWireVertex::new();
    analysis.init_wire_data(&data, 0.75);
    analysis.set_precision(1e-3);
    analysis.analyze()?;
    Ok((analysis, data))
}

fn line(start: f64, end: f64) -> Result<Edge, rustycad_tkernel::standard::Failure> {
    BRepBuilderAPIMakeEdge::from_points(Point3::new(start, 0.0, 0.0), Point3::new(end, 0.0, 0.0))?
        .edge()
}

fn set_status(
    analysis: &mut ShapeAnalysisWireVertex,
    status: i32,
) -> Result<(), rustycad_tkernel::standard::Failure> {
    match status {
        3 => analysis.set_end(1, Xyz::new(1.1, 0.0, 0.0), 0.25),
        4 => analysis.set_start(1, Xyz::new(0.9, 0.0, 0.0), 0.75),
        _ => analysis.set_inters(1, Xyz::new(1.2, 0.0, 0.0), 0.6, 0.4),
    }
}

fn assert_shared_endpoints(
    data: &Handle<ShapeExtendWireData>,
    x: f64,
) -> Result<(), rustycad_tkernel::standard::Failure> {
    let (previous, following) = endpoints(data)?;
    let previous_edge = data.edge(1)?;
    let following_edge = data.edge(2)?;
    assert!(previous.is_same(&following));
    assert!((BRepTool::pnt(&previous)?.x() - x).abs() < 1e-12);
    assert_eq!(BRepTool::vertex_tolerance(&previous)?, 1e-3);
    assert_eq!(BRepTool::parameter(&previous, &previous_edge)?, 1.0);
    assert_eq!(BRepTool::parameter(&following, &following_edge)?, 0.0);
    Ok(())
}

fn assert_reversed_join(
    data: &Handle<ShapeExtendWireData>,
    points: (f64, f64),
    parameters: (f64, f64),
    previous_tolerance: f64,
) {
    let (previous, following) = endpoints(data).unwrap();
    let previous_edge = data.edge(1).unwrap();
    let following_edge = data.edge(2).unwrap();
    assert!(!previous.is_same(&following));
    assert_eq!(BRepTool::pnt(&previous).unwrap().x(), points.0);
    assert_eq!(BRepTool::pnt(&following).unwrap().x(), points.1);
    assert_eq!(
        BRepTool::vertex_tolerance(&previous).unwrap(),
        previous_tolerance
    );
    assert_eq!(
        BRepTool::parameter(&previous, &previous_edge).unwrap(),
        parameters.0
    );
    assert_eq!(
        BRepTool::parameter(&following, &following_edge).unwrap(),
        parameters.1
    );
}

fn assert_shared_join(data: &Handle<ShapeExtendWireData>, x: f64, parameters: (f64, f64)) {
    let (previous, following) = endpoints(data).unwrap();
    assert!(previous.is_same(&following));
    assert_eq!(BRepTool::pnt(&previous).unwrap().x(), x);
    assert_eq!(
        BRepTool::parameter(&previous, &data.edge(1).unwrap()).unwrap(),
        parameters.0
    );
    assert_eq!(
        BRepTool::parameter(&following, &data.edge(2).unwrap()).unwrap(),
        parameters.1
    );
}

fn assert_not_reconnected(data: &Handle<ShapeExtendWireData>, close: bool) {
    let (previous, following) = endpoints(data).unwrap();
    assert!(!previous.is_same(&following));
    assert_eq!(BRepTool::pnt(&previous).unwrap().x(), 1.0);
    assert_eq!(BRepTool::pnt(&following).unwrap().x(), 1.0001);
    let expected_tolerance = if close { 1e-3 } else { 1e-7 };
    assert_eq!(
        BRepTool::vertex_tolerance(&previous).unwrap(),
        expected_tolerance
    );
    assert_eq!(BRepTool::vertex_tolerance(&following).unwrap(), 1e-7);
}

fn endpoints(
    data: &Handle<ShapeExtendWireData>,
) -> Result<(Vertex, Vertex), rustycad_tkernel::standard::Failure> {
    let edges = ShapeAnalysisEdge::new();
    Ok((
        edges.last_vertex(&data.edge(1)?),
        edges.first_vertex(&data.edge(2)?),
    ))
}
