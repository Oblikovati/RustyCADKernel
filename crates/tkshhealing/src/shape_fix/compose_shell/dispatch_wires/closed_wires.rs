use crate::shape_build::ShapeBuildEdge;
use crate::shape_fix::{FixRoot, ShapeFixWire, ShapeFixWireSegment};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topods::{self, Edge, Face};
use rustycad_tkernel::ncollection::Sequence;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::gp::Vector2;

pub(super) fn prepare_closed_wires(
    composer: &super::super::ShapeFixComposeShell,
    wires: &mut Sequence<ShapeFixWireSegment>,
    face: &Face,
) -> Result<(), Failure> {
    if !composer.closed_mode() {
        return Ok(());
    }
    translate_reversed_seams(composer, wires, face)?;
    fix_wire_pcurves(composer, wires, face)
}

fn translate_reversed_seams(
    composer: &super::super::ShapeFixComposeShell,
    wires: &Sequence<ShapeFixWireSegment>,
    face: &Face,
) -> Result<(), Failure> {
    let fields = composer.fields.read();
    for wire in wires.iter().filter(|wire| !wire.is_vertex()) {
        for index in 1..=wire.nb_edges() {
            let edge = wire.edge(index)?;
            if edge.orientation() == Orientation::Reversed
                && BRepTool::is_closed_on_face(&edge, face)?
            {
                translate_seam_pair(
                    &edge,
                    face,
                    fields.u_closed,
                    fields.v_closed,
                    fields.u_period,
                    fields.v_period,
                )?;
            }
        }
    }
    Ok(())
}

fn translate_seam_pair(
    edge: &Edge,
    face: &Face,
    u_closed: bool,
    v_closed: bool,
    u_period: f64,
    v_period: f64,
) -> Result<(), Failure> {
    let first = BRepTool::curve_on_face(edge, face)?;
    let reversed = topods::edge(&edge.reversed())?;
    let second = BRepTool::curve_on_face(&reversed, face)?;
    if !same_or_near_pcurve(
        &first.pcurve,
        &second.pcurve,
        (first.first, first.last),
        (second.first, second.last),
    )? {
        return Ok(());
    }
    shift_seam_pcurve(
        second.pcurve,
        (second.first, second.last),
        u_closed,
        v_closed,
        u_period,
        v_period,
    )
}

fn same_or_near_pcurve(
    first: &Handle<dyn Geom2dCurve>,
    second: &Handle<dyn Geom2dCurve>,
    first_range: (f64, f64),
    second_range: (f64, f64),
) -> Result<bool, Failure> {
    if first == second {
        return Ok(true);
    }
    let first_curve = first.get().ok_or_else(null_pcurve)?;
    let second_curve = second.get().ok_or_else(null_pcurve)?;
    let tolerance = precision::p_confusion().powi(2);
    let starts_near = first_curve
        .value(first_range.0)?
        .square_distance(second_curve.value(second_range.0)?)
        < tolerance;
    let ends_near = first_curve
        .value(first_range.1)?
        .square_distance(second_curve.value(second_range.1)?)
        < tolerance;
    Ok(starts_near || ends_near)
}

fn shift_seam_pcurve(
    curve: Handle<dyn Geom2dCurve>,
    range: (f64, f64),
    u_closed: bool,
    v_closed: bool,
    u_period: f64,
    v_period: f64,
) -> Result<(), Failure> {
    let geometry = curve.get().ok_or_else(null_pcurve)?;
    let first = geometry.value(range.0)?;
    let last = geometry.value(range.1)?;
    let mut shift = Vector2::default();
    if u_closed && (first.x() - last.x()).abs() < precision::p_confusion() {
        shift.set_x(u_period);
    }
    if v_closed && (first.y() - last.y()).abs() < precision::p_confusion() {
        shift.set_y(v_period);
    }
    geometry.translate(shift)
}

fn fix_wire_pcurves(
    composer: &super::super::ShapeFixComposeShell,
    wires: &mut Sequence<ShapeFixWireSegment>,
    face: &Face,
) -> Result<(), Failure> {
    let mut index = 1;
    while index <= wires.length() as usize {
        let segment = wires.value(index)?.clone();
        if segment.is_vertex() {
            index += 1;
            continue;
        }
        fix_segment_pcurves(composer, &segment, face)?;
        if is_empty_or_single_degenerate(&segment)? {
            wires.remove(index)?;
        } else {
            index += 1;
        }
    }
    Ok(())
}

fn fix_segment_pcurves(
    composer: &super::super::ShapeFixComposeShell,
    segment: &ShapeFixWireSegment,
    face: &Face,
) -> Result<(), Failure> {
    let data = segment.wire_data().get().ok_or_else(null_wire)?;
    let fixer = ShapeFixWire::new();
    fixer.set_face(face)?;
    fixer.set_precision(composer.precision());
    fixer.load_wire_data(segment.wire_data())?;
    fixer.fix_shifted()?;
    remove_degenerate_pcurves(data, face)?;
    fixer.fix_degenerated()?;
    Ok(())
}

fn remove_degenerate_pcurves(
    data: &crate::shape_extend::ShapeExtendWireData,
    face: &Face,
) -> Result<(), Failure> {
    for index in 1..=data.nb_edges() {
        let edge = data.edge(index)?;
        if BRepTool::degenerated(&edge)? {
            ShapeBuildEdge.remove_pcurve_on_face(&edge, face)?;
        }
    }
    Ok(())
}

fn is_empty_or_single_degenerate(segment: &ShapeFixWireSegment) -> Result<bool, Failure> {
    if segment.nb_edges() <= 0 {
        return Ok(true);
    }
    if segment.nb_edges() != 1 {
        return Ok(false);
    }
    BRepTool::degenerated(&segment.edge(1)?)
}

fn null_pcurve() -> Failure {
    Failure::with_message(
        "Standard_NullObject",
        "ShapeFix_ComposeShell::DispatchWires: null seam p-curve; expected stored p-curve",
    )
}

fn null_wire() -> Failure {
    Failure::with_message(
        "Standard_NullObject",
        "ShapeFix_ComposeShell::DispatchWires: null wire data; expected initialized wire segment",
    )
}
