// Rust translation of ModelingAlgorithms/TKBool/TopOpeBRepDS/TopOpeBRepDS_TKI.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TopOpeBRepDS_TKI`: interferences sorted by the kind and index of their geometry or
//! support, with an iteration over the (kind, index) pairs holding interferences.
use super::{ListOfInterference, TopOpeBRepDSInterference, TopOpeBRepDSKind};
use rustycad_tkernel::ncollection::DataMap;
use rustycad_tkernel::standard::{Failure, Handle};

/// The kind of an index (the native cast of an integer to the enumeration; the values
/// past COMPOUND are UNKNOWN).
pub(crate) fn kind_from_i32(k: i32) -> TopOpeBRepDSKind {
    use TopOpeBRepDSKind::*;
    [
        Point, Curve, Surface, Vertex, Edge, Wire, Face, Shell, Solid, CompSolid, Compound,
    ]
    .get(usize::try_from(k).unwrap_or(usize::MAX))
    .copied()
    .unwrap_or(Unknown)
}

/// The interferences by kind and index; `let mut tki = TopOpeBRepDSTKI::new();`.
pub struct TopOpeBRepDSTKI {
    /// `myT(f, l)`: one map per kind from POINT to SOLID (table index from 1).
    my_t: Vec<DataMap<i32, ListOfInterference>>,
    /// `myITM`: the keys of the map iterated and the current position.
    my_itm: (Vec<i32>, usize),
    my_ti: i32,
    my_k: TopOpeBRepDSKind,
    my_g: i32,
    my_empty_loi: ListOfInterference,
    mydelta: i32,
}

impl Default for TopOpeBRepDSTKI {
    fn default() -> Self {
        Self::new()
    }
}

impl TopOpeBRepDSTKI {
    /// `TopOpeBRepDS_TKI()`; `TopOpeBRepDSTKI::new()`.
    pub fn new() -> Self {
        let mut t = Self {
            my_t: Vec::new(),
            my_itm: (Vec::new(), 0),
            my_ti: 0,
            my_k: TopOpeBRepDSKind::Unknown,
            my_g: 0,
            my_empty_loi: ListOfInterference::default(),
            mydelta: 0,
        };
        t.reset();
        t
    }

    /// `Reset()` (protected): the table of the kinds POINT..SOLID, cleared.
    fn reset(&mut self) {
        let (ip, is) = (
            TopOpeBRepDSKind::Point as i32,
            TopOpeBRepDSKind::Solid as i32,
        );
        let f = 1;
        self.mydelta = f - ip;
        if self.my_t.is_empty() {
            self.my_t = (ip..=is).map(|_| DataMap::default()).collect();
        }
        self.clear();
        self.my_k = TopOpeBRepDSKind::Unknown;
        self.my_g = 0;
    }

    /// `Clear()`: every map emptied; `tki.clear()`.
    pub fn clear(&mut self) {
        for m in &mut self.my_t {
            m.clear(false);
        }
    }

    /// `FillOnGeometry(L)`: each interference added under its geometry;
    /// `tki.fill_on_geometry(&l)?`.
    pub fn fill_on_geometry(&mut self, l: &ListOfInterference) -> Result<(), Failure> {
        for i in l.iter() {
            let Some(ii) = i.get() else {
                continue;
            };
            self.add_interference(ii.geometry_type(), ii.geometry(), i)?;
        }
        Ok(())
    }

    /// `FillOnSupport(L)`: each interference added under its support;
    /// `tki.fill_on_support(&l)?`.
    pub fn fill_on_support(&mut self, l: &ListOfInterference) -> Result<(), Failure> {
        for i in l.iter() {
            let Some(ii) = i.get() else {
                continue;
            };
            self.add_interference(ii.support_type(), ii.support(), i)?;
        }
        Ok(())
    }

    /// The vector index of a valid kind.
    fn slot(&self, k: TopOpeBRepDSKind) -> usize {
        (self.kind_to_table_index(k) - 1) as usize
    }

