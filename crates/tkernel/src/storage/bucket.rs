//! `Storage_Bucket`, `Storage_BucketOfPersistent` and `Storage_BucketIterator`: the
//! chunked list of objects `Storage_Schema::Write` collects before writing.
//!
//! Native buckets keep raw, non-owning `Standard_Persistent*` in preallocated arrays and
//! read stale or uninitialised slots past the last append. Rust buckets hold handles and
//! report such slots as null, which is what the zero-filled arrays of a fresh native
//! bucket read as.
use crate::standard::{Handle, StandardPersistent};

/// One chunk of persistent objects; `Bucket::with_size(4)`.
///
/// ```
/// use rustycad_tkernel::storage::Bucket;
/// let mut bucket = Bucket::with_size(4);
/// bucket.clear();
/// assert_eq!(bucket.space_size(), 4);
/// ```
pub struct Bucket {
    space: Vec<Handle<StandardPersistent>>,
    space_size: i32,
}

impl Default for Bucket {
    /// `Storage_Bucket()`: 200000 slots.
    fn default() -> Self {
        Self::with_size(200_000)
    }
}

impl Bucket {
    /// `Storage_Bucket(theSpaceSize)`; `Bucket::with_size(10)`.
    pub fn with_size(space_size: i32) -> Self {
        Self {
            space: Vec::new(),
            space_size,
        }
    }

    /// The capacity given at construction; `bucket.space_size()`.
    pub fn space_size(&self) -> i32 {
        self.space_size
    }

    /// `Clear`: forgets the stored objects (`myCurrentSpace = -1`); `bucket.clear()`.
    pub fn clear(&mut self) {
        self.space.clear();
    }

    /// `myCurrentSpace`: index of the last stored object, -1 when empty.
    fn current_space(&self) -> i32 {
        self.space.len() as i32 - 1
    }

    /// `mySpace[index]`, null past the last append.
    fn slot(&self, index: i32) -> Handle<StandardPersistent> {
        usize::try_from(index)
            .ok()
            .and_then(|index| self.space.get(index))
            .cloned()
            .unwrap_or_default()
    }
}

/// Objects appended in order across buckets of `bucket_size` slots.
///
/// ```
/// use rustycad_tkernel::standard::{Handle, StandardPersistent};
/// use rustycad_tkernel::storage::BucketOfPersistent;
/// let mut objects = BucketOfPersistent::new(2, 1);
/// for _ in 0..3 {
///     objects.append(&Handle::new(StandardPersistent::default()));
/// }
/// assert_eq!(objects.length(), 3);
/// assert!(objects.value(3).is_some());
/// ```
pub struct BucketOfPersistent {
    buckets: Vec<Bucket>,
    bucket_size: i32,
    length: i32,
}

impl Default for BucketOfPersistent {
    /// `Storage_BucketOfPersistent()`: buckets of 300000 slots.
    fn default() -> Self {
        Self::new(300_000, 100)
    }
}

impl BucketOfPersistent {
    /// `Storage_BucketOfPersistent(theBucketSize, theBucketNumber)`; the bucket number only
    /// sizes the native pointer table, which grows by doubling; `BucketOfPersistent::new(8, 2)`.
    pub fn new(bucket_size: i32, _bucket_number: i32) -> Self {
        Self {
            buckets: vec![Bucket::with_size(bucket_size)],
            bucket_size,
            length: 0,
        }
    }

    /// `Length`; `objects.length()`.
    pub fn length(&self) -> i32 {
        self.length
    }

    /// `Append`: fills the current bucket, then opens a new one; `objects.append(&object)`.
    pub fn append(&mut self, object: &Handle<StandardPersistent>) {
        let full = self
            .buckets
            .last()
            .is_some_and(|bucket| bucket.current_space() + 1 == self.bucket_size);
        if full {
            self.buckets.push(Bucket::with_size(self.bucket_size));
        }
        self.length += 1;
        if let Some(bucket) = self.buckets.last_mut() {
            bucket.space.push(object.clone());
        }
    }

