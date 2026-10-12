//! Pinned GNU13 stable-sort structure preserves native touching-interval ties without inventing lexicographic order.
use super::StdPrsSegOnIso;
pub(super) fn sort_isoline_segments(segments: &mut [StdPrsSegOnIso]) {
    let middle = segments.len().div_ceil(2);
    let (left, right) = segments.split_at_mut(middle);
    buffered_isoline_sort(left);
    buffered_isoline_sort(right);
    let merged = if left.len() <= right.len() {
        merge_isoline_forward(left, right)
    } else {
        merge_isoline_backward(left, right)
    };
    segments.copy_from_slice(&merged);
}
fn buffered_isoline_sort(segments: &mut [StdPrsSegOnIso]) {
    for chunk in segments.chunks_mut(7) {
        insertion_isoline_sort(chunk);
    }
    let mut width = 7;
    while width < segments.len() {
        merge_isoline_pass(segments, width);
        width *= 2;
        merge_isoline_pass(segments, width);
        width *= 2;
    }
}
fn insertion_isoline_sort(segments: &mut [StdPrsSegOnIso]) {
    for current in 1..segments.len() {
        let value = segments[current];
        let position = isoline_insertion_position(segments, current, &value);
        segments.copy_within(position..current, position + 1);
        segments[position] = value;
    }
}
fn isoline_insertion_position(
    segments: &[StdPrsSegOnIso],
    current: usize,
    value: &StdPrsSegOnIso,
) -> usize {
    if value.less_than(&segments[0]) {
        return 0;
    }
    let mut position = current;
    while position > 0 && value.less_than(&segments[position - 1]) {
        position -= 1;
    }
    position
}
fn merge_isoline_pass(segments: &mut [StdPrsSegOnIso], width: usize) {
    for chunk in segments.chunks_mut(width * 2) {
        let middle = width.min(chunk.len());
        let merged = merge_isoline_forward(&chunk[..middle], &chunk[middle..]);
        chunk.copy_from_slice(&merged);
    }
}
fn merge_isoline_forward(left: &[StdPrsSegOnIso], right: &[StdPrsSegOnIso]) -> Vec<StdPrsSegOnIso> {
    let mut result = Vec::with_capacity(left.len() + right.len());
    let (mut a, mut b) = (0, 0);
    while a < left.len() && b < right.len() {
        if right[b].less_than(&left[a]) {
            result.push(right[b]);
            b += 1;
        } else {
            result.push(left[a]);
            a += 1;
        }
    }
    result.extend_from_slice(&left[a..]);
    result.extend_from_slice(&right[b..]);
    result
}
fn merge_isoline_backward(
    left: &[StdPrsSegOnIso],
    right: &[StdPrsSegOnIso],
) -> Vec<StdPrsSegOnIso> {
    let mut reversed = Vec::with_capacity(left.len() + right.len());
    let (mut a, mut b) = (left.len(), right.len());
    while a > 0 && b > 0 {
        if right[b - 1].less_than(&left[a - 1]) {
            a -= 1;
            reversed.push(left[a]);
        } else {
            b -= 1;
            reversed.push(right[b]);
        }
    }
    reversed.extend(left[..a].iter().rev());
    reversed.extend(right[..b].iter().rev());
    reversed.reverse();
    reversed
}
