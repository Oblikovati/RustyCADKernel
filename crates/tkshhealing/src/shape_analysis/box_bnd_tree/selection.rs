use super::ShapeAnalysisBoxBndTreeSelector as Selector;
use crate::{
    shape_analysis::ShapeAnalysis,
    shape_extend::{ShapeExtend, ShapeExtendStatus},
};
use rustycad_tkbrep::{
    brep::BRepTool,
    topods::{self, Shape, Vertex},
};
use rustycad_tkernel::standard::Failure;

pub(super) fn accept(selector: &mut Selector, index: i32) -> Result<bool, Failure> {
    validate_index(selector, index)?;
    if selector.visited.contains(&index) {
        return Ok(false);
    }
    let wire = selected_wire(selector, index)?;
    let (first, last) = ShapeAnalysis::find_bounds(&wire)?;
    if selector.shared {
        return Ok(shared(selector, index, &first, &last));
    }
    geometric(selector, index, &first, &last)
}
fn sequence(
    selector: &Selector,
) -> Result<&rustycad_tkernel::ncollection::HArray1<Shape>, Failure> {
    selector.sequence.get().ok_or_else(|| {
        Failure::with_type(
            "Standard_NullObject",
            "ShapeAnalysis_BoxBndTreeSelector sequence is null; expected allocated shape array",
            "",
        )
    })
}
fn validate_index(selector: &Selector, index: i32) -> Result<(), Failure> {
    let length = sequence(selector)?.array1().len();
    if index >= 1 && index as usize <= length {
        return Ok(());
    }
    Err(index_failure(
        "Standard_NoSuchObject",
        index,
        format!("1..={length}"),
    ))
}
fn selected_wire(selector: &Selector, index: i32) -> Result<Shape, Failure> {
    let array = sequence(selector)?.array1();
    let shape = array.value(index).map_err(|_| {
        index_failure(
            "Standard_OutOfRange",
            index,
            format!("{}..={}", array.lower(), array.upper()),
        )
    })?;
    Ok(topods::wire(shape)?.into_shape())
}
fn index_failure(kind: &'static str, index: i32, expected: String) -> Failure {
    Failure::with_type(
        kind,
        &format!("ShapeAnalysis_BoxBndTreeSelector index {index}; expected {expected}"),
        "",
    )
}
fn shared(selector: &mut Selector, index: i32, first: &Vertex, last: &Vertex) -> bool {
    let Some(position) = shared_match(selector, first, last) else {
        selector.status = ShapeExtend::encode_status(ShapeExtendStatus::Fail2);
        selector.stopped = false;
        return false;
    };
    selector.status = done_status(position);
    selector.indices[usize::from(position < 2)] = index;
    selector.selected = index;
    if selector.indices[1] != 0 {
        selector.stopped = true;
    }
    true
}
fn shared_match(selector: &Selector, first: &Vertex, last: &Vertex) -> Option<usize> {
    [
        selector.vertices[1].is_same(first),
        selector.vertices[1].is_same(last),
        selector.vertices[0].is_same(last),
        selector.vertices[0].is_same(first),
    ]
    .iter()
    .position(|same| *same)
}
struct NearestEndpoint {
    distance: f64,
    head_closer: bool,
    orientation: usize,
}
fn geometric(
    selector: &mut Selector,
    index: i32,
    first: &Vertex,
    last: &Vertex,
) -> Result<bool, Failure> {
    let nearest = nearest_endpoint(selector, first, last)?;
    if nearest.distance > selector.minimum {
        return Ok(false);
    }
    let position = usize::from(!nearest.head_closer);
    selector.indices[position] = index;
    // Preserve the native subtraction direction, including retained opposite candidates.
    if nearest.distance - selector.minimum > f64::MIN_POSITIVE {
        selector.indices[1 - position] = 0;
    }
    selector.minimum = nearest.distance;
    Ok(record_geometric(selector, index, &nearest))
}
fn nearest_endpoint(
    selector: &Selector,
    first: &Vertex,
    last: &Vertex,
) -> Result<NearestEndpoint, Failure> {
    let p1 = BRepTool::pnt(first)?;
    let p2 = BRepTool::pnt(last)?;
    let (tail, tail_kind) = endpoint_distance(&p1, &p2, &selector.points[1]);
    let (head, head_kind) = endpoint_distance(&p2, &p1, &selector.points[0]);
    Ok(NearestEndpoint {
        distance: if head < tail { head } else { tail },
        head_closer: tail > head,
        orientation: if tail > head {
            head_kind + 2
        } else {
            tail_kind
        },
    })
}
fn endpoint_distance(
    first: &rustycad_tkmath::gp::Point3,
    last: &rustycad_tkmath::gp::Point3,
    reference: &rustycad_tkmath::gp::Point3,
) -> (f64, usize) {
    let distances = [first.distance(reference), last.distance(reference)];
    let kind = usize::from(distances[0] > distances[1]);
    (distances[kind], kind)
}
fn record_geometric(selector: &mut Selector, index: i32, nearest: &NearestEndpoint) -> bool {
    if nearest.distance > selector.tolerance {
        selector.status = ShapeExtend::encode_status(ShapeExtendStatus::Fail2);
        return false;
    }
    selector.selected = if selector.indices[1] != 0 {
        selector.indices[1]
    } else {
        selector.indices[0]
    };
    if nearest.distance == 0. && !nearest.head_closer {
        selector.stopped = true;
    }
    if selector.selected == index {
        selector.status = done_status(nearest.orientation);
    }
    true
}
fn done_status(position: usize) -> i32 {
    ShapeExtend::encode_status(
        [
            ShapeExtendStatus::Done1,
            ShapeExtendStatus::Done2,
            ShapeExtendStatus::Done3,
            ShapeExtendStatus::Done4,
        ][position],
    )
}