    /// `Value`: the object at one-based `index`, null outside the stored range.
    /// Example: `objects.value(1)`.
    pub fn value(&self, index: i32) -> Handle<StandardPersistent> {
        let offset = index - 1;
        let bucket = offset.div_euclid(self.bucket_size.max(1));
        let slot = offset - self.bucket_size * bucket;
        usize::try_from(bucket)
            .ok()
            .and_then(|bucket| self.buckets.get(bucket))
            .map(|bucket| bucket.slot(slot))
            .unwrap_or_default()
    }

    /// `Clear`: keeps only the first bucket, emptied; `objects.clear()`.
    pub fn clear(&mut self) {
        self.buckets.truncate(1);
        if let Some(first) = self.buckets.first_mut() {
            first.clear();
        }
        self.length = 0;
    }
}

/// `Storage_BucketIterator`: walks every bucket slot up to each bucket's last object. On an
/// empty collection it still yields one (null) slot, as natively.
///
/// ```
/// use rustycad_tkernel::storage::{BucketIterator, BucketOfPersistent};
/// let objects = BucketOfPersistent::new(4, 1);
/// let mut walk = BucketIterator::new(Some(&objects));
/// assert!(walk.more() && walk.value().is_null());
/// walk.next();
/// assert!(!walk.more());
/// ```
pub struct BucketIterator<'a> {
    source: Option<&'a BucketOfPersistent>,
    bucket_index: usize,
    current: usize,
    index: i32,
    bucket_number: usize,
    more: bool,
}

impl<'a> BucketIterator<'a> {
    /// `Storage_BucketIterator(aBucketManager)`; `None` gives an exhausted iterator.
    /// Example: `BucketIterator::new(Some(&objects))`.
    pub fn new(source: Option<&'a BucketOfPersistent>) -> Self {
        let mut iterator = Self {
            source: None,
            bucket_index: 0,
            current: 0,
            index: 0,
            bucket_number: 0,
            more: false,
        };
        if source.is_some() {
            iterator.init(source);
        }
        iterator
    }

    /// `Init`: restarts on `source`; `None` keeps the previous collection but stops.
    /// Example: `walk.init(Some(&objects))`.
    pub fn init(&mut self, source: Option<&'a BucketOfPersistent>) {
        let Some(source) = source else {
            self.more = false;
            return;
        };
        self.source = Some(source);
        self.restart(source);
    }

    fn restart(&mut self, source: &BucketOfPersistent) {
        self.bucket_number = source.buckets.len();
        self.index = 0;
        self.bucket_index = 0;
        self.current = 0;
        self.more = true;
    }

    /// `Reset`: back to the first slot of the collection given last; `walk.reset()`.
    pub fn reset(&mut self) {
        match self.source {
            Some(source) => self.restart(source),
            None => self.more = false,
        }
    }

    /// `Value`: the current slot, null for an empty slot or no collection; `walk.value()`.
    pub fn value(&self) -> Handle<StandardPersistent> {
        self.current_bucket()
            .map(|bucket| bucket.slot(self.index))
            .unwrap_or_default()
    }

    /// `More`; `walk.more()`.
    pub fn more(&self) -> bool {
        self.more
    }

    /// `Next`: the next slot of the bucket, else the first slot of the next bucket.
    /// Example: `walk.next()`.
    pub fn next(&mut self) {
        if !self.more {
            return;
        }
        let last = self.current_bucket().map_or(-1, Bucket::current_space);
        if self.index < last {
            self.index += 1;
            return;
        }
        // Past the last bucket the native iterator keeps pointing at that bucket.
        self.index = 0;
        self.bucket_index += 1;
        if self.bucket_index < self.bucket_number {
            self.current = self.bucket_index;
        } else {
            self.more = false;
        }
    }

    fn current_bucket(&self) -> Option<&'a Bucket> {
        self.source?.buckets.get(self.current)
    }
}
