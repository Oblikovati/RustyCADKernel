// Rust translation of ModelingData/TKBRep/BRepGraph/BRepGraph_UID.hxx and
// BRepGraph_RefUID.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepGraph_UID` and `BRepGraph_RefUID`: persistent `(Kind, Counter)` identities.
//!
//! Counter 0 is the invalid sentinel; two invalid UIDs are equal whatever their kind,
//! and the hash of an invalid UID is `opencascade::hash(0)`, consistent with that.
use super::hashing::{combine, hash_i32, hash_u32};
use super::node_id::NodeKind;
use super::ref_id::RefKind;
use std::cmp::Ordering;
use std::hash::{Hash, Hasher};

/// `BRepGraph_UID`: the persistent identity of a definition node.
///
/// ```
/// use rustycad_tkbrep::brep_graph::{NodeKind, Uid};
/// let uid = Uid::new(NodeKind::FACE, 3);
/// assert!(uid.is_valid() && uid.is_topology());
/// assert_eq!(Uid::invalid(), Uid::new(NodeKind::EDGE, 0));
/// ```
#[derive(Clone, Copy, Debug, Default)]
pub struct Uid {
    /// `Kind`.
    pub kind: NodeKind,
    /// `Counter`: 0 is the invalid sentinel.
    pub counter: u32,
}

impl Uid {
    /// `BRepGraph_UID(theKind, theCounter)`.
    pub const fn new(kind: NodeKind, counter: u32) -> Self {
        Self { kind, counter }
    }

    /// `Invalid()`.
    pub const fn invalid() -> Self {
        Self::new(NodeKind::SOLID, 0)
    }

    /// `IsValid()`: a non-zero counter and a valid kind.
    pub fn is_valid(self) -> bool {
        self.counter > 0 && NodeKind::is_valid_kind(self.kind)
    }

    /// `IsTopology()`.
    pub fn is_topology(self) -> bool {
        self.is_valid() && NodeKind::is_topology_kind(self.kind)
    }

    /// `IsAssembly()`.
    pub fn is_assembly(self) -> bool {
        self.is_valid() && NodeKind::is_assembly_kind(self.kind)
    }

    /// `HashValue()`, compatible with `==`.
    pub fn hash_value(self) -> usize {
        kind_counter_hash(self.kind.0, self.counter)
    }
}

/// `BRepGraph_RefUID`: the persistent identity of a reference entry.
///
/// ```
/// use rustycad_tkbrep::brep_graph::{RefKind, RefUid};
/// assert!(RefUid::new(RefKind::WIRE, 1) < RefUid::new(RefKind::WIRE, 2));
/// ```
#[derive(Clone, Copy, Debug, Default)]
pub struct RefUid {
    /// `Kind`.
    pub kind: RefKind,
    /// `Counter`: 0 is the invalid sentinel.
    pub counter: u32,
}

impl RefUid {
    /// `BRepGraph_RefUID(theKind, theCounter)`.
    pub const fn new(kind: RefKind, counter: u32) -> Self {
        Self { kind, counter }
    }

    /// `Invalid()`.
    pub const fn invalid() -> Self {
        Self::new(RefKind::SHELL, 0)
    }

    /// `IsValid()`.
    pub fn is_valid(self) -> bool {
        self.counter > 0 && RefKind::is_valid_kind(self.kind)
    }

    /// `HashValue()`.
    pub fn hash_value(self) -> usize {
        kind_counter_hash(self.kind.0, self.counter)
    }
}

// The shared body of `BRepGraph_UID::HashValue` and `BRepGraph_RefUID::HashValue`.
fn kind_counter_hash(kind: i32, counter: u32) -> usize {
    if counter == 0 {
        return hash_i32(0);
    }
    combine(&[hash_i32(kind), hash_u32(counter)])
}

// The shared `operator==`: two invalid UIDs are equal, an invalid one equals no valid one.
fn kind_counter_eq(left: (i32, u32), right: (i32, u32)) -> bool {
    if left.1 == 0 || right.1 == 0 {
        return (left.1 == 0) == (right.1 == 0);
    }
    left == right
}

macro_rules! uid_traits {
    ($type:ident) => {
        impl PartialEq for $type {
            fn eq(&self, other: &Self) -> bool {
                kind_counter_eq((self.kind.0, self.counter), (other.kind.0, other.counter))
            }
        }
        impl Eq for $type {}
        impl PartialOrd for $type {
            fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
                Some(self.cmp(other))
            }
        }
        // `operator<`: by kind, then counter (not consistent with the invalid equality,
        // as natively).
        impl Ord for $type {
            fn cmp(&self, other: &Self) -> Ordering {
                (self.kind, self.counter).cmp(&(other.kind, other.counter))
            }
        }
        impl Hash for $type {
            fn hash<H: Hasher>(&self, state: &mut H) {
                state.write_usize(self.hash_value());
            }
        }
    };
}

uid_traits!(Uid);
uid_traits!(RefUid);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_uids_are_equal_and_hash_alike() {
        let a = Uid::new(NodeKind::EDGE, 0);
        let b = Uid::new(NodeKind::FACE, 0);
        assert_eq!(a, b);
        assert_eq!(a.hash_value(), b.hash_value());
        assert_ne!(Uid::new(NodeKind::EDGE, 1), Uid::new(NodeKind::FACE, 1));
        assert!(Uid::new(NodeKind::SOLID, 9) < Uid::new(NodeKind::SHELL, 1));
        assert!(Uid::new(NodeKind::PRODUCT, 1).is_assembly());
        assert!(!RefUid::invalid().is_valid());
        assert_eq!(RefUid::new(RefKind::FACE, 0), RefUid::invalid());
    }
}
