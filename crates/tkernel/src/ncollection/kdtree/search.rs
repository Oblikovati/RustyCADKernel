use super::{KDTree, KdPoint, Region};
impl<P: KdPoint, const D: usize, const R: bool> KDTree<P, D, R> {
    /// Collects original indices inside an inclusive sphere; `tree.range_search(&query,1.0)`.
    pub fn range_search(&self, query: &P, radius: f64) -> Vec<usize> {
        let mut result = Vec::new();
        self.for_each_in_range(query, radius, |index| result.push(index));
        result
    }
    /// Visits the same indices without allocating an output array; `tree.for_each_in_range(&q,1.0,|i| consume(i))`.
    pub fn for_each_in_range(&self, query: &P, radius: f64, mut visitor: impl FnMut(usize)) {
        if self.is_empty() || radius < 0.0 {
            return;
        }
        self.range_visit(
            query,
            radius * radius,
            Region::root(self.len()),
            &mut visitor,
        );
    }
    pub(super) fn range_visit(
        &self,
        query: &P,
        radius: f64,
        region: Region<D>,
        visitor: &mut impl FnMut(usize),
    ) {
        if region.lo == region.hi {
            return;
        }
        if region.hi - region.lo <= 32 {
            for &index in &self.indices[region.lo..region.hi] {
                self.range_candidate(query, radius, index, visitor);
            }
            return;
        }
        let index = self.indices[region.middle()];
        self.range_candidate(query, radius, index, visitor);
        let split = self.points[index - 1].coord(region.depth % D + 1);
        let diff = query.coord(region.depth % D + 1) - split;
        let (left, right) = region.children(split);
        let (near, far) = if diff <= 0.0 {
            (left, right)
        } else {
            (right, left)
        };
        self.range_visit(query, radius, near, visitor);
        if diff * diff <= radius {
            self.range_visit(query, radius, far, visitor);
        }
    }
    fn range_candidate(
        &self,
        query: &P,
        radius: f64,
        index: usize,
        visitor: &mut impl FnMut(usize),
    ) {
        if self.distance(query, index) <= radius {
            visitor(index);
        }
    }
    /// Collects points in the inclusive axis-aligned box; `tree.box_search(&minimum,&maximum)`.
    pub fn box_search(&self, min: &P, max: &P) -> Vec<usize> {
        let mut result = Vec::new();
        self.box_visit(min, max, Region::root(self.len()), &mut result);
        result
    }
    fn box_visit(&self, min: &P, max: &P, region: Region<D>, result: &mut Vec<usize>) {
        if region.lo == region.hi {
            return;
        }
        if region.hi - region.lo <= 32 {
            for &index in &self.indices[region.lo..region.hi] {
                self.box_candidate(min, max, index, result);
            }
            return;
        }
        let index = self.indices[region.middle()];
        self.box_candidate(min, max, index, result);
        let axis = region.depth % D + 1;
        let split = self.points[index - 1].coord(axis);
        let (left, right) = region.children(split);
        if split >= min.coord(axis) {
            self.box_visit(min, max, left, result);
        }
        if split <= max.coord(axis) {
            self.box_visit(min, max, right, result);
        }
    }
    fn box_candidate(&self, min: &P, max: &P, index: usize, result: &mut Vec<usize>) {
        let point = &self.points[index - 1];
        if (1..=D).all(|axis| {
            !(point.coord(axis) < min.coord(axis) || point.coord(axis) > max.coord(axis))
        }) {
            result.push(index);
        }
    }
}
