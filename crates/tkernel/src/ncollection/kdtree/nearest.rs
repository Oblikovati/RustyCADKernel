use super::{Array1, KDTree, KdPoint, Region, heap::NearestHeap};
trait NearestVisitor {
    fn threshold(&self) -> Option<f64>;
    fn offer(&mut self, distance: f64, index: usize);
}
struct BestPoint {
    index: usize,
    distance: f64,
}
impl NearestVisitor for BestPoint {
    fn threshold(&self) -> Option<f64> {
        Some(self.distance)
    }
    fn offer(&mut self, distance: f64, index: usize) {
        if distance < self.distance {
            self.distance = distance;
            self.index = index;
        }
    }
}
impl NearestVisitor for NearestHeap {
    fn threshold(&self) -> Option<f64> {
        self.threshold()
    }
    fn offer(&mut self, distance: f64, index: usize) {
        self.offer(distance, index);
    }
}
impl<P: KdPoint, const D: usize, const R: bool> KDTree<P, D, R> {
    /// Returns zero for an empty tree and optionally writes squared distance; `tree.nearest_point(&query,None)`.
    pub fn nearest_point(&self, query: &P, distance: Option<&mut f64>) -> usize {
        let mut best = BestPoint {
            index: 0,
            distance: if self.is_empty() { 0.0 } else { f64::MAX },
        };
        self.nearest_visit(query, Region::root(self.len()), &mut best);
        if let Some(output) = distance {
            *output = best.distance;
        }
        best.index
    }
    /// Collects all nearest-distance ties within an absolute distance tolerance; `tree.nearest_points(&q,0.0,&mut distance)`.
    pub fn nearest_points(&self, query: &P, tolerance: f64, distance: &mut f64) -> Vec<usize> {
        self.nearest_point(query, Some(distance));
        if self.is_empty() {
            return Vec::new();
        }
        let nearest = distance.sqrt();
        let maximum = (nearest + tolerance) * (nearest + tolerance);
        let minimum = if nearest - tolerance > 0.0 {
            (nearest - tolerance) * (nearest - tolerance)
        } else {
            0.0
        };
        let mut result = Vec::new();
        self.range_visit(query, maximum, Region::root(self.len()), &mut |index| {
            let squared = self.distance(query, index);
            if squared >= minimum && squared <= maximum {
                result.push(index);
            }
        });
        result
    }
    /// Resizes one-based output arrays and returns the count; `tree.k_nearest_points(&q,3,&mut indices,&mut distances)`.
    pub fn k_nearest_points(
        &self,
        query: &P,
        count: usize,
        indices: &mut Array1<'_, usize>,
        distances: &mut Array1<'_, f64>,
    ) -> usize {
        if count == 0 || self.is_empty() {
            *indices = Array1::default();
            *distances = Array1::default();
            return 0;
        }
        let mut heap = NearestHeap::new(count.min(self.len()));
        self.nearest_visit(query, Region::root(self.len()), &mut heap);
        let sorted = heap.sorted();
        indices
            .resize_bounds(1, sorted.len() as i32, false)
            .unwrap();
        distances
            .resize_bounds(1, sorted.len() as i32, false)
            .unwrap();
        for (position, (distance, index)) in sorted.iter().enumerate() {
            indices.as_mut_slice()[position] = *index;
            distances.as_mut_slice()[position] = *distance;
        }
        sorted.len()
    }
    fn nearest_visit(&self, query: &P, region: Region<D>, visitor: &mut impl NearestVisitor) {
        if region.lo == region.hi
            || visitor
                .threshold()
                .is_some_and(|limit| region.distance(query) >= limit)
        {
            return;
        }
        if region.hi - region.lo <= 32 {
            for &index in &self.indices[region.lo..region.hi] {
                visitor.offer(self.distance(query, index), index);
            }
            return;
        }
        let index = self.indices[region.middle()];
        visitor.offer(self.distance(query, index), index);
        let split = self.points[index - 1].coord(region.depth % D + 1);
        let (left, right) = region.children(split);
        let (near, far) = if query.coord(region.depth % D + 1) - split <= 0.0 {
            (left, right)
        } else {
            (right, left)
        };
        self.nearest_visit(query, near, visitor);
        self.nearest_visit(query, far, visitor);
    }
}
