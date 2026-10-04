// Rust translation of FoundationClasses/TKMath/Poly/Poly_MakeLoops.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

use super::{
    HeapOfInteger, Link, LinkFlag, LinkHasher, LinkTable, ListOfLink, LoopWay, contour::Contour,
    get_link, result_code,
};
use rustycad_tkernel::{ncollection::PackedMap, standard::Failure};
/// Nothing was consumed, so the native loop would repeat this step forever (seen when
/// helper links and stored links disagree on node order).
fn no_progress(start: i32) -> Failure {
    Failure::with_type(
        "Standard_ProgramError",
        &format!(
            "Poly_MakeLoops::Perform: start link {start} consumed nothing; the native loop never ends"
        ),
        "",
    )
}
/// The loop builder over a [`LoopWay`] strategy; `MakeLoops::new(LeftWay3D::new(&helper))`.
pub struct MakeLoops<W: LoopWay> {
    pub(super) way: W,
    pub(super) links: LinkTable,
    pub(super) loops: Vec<ListOfLink>,
    pub(super) start_indices: HeapOfInteger,
    pub(super) hang_indices: PackedMap<i32>,
}
impl<W: LoopWay> MakeLoops<W> {
    /// An empty builder; `MakeLoops::new(way)`.
    pub fn new(way: W) -> Self {
        Self {
            way,
            links: LinkTable::with_hasher(LinkHasher, 4000),
            loops: Vec::new(),
            start_indices: HeapOfInteger::new(4000),
            hang_indices: PackedMap::default(),
        }
    }
    /// Returns to the initial state with a new strategy (helper); `builder.reset(way)`.
    pub fn reset(&mut self, way: W) {
        self.way = way;
        self.links.clear(false);
        self.loops.clear();
        self.start_indices.clear();
    }
    /// The strategy, as `getHelper`; `builder.way()`.
    pub const fn way(&self) -> &W {
        &self.way
    }
    /// Adds a link or merges its flags into an equal one; a link on one node is ignored;
    /// `builder.add_link(&Link::new(1, 2))?`.
    pub fn add_link(&mut self, link: &Link) -> Result<(), Failure> {
        if link.node1 == link.node2 {
            return Ok(());
        }
        let index = self.links.add(*link);
        let mut stored = *self.stored(index)?;
        stored.flags |= link.flags;
        self.links.substitute(index, stored)
    }
    /// Replaces a link (added first when absent) by another one, e.g. to change node order;
    /// `builder.replace_link(&old, &new)?`.
    pub fn replace_link(&mut self, link: &Link, new_link: &Link) -> Result<(), Failure> {
        if new_link.node1 == new_link.node2 {
            return Ok(());
        }
        let index = self.links.add(*link);
        // A null link is stored first, as natively, so that the new key is looked up alone.
        self.links.substitute(index, Link::default())?;
        self.links.substitute(index, *new_link)
    }
    /// Replaces the flags of a stored link and returns its old orientation bits;
    /// `builder.set_link_orientation(&link, LinkFlag::None)?`.
    pub fn set_link_orientation(
        &mut self,
        link: &Link,
        orientation: LinkFlag,
    ) -> Result<LinkFlag, Failure> {
        let index = self.links.find_index(link);
        if index == 0 {
            return Ok(LinkFlag::None);
        }
        let mut stored = *self.stored(index)?;
        let old =
            LinkFlag::from_native(stored.flags & LinkFlag::Both as i32).unwrap_or(LinkFlag::None);
        stored.flags = orientation as i32;
        self.links.substitute(index, stored)?;
        Ok(old)
    }
    /// The stored link equal to `link`, or the null link; `builder.find_link(&link)`.
    pub fn find_link(&self, link: &Link) -> Link {
        let index = self.links.find_index(link);
        self.stored(index).copied().unwrap_or_default()
    }
    pub(super) fn stored(&self, index: usize) -> Result<&Link, Failure> {
        self.links.find_key(index)
    }
    /// The link of a signed index, as protected `getLink`; invalid indices fail;
    /// `builder.get_link(-1)?`.
    pub fn get_link(&self, segment: i32) -> Result<Link, Failure> {
        get_link(&self.links, segment)
    }
    /// Builds loops in two passes (the second one restarts from hanging links) and returns
    /// `result_code` bits; `builder.perform()?`.
    pub fn perform(&mut self) -> Result<i32, Failure> {
        self.fill_start_indices()?;
        for pass in 0..2 {
            self.hang_indices.clear();
            if !self.search_loops()? {
                return Ok(result_code::FAILURE);
            }
            if pass == 0 {
                let hanging: Vec<i32> = self.hang_indices.iter().collect();
                hanging
                    .into_iter()
                    .for_each(|index| self.start_indices.add(index));
            }
        }
        let mut result = 0;
        if !self.loops.is_empty() {
            result |= result_code::LOOPS_DONE;
        }
        if !self.hang_indices.is_empty() {
            result |= result_code::HANGING_LINKS;
        }
        Ok(result)
    }
    // Every allowed orientation of every link, signed by direction, in index order.
    fn fill_start_indices(&mut self) -> Result<(), Failure> {
        self.start_indices.clear();
        for index in 1..=self.links.len() as i32 {
            let flags = self.stored(index as usize)?.flags;
            if flags & LinkFlag::Fwd as i32 != 0 {
                self.start_indices.add(index);
            }
            if flags & LinkFlag::Rev as i32 != 0 {
                self.start_indices.add(-index);
            }
        }
        Ok(())
    }
    // One pass of the main loop; false when a contour search fails.
    fn search_loops(&mut self) -> Result<bool, Failure> {
        while !self.start_indices.is_empty() {
            let start = self.start_indices.top()?;
            let remaining = self.start_indices.extent();
            let contour = self.find_contour(start)?;
            if contour.start == 0 {
                return Ok(false);
            }
            if contour.start <= contour.len() {
                self.accept_contour(&contour)?;
            }
            if contour.start > 1 {
                self.mark_hanging_part(&contour, start)?;
            }
            if self.start_indices.extent() == remaining {
                return Err(no_progress(start));
            }
        }
        Ok(true)
    }
    // A closed contour hangs from the first node of its start link; an open one from the
    // last node of its last link.
    fn mark_hanging_part(&mut self, contour: &Contour, start: i32) -> Result<(), Failure> {
        if contour.start <= contour.len() {
            return self.mark_hang_chain(self.first_node(start)?, start);
        }
        let last = contour.at(contour.start - 1);
        self.mark_hang_chain(self.last_node(last)?, last)
    }
    /// Number of loops; `builder.get_nb_loops()`.
    pub fn get_nb_loops(&self) -> i32 {
        self.loops.len() as i32
    }
    /// The loop at a one-based index; `builder.get_loop(1)?`.
    pub fn get_loop(&self, index: i32) -> Result<&ListOfLink, Failure> {
        usize::try_from(index - 1)
            .ok()
            .and_then(|slot| self.loops.get(slot))
            .ok_or_else(|| {
                Failure::out_of_range(&format!(
                    "Poly_MakeLoops::GetLoop: index {index}; expected [1, {}]",
                    self.loops.len()
                ))
            })
    }
    /// Number of hanging links; `builder.get_nb_hanging()`.
    pub fn get_nb_hanging(&self) -> i32 {
        self.hang_indices.extent()
    }
    /// Appends hanging links in map order; invalid internal indices return a range failure;
    /// `builder.get_hanging_links(&mut list)?`.
    pub fn get_hanging_links(&self, links: &mut ListOfLink) -> Result<(), Failure> {
        for index in self.hang_indices.iter() {
            links.append(self.get_link(index)?);
        }
        Ok(())
    }
}
