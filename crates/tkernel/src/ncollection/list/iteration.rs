use super::{List, ListCursor, ListIter, ListIterMut, Node, address};
impl ListCursor {
    /// Repositions on the head of `list`, as OCCT's `Iterator::Init`; `cursor.init(&list)`.
    pub fn init<T>(&mut self, list: &List<T>) {
        *self = list.cursor();
    }
    /// Checks whether this cursor names a node; `cursor.more()`.
    pub fn more(self) -> bool {
        self.address.is_some()
    }
    /// Looks up the stable node in its list; `cursor.value(&list)`.
    pub fn value<T>(self, list: &List<T>) -> Option<&T> {
        self.node(list).map(|node| &node.value)
    }
    /// Borrows the positioned value for mutation; `*cursor.change_value(&mut list).unwrap() = 9`.
    /// An end or foreign cursor returns `None`, rejecting native's undefined dereference domain.
    pub fn change_value<'a, T>(&self, list: &'a mut List<T>) -> Option<&'a mut T> {
        let mut node = list.head.as_deref_mut();
        while let Some(current) = node {
            if Some(current as *mut Node<T> as usize) == self.address {
                return Some(&mut current.value);
            }
            node = current.next.as_deref_mut();
        }
        None
    }
    /// Advances to the next node; `cursor.next(&list)`.
    pub fn next<T>(&mut self, list: &List<T>) {
        self.address = self.node(list).and_then(|node| address(&node.next));
    }
    fn node<T>(self, list: &List<T>) -> Option<&Node<T>> {
        let mut node = list.head.as_deref();
        while let Some(current) = node {
            if Some(current as *const Node<T> as usize) == self.address {
                return node;
            }
            node = current.next.as_deref();
        }
        None
    }
}
impl<'a, T> Iterator for ListIter<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<Self::Item> {
        let node = self.next.take()?;
        self.next = node.next.as_deref();
        Some(&node.value)
    }
}
impl<T> PartialEq for ListIter<'_, T> {
    fn eq(&self, other: &Self) -> bool {
        self.next.map(|n| n as *const Node<T>) == other.next.map(|n| n as *const Node<T>)
    }
}
impl<T> Eq for ListIter<'_, T> {}
impl<'a, T> Iterator for ListIterMut<'a, T> {
    type Item = &'a mut T;
    fn next(&mut self) -> Option<Self::Item> {
        let node = self.next.take()?;
        self.next = node.next.as_deref_mut();
        Some(&mut node.value)
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
