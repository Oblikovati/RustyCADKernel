//! Forward borrows project item fields only, leaving raw links disjoint from yielded values.
use super::{List, ListCursor, ListIter, ListIterMut, address, owner};
// SAFETY: shared/mutable iterators borrow their List and carry the same bounds as slice iterators.
unsafe impl<T: Sync> Send for ListIter<'_, T> {}
unsafe impl<T: Sync> Sync for ListIter<'_, T> {}
unsafe impl<T: Send> Send for ListIterMut<'_, T> {}
unsafe impl<T: Sync> Sync for ListIterMut<'_, T> {}
impl ListCursor {
    /// Repositions on the head; `cursor.init(&list)`.
    pub fn init<T>(&mut self, list: &List<T>) {
        *self = list.cursor();
    }
    /// Checks whether the cursor names a node; `cursor.more()`.
    pub fn more(self) -> bool {
        self.address.is_some()
    }
    /// Looks up the stable node in its list; `cursor.value(&list)`.
    pub fn value<T>(self, list: &List<T>) -> Option<&T> {
        let node = list.find_cursor(self.address)?.current;
        Some(unsafe { owner::item_ref(node) })
    }
    /// Borrows the positioned item exclusively; `*cursor.change_value(&mut list).unwrap() = 9`.
    /// End or foreign cursors return None instead of native undefined dereferencing.
    pub fn change_value<'a, T>(&self, list: &'a mut List<T>) -> Option<&'a mut T> {
        let node = list.find_cursor(self.address)?.current;
        Some(unsafe { owner::item_mut(node) })
    }
    /// Advances while remembering current, including transition to end; `cursor.next(&list)`.
    pub fn next<T>(&mut self, list: &List<T>) {
        if !self.more() {
            return;
        }
        self.previous = self.address;
        self.address = list
            .find_cursor(self.address)
            .and_then(|raw| address(unsafe { (*raw.current).next() }));
    }
}
impl<'a, T> Iterator for ListIter<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<Self::Item> {
        if self.next.is_null() {
            return None;
        }
        let node = self.next;
        self.previous = address(node);
        self.next = unsafe { (*node).next() };
        Some(unsafe { owner::item_ref(node) })
    }
}
impl<T> PartialEq for ListIter<'_, T> {
    fn eq(&self, other: &Self) -> bool {
        self.next == other.next
    }
}
impl<T> Eq for ListIter<'_, T> {}
impl<'a, T> Iterator for ListIterMut<'a, T> {
    type Item = &'a mut T;
    fn next(&mut self) -> Option<Self::Item> {
        if self.next.is_null() {
            return None;
        }
        let node = self.next;
        self.previous = address(node);
        self.next = unsafe { (*node).next() };
        Some(unsafe { owner::item_mut(node) })
    }
}
impl<'a, T> IntoIterator for &'a List<T> {
    type Item = &'a T;
    type IntoIter = ListIter<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
impl<'a, T> IntoIterator for &'a mut List<T> {
    type Item = &'a mut T;
    type IntoIter = ListIterMut<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}
