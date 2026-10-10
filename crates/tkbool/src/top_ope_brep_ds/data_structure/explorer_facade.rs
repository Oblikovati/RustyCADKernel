// The native PointExplorer and CurveExplorer have identical cursor operations.
// Keep the typed API names while sharing the traversal and borrowing contracts.
macro_rules! geometry_explorer_facade {
    ($name:ident, $geometry:ty, $current:ident, $at:ident, $bound:ident, $kept:ident, $count:ident) => {
        #[doc = concat!("Traverses bound geometry indices, optionally retaining only kept entries.\n",
            "The borrow keeps returned geometries tied to their source; counting exhausts the cursor.\n",
            "Example: `let mut explorer = ", stringify!($name), "::new(&source, true);`.")]
        pub struct $name<'a> {
            cursor: super::explorer_core::GeometryExplorer<'a, $geometry>,
        }

        impl Default for $name<'_> {
            fn default() -> Self { Self { cursor: Default::default() } }
        }

        impl<'a> $name<'a> {
            /// Starts at the first matching bound index; `Self::new(&source, true)`.
            pub fn new(source: &'a super::TopOpeBRepDSDataStructure, find_keep: bool) -> Self {
                Self { cursor: super::explorer_core::GeometryExplorer::new(source, find_keep) }
            }
            /// Restarts against a source and filter; `explorer.init(&source, false)`.
            pub fn init(&mut self, source: &'a super::TopOpeBRepDSDataStructure, find_keep: bool) {
                self.cursor.init(source, find_keep);
            }
            /// Allows source edits between steps without detached pointers; `Self::new_mut(&mut ds, true)`.
            pub fn new_mut(source: &'a mut super::TopOpeBRepDSDataStructure, find_keep: bool) -> Self {
                Self { cursor: super::explorer_core::GeometryExplorer::new_mut(source, find_keep) }
            }
            /// Returns editable source for an exclusive explorer; `explorer.source_mut()?.init()`.
            /// Shared and unbound explorers return `None`.
            pub fn source_mut(&mut self) -> Option<&mut super::TopOpeBRepDSDataStructure> {
                self.cursor.source_mut()
            }
            /// Reports whether a matching entry is current; `assert!(explorer.more())`.
            pub fn more(&self) -> bool { self.cursor.more() }
            /// Advances past holes and, when requested, discarded entries; `explorer.next()`.
            pub fn next(&mut self) { self.cursor.next(); }
            #[doc = concat!("Tests binding independently of Keep and the traversal limit; `explorer.", stringify!($bound), "(4)`.")]
            pub fn $bound(&self, index: i32) -> bool { self.cursor.is_bound(index) }
            #[doc = concat!("Requires both a bound geometry and its Keep flag; `explorer.", stringify!($kept), "(4)`.")]
            pub fn $kept(&self, index: i32) -> bool { self.cursor.is_kept(index) }
            #[doc = concat!("Looks up a bound geometry regardless of the filter; `explorer.", stringify!($at), "(4)`.")]
            pub fn $at(&self, index: i32) -> &$geometry { self.cursor.at(index) }
            #[doc = concat!("Recounts from index one and leaves the explorer exhausted; `explorer.", stringify!($count), "()`.")]
            pub fn $count(&mut self) -> i32 { self.cursor.count() }
            /// Returns the cursor index, including past-the-end indices; `explorer.index()`.
            pub fn index(&self) -> i32 { self.cursor.index() }
        }
    };
}
pub(super) use geometry_explorer_facade;