    /// `IsBound(K, G)`; `tki.is_bound(TopOpeBRepDSKind::Edge, 1)`.
    pub fn is_bound(&self, k: TopOpeBRepDSKind, g: i32) -> bool {
        self.is_valid_kg(k, g) && self.my_t[self.slot(k)].is_bound(&g)
    }

    /// `Interferences(K, G)`: the interferences of the pair (an empty list when unbound);
    /// `tki.interferences(TopOpeBRepDSKind::Edge, 1)`.
    pub fn interferences(&self, k: TopOpeBRepDSKind, g: i32) -> &ListOfInterference {
        if !self.is_bound(k, g) {
            return &self.my_empty_loi;
        }
        self.my_t[self.slot(k)]
            .seek(&g)
            .unwrap_or(&self.my_empty_loi)
    }

    /// `ChangeInterferences(K, G)`: the interferences of the pair, changed (a shared
    /// list, kept between calls, when unbound, as natively);
    /// `tki.change_interferences(TopOpeBRepDSKind::Edge, 1).clear()`.
    pub fn change_interferences(&mut self, k: TopOpeBRepDSKind, g: i32) -> &mut ListOfInterference {
        if !self.is_bound(k, g) {
            return &mut self.my_empty_loi;
        }
        let slot = self.slot(k);
        self.my_t[slot].change_seek(&g).expect("a bound key")
    }

    /// `HasInterferences(K, G)`: bound with a non-empty list;
    /// `tki.has_interferences(TopOpeBRepDSKind::Edge, 1)`.
    pub fn has_interferences(&self, k: TopOpeBRepDSKind, g: i32) -> bool {
        self.is_bound(k, g) && !self.interferences(k, g).is_empty()
    }

    /// `Add(K, G)`: the pair bound to an empty list when unbound; Standard_ProgramError
    /// for an invalid pair; `tki.add(TopOpeBRepDSKind::Edge, 1)?`.
    pub fn add(&mut self, k: TopOpeBRepDSKind, g: i32) -> Result<(), Failure> {
        if !self.is_valid_kg(k, g) {
            return Err(Failure::with_type(
                "Standard_ProgramError",
                &format!(
                    "TopOpeBRepDS_TKI : Add K G ({k:?}, {g}); expected a kind POINT..SOLID and an index > 0"
                ),
                "TopOpeBRepDS_TKI",
            ));
        }
        if !self.is_bound(k, g) {
            let slot = self.slot(k);
            self.my_t[slot].bind(g, ListOfInterference::default());
        }
        Ok(())
    }

    /// `Add(K, G, HI)`: the interference appended to the pair;
    /// `tki.add_interference(TopOpeBRepDSKind::Edge, 1, &i)?`.
    pub fn add_interference(
        &mut self,
        k: TopOpeBRepDSKind,
        g: i32,
        hi: &Handle<TopOpeBRepDSInterference>,
    ) -> Result<(), Failure> {
        self.add(k, g)?;
        self.change_interferences(k, g).append(hi.clone());
        Ok(())
    }

    /// `DumpTKIIterator(s1, s2)`: the two texts printed around an iteration;
    /// `tki.dump_tki_iterator("", "")`.
    pub fn dump_tki_iterator(&mut self, s1: &str, s2: &str) {
        print!("{s1}");
        self.init();
        while self.more() {
            self.next();
        }
        print!("{s2}");
    }

    /// The map iteration started on the current table index.
    fn init_itm(&mut self) {
        let keys = self.my_t[(self.my_ti - 1) as usize]
            .items()
            .map(|(k, _)| *k)
            .collect();
        self.my_itm = (keys, 0);
    }

    /// `Init()`: the iteration on the first pair holding interferences; `tki.init()`.
    pub fn init(&mut self) {
        self.my_k = TopOpeBRepDSKind::Unknown;
        self.my_g = 0;
        if self.my_t.is_empty() {
            return;
        }
        self.my_ti = 1;
        self.my_k = self.table_index_to_kind(self.my_ti);
        self.init_itm();
        self.find();
    }

