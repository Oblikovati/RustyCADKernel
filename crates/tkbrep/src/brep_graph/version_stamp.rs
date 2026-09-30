// Rust translation of ModelingData/TKBRep/BRepGraph/BRepGraph_VersionStamp.hxx and .cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepGraph_VersionStamp`: a graph item identity with its freshness generations.
use crate::brep_graph::ids::hashing::{combine, hash_i32, hash_u32};
use crate::brep_graph::{ItemUid, RefUid, Uid};
use rustycad_tkernel::standard::hash_bytes;
use rustycad_tkernel::standard::{Guid, Uuid};
use std::hash::{Hash, Hasher};

/// `BRepGraph_VersionStamp::Domain`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum StampDomain {
    /// `Domain::None`.
    #[default]
    None = 0,
    /// `Domain::Node`.
    Node = 1,
    /// `Domain::Reference`.
    Reference = 2,
}

/// `BRepGraph_VersionStamp`: a node or reference UID with the OwnGen and the graph
/// generation at snapshot time; the default is the invalid stamp.
///
/// ```
/// use rustycad_tkbrep::brep_graph::{NodeKind, Uid, VersionStamp};
/// let stamp = VersionStamp::of_node(Uid::new(NodeKind::FACE, 1), 0, 2);
/// assert!(stamp.is_node_stamp() && !stamp.is_ref_stamp());
/// assert!(stamp.is_same_item(&VersionStamp::of_node(Uid::new(NodeKind::FACE, 1), 5, 2)));
/// ```
#[derive(Clone, Copy, Debug, Default)]
pub struct VersionStamp {
    /// `myNodeUID`: the definition identity of a node stamp.
    pub node_uid: Uid,
    /// `myRefUID`: the reference identity of a reference stamp.
    pub ref_uid: RefUid,
    /// `myMutationGen`: OwnGen at snapshot time.
    pub mutation_gen: u32,
    /// `myGeneration`: the graph `Clear()` generation at snapshot time.
    pub generation: u32,
    /// `myDomain`.
    pub domain: StampDomain,
}

impl VersionStamp {
    /// `BRepGraph_VersionStamp(theUID, theMutationGen, theGeneration)`.
    pub fn of_node(uid: Uid, mutation_gen: u32, generation: u32) -> Self {
        Self {
            node_uid: uid,
            mutation_gen,
            generation,
            domain: StampDomain::Node,
            ..Self::default()
        }
    }

    /// `BRepGraph_VersionStamp(theRefUID, theMutationGen, theGeneration)`.
    pub fn of_ref(uid: RefUid, mutation_gen: u32, generation: u32) -> Self {
        Self {
            ref_uid: uid,
            mutation_gen,
            generation,
            domain: StampDomain::Reference,
            ..Self::default()
        }
    }

    /// `IsValid()`: a valid identity in the stamp's domain.
    pub fn is_valid(&self) -> bool {
        match self.domain {
            StampDomain::Node => self.node_uid.is_valid(),
            StampDomain::Reference => self.ref_uid.is_valid(),
            StampDomain::None => self.node_uid.is_valid() || self.ref_uid.is_valid(),
        }
    }

    /// `IsNodeStamp()`.
    pub fn is_node_stamp(&self) -> bool {
        match self.domain {
            StampDomain::Node => self.node_uid.is_valid(),
            StampDomain::None => self.node_uid.is_valid() && !self.ref_uid.is_valid(),
            StampDomain::Reference => false,
        }
    }

    /// `IsRefStamp()`.
    pub fn is_ref_stamp(&self) -> bool {
        match self.domain {
            StampDomain::Reference => self.ref_uid.is_valid(),
            StampDomain::None => self.ref_uid.is_valid() && !self.node_uid.is_valid(),
            StampDomain::Node => false,
        }
    }

    /// `ItemUID()`: the active identity as a generic item UID.
    pub fn item_uid(&self) -> ItemUid {
        match self.domain {
            StampDomain::Node => ItemUid::node(self.node_uid.kind, self.node_uid.counter as usize),
            StampDomain::Reference => {
                ItemUid::reference(self.ref_uid.kind, self.ref_uid.counter as usize)
            }
            StampDomain::None => ItemUid::invalid(),
        }
    }

    /// `IsSameItem(theOther)`: same domain and UID, whatever the generations.
    pub fn is_same_item(&self, other: &Self) -> bool {
        if self.domain != other.domain {
            return false;
        }
        self.same_uids(other)
    }

    // The UID comparison of `operator==` and `IsSameItem` for the (equal) domain.
    fn same_uids(&self, other: &Self) -> bool {
        match self.domain {
            StampDomain::Node => self.node_uid == other.node_uid,
            StampDomain::Reference => self.ref_uid == other.ref_uid,
            StampDomain::None => self.node_uid == other.node_uid && self.ref_uid == other.ref_uid,
        }
    }

