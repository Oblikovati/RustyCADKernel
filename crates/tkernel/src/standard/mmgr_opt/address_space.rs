//! Real heap regions addressed by their numeric start, so the manager can keep OCCT's
//! intrusive headers and free-list links inside the memory it hands out.
use std::collections::BTreeMap;

/// One system allocation: `malloc`/`calloc` or an anonymous mapping.
struct Region {
    storage: Vec<u8>,
    offset: usize,
    length: usize,
}
impl Region {
    fn bytes(&self) -> &[u8] {
        &self.storage[self.offset..self.offset + self.length]
    }
    fn bytes_mut(&mut self) -> &mut [u8] {
        &mut self.storage[self.offset..self.offset + self.length]
    }
}

/// Every region a manager obtained from the system, keyed by start address; `space.reserve(64, 16)`.
///
/// Each region is a zero-filled heap buffer that never moves while it lives, so its start
/// address is a stable, real address. Words are stored in native byte order, as `size_t`.
#[derive(Default)]
pub(super) struct AddressSpace {
    regions: BTreeMap<usize, Region>,
}
impl AddressSpace {
    /// Obtains `length` zeroed bytes starting on an `alignment` boundary; `space.reserve(4096, 4096)`.
    pub(super) fn reserve(&mut self, length: usize, alignment: usize) -> Option<usize> {
        let storage = crate::standard::memory::zeroed_bytes(length.checked_add(alignment - 1)?)?;
        let address = storage.as_ptr() as usize;
        let offset = (alignment - address % alignment) % alignment;
        self.regions.insert(
            address + offset,
            Region {
                storage,
                offset,
                length,
            },
        );
        Some(address + offset)
    }
    /// Returns a region to the system; `space.release(start)`.
    pub(super) fn release(&mut self, start: usize) -> bool {
        self.regions.remove(&start).is_some()
    }
    /// Borrows `length` bytes at `address` inside one region; `space.bytes(address, 8)`.
    pub(super) fn bytes(&self, address: usize, length: usize) -> Option<&[u8]> {
        let (start, region) = self.regions.range(..=address).next_back()?;
        let offset = address - start;
        region.bytes().get(offset..offset.checked_add(length)?)
    }
    /// Borrows `length` writable bytes at `address` inside one region; `space.bytes_mut(address, 8)`.
    pub(super) fn bytes_mut(&mut self, address: usize, length: usize) -> Option<&mut [u8]> {
        let (start, region) = self.regions.range_mut(..=address).next_back()?;
        let offset = address - *start;
        region
            .bytes_mut()
            .get_mut(offset..offset.checked_add(length)?)
    }
    /// Reads the `size_t` stored at `address`; `space.read_word(block)`.
    pub(super) fn read_word(&self, address: usize) -> Option<usize> {
        let bytes = self.bytes(address, WORD)?;
        Some(usize::from_ne_bytes(bytes.try_into().ok()?))
    }
    /// Stores a `size_t` at `address`; `space.write_word(block, round_size)`.
    pub(super) fn write_word(&mut self, address: usize, value: usize) -> Option<()> {
        self.bytes_mut(address, WORD)?
            .copy_from_slice(&value.to_ne_bytes());
        Some(())
    }
}

/// `sizeof(size_t)`: the header and link size of every block.
pub(super) const WORD: usize = size_of::<usize>();
