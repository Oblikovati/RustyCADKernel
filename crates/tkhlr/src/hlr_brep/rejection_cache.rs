//! Private HLRBRep_Data::RejectionCache at OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
use rustycad_tkgeomalgo::int_res2d::{IntersectionPoint, Position, TypeTrans};

struct CachedIntersection {
    edge_index: u32,
    parameter: f64,
}

pub(crate) struct RejectionCache {
    intersections: Vec<Vec<CachedIntersection>>,
    rejected_pairs: Vec<u32>,
    words_per_row: usize,
}

impl RejectionCache {
    pub(crate) fn new(edge_count: usize) -> Self {
        let words_per_row = edge_count.div_ceil(32);
        Self {
            intersections: (0..edge_count).map(|_| Vec::new()).collect(),
            rejected_pairs: vec![0; edge_count * words_per_row],
            words_per_row,
        }
    }

    fn set(&mut self, first: usize, second: usize, parameter: f64) {
        let row = &mut self.intersections[first - 1];
        if row.capacity() == 0 {
            row.reserve(8);
        }
        let edge_index = second as u32;
        let offset = row.partition_point(|entry| entry.edge_index > edge_index);
        // Native lower_bound inserts before equal keys: the newest value wins.
        row.insert(
            offset,
            CachedIntersection {
                edge_index,
                parameter,
            },
        );
    }

    fn get(&self, first: usize, second: usize) -> f64 {
        let row = &self.intersections[first - 1];
        let edge_index = second as u32;
        let offset = row.partition_point(|entry| entry.edge_index > edge_index);
        row.get(offset)
            .filter(|entry| entry.edge_index == edge_index)
            .map_or(f64::MAX, |entry| entry.parameter)
    }

    fn rejection_address(&self, first: usize, second: usize) -> (usize, u32) {
        let row = first.min(second) - 1;
        let column = first.max(second) - 1;
        (row * self.words_per_row + column / 32, 1 << (column % 32))
    }

    pub(crate) fn set_no_intersection(&mut self, first: usize, second: usize) {
        let (word, mask) = self.rejection_address(first, second);
        self.rejected_pairs[word] |= mask;
    }

    pub(crate) fn no_intersection(&self, first: usize, second: usize) -> bool {
        let (word, mask) = self.rejection_address(first, second);
        self.rejected_pairs[word] & mask != 0
    }

    pub(crate) fn set_intersection(
        &mut self,
        first: usize,
        second: usize,
        intersection: &IntersectionPoint,
    ) {
        let first_transition = intersection.transition_of_first();
        let second_transition = intersection.transition_of_second();
        if first_transition.position_on_curve() != Position::Middle
            || second_transition.position_on_curve() != Position::Middle
            || !matches!(
                first_transition.transition_type(),
                TypeTrans::In | TypeTrans::Out
            )
        {
            return;
        }
        self.set(first, second, intersection.param_on_first());
        self.set(second, first, intersection.param_on_second());
    }

    pub(crate) fn single_intersection(&self, first: usize, second: usize) -> (f64, f64) {
        let first_parameter = self.get(first, second);
        let second_parameter = if first_parameter == f64::MAX {
            f64::MAX
        } else {
            self.get(second, first)
        };
        (first_parameter, second_parameter)
    }
}

#[cfg(test)]
mod tests;
