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
