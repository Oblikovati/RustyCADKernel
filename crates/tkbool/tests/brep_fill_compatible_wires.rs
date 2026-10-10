//! BRepFill_CompatibleWires: sections cut to the same edge count and oriented alike.
use rustycad_tkbool::brep_fill::{BRepFillCompatibleWires, ThruSectionErrorStatus};
use rustycad_tkbrep::brep_tools::WireExplorer;
use rustycad_tkbrep::topods::{self, Shape};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::Point3;
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakePolygon;

/// A polygon wire through the points, closed when asked.
fn polygon(points: &[(f64, f64, f64)], closed: bool) -> Result<Shape, Failure> {
    let mut maker = BRepBuilderAPIMakePolygon::new();
    for &(x, y, z) in points {
        maker.add_point(Point3::new(x, y, z))?;
    }
    if closed {
        maker.close()?;
    }
    Ok(maker.wire()?.into_shape())
}

/// The edge count of a section for the wire explorer.
fn edge_count(section: &Shape) -> usize {
    let mut explorer = WireExplorer::with_wire(&topods::wire(section).unwrap()).unwrap();
    let mut n = 0;
    while explorer.more() {
        n += 1;
        explorer.next().unwrap();
    }
    n
}

#[test]
fn a_square_and_a_triangle_get_the_same_edge_count() -> Result<(), Failure> {
    let square = polygon(
        &[(-1., -1., 0.), (1., -1., 0.), (1., 1., 0.), (-1., 1., 0.)],
        true,
    )?;
    let triangle = polygon(&[(1., 0., 1.), (-0.5, 0.866, 1.), (-0.5, -0.866, 1.)], true)?;
    let mut compatible = BRepFillCompatibleWires::with_sections(&[square, triangle]);
    compatible.perform(true)?;
    assert!(compatible.is_done());
    let counts: Vec<usize> = compatible.shape().iter().map(edge_count).collect();
    assert_eq!(counts[0], counts[1]);
    assert!(counts[0] >= 4);
    Ok(())
}

#[test]
fn open_and_closed_sections_are_not_processed() -> Result<(), Failure> {
    let square = polygon(
        &[(-1., -1., 0.), (1., -1., 0.), (1., 1., 0.), (-1., 1., 0.)],
        true,
    )?;
    let segment = polygon(&[(0., 0., 1.), (1., 0., 1.)], false)?;
    let mut compatible = BRepFillCompatibleWires::new();
    compatible.init(&[square, segment]);
    compatible.set_percent(2.);
    compatible.perform(true)?;
    assert_eq!(
        compatible.get_status(),
        ThruSectionErrorStatus::NotSameTopology
    );
    assert!(!compatible.is_done());
    Ok(())
}

#[test]
fn an_open_section_running_backwards_is_reversed() -> Result<(), Failure> {
    let first = polygon(&[(0., 0., 0.), (1., 1., 0.), (2., 0., 0.)], false)?;
    let second = polygon(&[(2., 0., 1.), (1., 1., 1.), (0., 0., 1.)], false)?;
    let mut compatible = BRepFillCompatibleWires::with_sections(&[first, second.clone()]);
    compatible.perform(true)?;
    assert!(compatible.is_done());
    assert!(!compatible.is_degenerated_first_section());
    let explorer = WireExplorer::with_wire(&topods::wire(&compatible.shape()[1])?)?;
    let start = rustycad_tkbrep::brep::BRepTool::pnt(explorer.current_vertex())?;
    assert_eq!(start, Point3::new(0., 0., 1.));
    assert_eq!(compatible.generated().extent(), 4);
    Ok(())
}
