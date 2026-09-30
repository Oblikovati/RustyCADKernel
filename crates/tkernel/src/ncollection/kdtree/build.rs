use super::{KDTree, KdPoint, selection};
impl<P: KdPoint, const D: usize> KDTree<P, D, false> {
    /// Copies a contiguous cloud and builds the balanced index; `tree.build(&points)`.
    pub fn build(&mut self, points: &[P]) {
        self.initialize(points, &[]);
    }
}
impl<P: KdPoint, const D: usize> KDTree<P, D, true> {
    /// Copies the cloud and its first `points.len()` radii (native pointer overload, whose
    /// count governs both buffers); `tree.build_radii(&points,&radii)`.
    ///
    /// # Panics
    /// When `radii` is shorter than `points`: the native read runs past the buffer.
    pub fn build_radii(&mut self, points: &[P], radii: &[f64]) {
        assert!(
            radii.len() >= points.len(),
            "KDTree radii length {}; expected at least {}",
            radii.len(),
            points.len()
        );
        self.initialize(points, &radii[..points.len()]);
    }
}
impl<P: KdPoint, const D: usize, const R: bool> KDTree<P, D, R> {
    fn initialize(&mut self, points: &[P], radii: &[f64]) {
        assert!(D > 0, "KDTree dimension {D}; expected positive dimension");
        self.points = points.to_vec();
        self.indices = (1..=points.len()).collect();
        self.radii = radii.to_vec();
        self.max_radius = if R {
            vec![0.0; points.len()]
        } else {
            Vec::new()
        };
        self.build_range(0, points.len(), 0);
    }
    fn build_range(&mut self, lo: usize, hi: usize, depth: usize) -> f64 {
        if lo == hi {
            return 0.0;
        }
        let mid = lo + (hi - lo - 1) / 2;
        let axis = depth % D + 1;
        selection::select(&mut self.indices[lo..hi], mid - lo, |a, b| {
            self.points[a - 1].coord(axis) < self.points[b - 1].coord(axis)
        });
        let left = self.build_range(lo, mid, depth + 1);
        let right = self.build_range(mid + 1, hi, depth + 1);
        if !R {
            return 0.0;
        }
        let radius = self.radii[self.indices[mid] - 1];
        let maximum = if hi - lo == 1 {
            radius
        } else {
            cpp_max(cpp_max(radius, left), right)
        };
        self.max_radius[mid] = maximum;
        maximum
    }
}
fn cpp_max(left: f64, right: f64) -> f64 {
    if left < right { right } else { left }
}