    /// `ToGUID(theGraphGUID)`: four Murmur hashes of the four slices of the packed
    /// 37-byte buffer, each truncated to 32 bits.
    pub fn to_guid(&self, graph_guid: Guid) -> Guid {
        let buffer = self.guid_buffer(graph_guid.to_uuid());
        let quarter = buffer.len() / 4;
        let slices = [
            &buffer[..quarter],
            &buffer[quarter..quarter * 2],
            &buffer[quarter * 2..quarter * 3],
            &buffer[quarter * 3..],
        ];
        let mut bytes = [0_u8; 16];
        for (slot, slice) in bytes.chunks_mut(4).zip(slices) {
            slot.copy_from_slice(&(hash_bytes(slice) as u32).to_ne_bytes());
        }
        Guid::from_uuid(uuid_from_bytes(&bytes))
    }

    // The native `aBuffer`: the UUID, the domain byte, the 8-byte counter, the kind, the
    // OwnGen and the generation, in native byte order.
    fn guid_buffer(&self, graph: Uuid) -> Vec<u8> {
        let (counter, kind): (u64, i32) = match self.domain {
            StampDomain::Node => (u64::from(self.node_uid.counter), self.node_uid.kind.0),
            StampDomain::Reference => (u64::from(self.ref_uid.counter), self.ref_uid.kind.0),
            StampDomain::None => (0, 0),
        };
        let mut buffer = uuid_bytes(graph).to_vec();
        buffer.push(self.domain as u8);
        buffer.extend_from_slice(&counter.to_ne_bytes());
        buffer.extend_from_slice(&kind.to_ne_bytes());
        buffer.extend_from_slice(&self.mutation_gen.to_ne_bytes());
        buffer.extend_from_slice(&self.generation.to_ne_bytes());
        buffer
    }

    /// `HashValue()`: the domain, the active UID hash and the two generations.
    pub fn hash_value(&self) -> usize {
        let uid_hash = match self.domain {
            StampDomain::Node => self.node_uid.hash_value(),
            StampDomain::Reference => self.ref_uid.hash_value(),
            StampDomain::None => hash_i32(0),
        };
        combine(&[
            hash_i32(self.domain as i32),
            uid_hash,
            hash_u32(self.mutation_gen),
            hash_u32(self.generation),
        ])
    }
}

impl PartialEq for VersionStamp {
    /// `operator==`: same domain, UID and generations; two invalid stamps are equal.
    fn eq(&self, other: &Self) -> bool {
        if !self.is_valid() && !other.is_valid() {
            return true;
        }
        if self.domain != other.domain {
            return false;
        }
        if self.mutation_gen != other.mutation_gen || self.generation != other.generation {
            return false;
        }
        self.same_uids(other)
    }
}

impl Eq for VersionStamp {}

impl Hash for VersionStamp {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_usize(self.hash_value());
    }
}

// The 16 object bytes of a `Standard_UUID` (`Data1`, `Data2`, `Data3`, `Data4`).
fn uuid_bytes(uuid: Uuid) -> [u8; 16] {
    let mut bytes = [0_u8; 16];
    bytes[..4].copy_from_slice(&uuid.data1.to_ne_bytes());
    bytes[4..6].copy_from_slice(&uuid.data2.to_ne_bytes());
    bytes[6..8].copy_from_slice(&uuid.data3.to_ne_bytes());
    bytes[8..].copy_from_slice(&uuid.data4);
    bytes
}

// The `Standard_UUID` whose object bytes are `bytes` (the native `memcpy` into it).
fn uuid_from_bytes(bytes: &[u8; 16]) -> Uuid {
    let mut data4 = [0_u8; 8];
    data4.copy_from_slice(&bytes[8..]);
    Uuid {
        data1: u32::from_ne_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
        data2: u16::from_ne_bytes([bytes[4], bytes[5]]),
        data3: u16::from_ne_bytes([bytes[6], bytes[7]]),
        data4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::brep_graph::{NodeKind, RefKind};

    #[test]
    fn stamps_compare_by_domain_uid_and_generations() {
        let node = VersionStamp::of_node(Uid::new(NodeKind::EDGE, 2), 1, 0);
        let moved = VersionStamp::of_node(Uid::new(NodeKind::EDGE, 2), 2, 0);
        let reference = VersionStamp::of_ref(RefUid::new(RefKind::WIRE, 2), 1, 0);
        assert_ne!(node, moved);
        assert!(node.is_same_item(&moved));
        assert!(!node.is_same_item(&reference));
        assert_eq!(
            VersionStamp::default(),
            VersionStamp::of_node(Uid::invalid(), 3, 4)
        );
        assert_eq!(reference.item_uid(), ItemUid::reference(RefKind::WIRE, 2));
        assert_ne!(node.hash_value(), moved.hash_value());
    }

    #[test]
    fn guid_packs_the_uuid_round_trip() {
        let uuid = Uuid {
            data1: 0x0102_0304,
            data2: 0x0506,
            data3: 0x0708,
            data4: [9, 10, 11, 12, 13, 14, 15, 16],
        };
        assert_eq!(uuid_from_bytes(&uuid_bytes(uuid)), uuid);
        let stamp = VersionStamp::of_node(Uid::new(NodeKind::FACE, 1), 0, 0);
        assert_eq!(stamp.guid_buffer(uuid).len(), 37);
        assert_ne!(stamp.to_guid(Guid::new()), Guid::new());
    }
}
