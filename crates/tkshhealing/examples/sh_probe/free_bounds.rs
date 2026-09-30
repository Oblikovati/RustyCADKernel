//! Native boundary chaining, splitting and connection-status differential requests.
use super::{
    brep_lib_common::shape,
    common::{Reader, attempt, end, flag, int, real},
    shapes::{TpShapes, tp_shapes},
    wire_data::sh_shape,
};
use rustycad_tkbrep::topods::{self, Compound, Shape};
use rustycad_tkernel::standard::Failure;
use rustycad_tkshhealing::{
    shape_analysis::{ShapeAnalysisFreeBounds, ShapeAnalysisWire},
    shape_extend::ShapeExtendStatus,
};
pub fn free_bounds_case(values: &[f64]) {
    let mut reader = Reader::new(values);
    attempt(|| {
        let table = tp_shapes(&mut reader)?;
        boundary_request(&table, &mut reader)
    });
    end();
}
fn boundary_request(table: &TpShapes, reader: &mut Reader<'_>) -> Result<(), Failure> {
    let mode = reader.integer();
    let tolerance = reader.real();
    let shared = reader.flag();
    let mut wires = (0..reader.integer())
        .map(|_| sh_shape(table, reader.integer()))
        .collect::<Vec<_>>();
    if mode <= 2 || (8..=10).contains(&mode) {
        chain_request(table, &mut wires, mode, tolerance, shared)?;
    } else {
        boundary_other(&wires, mode, tolerance, shared, reader)?;
    }
    print_boundary_shapes(&wires)
}
fn boundary_other(
    wires: &[Shape],
    mode: i32,
    tolerance: f64,
    shared: bool,
    reader: &mut Reader<'_>,
) -> Result<(), Failure> {
    match mode {
        3 | 11 => split_request(wires, tolerance, shared, mode == 11),
        4 => dispatch_request(wires),
        7 => connection_request(wires, tolerance, reader),
        12 => print_boundary_result(&ShapeAnalysisFreeBounds::new()),
        _ => constructor_request(wires, mode, tolerance, shared, reader),
    }
}
fn dispatch_request(wires: &[Shape]) -> Result<(), Failure> {
    let mut closed = Compound::default();
    let mut open = Compound::default();
    ShapeAnalysisFreeBounds::dispatch_wires(wires, &mut closed, &mut open)?;
    shape(&closed)?;
    shape(&open)
}
fn constructor_request(
    wires: &[Shape],
    mode: i32,
    tolerance: f64,
    shared: bool,
    reader: &mut Reader<'_>,
) -> Result<(), Failure> {
    let split_closed = reader.flag();
    let split_open = reader.flag();
    let result = if mode == 5 {
        ShapeAnalysisFreeBounds::forecast(&wires[0], tolerance, split_closed, split_open)?
    } else {
        ShapeAnalysisFreeBounds::from_shape(&wires[0], split_closed, split_open, shared)?
    };
    print_boundary_result(&result)
}
fn print_boundary_result(result: &ShapeAnalysisFreeBounds) -> Result<(), Failure> {
    shape(result.closed_wires())?;
    shape(result.open_wires())
}
fn chain_request(
    table: &TpShapes,
    wires: &mut [Shape],
    mode: i32,
    tolerance: f64,
    shared: bool,
) -> Result<(), Failure> {
    let mut vertices = Vec::new();
    let result = if mode >= 8 {
        chain_legacy(wires, mode, tolerance, shared, &mut vertices)
    } else {
        chain_return(wires, mode, tolerance, shared, &mut vertices)?
    };
    print_boundary_shapes(&result)?;
    if mode == 2 || mode == 10 {
        print_boundary_replacements(table, &vertices)?;
    }
    Ok(())
}
fn chain_return(
    wires: &mut [Shape],
    mode: i32,
    tolerance: f64,
    shared: bool,
    vertices: &mut Vec<(Shape, Shape)>,
) -> Result<Vec<Shape>, Failure> {
    match mode {
        0 => ShapeAnalysisFreeBounds::connect_edges_to_wires(wires, tolerance, shared),
        1 => ShapeAnalysisFreeBounds::connect_wires_to_wires(wires, tolerance, shared),
        _ => {
            ShapeAnalysisFreeBounds::connect_wires_with_vertices(wires, tolerance, shared, vertices)
        }
    }
}
fn chain_legacy(
    wires: &mut [Shape],
    mode: i32,
    tolerance: f64,
    shared: bool,
    vertices: &mut Vec<(Shape, Shape)>,
) -> Vec<Shape> {
    let mut result = wires.first().cloned().into_iter().collect();
    attempt(|| match mode {
        8 => ShapeAnalysisFreeBounds::connect_edges_into(wires, tolerance, shared, &mut result),
        9 => ShapeAnalysisFreeBounds::connect_wires_into(wires, tolerance, shared, &mut result),
        _ => ShapeAnalysisFreeBounds::connect_wires_with_vertices_into(
            wires,
            tolerance,
            shared,
            &mut result,
            vertices,
        ),
    });
    result
}
fn print_boundary_replacements(
    table: &TpShapes,
    vertices: &[(Shape, Shape)],
) -> Result<(), Failure> {
    for original in &table.shapes {
        let found = vertices.iter().find(|(key, _)| key.is_same(original));
        flag(found.is_some());
        if let Some((_, new)) = found {
            shape(new)?;
        }
    }
    Ok(())
}
fn split_request(
    wires: &[Shape],
    tolerance: f64,
    shared: bool,
    output: bool,
) -> Result<(), Failure> {
    let (closed, open) = if output {
        let (mut closed, mut open) = (wires.to_vec(), wires.to_vec());
        attempt(|| {
            ShapeAnalysisFreeBounds::split_wires_into(
                wires,
                tolerance,
                shared,
                &mut closed,
                &mut open,
            )
        });
        (closed, open)
    } else {
        ShapeAnalysisFreeBounds::split_wires(wires, tolerance, shared)?
    };
    print_boundary_shapes(&closed)?;
    print_boundary_shapes(&open)
}
fn print_boundary_shapes(shapes: &[Shape]) -> Result<(), Failure> {
    int(shapes.len() as i32);
    for item in shapes {
        shape(item)?;
    }
    Ok(())
}
fn connection_request(
    wires: &[Shape],
    tolerance: f64,
    reader: &mut Reader<'_>,
) -> Result<(), Failure> {
    let analyzer = ShapeAnalysisWire::new();
    if !wires.is_empty() {
        analyzer.load_wire(&topods::wire(&wires[0])?)?;
    }
    analyzer.set_precision(tolerance);
    for _ in 0..reader.integer() {
        let index = reader.integer();
        let precision = reader.real();
        attempt(|| {
            flag(if index == -99 {
                analyzer.check_connected(precision)?
            } else {
                analyzer.check_connected_at(index, precision)?
            });
            connection_status(&analyzer);
            Ok(())
        });
    }
    Ok(())
}
fn connection_status(analyzer: &ShapeAnalysisWire) {
    real(analyzer.min_distance_3d());
    for status in [
        ShapeExtendStatus::Ok,
        ShapeExtendStatus::Done,
        ShapeExtendStatus::Done1,
        ShapeExtendStatus::Done2,
        ShapeExtendStatus::Done3,
        ShapeExtendStatus::Fail,
        ShapeExtendStatus::Fail1,
        ShapeExtendStatus::Fail2,
    ] {
        flag(analyzer.last_check_status(status));
        flag(analyzer.status_connected(status));
    }
}
