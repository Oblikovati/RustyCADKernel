use super::{BaseMapBasicIterator, BasicMapIteratorSource, BasicMapNode};

impl<'a, N: BasicMapNode, const M: bool, const K: bool, const O: bool>
    BaseMapBasicIterator<'a, N, M, K, O>
{
    fn from_raw_node(node: *mut N) -> Self {
        Self {
            node,
            ..Self::default()
        }
    }
    fn from_source(source: &'a impl BasicMapIteratorSource<Node = N>) -> Self {
        if O {
            return Self::from_raw_node(source.basic_first_node());
        }
        let mut cursor = Self {
            table: source.basic_bucket_slots(),
            ..Self::default()
        };
        cursor.scan_buckets();
        cursor
    }
}

impl<'a, N: BasicMapNode, const K: bool, const O: bool> BaseMapBasicIterator<'a, N, false, K, O> {
    /// Attaches the const native specialization to a container; `BaseMapBasicIterator::<MyNode>::new(&map)`.
    pub fn new(source: &'a impl BasicMapIteratorSource<Node = N>) -> Self {
        Self::from_source(source)
    }
    /// Starts from a const raw node, traversing only its chain; `Self::from_node(node)`.
    pub fn from_node(node: *const N) -> Self {
        Self::from_raw_node(node.cast_mut())
    }
    /// Copies mutable state into a const cursor (an extension: the pinned native overload is ill-formed); `Self::from_mutable(&writable)`.
    pub fn from_mutable(source: &BaseMapBasicIterator<'a, N, true, K, O>) -> Self {
        source.as_const()
    }
    /// Assigns mutable state into a const cursor (an extension: the pinned native overload is ill-formed); `readonly.assign_from_mutable(&writable)`.
    pub fn assign_from_mutable(
        &mut self,
        source: &BaseMapBasicIterator<'a, N, true, K, O>,
    ) -> &mut Self {
        *self = source.as_const();
        self
    }
}

impl<'a, N: BasicMapNode, const K: bool, const O: bool> BaseMapBasicIterator<'a, N, true, K, O> {
    /// Attaches the mutable native specialization, retaining the table borrow; `BaseMapBasicIterator::<MyNode, true>::new(&mut map)`.
    pub fn new(source: &'a mut impl BasicMapIteratorSource<Node = N>) -> Self {
        Self::from_source(source)
    }
    /// Starts from a writable raw node, traversing only its chain; `Self::from_node(node)`.
    pub fn from_node(node: *mut N) -> Self {
        Self::from_raw_node(node)
    }
}
