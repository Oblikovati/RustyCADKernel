// Rust port of DataExchange/TKXSBase/Interface/Interface_BitMap (.hxx, .cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Interface_BitMap`: one or more flags (bits) for each of a number of items, packed
//! in 32-bit words, flag after flag.
//!
//! The words and the flag names are shared by the bit maps built from each other without
//! copy, as the native handles; the `const` members that set bits edit the shared words.
//! The native code dereferences a null name list (before the first flag) in `RemoveFlag`,
//! `SetFlagName` and `FlagName`; the port reads it as empty.
use super::int_array::{
    InterfaceIntArray, int_at, int_upper, new_int_array, regrown_int_array, set_int,
};
use rustycad_tkernel::{
    standard::{Failure, SharedState},
    tcollection::AsciiString,
};
use std::sync::Arc;

/// The shared flag names (`occ::handle<NCollection_HSequence<TCollection_AsciiString>>`).
type FlagNames = Arc<SharedState<Vec<AsciiString>>>;

/// `Interface_BitMap`.
///
/// ```
/// use rustycad_tkxsbase::interface::InterfaceBitMap;
/// let mut map = InterfaceBitMap::with_items(40, 0).unwrap();
/// let flag = map.add_flag("seen").unwrap();
/// map.set_true(35, flag).unwrap();
/// assert!(map.value(35, flag).unwrap());
/// assert!(!map.value(35, 0).unwrap());
/// ```
#[derive(Clone, Default)]
pub struct InterfaceBitMap {
    thenbitems: i32,
    thenbwords: i32,
    thenbflags: i32,
    theflags: InterfaceIntArray,
    thenames: Option<FlagNames>,
}

