use super::SplitIntersection;
use rustycad_tkbrep::topods::Edge;
use rustycad_tkernel::standard::Failure;

pub(super) const IOR_BOTH: i32 = 3;
pub(super) const ITP_INTER: i32 = 8;
pub(super) const ITP_BEGSEG: i32 = 16;
pub(super) const ITP_ENDSEG: i32 = 32;
pub(super) const ITP_TANG: i32 = 64;

#[derive(Default)]
pub(super) struct JunctionState {
    pub(super) start_code: i32,
    pub(super) previous_code: i32,
    pub(super) start_point: Option<rustycad_tkmath::gp::Point2>,
    pub(super) previous_point: rustycad_tkmath::gp::Point2,
    pub(super) last_parameter: f64,
}

pub(super) struct SplitArguments {
    pub(super) indices: rustycad_tkernel::ncollection::Sequence<i32>,
    pub(super) values: rustycad_tkernel::ncollection::Sequence<f64>,
    pub(super) segment_codes: rustycad_tkernel::ncollection::Sequence<i32>,
}

pub(super) fn edge_start_parameter(edge: &Edge, first: f64, last: f64) -> f64 {
    if edge.orientation() == rustycad_tkg3d::top_abs::Orientation::Reversed {
        last
    } else {
        first
    }
}

pub(super) fn edge_end_parameter(edge: &Edge, first: f64, last: f64) -> f64 {
    if edge.orientation() == rustycad_tkg3d::top_abs::Orientation::Reversed {
        first
    } else {
        last
    }
}

pub(super) fn shared_tangential_point(segment_codes: &[i32]) -> Option<usize> {
    let count = segment_codes.len();
    (0..count)
        .find(|index| segment_codes[(index + count - 1) % count] == 0 && segment_codes[*index] == 0)
}

pub(super) fn point_code(
    index: usize,
    segment_codes: &[i32],
    intersections: &[SplitIntersection],
    is_internal: bool,
) -> i32 {
    let count = segment_codes.len();
    let previous = (index + count - 1) % count;
    let code_before = segment_codes[previous];
    let code_after = segment_codes[index];
    let mut code = code_before | code_after;
    if code_before == 0 {
        return tangential_start_code(index, previous, intersections, code);
    }
    if code_after == 0 {
        return tangential_end_code(index, count, intersections, code);
    }
    if index == previous {
        return single_segment_code(code, is_internal);
    }
    if is_internal || code_before == code_after {
        code |= ITP_TANG;
        return code;
    }
    if (code_before | code_after) == IOR_BOTH {
        code |= ITP_INTER;
        return code;
    }
    code | ITP_TANG
}

fn tangential_start_code(
    index: usize,
    previous: usize,
    intersections: &[SplitIntersection],
    code: i32,
) -> i32 {
    let next = intersections[index].line_parameter;
    let prior = intersections[previous].line_parameter;
    code | if next > prior { ITP_ENDSEG } else { ITP_BEGSEG }
}

fn tangential_end_code(
    index: usize,
    count: usize,
    intersections: &[SplitIntersection],
    code: i32,
) -> i32 {
    let next = intersections[(index + 1) % count].line_parameter;
    let current = intersections[index].line_parameter;
    code | if next > current {
        ITP_BEGSEG
    } else {
        ITP_ENDSEG
    }
}

fn single_segment_code(code: i32, is_internal: bool) -> i32 {
    if code & IOR_BOTH == IOR_BOTH && !is_internal {
        code | ITP_INTER
    } else {
        code | ITP_TANG
    }
}

pub(super) fn null_wire() -> Failure {
    Failure::with_message(
        "Standard_NullObject",
        "ShapeFix_ComposeShell::SplitByLine: null wire data",
    )
}

pub(super) fn null_context() -> Failure {
    Failure::with_message(
        "Standard_NullObject",
        "ShapeFix_ComposeShell::SplitByLine: null context",
    )
}

pub(super) fn null_pcurve() -> Failure {
    Failure::with_message(
        "Standard_NullObject",
        "ShapeFix_ComposeShell::SplitByLine: null p-curve",
    )
}

pub(super) fn not_implemented(detail: &str) -> Failure {
    Failure::with_message(
        "Standard_NotImplemented",
        &format!("ShapeFix_ComposeShell::SplitByLine: {detail}"),
    )
}
