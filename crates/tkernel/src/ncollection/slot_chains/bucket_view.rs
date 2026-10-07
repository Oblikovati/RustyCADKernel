//! Safe form of the public `NCollection_BaseMap::Buckets()` table of the hashed maps.
use super::{ChainView, SlotLink};
use crate::standard::Failure;

/// Read-only view of an allocated bucket table, as native `Buckets()` returns it.
///
/// The view gives what a native caller reads through the table: its identity (it changes only
/// when `ReSize` replaces the table), `NbBuckets() + 1` heads and each chain in native order.
/// Node addresses stay private (plan D1 class B5). Example:
/// ```
/// use rustycad_tkernel::ncollection::DataMap;
/// let mut map = DataMap::<i32, i32>::default();
/// assert!(map.buckets().is_none());
/// map.bind(1, 10);
/// let table = map.buckets().unwrap();
/// assert_eq!(table.len(), 102);
/// let mut entries = Vec::new();
/// for bucket in 0..table.len() {
///     entries.extend(table.chain(bucket)?);
/// }
/// assert_eq!(entries, [(&1, &10)]);
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
pub struct HashBucketTable<'a, K, V> {
    view: ChainView<'a, K, V>,
}

/// One bucket chain of a [`HashBucketTable`], head first; yields `(key, value)` pairs.
///
/// Example: `table.chain(1)?.map(|(key, _)| *key).collect::<Vec<_>>()`.
pub struct HashBucketChain<'a, K, V> {
    view: ChainView<'a, K, V>,
    link: SlotLink,
}

impl<'a, K, V> HashBucketTable<'a, K, V> {
    /// The allocated table of `view`, or `None` where native `Buckets()` returns null.
    pub(in crate::ncollection) fn of(view: ChainView<'a, K, V>) -> Option<Self> {
        view.chains().is_allocated().then_some(Self { view })
    }
    /// Address of the table storage, the native `Buckets()` pointer value; `table.identity()`.
    pub fn identity(&self) -> *const () {
        self.heads().as_ptr().cast()
    }
    /// Number of heads, `NbBuckets() + 1` (bucket zero stays empty); `table.len()`.
    pub fn len(&self) -> usize {
        self.heads().len()
    }
    /// Always false: an allocated table holds at least bucket zero; `table.is_empty()`.
    pub fn is_empty(&self) -> bool {
        self.heads().is_empty()
    }
    /// The chain of `bucket` in native order, or Standard_OutOfRange past the last head.
    pub fn chain(&self, bucket: usize) -> Result<HashBucketChain<'a, K, V>, Failure> {
        let link = *self
            .heads()
            .get(bucket)
            .ok_or_else(|| out_of_table(bucket, self.len()))?;
        Ok(HashBucketChain {
            view: self.view,
            link,
        })
    }
    fn heads(&self) -> &'a [SlotLink] {
        self.view.chains().table()
    }
}

impl<'a, K, V> Iterator for HashBucketChain<'a, K, V> {
    type Item = (&'a K, &'a V);
    fn next(&mut self) -> Option<Self::Item> {
        let slot = self.link?;
        self.link = self.view.chains().next(slot);
        let entry = self.view.slot(slot);
        Some((entry.key(), entry.value()))
    }
}

fn out_of_table(bucket: usize, length: usize) -> Failure {
    Failure::with_type(
        "Standard_OutOfRange",
        &format!("bucket {bucket}; expected an index below the table length {length}"),
        "",
    )
}
