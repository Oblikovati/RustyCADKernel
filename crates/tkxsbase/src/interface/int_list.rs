// Rust port of DataExchange/TKXSBase/Interface/Interface_IntList (.hxx, .cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Interface_IntList`: for each entity number, a list of integers (references), kept in
//! two shared arrays with the native packed layout: an entity entry is 0 (empty), the
//! single value (>0), -1 (empty but redefined) or `-rank` of its list in the references,
//! whose last value is stored negated.
use super::int_array::{
    InterfaceIntArray, int_at, int_lower, int_upper, new_int_array, regrown_int_array, set_int,
};
use rustycad_tkernel::standard::Failure;

/// `Interface_IntList`. Copies share the arrays unless built with `copied`.
///
/// ```
/// use rustycad_tkxsbase::interface::InterfaceIntList;
/// let mut list = InterfaceIntList::with_entities(3).unwrap();
/// list.set_number(2).unwrap();
/// list.add(7).unwrap();
/// list.add(9).unwrap();
/// assert_eq!(list.length(), 2);
/// assert_eq!(list.value(2).unwrap(), 9);
/// ```
#[derive(Clone, Default)]
pub struct InterfaceIntList {
    thenbe: i32,
    thenbr: i32,
    thenum: i32,
    thecount: i32,
    therank: i32,
    theents: InterfaceIntArray,
    therefs: InterfaceIntArray,
}

