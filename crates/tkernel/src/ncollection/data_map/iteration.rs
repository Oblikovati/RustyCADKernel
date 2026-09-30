use super::cursor::settle;
use super::{DataMapItems, Entry};
impl<K, V> DataMapItems<'_, K, V> {
    pub(super) fn seek(&mut self) {
        (self.bucket, self.index) = settle(self.buckets, self.bucket, self.index);
    }
    pub(super) fn current(&self) -> Option<*const Entry<K, V>> {
        self.buckets
            .get(self.bucket)?
            .get(self.index)
            .map(|node| &**node as *const Entry<K, V>)
    }
}
impl<'a, K, V> Iterator for DataMapItems<'a, K, V> {
    type Item = (&'a K, &'a V);
    fn next(&mut self) -> Option<Self::Item> {
        let entry = self.buckets.get(self.bucket)?.get(self.index)?;
        self.index += 1;
        self.seek();
        Some((&entry.key, &entry.value))
    }
}
impl<K, V> Clone for DataMapItems<'_, K, V> {
    fn clone(&self) -> Self {
        Self {
            buckets: self.buckets,
            bucket: self.bucket,
            index: self.index,
        }
    }
}
impl<K, V> PartialEq for DataMapItems<'_, K, V> {
    fn eq(&self, other: &Self) -> bool {
        self.current() == other.current()
    }
}
impl<K, V> Eq for DataMapItems<'_, K, V> {}
