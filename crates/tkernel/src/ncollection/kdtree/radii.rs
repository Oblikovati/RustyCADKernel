use super::{Failure, KDTree, KdPoint, Region};
impl<P: KdPoint, const D: usize> KDTree<P, D, true> {
    /// Returns the stored radius by original index; `tree.radius(1)?`.
    pub fn radius(&self, index: usize) -> Result<f64, Failure> {
        self.point(index)?;
        Ok(self.radii[index - 1])
    }
    /// Collects every sphere containing the query; `tree.containing_search(&query)`.
    pub fn containing_search(&self, query: &P) -> Vec<usize> {
        let mut result = Vec::new();
        self.containing_visit(query, Region::root(self.len()), &mut result);
        result
    }
    fn containing_visit(&self, query: &P, region: Region<D>, result: &mut Vec<usize>) {
        if region.lo == region.hi {
            return;
        }
        let radius = self.max_radius[region.middle()];
        if region.distance(query) > radius * radius {
            return;
        }
        if region.hi - region.lo <= 32 {
            for &index in &self.indices[region.lo..region.hi] {
                self.containing_candidate(query, index, result);
            }
            return;
        }
        let index = self.indices[region.middle()];
        self.containing_candidate(query, index, result);
        let (left, right) = region.children(self.points[index - 1].coord(region.depth % D + 1));
        self.containing_visit(query, left, result);
        self.containing_visit(query, right, result);
    }
    fn containing_candidate(&self, query: &P, index: usize, result: &mut Vec<usize>) {
        let radius = self.radii[index - 1];
        if self.distance(query, index) <= radius * radius {
            result.push(index);
        }
    }
    /// Minimizes center distance minus radius, including negative gaps; `tree.nearest_weighted(&q,None)`.
    pub fn nearest_weighted(&self, query: &P, gap: Option<&mut f64>) -> usize {
        let mut best = (0, if self.is_empty() { 0.0 } else { f64::MAX });
        self.weighted_visit(query, Region::root(self.len()), &mut best);
        if let Some(output) = gap {
            *output = best.1;
        }
        best.0
    }
    fn weighted_visit(&self, query: &P, region: Region<D>, best: &mut (usize, f64)) {
        if region.lo == region.hi {
            return;
        }
        let threshold = best.1 + self.max_radius[region.middle()];
        if threshold >= 0.0 && region.distance(query) >= threshold * threshold {
            return;
        }
        if region.hi - region.lo <= 32 {
            for &index in &self.indices[region.lo..region.hi] {
                self.weighted_candidate(query, index, best);
            }
            return;
        }
        let index = self.indices[region.middle()];
        self.weighted_candidate(query, index, best);
        let axis = region.depth % D + 1;
        let split = self.points[index - 1].coord(axis);
        let (left, right) = region.children(split);
        let (near, far) = if query.coord(axis) - split <= 0.0 {
            (left, right)
        } else {
            (right, left)
        };
        self.weighted_visit(query, near, best);
        self.weighted_visit(query, far, best);
    }
    fn weighted_candidate(&self, query: &P, index: usize, best: &mut (usize, f64)) {
        let gap = self.distance(query, index).sqrt() - self.radii[index - 1];
        if gap < best.1 {
            *best = (index, gap);
        }
    }
}