impl InterfaceIntList {
    /// `Interface_IntList()`: no entity (no arrays).
    pub fn new() -> Self {
        Self::default()
    }
    /// `Interface_IntList(nbe)`.
    pub fn with_entities(nbe: i32) -> Result<Self, Failure> {
        let mut list = Self::default();
        list.initialize(nbe)?;
        Ok(list)
    }
    /// `Interface_IntList(other, copied)`: shares or copies the arrays of `other`, then
    /// sets its current number.
    pub fn from_other(other: &InterfaceIntList, copied: bool) -> Result<Self, Failure> {
        let mut list = Self {
            thenbe: other.thenbe,
            thenbr: other.thenbr,
            theents: other.theents.clone(),
            therefs: other.therefs.clone(),
            ..Self::default()
        };
        if copied {
            list.theents = regrown_int_array(&other.theents, 0, list.thenbe, 1..=list.thenbe)?;
            list.therefs = regrown_int_array(&other.therefs, 0, list.thenbr, 1..=list.thenbr)?;
        }
        list.set_number(other.thenum)?;
        Ok(list)
    }
    /// `Initialize(nbe)`: `nbe` empty entities, no reference.
    pub fn initialize(&mut self, nbe: i32) -> Result<(), Failure> {
        *self = Self {
            thenbe: nbe,
            therefs: std::mem::take(&mut self.therefs),
            theents: new_int_array(0, nbe)?,
            ..Self::default()
        };
        Ok(())
    }
    /// `NbReferences`.
    pub fn nb_references(&self) -> i32 {
        self.thenbr
    }
    /// `Entities`: the shared array of entity entries.
    pub fn entities(&self) -> &InterfaceIntArray {
        &self.theents
    }
    /// `References`: the shared array of references (null before the first list).
    pub fn references(&self) -> &InterfaceIntArray {
        &self.therefs
    }
    /// `Internals` (deprecated natively): (references count, entities, references).
    pub fn internals(&self) -> (i32, InterfaceIntArray, InterfaceIntArray) {
        (self.thenbr, self.theents.clone(), self.therefs.clone())
    }
    /// `NbEntities`.
    pub fn nb_entities(&self) -> i32 {
        self.thenbe
    }
    /// `SetNbEntities(nbe)`: only grows.
    pub fn set_nb_entities(&mut self, nbe: i32) -> Result<(), Failure> {
        if nbe <= int_upper(&self.theents)? {
            return Ok(());
        }
        self.theents = regrown_int_array(&self.theents, 0, nbe, 1..=self.thenbe)?;
        self.thenbe = nbe;
        Ok(())
    }
    /// `SetNumber(number)`: the current entity; a negative number keeps the reserved
    /// count of a list (to fill it), a positive one counts its values.
    pub fn set_number(&mut self, number: i32) -> Result<(), Failure> {
        if number < 0 {
            if self.thenum == -number || number < -self.thenbe {
                return Ok(());
            }
            self.thenum = -number;
            if self.reserved_count()? {
                return Ok(());
            }
        } else if number > 0 {
            if self.thenum == number || number > self.thenbe {
                return Ok(());
            }
            self.thenum = number;
        } else {
            return Ok(());
        }
        self.count_current()
    }
    /// The first part of a negative `SetNumber`: true when it is complete.
    fn reserved_count(&mut self) -> Result<bool, Failure> {
        let val = int_at(&self.theents, self.thenum)?;
        if val == 0 {
            (self.thecount, self.therank) = (0, 0);
        } else if val > 0 {
            (self.thecount, self.therank) = (1, -1);
        }
        if val < -1 {
            self.therank = -val;
            self.thecount = int_at(&self.therefs, self.therank)?;
            return Ok(self.thecount > 0);
        }
        Ok(true)
    }
    /// The count and rank of the current entity's list.
    fn count_current(&mut self) -> Result<(), Failure> {
        let val = int_at(&self.theents, self.thenum)?;
        if val == 0 {
            (self.thecount, self.therank) = (0, 0);
        } else if val > 0 {
            (self.thecount, self.therank) = (1, -1);
        } else if val < -1 {
            self.therank = -val;
            self.thecount = self.listed_count()?;
        } else {
            (self.thecount, self.therank) = (0, -1); // val == -1 stays
        }
        Ok(())
    }
    /// The number of values of the list at `therank` (negative values up to the last).
    fn listed_count(&self) -> Result<i32, Failure> {
        if int_at(&self.therefs, self.therank + 1)? == 0 {
            return Ok(-int_at(&self.therefs, self.therank)?);
        }
        let mut count = 0;
        let mut j = 1;
        let last = loop {
            let val = int_at(&self.therefs, self.therank + j)?;
            if val >= 0 {
                break val;
            }
            count += 1;
            j += 1;
        };
        Ok(if last > 0 { count + 1 } else { count })
    }
    /// `Number`: the current entity.
    pub fn number(&self) -> i32 {
        self.thenum
    }
    /// `List(number, copied = false)`: a list positioned on `number`.
    pub fn list(&self, number: i32, copied: bool) -> Result<InterfaceIntList, Failure> {
        let mut alist = Self::from_other(self, copied)?;
        alist.set_number(number)?;
        Ok(alist)
    }
    /// `SetRedefined(mode)`: marks (or unmarks) the current entity as redefined.
    pub fn set_redefined(&mut self, mode: bool) -> Result<(), Failure> {
        if self.nb_entities() == 0 || self.thenum == 0 {
            return Ok(());
        }
        let val = int_at(&self.theents, self.thenum)?;
        if val < -1 {
            return Ok(());
        }
        if mode {
            return self.mark_redefined(val);
        }
        if val == -1 {
            return set_int(&self.theents, self.thenum, 0);
        }
        let next = int_at(&self.therefs, self.therank + 1)?;
        if next >= 0 {
            set_int(&self.theents, self.thenum, next)?;
            if self.thenbr == self.therank + 1 {
                self.thenbr -= 1;
            }
        }
        Ok(())
    }
    fn mark_redefined(&mut self, val: i32) -> Result<(), Failure> {
        if val == 0 {
            return set_int(&self.theents, self.thenum, -1);
        }
        if val > 0 {
            self.reservate(2)?;
            set_int(&self.theents, self.thenum, -self.thenbr)?;
            set_int(&self.therefs, self.thenbr + 1, val)?;
            self.thenbr += 1;
        }
        Ok(())
    }
    /// `Reservate(count)`: room for `count` more references; a negative count reserves
    /// `-count - 1` places for the current entity, its count stored before them.
    pub fn reservate(&mut self, count: i32) -> Result<(), Failure> {
        if count < 0 {
            return self.reservate_for_current(count);
        }
        if self.thenbr == 0 {
            // Not yet allocated; the lists start after rank 2 (convenience of addressing).
            let mut up = (self.thenbe / 2 + 1).max(2);
            if up < count {
                up = count * 3 / 2;
            }
            self.therefs = new_int_array(0, up)?;
            self.thenbr = 2;
        }
        let oldup = int_upper(&self.therefs)?;
        if self.thenbr + count < oldup {
            return Ok(());
        }
        let up = (oldup * 3 / 2 + count).max(2);
        self.therefs = regrown_int_array(&self.therefs, 0, up, 1..=oldup)?;
        Ok(())
    }
    fn reservate_for_current(&mut self, count: i32) -> Result<(), Failure> {
        self.reservate(-count - 1)?;
        if self.thenum == 0 {
            return Ok(());
        }
        self.thenbr += 1;
        set_int(&self.therefs, self.thenbr, 0)?; // will contain the number
        self.therank = self.thenbr;
        set_int(&self.theents, self.thenum, -self.thenbr)?;
        self.thenbr -= count;
        Ok(())
    }
    /// `Add(ref)`: appends a reference to the current entity's list; a negative one is
    /// added (as its opposite) to a reserved list and updates its stored count.
    pub fn add(&mut self, reference: i32) -> Result<(), Failure> {
        if self.thenum == 0 {
            return Ok(());
        }
        if reference < 0 {
            self.add(-reference)?;
            if self.therank > 0 && int_at(&self.therefs, self.therank)? >= 0 {
                set_int(&self.therefs, self.therank, self.thecount)?;
            }
            return Ok(());
        }
        if self.therank == 0 {
            set_int(&self.theents, self.thenum, reference)?;
            (self.thecount, self.therank) = (1, -1);
            return Ok(());
        }
        if self.therank < 0 {
            return self.add_second(reference);
        }
        self.add_to_list(reference)
    }
    /// `Add` when the entity holds one value inline: moves it to a list.
    fn add_second(&mut self, reference: i32) -> Result<(), Failure> {
        self.reservate(2)?;
        self.therank = self.thenbr;
        let val = int_at(&self.theents, self.thenum)?;
        set_int(&self.theents, self.thenum, -self.thenbr)?;
        if self.thecount == 1 {
            set_int(&self.therefs, self.thenbr + 1, -val)?;
            self.thenbr += 1;
        }
        set_int(&self.therefs, self.thenbr + 1, reference)?;
        self.thenbr += 1;
        self.thecount += 1;
        Ok(())
    }
    /// `Add` to an existing list: at the end, in the free place after it, or by moving
    /// the list to the end.
    fn add_to_list(&mut self, reference: i32) -> Result<(), Failure> {
        let last = self.therank + self.thecount;
        if self.thenbr == last {
            negate_at(&self.therefs, self.thenbr)?;
            set_int(&self.therefs, self.thenbr + 1, reference)?;
            self.thenbr += 1;
            self.thecount += 1;
            return Ok(());
        }
        if int_at(&self.therefs, last + 1)? == 0 {
            negate_at(&self.therefs, last)?;
            set_int(&self.therefs, last + 1, reference)?;
            self.thecount += 1;
            return Ok(());
        }
        self.move_list_to_end(reference)
    }
    fn move_list_to_end(&mut self, reference: i32) -> Result<(), Failure> {
        self.reservate(self.thecount + 2)?;
        let rank = self.therank;
        self.therank = self.thenbr;
        set_int(&self.theents, self.thenum, -self.therank)?;
        for i in 1..self.thecount {
            set_int(
                &self.therefs,
                self.therank + i,
                int_at(&self.therefs, rank + i)?,
            )?;
            set_int(&self.therefs, rank + i, 0)?;
        }
        let last = int_at(&self.therefs, rank + self.thecount)?;
        set_int(&self.therefs, self.therank + self.thecount, -last)?;
        set_int(&self.therefs, rank + self.thecount, 0)?;
        set_int(&self.therefs, self.therank + self.thecount + 1, reference)?;
        self.thecount += 1;
        self.thenbr = self.therank + self.thecount + 1;
        Ok(())
    }
    /// `Length`: the count of values of the current entity.
    pub fn length(&self) -> i32 {
        self.thecount
    }
    /// `IsRedefined(num = 0)`: the entity (the current one for 0) is redefined.
    pub fn is_redefined(&self, num: i32) -> Result<bool, Failure> {
        let n = if num == 0 { self.thenum } else { num };
        if self.nb_entities() == 0 || n == 0 {
            return Ok(false);
        }
        Ok(int_at(&self.theents, n)? < 0)
    }
    /// `Value(num)`: the value of rank `num` of the current entity, 0 out of range.
    pub fn value(&self, num: i32) -> Result<i32, Failure> {
        if self.thenum == 0 || num <= 0 || num > self.thecount || self.thecount == 0 {
            return Ok(0);
        }
        if self.therank <= 0 {
            return int_at(&self.theents, self.thenum);
        }
        Ok(int_at(&self.therefs, self.therank + num)?.abs())
    }
    /// `Remove(num)`: not implemented natively; always false.
    pub fn remove(&mut self, _num: i32) -> bool {
        false
    }
    /// `Clear`: zeroes both arrays (nothing when already clear).
    pub fn clear(&mut self) -> Result<(), Failure> {
        if self.thenbr == 0 {
            return Ok(());
        }
        zero_all(&self.theents)?;
        self.thenbr = 0;
        if self.therefs.is_null() {
            return Ok(());
        }
        zero_all(&self.therefs)
    }
    /// `AdjustSize(margin = 0)`: shrinks the arrays to the used size plus `margin`.
    pub fn adjust_size(&mut self, margin: i32) -> Result<(), Failure> {
        if int_upper(&self.theents)? > self.thenbe {
            self.theents = regrown_int_array(&self.theents, 0, self.thenbe, 1..=self.thenbe)?;
        }
        if self.thenbr == 0 {
            return self.reservate(margin);
        }
        let up = int_upper(&self.therefs)?;
        if up >= self.thenbr && up <= self.thenbr + margin {
            return Ok(());
        }
        let size = self.thenbr + margin;
        self.therefs = regrown_int_array(&self.therefs, 0, size, 1..=self.thenbr)?;
        Ok(())
    }
}

