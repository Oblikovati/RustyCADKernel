//! The declaration macros of the list attributes over the list core (ocaf-design.md
//! section 7). Every generated member forwards to `list_core`.

/// Declares the struct, RTTI, constants and the members every list class has.
macro_rules! list_attribute_base {
    (
        $(#[$doc:meta])* $name:ident, $occt:literal, $dump_name:literal, stored: $stored:ty,
        guid: $guid:literal, dump_item: $dump_item:path
    ) => {
        $(#[$doc])*
        pub struct $name {
            transient: StandardTransient,
            base: TdfAttributeBase,
            state: SharedState<ListState<$stored>>,
        }
        impl_transient!($name, $occt, dyn TdfAttribute, transient, interfaces: [dyn TdfAttribute]);

        impl Default for $name {
            /// The native constructor: an empty list and the default ID.
            fn default() -> Self {
                Self {
                    transient: StandardTransient::default(),
                    base: TdfAttributeBase::default(),
                    state: SharedState::new(ListState::new(guid_from_text($guid))),
                }
            }
        }

        impl ListAttribute for $name {
            type Item = $stored;
            const DEFAULT_ID: Guid = guid_from_text($guid);
            const DUMP_NAME: &'static str = $dump_name;
            fn make() -> Self {
                Self::default()
            }
            fn list_state(&self) -> &SharedState<ListState<$stored>> {
                &self.state
            }
            fn dump_item(stream: &mut DumpStream, item: &$stored) -> io::Result<()> {
                $dump_item(stream, item)
            }
        }

        impl $name {
            /// `GetID`: the default ID.
            pub fn get_id() -> Guid {
                guid_from_text($guid)
            }
            /// The default constructor (an empty list).
            pub fn new() -> Self {
                Self::default()
            }
            /// `Set(label)`: the attribute of the default ID, added when absent.
            pub fn set(label: &TdfLabel) -> Result<Handle<Self>, Failure> {
                set_attr(label, &Self::get_id())
            }
            /// `Set(label, guid)`: the attribute of a user ID, added when absent.
            pub fn set_with_id(label: &TdfLabel, guid: &Guid) -> Result<Handle<Self>, Failure> {
                set_attr(label, guid)
            }
            /// `IsEmpty`.
            pub fn is_empty(&self) -> bool {
                extent(self) == 0
            }
            /// `Extent`.
            pub fn extent(&self) -> i32 {
                extent(self)
            }
            /// `Clear`: backs up, then empties the list.
            pub fn clear(&self) -> Result<(), Failure> {
                edit(self, Vec::clear)
            }
            /// `Remove(index)`: removes the item at the 1-based index; false outside the list.
            pub fn remove_by_index(&self, index: i32) -> Result<bool, Failure> {
                remove_by_index(self, index)
            }
        }
    };
}
pub(crate) use list_attribute_base;

/// Declares a list class whose items are stored as given: the base members, the value
/// members and the `TdfAttribute` impl (with `paste` and `references` from the caller).
macro_rules! list_attribute {
    (
        $(#[$doc:meta])* $name:ident, $occt:literal, $dump_name:literal, item: $item:ty,
        guid: $guid:literal, dump_item: $dump_item:path, paste: $paste:path,
        references: $references:path
    ) => {
        list_attribute_base!($(#[$doc])* $name, $occt, $dump_name, stored: $item, guid: $guid, dump_item: $dump_item);

        impl $name {
            /// `Prepend`: backs up, then adds the item in front.
            pub fn prepend(&self, value: $item) -> Result<(), Failure> {
                edit(self, |list| list.insert(0, value))
            }
            /// `Append`: backs up, then adds the item at the end.
            pub fn append(&self, value: $item) -> Result<(), Failure> {
                edit(self, |list| list.push(value))
            }
            /// `InsertBefore(value, before_value)`: false when no item equals `before_value`.
            pub fn insert_before(&self, value: $item, before_value: &$item) -> Result<bool, Failure> {
                insert_before(self, value, before_value)
            }
            /// `InsertBefore(index, value)` (`InsertBeforeByIndex`): false outside the list.
            pub fn insert_before_by_index(&self, index: i32, value: $item) -> Result<bool, Failure> {
                insert_before_by_index(self, index, value)
            }
            /// `InsertAfter(value, after_value)`: false when no item equals `after_value`.
            pub fn insert_after(&self, value: $item, after_value: &$item) -> Result<bool, Failure> {
                insert_after(self, value, after_value)
            }
            /// `InsertAfter(index, value)` (`InsertAfterByIndex`): false outside the list.
            pub fn insert_after_by_index(&self, index: i32, value: $item) -> Result<bool, Failure> {
                insert_after_by_index(self, index, value)
            }
            /// `Remove(value)`: removes the first equal item; false when there is none.
            pub fn remove(&self, value: &$item) -> Result<bool, Failure> {
                remove(self, value)
            }
            /// `First`; `Standard_NoSuchObject` for an empty list.
            pub fn first(&self) -> Result<$item, Failure> {
                first(self)
            }
            /// `Last`; `Standard_NoSuchObject` for an empty list.
            pub fn last(&self) -> Result<$item, Failure> {
                last(self)
            }
            /// `List`: a copy of the items in order.
            pub fn list(&self) -> Vec<$item> {
                items(self)
            }
        }

        impl TdfAttribute for $name {
            fn attribute_base(&self) -> &TdfAttributeBase {
                &self.base
            }
            fn id(&self) -> Guid {
                self.state.read().id
            }
            fn set_id(&self, guid: &Guid) -> Result<(), Failure> {
                set_id(self, guid)
            }
            fn set_default_id(&self) -> Result<(), Failure> {
                set_default_id(self)
            }
            fn restore(&self, with: &Handle<dyn TdfAttribute>) -> Result<(), Failure> {
                restore(self, with)
            }
            fn new_empty(&self) -> Handle<dyn TdfAttribute> {
                Handle::new(Self::new()).cast()
            }
            fn paste(&self, into: &Handle<dyn TdfAttribute>, table: &Handle<TdfRelocationTable>) -> Result<(), Failure> {
                $paste(self, into, table)
            }
            fn references(&self, data_set: &Handle<TdfDataSet>) {
                $references(self, data_set)
            }
            fn dump(&self, stream: &mut dyn io::Write) -> io::Result<()> {
                dump(self, stream)
            }
            fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
                dump_json(self, stream, depth)
            }
        }
    };
}
pub(crate) use list_attribute;
