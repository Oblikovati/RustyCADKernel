//! Real heap regions addressed by their numeric start, so the manager can keep OCCT's
//! intrusive headers and free-list links inside the memory it hands out.
#[cfg(unix)]
use super::mapped_region::MappedRegion;
use std::collections::BTreeMap;

enum RegionStorage {
    Heap {
        bytes: Vec<u8>,
        offset: usize,
    },
    #[cfg(unix)]
    Mapped(MappedRegion),
}

/// One system allocation: `malloc`/`calloc` or an anonymous mapping.
struct Region {
    storage: RegionStorage,
    length: usize,
}
impl Region {
    fn bytes(&self) -> &[u8] {
        match &self.storage {
            RegionStorage::Heap { bytes, offset } => &bytes[*offset..*offset + self.length],
            #[cfg(unix)]
            RegionStorage::Mapped(mapping) => mapping.bytes(),
        }
    }
    fn bytes_mut(&mut self) -> &mut [u8] {
        match &mut self.storage {
            RegionStorage::Heap { bytes, offset } => &mut bytes[*offset..*offset + self.length],
            #[cfg(unix)]
            RegionStorage::Mapped(mapping) => mapping.bytes_mut(),
        }
    }
    fn release_prefix(&mut self, length: usize) -> bool {
        match &mut self.storage {
            RegionStorage::Heap { offset, .. } => {
                *offset += length;
                true
            }
            #[cfg(unix)]
            RegionStorage::Mapped(mapping) => mapping.release_prefix(length),
        }
    }
}

/// Every region a manager obtained from the system, keyed by start address; `space.reserve(64, 16)`.
///
/// Each region keeps stable owned backing, either a zeroed heap buffer or OS mapping. Words are
/// stored in native byte order, as `size_t`.
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
        let address = address + offset;
        self.regions.insert(
            address,
            Region {
                storage: RegionStorage::Heap {
                    bytes: storage,
                    offset,
                },
                length,
            },
        );
        Some(address)
    }
    /// Obtains page-aligned bytes with an OS mapping where supported.
    pub(super) fn reserve_mapped(&mut self, length: usize, alignment: usize) -> Option<usize> {
        #[cfg(unix)]
        {
            self.insert_mapped(length, alignment)
        }
        #[cfg(not(unix))]
        {
            self.reserve(length, alignment)
        }
    }
    #[cfg(unix)]
    fn insert_mapped(&mut self, length: usize, alignment: usize) -> Option<usize> {
        let mapping = MappedRegion::new(length)?;
        let address = mapping.address();
        if address % alignment != 0 {
            return None;
        }
        self.regions.insert(
            address,
            Region {
                storage: RegionStorage::Mapped(mapping),
                length,
            },
        );
        Some(address)
    }
    /// Returns a region to the system; `space.release(start)`.
    pub(super) fn release(&mut self, start: usize) -> bool {
        self.regions.remove(&start).is_some()
    }
    /// Unmaps a prefix and relocates the tracked suffix; `space.trim_prefix(base, 4096)`.
    pub(super) fn trim_prefix(&mut self, start: usize, length: usize) -> Option<(usize, usize)> {
        let region_length = self.region_len(start)?;
        if length > region_length {
            return None;
        }
        if length == 0 {
            return Some((start, region_length));
        }
        let region = self.regions.remove(&start)?;
        self.trim_region(start, region, length)
    }
    fn trim_region(
        &mut self,
        start: usize,
        mut region: Region,
        length: usize,
    ) -> Option<(usize, usize)> {
        if length == region.length {
            return Some((0, 0));
        }
        if !region.release_prefix(length) {
            self.regions.insert(start, region);
            return None;
        }
        region.length -= length;
        let suffix = start + length;
        let suffix_length = region.length;
        self.regions.insert(suffix, region);
        Some((suffix, suffix_length))
    }
    /// Returns a region's complete mapped length only at its base address.
    pub(super) fn region_len(&self, start: usize) -> Option<usize> {
        self.regions.get(&start).map(|region| region.length)
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
