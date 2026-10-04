use core::marker::PhantomData;

mod private {
    use core::marker::PhantomData;

    pub trait Dialect {}
    pub trait Fast {}

    pub struct Text;
    pub struct Compatible;
    pub struct Generic<DialectImplementationParameter: Dialect>(pub PhantomData<DialectImplementationParameter>);

    impl Dialect for Text {}
    impl Dialect for Compatible {}
    impl Fast for Text {}

    impl<DialectImplementationParameter: Dialect> Generic<DialectImplementationParameter> {
        pub fn common(&self) -> &'static str {
            "common"
        }
    }

    pub trait Open {
        fn open(&self) -> &'static str;
    }

    impl<DialectImplementationParameter: Dialect> Open for Generic<DialectImplementationParameter> {
        fn open(&self) -> &'static str {
            "open"
        }
    }

    impl<DialectImplementationParameter: Dialect + Fast> Generic<DialectImplementationParameter> {
        pub fn fast_only(&self) -> &'static str {
            "fast"
        }
    }

    impl<DialectImplementationParameter: Dialect> Generic<DialectImplementationParameter>
    where
        DialectImplementationParameter: Clone,
    {
        pub fn where_bound(&self) -> &'static str {
            "where"
        }
    }
}

pub use private::{Compatible, Dialect, Fast, Generic as TextFile, Open, Text};
pub type File = TextFile<Text>;
pub type CmpFile = TextFile<Compatible>;
