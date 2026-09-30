use super::{HashPolicy, OrderedTable};
type BorrowedNode<'a, K, V> = (&'a K, &'a mut V, Option<usize>);
pub(in crate::ncollection) struct OrderedItemsMut<'a, K, V> {
    entries: Vec<Option<BorrowedNode<'a, K, V>>>,
    current: Option<usize>,
}
impl<K, V, H: HashPolicy<K>> OrderedTable<K, V, H> {
    pub fn items_mut(&mut self) -> OrderedItemsMut<'_, K, V> {
        let entries = self
            .slots
            .iter_mut()
            .map(|slot| {
                slot.as_mut().map(|node| {
                    let node = &mut **node;
                    (&node.key, &mut node.value, node.next)
                })
            })
            .collect();
        OrderedItemsMut {
            entries,
            current: self.first,
        }
    }
}
impl<'a, K, V> Iterator for OrderedItemsMut<'a, K, V> {
    type Item = (&'a K, &'a mut V);
    fn next(&mut self) -> Option<Self::Item> {
        let (key, value, next) = self.entries.get_mut(self.current?)?.take()?;
        self.current = next;
        Some((key, value))
    }
}
