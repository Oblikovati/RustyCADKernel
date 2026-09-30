//! Merge in discovery order; whole-range matches clear previously accumulated common parts.
use super::EdgeEdge;
use crate::int_tools::Range;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::native::{std_max, std_min};
use rustycad_tkg3d::top_abs::ShapeEnum;
impl EdgeEdge {
    pub(super) fn merge_solutions(
        &mut self,
        solutions: &[[Range; 2]],
        split: bool,
    ) -> Result<(), Failure> {
        if solutions.is_empty() {
            return Ok(());
        }
        let allowances = [
            20. * self.curve_resolution(0, self.tolerance)?,
            20. * self.curve_resolution(1, self.tolerance)?,
        ];
        let mut consumed = vec![false; solutions.len()];
        let mut index = 0;
        while index < solutions.len() {
            if consumed[index] {
                index += 1;
                continue;
            }
            let mut range = solutions[index];
            consumed[index] = true;
            self.merge_following(
                solutions,
                split,
                allowances,
                &mut consumed,
                &mut index,
                &mut range,
            );
            let complete = self.covers_input(range);
            if complete {
                self.parts.clear();
            }
            self.add_solution(
                range,
                if complete {
                    ShapeEnum::Edge
                } else {
                    ShapeEnum::Vertex
                },
            )?;
            if complete {
                break;
            }
            if split {
                index += 1;
            }
        }
        Ok(())
    }
    fn merge_following(
        &self,
        solutions: &[[Range; 2]],
        split: bool,
        allowances: [f64; 2],
        consumed: &mut [bool],
        index: &mut usize,
        range: &mut [Range; 2],
    ) {
        for next in (*index + 1)..solutions.len() {
            if consumed[next] {
                continue;
            }
            if mergeable(*range, solutions[next], allowances, split) {
                range[0] = union(range[0], solutions[next][0]);
                range[1] = union(range[1], solutions[next][1]);
                consumed[next] = true;
            } else if !split {
                *index = next;
                break;
            }
        }
    }
    fn covers_input(&self, ranges: [Range; 2]) -> bool {
        (0..2).any(|index| {
            (self.ranges[index].first() - ranges[index].first()).abs() < self.resolutions[index]
                && (self.ranges[index].last() - ranges[index].last()).abs()
                    < self.resolutions[index]
        })
    }
}
fn overlapping(first: Range, second: Range) -> bool {
    (second.first() > first.first() && second.first() < first.last())
        || (first.first() > second.first() && first.first() < second.last())
}
fn mergeable(first: [Range; 2], second: [Range; 2], allowance: [f64; 2], split: bool) -> bool {
    let contiguous = (first[0].last() - second[0].first()).abs() < allowance[0]
        || overlapping(first[0], second[0])
        || (split && (second[0].last() - first[0].first()).abs() < allowance[0]);
    if !contiguous || !split {
        return contiguous;
    }
    let combined =
        std_max(first[1].last(), second[1].last()) - std_min(first[1].first(), second[1].first());
    let lengths = (first[1].last() - first[1].first()) + (second[1].last() - second[1].first());
    (combined - lengths).abs() < allowance[1] || overlapping(first[1], second[1])
}
fn union(first: Range, second: Range) -> Range {
    Range::new(
        std_min(first.first(), second.first()),
        std_max(first.last(), second.last()),
    )
}