fn negate_at(array: &InterfaceIntArray, index: i32) -> Result<(), Failure> {
    set_int(array, index, -int_at(array, index)?)
}

fn zero_all(array: &InterfaceIntArray) -> Result<(), Failure> {
    (int_lower(array)?..=int_upper(array)?).try_for_each(|i| set_int(array, i, 0))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn values(list: &InterfaceIntList) -> Vec<i32> {
        (1..=list.length())
            .map(|i| list.value(i).unwrap())
            .collect()
    }

    #[test]
    fn lists_grow_and_move() {
        let mut list = InterfaceIntList::with_entities(4).unwrap();
        list.set_number(1).unwrap();
        list.add(5).unwrap();
        list.set_number(2).unwrap();
        list.add(6).unwrap();
        list.add(7).unwrap();
        list.set_number(1).unwrap();
        list.add(8).unwrap();
        list.add(9).unwrap();
        assert_eq!(values(&list), vec![5, 8, 9]);
        list.set_number(2).unwrap();
        list.add(10).unwrap();
        assert_eq!(values(&list), vec![6, 7, 10]);
        let copy = list.list(1, true).unwrap();
        assert_eq!(values(&copy), vec![5, 8, 9]);
        assert_eq!(list.value(9).unwrap(), 0);
        assert!(!list.remove(1));
    }

    #[test]
    fn reserved_lists_and_redefinition() {
        let mut list = InterfaceIntList::with_entities(3).unwrap();
        list.set_number(3).unwrap();
        list.reservate(-3).unwrap();
        list.add(-4).unwrap();
        list.add(-5).unwrap();
        let mut other = list.list(0, false).unwrap();
        other.set_number(-3).unwrap();
        assert_eq!(values(&other), vec![4, 5]);
        list.set_number(2).unwrap();
        list.set_redefined(true).unwrap();
        assert!(list.is_redefined(2).unwrap());
        list.set_redefined(false).unwrap();
        assert!(!list.is_redefined(0).unwrap());
        list.set_nb_entities(6).unwrap();
        assert_eq!(list.nb_entities(), 6);
        // Native quirk kept: Reservate(-3) reserves 2 places but advances the count of
        // references by 4, past the array, so AdjustSize raises Standard_OutOfRange.
        assert!(list.adjust_size(1).is_err());
        list.clear().unwrap();
        assert_eq!(list.nb_references(), 0);
        assert_eq!(int_at(list.entities(), 3).unwrap(), 0);
    }

    #[test]
    fn adjust_size_shrinks_to_the_margin() {
        let mut list = InterfaceIntList::with_entities(2).unwrap();
        list.set_number(1).unwrap();
        list.add(3).unwrap();
        list.add(4).unwrap();
        list.adjust_size(2).unwrap();
        assert_eq!(super::int_upper(list.references()).unwrap(), 5);
        list.adjust_size(0).unwrap();
        let upper = super::int_upper(list.references()).unwrap();
        assert_eq!(upper, list.nb_references());
        assert_eq!(values(&list), vec![3, 4]);
        let (count, _, refs) = list.internals();
        assert_eq!(
            (count, refs == *list.references()),
            (list.nb_references(), true)
        );
    }
}
