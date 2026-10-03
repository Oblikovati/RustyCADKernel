use super::{
    Failure, Shape, ShapeExtend, ShapeExtendStatus, ShapeExtendWireData, ShapeFixFace, Wire,
};
use rustycad_tkbrep::topexp;
use rustycad_tkernel::ncollection::Sequence;
pub(super) fn join_open_paths(
    open: &mut Vec<Wire>,
    output: &mut Sequence<Shape>,
    tool: &ShapeFixFace,
) -> Result<bool, Failure> {
    if open.len() == 1 {
        output.append(open[0].clone().into_shape());
        return Ok(false);
    }
    let mut changed = join_matching_ends(open, output, tool)?;
    if open.len() < 3 {
        for wire in open {
            output.append(wire.clone().into_shape());
        }
        return Ok(changed);
    }
    changed |= join_shared_end(open, output, tool)?;
    Ok(changed)
}
fn join_matching_ends(
    open: &mut Vec<Wire>,
    output: &mut Sequence<Shape>,
    tool: &ShapeFixFace,
) -> Result<bool, Failure> {
    let mut changed = false;
    let mut index = 0;
    while index < open.len() {
        let ends = topexp::wire_vertices(&open[index]);
        let other = (index + 1..open.len())
            .find(|&other| same_ends(&ends, &topexp::wire_vertices(&open[other]), true));
        if let Some(other) = other {
            output.append(merge_paths(&open[index], &open[other])?.into_shape());
            open.remove(other);
            open.remove(index);
            changed = true;
            *tool.status.write() |= ShapeExtend::encode_status(ShapeExtendStatus::Done7);
        } else {
            index += 1;
        }
    }
    Ok(changed)
}
fn join_shared_end(
    open: &mut Vec<Wire>,
    output: &mut Sequence<Shape>,
    tool: &ShapeFixFace,
) -> Result<bool, Failure> {
    let mut changed = false;
    for index in 0..open.len() {
        if index >= open.len() {
            break;
        }
        let (wire, joined) = join_following_paths(open, index, tool)?;
        changed |= joined;
        output.append(wire.into_shape());
    }
    Ok(changed)
}
fn join_following_paths(
    open: &mut Vec<Wire>,
    index: usize,
    tool: &ShapeFixFace,
) -> Result<(Wire, bool), Failure> {
    let mut changed = false;
    let mut wire = open[index].clone();
    let mut other = index + 1;
    while other < open.len() {
        if !same_ends(
            &topexp::wire_vertices(&wire),
            &topexp::wire_vertices(&open[other]),
            false,
        ) {
            other += 1;
            continue;
        }
        wire = merge_paths(&wire, &open[other])?;
        open.remove(other);
        changed = true;
        // OCCT exposes completed joins even when a later reorder fails.
        *tool.status.write() |= ShapeExtend::encode_status(ShapeExtendStatus::Done7);
    }
    Ok((wire, changed))
}

fn same_ends(
    first: &(
        rustycad_tkbrep::topods::Vertex,
        rustycad_tkbrep::topods::Vertex,
    ),
    second: &(
        rustycad_tkbrep::topods::Vertex,
        rustycad_tkbrep::topods::Vertex,
    ),
    both: bool,
) -> bool {
    let start = first.0.is_same(&second.0) || first.0.is_same(&second.1);
    let end = first.1.is_same(&second.0) || first.1.is_same(&second.1);
    if both { start && end } else { start || end }
}
fn merge_paths(first: &Wire, second: &Wire) -> Result<Wire, Failure> {
    let combined = ShapeExtendWireData::from_wire(first, true, true)?;
    combined.add_wire(second, 0)?;
    let fixer = super::ShapeFixWire::new();
    fixer.load_wire_data(&rustycad_tkernel::standard::Handle::new(combined))?;
    fixer.fix_reorder()?;
    fixer.wire()
}
