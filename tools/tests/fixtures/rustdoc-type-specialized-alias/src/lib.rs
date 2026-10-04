use core::marker::PhantomData;

mod private {
    use core::marker::PhantomData;

    pub trait Dialect {}
    pub trait Fast {}

    pub struct Text;
    pub struct Compatible;
    pub struct Generic<D: Dialect>(pub PhantomData<D>);

    impl Dialect for Text {}
    impl Dialect for Compatible {}
    impl Fast for Text {}

    impl<D: Dialect> Generic<D> {
        pub fn common(&self) -> &'static str {
            "common"
        }
    }

    pub trait Open {
        fn open(&self) -> &'static str;
    }

    impl<D: Dialect> Open for Generic<D> {
        fn open(&self) -> &'static str {
            "open"
        }
    }

    impl<D: Dialect + Fast> Generic<D> {
        pub fn fast_only(&self) -> &'static str {
            "fast"
        }
    }

    impl<D: Dialect> Generic<D>
    where
        D: Clone,
    {
        pub fn where_bound(&self) -> &'static str {
            "where"
        }
    }
}

pub use private::{Compatible, Dialect, Fast, Generic as TextFile, Open, Text};
pub type File = TextFile<Text>;
pub type CmpFile = TextFile<Compatible>;
