// Rust translation of FoundationClasses/TKMath/Bnd/Bnd_B3.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `IsOut(theBox, theTrsf)` and `IsIn(theBox, theTrsf)` shared by `Bnd_B2` and `Bnd_B3`.
use super::{BoxReal, storage::CompactBounds};

/// The transform facts both overloads read, in dimension-independent form.
///
/// `simple` is true for the Translation, Identity, PntMirror and Scale forms, `matrix`
/// is `HVectorialPart()` by rows and `moved_center` is `theTrsf.Transforms(center)` of the
/// other box's center promoted to double. Example: built by `CompactBox3::is_out_transformed_box`.
pub(super) struct TransformView<const N: usize> {
    pub(super) simple: bool,
    pub(super) scale: f64,
    pub(super) translation: [f64; N],
    pub(super) matrix: [[f64; N]; N],
    pub(super) moved_center: [f64; N],
}

/// Native left-to-right sum `t0 + t1 + ...` (no leading `0.0 +`, which would turn `-0.0` into `+0.0`).
fn left_sum(terms: impl Iterator<Item = f64>) -> f64 {
    terms.reduce(|sum, term| sum + term).unwrap_or(0.0)
}

impl<T: BoxReal, const N: usize> CompactBounds<T, N> {
    /// Separation of `other` mapped by the transform, as `Bnd_B3::IsOut(theBox, theTrsf)`.
    pub(super) fn out_of_transformed(&self, other: &Self, view: &TransformView<N>) -> bool {
        if view.simple {
            return (0..N).any(|i| {
                let (distance, reach) = self.simple_axis(other, view, i);
                distance > reach + self.half[i]
            });
        }
        let distance = self.center_distance(view);
        self.rows_separate(other, view, &distance) || self.columns_separate(other, view, &distance)
    }
    /// Strict inclusion of `self` in the transformed `other`, as `Bnd_B3::IsIn(theBox, theTrsf)`.
    pub(super) fn in_transformed(&self, other: &Self, view: &TransformView<N>) -> bool {
        if view.simple {
            return (0..N).all(|i| {
                let (distance, reach) = self.simple_axis(other, view, i);
                distance < reach - self.half[i]
            });
        }
        let distance = self.center_distance(view);
        (0..N).all(|column| {
            let (projection, own) = self.column_terms(view, &distance, column);
            projection < other.half[column].to_double() * view.scale.abs() - own
        })
    }
    /// Scalar-typed center distance and scaled half size along `axis` for simple forms:
    /// the products are formed in double, then rounded to the storage type.
    fn simple_axis(&self, other: &Self, view: &TransformView<N>, axis: usize) -> (T, T) {
        let moved = other.center[axis].to_double() * view.scale + view.translation[axis];
        let distance = (T::from_double(moved) - self.center[axis]).absolute();
        let reach = T::from_double(other.half[axis].to_double() * view.scale.abs());
        (distance, reach)
    }
    fn center_distance(&self, view: &TransformView<N>) -> [f64; N] {
        std::array::from_fn(|i| view.moved_center[i] - self.center[i].to_double())
    }
    /// The transformed `other` box, enlarged to its axis-aligned hull, against `self`.
    fn rows_separate(&self, other: &Self, view: &TransformView<N>, distance: &[f64; N]) -> bool {
        (0..N).any(|row| {
            let hull =
                left_sum((0..N).map(|j| view.matrix[row][j].abs() * other.half[j].to_double()));
            distance[row].abs() > view.scale.abs() * hull + self.half[row].to_double()
        })
    }
    /// `self`, rotated back into the frame of `other`, against the scaled `other` extents.
    fn columns_separate(&self, other: &Self, view: &TransformView<N>, distance: &[f64; N]) -> bool {
        (0..N).any(|column| {
            let (projection, own) = self.column_terms(view, distance, column);
            projection > other.half[column].to_double() * view.scale.abs() + own
        })
    }
    /// `|Σ M[i][c]·dist[i]|` and `Σ |M[i][c]|·half[i]` for one matrix column.
    fn column_terms(
        &self,
        view: &TransformView<N>,
        distance: &[f64; N],
        column: usize,
    ) -> (f64, f64) {
        let projection = left_sum((0..N).map(|i| view.matrix[i][column] * distance[i])).abs();
        let own = left_sum((0..N).map(|i| view.matrix[i][column].abs() * self.half[i].to_double()));
        (projection, own)
    }
}
