#![allow(dead_code)]

pub struct Xy;
pub struct Matrix2;

impl core::ops::Mul<f64> for Xy {
    type Output = Matrix2;

    fn mul(self, _right: f64) -> Matrix2 {
        Matrix2
    }
}

impl core::ops::Mul<Matrix2> for Xy {
    type Output = Xy;

    fn mul(self, _right: Matrix2) -> Xy {
        Xy
    }
}

pub trait Operation<T> {
    fn apply(&self, value: T);
}

pub use Operation as PublicOperation;

pub struct Value;

impl Operation<f64> for Value {
    fn apply(&self, _value: f64) {}
}

pub trait ConstOperation<const N: usize> {
    fn count(&self) -> usize;
}

pub struct Flag;

impl ConstOperation<2> for Flag {
    fn count(&self) -> usize {
        2
    }
}

pub struct DefaultSelf;

impl core::ops::Mul for DefaultSelf {
    type Output = DefaultSelf;

    fn mul(self, _right: Self) -> Self::Output {
        DefaultSelf
    }
}

pub struct ExplicitSelf;

impl core::ops::Mul<Self> for ExplicitSelf {
    type Output = ExplicitSelf;

    fn mul(self, _right: Self) -> Self::Output {
        ExplicitSelf
    }
}

pub struct ReferenceArgument;

impl core::ops::Add<&'static Matrix2> for ReferenceArgument {
    type Output = ReferenceArgument;

    fn add(self, _right: &'static Matrix2) -> Self::Output {
        ReferenceArgument
    }
}

pub struct CollectionArgument;

impl core::convert::From<Vec<Matrix2>> for CollectionArgument {
    fn from(_value: Vec<Matrix2>) -> Self {
        CollectionArgument
    }
}
