//! Shape-interference records corresponding to `BOPDS_Interf` and vertex subtypes.
use std::ops::{Deref, DerefMut};

/// Shared shape indices carried by every interference record; use `InterfVV::default()`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Interference {
    index1: i32,
    index2: i32,
    index_new: i32,
}

impl Default for Interference {
    fn default() -> Self {
        Self {
            index1: -1,
            index2: -1,
            index_new: -1,
        }
    }
}

impl Interference {
    /// Replaces both source indices; `record.set_indices(2, 5)`.
    pub fn set_indices(&mut self, first: i32, second: i32) {
        self.index1 = first;
        self.index2 = second;
    }

    /// Reads both source indices in argument order; `record.indices() == (2, 5)`.
    pub fn indices(&self) -> (i32, i32) {
        (self.index1, self.index2)
    }

    /// Replaces the first source index; `record.set_index1(2)`.
    pub fn set_index1(&mut self, index: i32) {
        self.index1 = index;
    }

    /// Replaces the second source index; `record.set_index2(5)`.
    pub fn set_index2(&mut self, index: i32) {
        self.index2 = index;
    }

    /// Reads the first source index; `record.index1()`.
    pub fn index1(&self) -> i32 {
        self.index1
    }

    /// Reads the second source index; `record.index2()`.
    pub fn index2(&self) -> i32 {
        self.index2
    }

    /// Returns the opposite source index, or `-1` when `index` is absent.
    pub fn opposite_index(&self, index: i32) -> i32 {
        if index == self.index1 {
            return self.index2;
        }
        if index == self.index2 {
            return self.index1;
        }
        -1
    }

    /// Reports whether either source index equals `index`; `record.contains(5)`.
    pub fn contains(&self, index: i32) -> bool {
        self.index1 == index || self.index2 == index
    }

    /// Replaces the generated shape index; `record.set_index_new(9)`.
    pub fn set_index_new(&mut self, index: i32) {
        self.index_new = index;
    }

    /// Reads the generated shape index; `record.index_new()`.
    pub fn index_new(&self) -> i32 {
        self.index_new
    }

    /// Writes the generated index and reports whether it is nonnegative; `record.has_index_new_into(&mut index)`.
    pub fn has_index_new_into(&self, index: &mut i32) -> bool {
        *index = self.index_new;
        self.index_new >= 0
    }

    /// Uses the native sentinel test (`-1` alone means absent); `record.has_index_new()`.
    pub fn has_index_new(&self) -> bool {
        self.index_new != -1
    }

    /// Returns the generated index unless its sentinel is exactly `-1`; `record.get_index_new() == None`.
    pub fn get_index_new(&self) -> Option<i32> {
        (self.index_new != -1).then_some(self.index_new)
    }
}

/// Vertex/vertex interference carrying only common indices; `InterfVV::default()`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct InterfVV(Interference);

/// Vertex/edge interference carrying a curve parameter; `InterfVE::default()`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct InterfVE {
    base: Interference,
    parameter: f64,
}

/// Vertex/face interference carrying surface parameters; `InterfVF::default()`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct InterfVF {
    base: Interference,
    uv: (f64, f64),
}

impl InterfVE {
    /// Replaces the vertex parameter on the edge; `record.set_parameter(0.25)`.
    pub fn set_parameter(&mut self, parameter: f64) {
        self.parameter = parameter;
    }

    /// Reads the vertex parameter on the edge; `record.parameter()`.
    pub fn parameter(&self) -> f64 {
        self.parameter
    }
}

impl InterfVF {
    /// Replaces the vertex UV parameters on the face; `record.set_uv(0.25, 0.75)`.
    pub fn set_uv(&mut self, u: f64, v: f64) {
        self.uv = (u, v);
    }

    /// Reads the vertex UV parameters in U/V order; `record.uv() == (0.25, 0.75)`.
    pub fn uv(&self) -> (f64, f64) {
        self.uv
    }
}

impl Deref for InterfVV {
    type Target = Interference;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for InterfVV {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Deref for InterfVE {
    type Target = Interference;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl DerefMut for InterfVE {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

impl Deref for InterfVF {
    type Target = Interference;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl DerefMut for InterfVF {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