    /// `More()`; `tki.more()`.
    pub fn more(&self) -> bool {
        self.is_valid_kg(self.my_k, self.my_g)
    }

    /// `Next()`; `tki.next()`.
    #[allow(clippy::should_implement_trait)] // OCCT's More/Next protocol.
    pub fn next(&mut self) {
        if self.more_itm() {
            self.next_itm();
            self.find();
        } else if self.more_ti() {
            self.next_ti();
            if self.more_ti() {
                self.init_itm();
            }
            self.find();
        }
    }

    /// `Value(K, G)`: the current pair and its interferences (empty at the end, the pair
    /// then unwritten: `None`); `let (k, g, l) = tki.value().unwrap();`.
    pub fn value(&self) -> Option<(TopOpeBRepDSKind, i32, &ListOfInterference)> {
        if !self.more() {
            return None;
        }
        Some((
            self.my_k,
            self.my_g,
            self.interferences(self.my_k, self.my_g),
        ))
    }

    /// `ChangeValue(K, G)`: the current pair and its interferences, changed;
    /// `let (k, g, l) = tki.change_value().unwrap();`.
    pub fn change_value(&mut self) -> Option<(TopOpeBRepDSKind, i32, &mut ListOfInterference)> {
        if !self.more() {
            return None;
        }
        let (k, g) = (self.my_k, self.my_g);
        Some((k, g, self.change_interferences(k, g)))
    }

    /// `MoreTI()` (protected).
    fn more_ti(&self) -> bool {
        self.is_valid_ti(self.my_ti)
    }

    /// `NextTI()` (protected).
    fn next_ti(&mut self) {
        self.my_ti += 1;
        self.my_k = self.table_index_to_kind(self.my_ti);
    }

    /// `MoreITM()` (protected).
    fn more_itm(&self) -> bool {
        self.my_itm.1 < self.my_itm.0.len()
    }

    /// `FindITM()` (protected): the map iteration advanced to a key with interferences.
    fn find_itm(&mut self) {
        while self.more_itm() {
            self.my_g = self.my_itm.0[self.my_itm.1];
            if self.has_interferences(self.my_k, self.my_g) {
                break;
            }
            self.my_itm.1 += 1;
        }
    }

    /// `NextITM()` (protected).
    fn next_itm(&mut self) {
        if !self.more_itm() {
            return;
        }
        self.my_itm.1 += 1;
        self.find_itm();
    }

    /// `Find()` (protected): the iteration advanced to a pair with interferences.
    fn find(&mut self) {
        let mut f = false;
        while self.more_ti() {
            while self.more_itm() {
                self.find_itm();
                f = self.has_interferences(self.my_k, self.my_g);
                if f {
                    break;
                }
            }
            if f {
                break;
            }
            self.next_ti();
            if self.more_ti() {
                self.init_itm();
            }
        }
    }

    /// `KindToTableIndex(K)` (protected).
    fn kind_to_table_index(&self, k: TopOpeBRepDSKind) -> i32 {
        k as i32 + self.mydelta
    }

    /// `TableIndexToKind(TI)` (protected).
    fn table_index_to_kind(&self, ti: i32) -> TopOpeBRepDSKind {
        kind_from_i32(ti - self.mydelta)
    }

    /// `IsValidTI(TI)` (protected).
    fn is_valid_ti(&self, ti: i32) -> bool {
        !self.my_t.is_empty() && ti >= 1 && ti <= self.my_t.len() as i32
    }

    /// `IsValidK(K)` (protected).
    fn is_valid_k(&self, k: TopOpeBRepDSKind) -> bool {
        k >= TopOpeBRepDSKind::Point && k <= TopOpeBRepDSKind::Solid
    }

    /// `IsValidG(G)` (protected).
    fn is_valid_g(&self, g: i32) -> bool {
        g > 0
    }

    /// `IsValidKG(K, G)` (protected).
    fn is_valid_kg(&self, k: TopOpeBRepDSKind, g: i32) -> bool {
        self.is_valid_k(k) && self.is_valid_g(g)
    }
}
