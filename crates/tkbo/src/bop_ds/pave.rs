// BOPDS_Pave.hxx/.lxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
//! A vertex index and its parameter on an edge.
use rustycad_tkernel::standard::{hash_bytes, hashing::hash};
/// `BOPDS_Pave`; `Pave::new(2, 0.5).contents() == (2, 0.5)`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pave {
    index: i32,
    parameter: f64,
}
impl Default for Pave {
    fn default() -> Self {
        Self::new(-1, 99.0)
    }
}
impl Pave {
    /// Stores the index and edge parameter; `Pave::new(2, 0.5)`.
    pub fn new(index: i32, parameter: f64) -> Self {
        Self { index, parameter }
    }
    /// Replaces the index; `pave.set_index(3)`.
    pub fn set_index(&mut self, index: i32) {
        self.index = index;
    }
    /// Reads the index; `Pave::default().index() == -1`.
    pub fn index(&self) -> i32 {
        self.index
    }
    /// Replaces the parameter; `pave.set_parameter(0.25)`.
    pub fn set_parameter(&mut self, parameter: f64) {
        self.parameter = parameter;
    }
    /// Reads the parameter; `Pave::default().parameter() == 99.0`.
    pub fn parameter(&self) -> f64 {
        self.parameter
    }
    /// Reads both fields; `pave.contents()`.
    pub fn contents(&self) -> (i32, f64) {
        (self.index, self.parameter)
    }
    /// Compares parameters only; `Pave::new(9, 0.0).is_less(&Pave::new(1, 1.0))`.
    pub fn is_less(&self, other: &Self) -> bool {
        self.parameter < other.parameter
    }
    /// Compares both fields using native floating equality; `pave.is_equal(&pave)`.
    pub fn is_equal(&self, other: &Self) -> bool {
        self == other
    }
    /// Preserves native hashing of raw parameter bits, including signed zero; `pave.hash_code()`.
    pub fn hash_code(&self) -> usize {
        let hashes: [usize; 2] = [hash(&self.index), hash(&self.parameter)];
        let bytes: Vec<u8> = hashes.into_iter().flat_map(usize::to_ne_bytes).collect();
        hash_bytes(&bytes)
    }
    /// Writes native Dump text to an injected writer; `pave.dump(&mut String::new())?`.
    pub fn dump(&self, out: &mut dyn std::fmt::Write) -> std::fmt::Result {
        write!(out, " {{ {} {:<7.3}}}", self.index, self.parameter)
    }
}
