pub struct CompactBounds<T, const N: usize>(pub [T; N]);

impl<T, const N: usize> CompactBounds<T, N> {
    pub fn compare_dist(&self) -> usize {
        N
    }

    pub fn compare_dist_d(&self) -> usize {
        N + 1
    }
}

pub type CompactBox2<T> = CompactBounds<T, 2>;
pub type CompactBox3<T> = CompactBounds<T, 3>;
