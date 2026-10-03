//! Joint-line event parity and seam generation across all boundary segments.
// ShapeFix_ComposeShell.cxx 1918–2130 at 3d097a0328e71b826377d4814ab05ec3c3d23871.
mod events;
mod seams;
use super::{ShapeFixComposeShell, split_by_line, split_wire};
use crate::shape_extend::{ShapeExtend, ShapeExtendStatus};
use crate::shape_fix::{FixRoot, ShapeFixWireSegment};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::{ncollection::Sequence, standard::Failure};
use rustycad_tkmath::gp::Line2;

struct LineEvent {
    parameter: f64,
    code: i32,
    vertex: Shape,
}

#[derive(Clone, Copy)]
struct GridCut {
    line: Line2,
    along_u: bool,
    index: i32,
}

pub(super) fn split_sequence(
    composer: &ShapeFixComposeShell,
    wires: &mut Sequence<ShapeFixWireSegment>,
    line: Line2,
    along_u: bool,
    index: i32,
) -> Result<(), Failure> {
    let cut = GridCut {
        line,
        along_u,
        index,
    };
    let mut events = collect_events(composer, wires, cut)?;
    events::sort_and_merge(&mut events);
    append_interior_seams(composer, wires, &events, cut)?;
    apply_wire_context(composer, wires)
}

fn collect_events(
    composer: &ShapeFixComposeShell,
    wires: &mut Sequence<ShapeFixWireSegment>,
    cut: GridCut,
) -> Result<Vec<LineEvent>, Failure> {
    let (mut parameters, mut codes, mut vertices) = (
        Sequence::default(),
        Sequence::default(),
        Sequence::default(),
    );
    for index in 1..=wires.length() {
        split_by_line::split_by_line(
            composer,
            split_by_line::SplitByLineInput {
                wire: wires.change_value(index as usize)?,
                line: cut.line,
                cut_by_u: cut.along_u,
                cut_index: cut.index,
                line_parameters: &mut parameters,
                point_codes: &mut codes,
                vertices: &mut vertices,
            },
        )?;
    }
    (1..=parameters.len())
        .map(|index| {
            Ok(LineEvent {
                parameter: *parameters.value(index)?,
                code: *codes.value(index)?,
                vertex: vertices.value(index)?.clone(),
            })
        })
        .collect()
}

fn append_interior_seams(
    composer: &ShapeFixComposeShell,
    wires: &mut Sequence<ShapeFixWireSegment>,
    events: &[LineEvent],
    cut: GridCut,
) -> Result<(), Failure> {
    let mut parity = events::LineParity::default();
    for (index, event) in events.iter().enumerate() {
        if !parity.advance(event.code) {
            continue;
        }
        let inner_pair = index > 1 || index + 1 < events.len();
        seams::append_seam(composer, wires, &events[index - 1..=index], cut, inner_pair)?;
    }
    if parity.is_odd() {
        composer.fields.write().status |= ShapeExtend::encode_status(ShapeExtendStatus::Fail4);
    }
    Ok(())
}

fn apply_wire_context(
    composer: &ShapeFixComposeShell,
    wires: &Sequence<ShapeFixWireSegment>,
) -> Result<(), Failure> {
    for wire_index in 1..=wires.len() {
        let wire = wires.value(wire_index)?;
        let mut edge_index = 1;
        while edge_index <= wire.nb_edges() {
            let advance = split_wire::apply_context(wire, edge_index, &composer.context())?;
            if advance <= 0 {
                return Err(dismissed_context_edge(edge_index));
            }
            edge_index += advance;
        }
    }
    Ok(())
}

fn dismissed_context_edge(index: i32) -> Failure {
    Failure::with_message(
        "Standard_NotImplemented",
        &format!(
            "ShapeFix_ComposeShell::SplitByLine: context removed edge {index}; expected at least one replacement edge (native loop cannot advance)"
        ),
    )
}
