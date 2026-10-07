// Rust port of ApplicationFramework/TKLCAF/TDF/TDF_IDFilter.hxx, .lxx and .cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TDF_IDFilter`: keeps or ignores attributes by ID.
use super::TdfAttribute;
use rustycad_tkernel::{
    ncollection::{HashPolicy, Map},
    standard::{Guid, Handle},
};
use std::io;

/// `TDF_IDFilterMapSize`: "to avoid too much resizing actions, 23 is a prime number".
const ID_FILTER_MAP_SIZE: usize = 23;

/// `std::hash<Standard_GUID>` and `operator==`, the native default hasher of a GUID map.
#[derive(Clone, Copy, Debug, Default)]
pub struct GuidHashPolicy;
impl HashPolicy<Guid> for GuidHashPolicy {
    fn hash(&self, key: &Guid) -> usize {
        key.hash_code()
    }
    fn equal(&self, first: &Guid, second: &Guid) -> bool {
        first == second
    }
}

/// `NCollection_Map<Standard_GUID>`.
pub type GuidMap = Map<Guid, GuidHashPolicy>;

/// `TDF_IDFilter`: an exclusive (ignore all but the listed IDs) or inclusive (keep all but
/// the listed IDs) filter; not copyable natively (use [`TdfIdFilter::assign`]).
///
/// ```
/// use rustycad_tklcaf::tdf::{TdfIdFilter, TdfTagSource};
/// let mut filter = TdfIdFilter::new(true);
/// filter.keep(&TdfTagSource::get_id());
/// assert!(filter.is_kept(&TdfTagSource::get_id()));
/// assert!(filter.is_ignored(&rustycad_tkernel::standard::Guid::new()));
/// ```
pub struct TdfIdFilter {
    ignore: bool,
    id_map: GuidMap,
}
impl Default for TdfIdFilter {
    /// `TDF_IDFilter()`: ignores everything.
    fn default() -> Self {
        Self::new(true)
    }
}
impl TdfIdFilter {
    /// `TDF_IDFilter(ignoreMode)`.
    pub fn new(ignore_mode: bool) -> Self {
        Self {
            ignore: ignore_mode,
            id_map: GuidMap::default(),
        }
    }
    /// `IgnoreAll(ignore)`: the mode, with an empty list.
    pub fn set_ignore_all(&mut self, ignore: bool) {
        self.ignore = ignore;
        self.id_map.clear(true);
        self.id_map.resize(ID_FILTER_MAP_SIZE);
    }
    /// `IgnoreAll()`: true for an exclusive filter.
    pub fn ignore_all(&self) -> bool {
        self.ignore
    }
    /// `Keep(anID)`.
    pub fn keep(&mut self, id: &Guid) {
        if self.ignore {
            self.id_map.add(*id);
        } else {
            self.id_map.remove(id);
        }
    }
    /// `Keep(anIDList)`.
    pub fn keep_list(&mut self, ids: &[Guid]) {
        if ids.is_empty() {
            return;
        }
        if self.ignore {
            self.add_all(ids);
        } else {
            ids.iter().for_each(|id| {
                self.id_map.remove(id);
            });
        }
    }
    /// `Ignore(anID)`.
    pub fn ignore(&mut self, id: &Guid) {
        if self.ignore {
            self.id_map.remove(id);
        } else {
            self.id_map.add(*id);
        }
    }
    /// `Ignore(anIDList)`.
    pub fn ignore_list(&mut self, ids: &[Guid]) {
        if ids.is_empty() {
            return;
        }
        if self.ignore {
            ids.iter().for_each(|id| {
                self.id_map.remove(id);
            });
        } else {
            self.add_all(ids);
        }
    }
    /// Adds a list after the native `ReSize(n + NbBuckets + 1)`.
    fn add_all(&mut self, ids: &[Guid]) {
        let size = ids.len() + self.id_map.bucket_count() + 1;
        self.id_map.resize(size);
        for id in ids {
            self.id_map.add(*id);
        }
    }
    /// `IsKept(anID)`.
    pub fn is_kept(&self, id: &Guid) -> bool {
        self.ignore == self.id_map.contains(id)
    }
    /// `IsKept(anAtt)`: by the attribute's ID; false for a null attribute (a crash
    /// natively).
    pub fn is_kept_attribute(&self, attribute: &Handle<dyn TdfAttribute>) -> bool {
        attribute
            .get()
            .is_some_and(|attribute| self.is_kept(&attribute.id()))
    }
    /// `IsIgnored(anID)`.
    pub fn is_ignored(&self, id: &Guid) -> bool {
        !self.is_kept(id)
    }
    /// `IsIgnored(anAtt)`.
    pub fn is_ignored_attribute(&self, attribute: &Handle<dyn TdfAttribute>) -> bool {
        attribute
            .get()
            .is_some_and(|attribute| self.is_ignored(&attribute.id()))
    }
    /// `IDList`: the listed IDs, in map order.
    pub fn id_list(&self, ids: &mut Vec<Guid>) {
        ids.clear();
        ids.extend(self.id_map.iter().copied());
    }
    /// `Copy(fromFilter)`: the mode and list of another filter, added to this list.
    pub fn copy(&mut self, from_filter: &TdfIdFilter) {
        self.ignore = from_filter.ignore_all();
        let mut ids = Vec::new();
        from_filter.id_list(&mut ids);
        if self.ignore {
            self.keep_list(&ids);
        } else {
            self.ignore_list(&ids);
        }
    }
    /// `Assign`: the mode and the map of another filter.
    pub fn assign(&mut self, filter: &TdfIdFilter) {
        self.ignore = filter.ignore;
        self.id_map = filter.id_map.clone();
    }
    /// `Dump`: `EXCLUSIVE filter: ignores all IDs BUT:` and the listed IDs.
    pub fn dump(&self, stream: &mut dyn io::Write) -> io::Result<()> {
        let (prefix, verb) = if self.ignore {
            ("EX", "ignores")
        } else {
            ("IN", "keeps  ")
        };
        write!(stream, "{prefix}CLUSIVE filter: {verb} all IDs")?;
        if self.id_map.is_empty() {
            return Ok(());
        }
        writeln!(stream, " BUT:")?;
        for id in self.id_map.iter() {
            let mut text = String::new();
            let _ = id.shallow_dump(&mut text);
            writeln!(stream, "{text}")?;
        }
        Ok(())
    }
}
