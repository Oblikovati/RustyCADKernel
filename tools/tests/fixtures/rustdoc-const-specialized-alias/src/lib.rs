pub struct CompactBounds<T, const N: usize>(pub [T; N]);

impl<T> CompactBounds<T, 2> {
    pub fn compare_dist(&self) -> usize {
        2
    }

    pub fn compare_dist_d(&self) -> usize {
        3
    }
}

impl<T> CompactBounds<T, 3> {
    pub fn compare_dist(&self) -> usize {
        3
    }

    pub fn compare_dist_d(&self) -> usize {
        4
    }
}

pub type CompactBox2<T> = CompactBounds<T, 2>;
pub type CompactBox3<T> = CompactBounds<T, 3>;

pub struct GenericBounds<T, const N: usize>(pub [T; N]);

impl<T, const N: usize> GenericBounds<T, N> {
    pub fn compare_generic_dist(&self) -> usize {
        N
    }
}

pub type GenericBox2<T> = GenericBounds<T, 2>;

pub struct BoundedBounds<T, const N: usize>(pub [T; N]);

impl<T, const N: usize> BoundedBounds<T, N>
where
    [T; N]: Clone,
{
    pub fn constrained(&self) -> usize {
        N
    }
}

pub type BoundedBox2<T> = BoundedBounds<T, 2>;
