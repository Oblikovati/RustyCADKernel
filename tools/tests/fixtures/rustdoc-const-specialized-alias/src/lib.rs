mod compact {
    mod storage {
        pub struct CompactBounds<T, const N: usize>(pub [T; N]);

        pub struct GenericBounds<T, const N: usize>(pub [T; N]);

        pub struct BoundedBounds<T, const N: usize>(pub [T; N]);

        impl<T, const N: usize> Default for CompactBounds<T, N> {
            fn default() -> Self {
                Self(unimplemented!())
            }
        }
    }

    mod box2 {
        use super::storage::{BoundedBounds, CompactBounds, GenericBounds};

        pub type CompactBox2<T> = CompactBounds<T, 2>;
        pub type GenericBox2<T> = GenericBounds<T, 2>;
        pub type BoundedBox2<T> = BoundedBounds<T, 2>;

        impl<T> CompactBounds<T, 2> {
            pub fn compare_dist(&self) -> usize {
                2
            }
            pub fn compare_dist_d(&self) -> usize {
                3
            }
        }

        impl<T, const N: usize> GenericBounds<T, N> {
            pub fn compare_generic_dist(&self) -> usize {
                N
            }
        }

        impl<T, const N: usize> BoundedBounds<T, N>
        where
            [T; N]: Clone,
        {
            pub fn constrained(&self) -> usize {
                N
            }
        }
    }

    mod box3 {
        use super::storage::CompactBounds;

        pub type CompactBox3<T> = CompactBounds<T, 3>;

        impl<T> CompactBounds<T, 3> {
            pub fn compare_dist(&self) -> usize {
                3
            }
            pub fn compare_dist_d(&self) -> usize {
                4
            }
        }
    }

    pub use box2::{BoundedBox2, CompactBox2, GenericBox2};
    pub use box3::CompactBox3;
}

pub use compact::{BoundedBox2, CompactBox2, CompactBox3, GenericBox2};
