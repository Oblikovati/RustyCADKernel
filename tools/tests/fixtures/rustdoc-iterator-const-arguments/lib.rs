#![allow(dead_code)]
use core::marker::PhantomData;
pub struct Cursor<T, const MUTABLE: bool> {
    position: usize,
    marker: PhantomData<T>,
}
impl<T, const M: bool, const O: bool> PartialEq<Cursor<T, O>> for Cursor<T, M> {
    fn eq(&self, other: &Cursor<T, O>) -> bool {
        self.position == other.position
    }
}
pub trait ConstLookup<const SLOT: usize> {
    fn lookup(&self) -> usize;
}
pub struct Lookup<const N: usize>;
impl<const N: usize> ConstLookup<N> for Lookup<N> {
    fn lookup(&self) -> usize {
        N
    }
}
