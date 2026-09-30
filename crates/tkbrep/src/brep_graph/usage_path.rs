// Rust translation of ModelingData/TKBRep/BRepGraph/BRepGraph_UsagePath.hxx and .cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepGraph_UsagePath`: the explicit steps of one usage from a traversal root.
use crate::brep_graph::ids::hashing::{combine, hash_i32, hash_kind_index, hash_usize};
use crate::brep_graph::{NodeId, RefId};
use std::hash::{Hash, Hasher};

/// `BRepGraph_UsagePath::Step`: the node reached, the reference it was reached through
/// (invalid for structural links) and the sibling order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UsageStep {
    /// `Node`.
    pub node: NodeId,
    /// `Ref`.
    pub reference: RefId,
    /// `StepIndex` (-1 by default).
    pub step_index: i32,
}

impl Default for UsageStep {
    fn default() -> Self {
        Self {
            node: NodeId::default(),
            reference: RefId::default(),
            step_index: -1,
        }
    }
}

impl UsageStep {
    // The three field hashes of a step, in the native combination order.
    fn field_hashes(&self) -> [usize; 3] {
        [
            hash_kind_index(self.node.kind.0, self.node.index),
            hash_kind_index(self.reference.kind.0, self.reference.index),
            hash_i32(self.step_index),
        ]
    }
}

/// `BRepGraph_UsagePath`: an ordered sequence of steps.
///
/// ```
/// use rustycad_tkbrep::brep_graph::{NodeId, NodeKind, UsagePath, UsageStep};
/// let mut path = UsagePath::new();
/// path.append(UsageStep { node: NodeId::new(NodeKind::SOLID, 0), ..Default::default() });
/// assert_eq!(path.size(), 1);
/// assert_eq!(path, path.clone());
/// ```
#[derive(Clone, Debug, Default)]
pub struct UsagePath {
    steps: Vec<UsageStep>,
}

impl UsagePath {
    /// `BRepGraph_UsagePath()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `BRepGraph_UsagePath(theCapacity)`.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            steps: Vec::with_capacity(capacity),
        }
    }

    /// `Size()`.
    pub fn size(&self) -> usize {
        self.steps.len()
    }

    /// `IsEmpty()`.
    pub fn is_empty(&self) -> bool {
        self.steps.is_empty()
    }

    /// `Value(theIdx)`; an index out of range panics with the size (undefined natively).
    pub fn value(&self, index: usize) -> &UsageStep {
        &self.steps[index]
    }

    /// `First()`; panics on an empty path (undefined natively).
    pub fn first(&self) -> &UsageStep {
        &self.steps[0]
    }

    /// `Last()`; panics on an empty path (undefined natively).
    pub fn last(&self) -> &UsageStep {
        &self.steps[self.steps.len() - 1]
    }

    /// `Append(theStep)`.
    pub fn append(&mut self, step: UsageStep) {
        self.steps.push(step);
    }

    /// `InsertBefore(theIdx, theStep)`.
    pub fn insert_before(&mut self, index: usize, step: UsageStep) {
        self.steps.insert(index, step);
    }

    /// `Clear()`.
    pub fn clear(&mut self) {
        self.steps.clear();
    }

    /// `IsEqual(theOther)` and `operator==`: same steps in the same order.
    pub fn is_equal(&self, other: &Self) -> bool {
        self.steps == other.steps
    }

    /// `HashCode()`: the size and the first and last steps (O(1)).
    pub fn hash_code(&self) -> usize {
        let size = self.size();
        if size == 0 {
            return hash_usize(size);
        }
        let mut words = vec![hash_usize(size)];
        words.extend(self.first().field_hashes());
        if size > 1 {
            words.extend(self.last().field_hashes());
        }
        combine(&words)
    }
}

impl PartialEq for UsagePath {
    fn eq(&self, other: &Self) -> bool {
        self.is_equal(other)
    }
}

impl Eq for UsagePath {}

impl Hash for UsagePath {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_usize(self.hash_code());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::brep_graph::{NodeKind, RefKind};

    fn step(index: u32) -> UsageStep {
        UsageStep {
            node: NodeId::new(NodeKind::FACE, index),
            reference: RefId::new(RefKind::FACE, index),
            step_index: index as i32,
        }
    }

    #[test]
    fn paths_compare_and_hash_by_ends() {
        let mut path = UsagePath::with_capacity(3);
        path.append(step(0));
        path.append(step(2));
        path.insert_before(1, step(1));
        assert_eq!(path.value(1), &step(1));
        let mut other = UsagePath::new();
        other.append(step(0));
        other.append(step(5));
        other.append(step(2));
        assert_ne!(path, other);
        assert_eq!(path.hash_code(), other.hash_code());
        path.clear();
        assert!(path.is_empty());
        assert_eq!(UsageStep::default().step_index, -1);
    }
}