impl InterfaceBitMap {
    /// `Interface_BitMap()`: no item.
    pub fn new() -> Result<Self, Failure> {
        Self::with_items(0, 0)
    }
    /// `Interface_BitMap(nbitems, resflags = 0)`.
    pub fn with_items(nbitems: i32, resflags: i32) -> Result<Self, Failure> {
        let mut map = Self::default();
        map.initialize(nbitems, resflags)?;
        Ok(map)
    }
    /// `Interface_BitMap(other, copied = false)`: shares the words and names of `other`,
    /// or copies them.
    pub fn from_other(other: &InterfaceBitMap, copied: bool) -> Result<Self, Failure> {
        let mut map = Self::default();
        map.initialize_from(other, copied)?;
        Ok(map)
    }
    /// `Initialize(nbitems, resflags = 0)`: room for `resflags` flags besides flag 0.
    pub fn initialize(&mut self, nbitems: i32, resflags: i32) -> Result<(), Failure> {
        self.thenbitems = nbitems;
        self.thenbwords = nbitems / 32 + 1;
        self.thenbflags = 0;
        self.theflags = new_int_array(0, self.thenbwords * (resflags + 1))?;
        Ok(())
    }
    /// `Initialize(other, copied = false)`.
    pub fn initialize_from(
        &mut self,
        other: &InterfaceBitMap,
        copied: bool,
    ) -> Result<(), Failure> {
        self.thenbitems = other.thenbitems;
        self.thenbwords = other.thenbwords;
        self.thenbflags = other.thenbflags;
        if !copied {
            self.theflags = other.theflags.clone();
            self.thenames = other.thenames.clone();
            return Ok(());
        }
        let upper = int_upper(&other.theflags)?;
        self.theflags = regrown_int_array(&other.theflags, 0, upper, 0..=upper)?;
        if let Some(names) = &other.thenames {
            self.thenames = Some(Arc::new(SharedState::new(names.snapshot())));
        }
        Ok(())
    }
    /// `Reservate(moreflags)`: grows the words for `moreflags` more flags (and one spare).
    pub fn reservate(&mut self, moreflags: i32) -> Result<(), Failure> {
        let nb = int_upper(&self.theflags)?;
        let nbflags = nb / self.thenbwords - 1; // flag 0 not counted
        if nbflags >= self.thenbflags + moreflags {
            return Ok(());
        }
        let nbw = self.thenbwords * (self.thenbflags + moreflags + 2);
        self.theflags = regrown_int_array(&self.theflags, 0, nbw, 0..=nb)?;
        Ok(())
    }
    /// `SetLength(nbitems)`: changes the number of items, keeping the words of each flag
    /// that fit.
    pub fn set_length(&mut self, nbitems: i32) -> Result<(), Failure> {
        let nbw = nbitems / 32 + 1;
        if nbw == self.thenbwords {
            return Ok(());
        }
        let flags = new_int_array(0, nbw * (self.thenbflags + 1))?;
        let nbmots = nbw.min(self.thenbwords);
        let (mut i0, mut i1) = (0, 0);
        for _ in 0..=self.thenbflags {
            for i in 0..nbmots {
                set_int(&flags, i1 + i, int_at(&self.theflags, i0 + i)?)?;
            }
            i0 += self.thenbwords;
            i1 += nbw;
        }
        self.theflags = flags;
        self.thenbitems = nbitems;
        self.thenbwords = nbw;
        Ok(())
    }
    /// `AddFlag(name = "")`: a new flag, reusing the place of a removed one ("." name);
    /// returns its number.
    pub fn add_flag(&mut self, name: &str) -> Result<i32, Failure> {
        self.reservate(1)?;
        let names = self.names_or_create();
        let mut deja = 0;
        let mut list = names.write();
        for (i, text) in list.iter_mut().enumerate() {
            if text.is_equal(".") {
                text.assign_cat(name);
                deja = i as i32 + 1;
            }
        }
        if deja == 0 {
            list.push(AsciiString::new(name));
        }
        self.thenbflags += 1;
        Ok(if deja != 0 { deja } else { self.thenbflags })
    }
    /// `AddSomeFlags(more)`: `more` unnamed flags; returns the new count of flags.
    pub fn add_some_flags(&mut self, more: i32) -> Result<i32, Failure> {
        self.reservate(more)?;
        let names = self.names_or_create();
        let mut list = names.write();
        (0..more).for_each(|_| list.push(AsciiString::default()));
        self.thenbflags += more;
        Ok(self.thenbflags)
    }
    /// `RemoveFlag(num)`: the last flag is removed, another one is marked "." for reuse.
    pub fn remove_flag(&mut self, num: i32) -> bool {
        let Some(names) = &self.thenames else {
            return false;
        };
        let mut list = names.write();
        let length = list.len() as i32;
        if num < 1 || num > length {
            return false;
        }
        if num == length {
            list.pop();
        } else {
            list[(num - 1) as usize].assign_cat(".");
        }
        self.thenbflags -= 1;
        true
    }
    /// `SetFlagName(num, name)`: false when out of range or the name is another flag's.
    /// As natively, the name is appended to the current one.
    pub fn set_flag_name(&mut self, num: i32, name: &str) -> bool {
        let length = self.names_len();
        if num < 1 || num > length {
            return false;
        }
        let deja = if name.is_empty() {
            0
        } else {
            self.flag_number(name)
        };
        if deja != 0 && deja != num {
            return false;
        }
        if let Some(names) = &self.thenames {
            names.write()[(num - 1) as usize].assign_cat(name);
        }
        true
    }
    /// `NbFlags`.
    pub fn nb_flags(&self) -> i32 {
        self.thenbflags
    }
    /// `Length`: the number of items.
    pub fn length(&self) -> i32 {
        self.thenbitems
    }
    /// `FlagName(num)`: empty when out of range.
    pub fn flag_name(&self, num: i32) -> AsciiString {
        if self.theflags.is_null() || num < 1 || num > self.names_len() {
            return AsciiString::default();
        }
        self.thenames
            .as_ref()
            .map(|n| n.read()[(num - 1) as usize].clone())
            .unwrap_or_default()
    }
    /// `FlagNumber(name)`: 0 for an empty or unknown name.
    pub fn flag_number(&self, name: &str) -> i32 {
        let Some(names) = self.thenames.as_ref().filter(|_| !name.is_empty()) else {
            return 0;
        };
        let list = names.read();
        list.iter()
            .position(|text| text.is_equal(name))
            .map_or(0, |i| i as i32 + 1)
    }
    /// `Value(item, flag = 0)`: `Standard_OutOfRange` for an item or flag out of the words.
    pub fn value(&self, item: i32, flag: i32) -> Result<bool, Failure> {
        let val = int_at(&self.theflags, self.word(item, flag))?;
        if val == 0 {
            return Ok(false);
        }
        if val == !0 {
            return Ok(true);
        }
        Ok(((1 << (item & 31)) & val) != 0)
    }
    /// `SetValue(item, val, flag = 0)`.
    pub fn set_value(&self, item: i32, val: bool, flag: i32) -> Result<(), Failure> {
        if val {
            self.set_true(item, flag)
        } else {
            self.set_false(item, flag)
        }
    }
    /// `SetTrue(item, flag = 0)`.
    pub fn set_true(&self, item: i32, flag: i32) -> Result<(), Failure> {
        let numw = self.word(item, flag);
        let val = int_at(&self.theflags, numw)?;
        set_int(&self.theflags, numw, val | (1 << (item & 31)))
    }
    /// `SetFalse(item, flag = 0)`.
    pub fn set_false(&self, item: i32, flag: i32) -> Result<(), Failure> {
        let numw = self.word(item, flag);
        let val = int_at(&self.theflags, numw)?;
        if val == 0 {
            return Ok(());
        }
        set_int(&self.theflags, numw, val & !(1 << (item & 31)))
    }
    /// `CTrue(item, flag = 0)`: sets the bit, returns its former value.
    pub fn c_true(&self, item: i32, flag: i32) -> Result<bool, Failure> {
        let numw = self.word(item, flag);
        let mot = 1 << (item & 31);
        let val = int_at(&self.theflags, numw)?;
        set_int(&self.theflags, numw, if val == 0 { mot } else { val | mot })?;
        Ok(val != 0 && (val & mot) != 0)
    }
    /// `CFalse(item, flag = 0)`: clears the bit. Native quirk kept: the result is
    /// `(val | ~bit) != 0`, true unless the word was all ones (then false).
    pub fn c_false(&self, item: i32, flag: i32) -> Result<bool, Failure> {
        let numw = self.word(item, flag);
        let mot = !(1 << (item & 31));
        let val = int_at(&self.theflags, numw)?;
        set_int(
            &self.theflags,
            numw,
            if val == !0 { mot } else { val & mot },
        )?;
        Ok(val != !0 && (val | mot) != 0)
    }
    /// `Init(val, flag = 0)`: sets every bit of a flag (of all flags for a negative one).
    pub fn init(&self, val: bool, flag: i32) -> Result<(), Failure> {
        let (first, count) = if flag < 0 {
            (0, self.thenbwords * (self.thenbflags + 1))
        } else {
            (self.thenbwords * flag, self.thenbwords)
        };
        let word = if val { !0 } else { 0 };
        (first..first + count).try_for_each(|i| set_int(&self.theflags, i, word))
    }
    /// `Clear`: no item, no flag (the names are kept, as natively).
    pub fn clear(&mut self) -> Result<(), Failure> {
        self.initialize(0, 0)
    }
    fn word(&self, item: i32, flag: i32) -> i32 {
        self.thenbwords * flag + (item >> 5)
    }
    fn names_len(&self) -> i32 {
        self.thenames.as_ref().map_or(0, |n| n.read().len() as i32)
    }
    fn names_or_create(&mut self) -> FlagNames {
        self.thenames
            .get_or_insert_with(|| Arc::new(SharedState::new(Vec::new())))
            .clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flags_and_names() {
        let mut map = InterfaceBitMap::with_items(10, 1).unwrap();
        assert_eq!(map.add_flag("a").unwrap(), 1);
        assert_eq!(map.add_some_flags(2).unwrap(), 3);
        assert_eq!(map.flag_number("a"), 1);
        assert!(map.set_flag_name(2, "b"));
        assert!(!map.set_flag_name(3, "a"));
        assert!(map.remove_flag(1));
        assert_eq!(map.flag_name(1).as_bytes(), b"a.");
        assert!(map.remove_flag(3));
        assert_eq!(map.nb_flags(), 1);
        assert!(!map.remove_flag(9));
    }

    #[test]
    fn bits_shared_or_copied() {
        let map = InterfaceBitMap::with_items(70, 0).unwrap();
        let shared = InterfaceBitMap::from_other(&map, false).unwrap();
        let copy = InterfaceBitMap::from_other(&map, true).unwrap();
        map.set_true(65, 0).unwrap();
        assert!(shared.value(65, 0).unwrap());
        assert!(!copy.value(65, 0).unwrap());
        assert!(!map.c_true(3, 0).unwrap());
        assert!(map.c_true(3, 0).unwrap());
        assert!(map.c_false(3, 0).unwrap());
        assert!(!map.value(3, 0).unwrap());
        map.init(true, 0).unwrap();
        assert!(!map.c_false(1, 0).unwrap());
        assert_eq!(
            map.value(5, 4).err().unwrap().exception_type(),
            "Standard_OutOfRange"
        );
    }

    #[test]
    fn length_changes_keep_bits() {
        let mut map = InterfaceBitMap::with_items(10, 0).unwrap();
        map.add_flag("").unwrap();
        map.set_true(4, 1).unwrap();
        map.set_length(100).unwrap();
        assert_eq!(map.length(), 100);
        assert!(map.value(4, 1).unwrap());
        map.set_value(99, true, 1).unwrap();
        map.set_value(99, false, 1).unwrap();
        assert!(!map.value(99, 1).unwrap());
        map.clear().unwrap();
        assert_eq!((map.length(), map.nb_flags()), (0, 0));
    }
}
