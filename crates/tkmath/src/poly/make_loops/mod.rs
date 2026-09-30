// Rust translation of FoundationClasses/TKMath/Poly/Poly_MakeLoops.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Poly_MakeLoops`: builds closed loops from a set of oriented links between nodes.
//!
//! The native class is abstract: `Poly_MakeLoops3D` and `Poly_MakeLoops2D` override
//! `chooseLeftWay` and take a helper that knows the links around a node. Here the
//! algorithm is [`MakeLoops`] and the override is a [`LoopWay`] strategy that owns the
//! helper reference; [`LeftWay3D`] and [`LeftWay2D`] are the two native strategies.
mod algo;
mod contour;
mod heap;
mod ways;
pub use algo::MakeLoops;
pub use heap::HeapOfInteger;
use rustycad_tkernel::{
    ncollection::{HashPolicy, IndexedMap, List},
    standard::hash_bytes,
};
pub use ways::{
    LeftWay2D, LeftWay3D, MakeLoops2D, MakeLoops2DHelper, MakeLoops3D, MakeLoops3DHelper,
};
/// `Poly_MakeLoops::LinkFlag`: which orientations of a link may be used.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(i32)]
pub enum LinkFlag {
    /// Excluded from consideration.
    None = 0,
    /// Forward orientation.
    Fwd = 1,
    /// Reversed orientation.
    Rev = 2,
    /// Both orientations.
    Both = 3,
    /// The link is reversed.
    Reversed = 4,
}
impl LinkFlag {
    /// The flag with a native value, `None` for combinations the enum does not name;
    /// `LinkFlag::from_native(3)`.
    pub const fn from_native(value: i32) -> Option<Self> {
        match value {
            0 => Some(Self::None),
            1 => Some(Self::Fwd),
            2 => Some(Self::Rev),
            3 => Some(Self::Both),
            4 => Some(Self::Reversed),
            _ => None,
        }
    }
}
/// `Poly_MakeLoops::ResultCode` bits returned by `perform`.
pub mod result_code {
    /// At least one loop was built.
    pub const LOOPS_DONE: i32 = 1;
    /// Some links hang outside every loop.
    pub const HANGING_LINKS: i32 = 2;
    /// The contour search failed.
    pub const FAILURE: i32 = 4;
}
/// `Poly_MakeLoops::Link`: two nodes and orientation flags; equality ignores direction;
/// `Link::new(1, 2)`.
#[derive(Clone, Copy, Debug, Default)]
pub struct Link {
    /// First node.
    pub node1: i32,
    /// Second node.
    pub node2: i32,
    /// `LinkFlag` bits.
    pub flags: i32,
}
impl Link {
    /// A forward link between two nodes; `Link::new(1, 2)`.
    pub const fn new(node1: i32, node2: i32) -> Self {
        Self {
            node1,
            node2,
            flags: LinkFlag::Fwd as i32,
        }
    }
    /// Toggles the reversed bit; `link.reverse()`.
    pub const fn reverse(&mut self) {
        self.flags ^= LinkFlag::Reversed as i32;
    }
    /// True when the reversed bit is set; `link.is_reversed()`.
    pub const fn is_reversed(&self) -> bool {
        self.flags & LinkFlag::Reversed as i32 != 0
    }
    /// Sets both nodes to 0; `link.nullify()`.
    pub const fn nullify(&mut self) {
        self.node1 = 0;
        self.node2 = 0;
    }
    /// True when either node is 0; `link.is_null()`.
    pub const fn is_null(&self) -> bool {
        self.node1 == 0 || self.node2 == 0
    }
}
impl PartialEq for Link {
    fn eq(&self, other: &Self) -> bool {
        (other.node1 == self.node1 && other.node2 == self.node2)
            || (other.node1 == self.node2 && other.node2 == self.node1)
    }
}
impl Eq for Link {}
/// `Poly_MakeLoops::Hasher`: Murmur hash of the ascending node pair; `LinkHasher::default()`.
#[derive(Clone, Copy, Debug, Default)]
pub struct LinkHasher;
impl HashPolicy<Link> for LinkHasher {
    fn hash(&self, link: &Link) -> usize {
        let (low, high) = if link.node1 > link.node2 {
            (link.node2, link.node1)
        } else {
            (link.node1, link.node2)
        };
        let mut bytes = [0_u8; 8];
        bytes[..4].copy_from_slice(&low.to_ne_bytes());
        bytes[4..].copy_from_slice(&high.to_ne_bytes());
        hash_bytes(&bytes)
    }
    fn equal(&self, first: &Link, second: &Link) -> bool {
        first == second
    }
}
/// `Poly_MakeLoops::ListOfLink` and `Loop`; `ListOfLink::default()`.
pub type ListOfLink = List<Link>;
/// The indexed link set of the algorithm, readable by a [`LoopWay`].
pub type LinkTable = IndexedMap<Link, LinkHasher>;
/// `Poly_MakeLoops::Helper`: the links incident to a node.
pub trait MakeLoopsHelper {
    /// Links adjacent to a node, as `GetAdjacentLinks`; `helper.get_adjacent_links(3)`.
    fn get_adjacent_links(&self, node: i32) -> &ListOfLink;
    /// Hook the native debug build calls after each link change; release builds, and
    /// this port, never call it; `helper.on_add_link(1, &link)`.
    fn on_add_link(&self, _number: i32, _link: &Link) {}
}
/// The overridable part of `Poly_MakeLoops`: its helper and `chooseLeftWay`.
pub trait LoopWay {
    /// The helper that lists adjacent links; `way.helper()`.
    fn helper(&self) -> &dyn MakeLoopsHelper;
    /// Picks the continuation among `candidates` (signed link indices) after `segment` at
    /// `node`; 0 means none; `way.choose_left_way(&links, node, segment, &candidates)`.
    fn choose_left_way(
        &self,
        links: &LinkTable,
        node: i32,
        segment: i32,
        candidates: &[i32],
    ) -> i32;
}
/// The link of a signed index, reversed for a negative index, as `getLink`; a missing
/// index gives the null link; `get_link(&links, -2)`.
pub fn get_link(links: &LinkTable, segment: i32) -> Link {
    let mut link = links
        .find_key(segment.unsigned_abs() as usize)
        .copied()
        .unwrap_or_default();
    if segment < 0 {
        link.reverse();
    }
    link
}
