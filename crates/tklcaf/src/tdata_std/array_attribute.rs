//! The declaration macro of the four array attributes over the array core
//! (ocaf-design.md section 7). Every generated member forwards to `array_core`.

/// Declares one array attribute class: its struct, RTTI, constants and members.
macro_rules! array_attribute {
    (
        $(#[$doc:meta])* $name:ident, $occt:literal, item: $item:ty, guid: $guid:literal,
        fill: $fill:expr, null_value: $null_value:expr, null_upper: $null_upper:expr,
        guards_null: $guards_null:expr, id_first: $id_first:expr, delta: $delta:ident,
        array: $array_fn:ident, dump: $dump:path, dump_item: $dump_item:path
    ) => {
        $(#[$doc])*
        pub struct $name {
            transient: StandardTransient,
            base: TdfAttributeBase,
            state: SharedState<ArrayState<$item>>,
        }
        impl_transient!($name, $occt, dyn TdfAttribute, transient, interfaces: [dyn TdfAttribute]);

        impl Default for $name {
            /// The native constructor: no array, delta off, the default ID.
            fn default() -> Self {
                Self {
                    transient: StandardTransient::default(),
                    base: TdfAttributeBase::default(),
                    state: SharedState::new(ArrayState::new(guid_from_text($guid))),
                }
            }
        }

        impl ArrayAttribute for $name {
            type Item = $item;
            const FILL: fn() -> $item = $fill;
            const NULL_VALUE: fn() -> $item = $null_value;
            const NULL_UPPER: i32 = $null_upper;
            const CHANGE_GUARDS_NULL: bool = $guards_null;
            const SET_ID_FIRST: bool = $id_first;
            const DEFAULT_ID: Guid = guid_from_text($guid);
            fn make() -> Self {
                Self::default()
            }
            fn array_state(&self) -> &SharedState<ArrayState<$item>> {
                &self.state
            }
            fn dump_item(stream: &mut DumpStream, item: &$item) -> io::Result<()> {
                $dump_item(stream, item)
            }
        }

        impl $name {
            /// `GetID`: the default ID.
            pub fn get_id() -> Guid {
                guid_from_text($guid)
            }
            /// The default constructor (an attribute without array).
            pub fn new() -> Self {
                Self::default()
            }
            /// `Set(label, lower, upper, isDelta)`: the attribute of the default ID, added
            /// with a new array when absent, re-initialized when its bounds differ.
            pub fn set(label: &TdfLabel, lower: i32, upper: i32, is_delta: bool) -> Result<Handle<Self>, Failure> {
                set_attr(label, &Self::get_id(), (lower, upper), is_delta)
            }
            /// `Set(label, guid, lower, upper, isDelta)`: the same with a user ID.
            pub fn set_with_id(
                label: &TdfLabel,
                guid: &Guid,
                lower: i32,
                upper: i32,
                is_delta: bool,
            ) -> Result<Handle<Self>, Failure> {
                set_attr(label, guid, (lower, upper), is_delta)
            }
            /// `Init(lower, upper)`: a new filled array; `Standard_RangeError` when
            /// `upper < lower`.
            pub fn init(&self, lower: i32, upper: i32) -> Result<(), Failure> {
                init(self, lower, upper)
            }
            /// `SetValue(index, value)`: backed up when the item changes.
            pub fn set_value(&self, index: i32, value: $item) -> Result<(), Failure> {
                set_value(self, index, value)
            }
            /// `Value(index)` and `operator()`; `Standard_OutOfRange` outside the bounds.
            pub fn value(&self, index: i32) -> Result<$item, Failure> {
                value(self, index)
            }
            /// `Lower`.
            pub fn lower(&self) -> i32 {
                lower_of(self)
            }
            /// `Upper`.
            pub fn upper(&self) -> i32 {
                upper_of(self)
            }
            /// `Length`.
            pub fn length(&self) -> i32 {
                length_of(self)
            }
            /// `ChangeArray(newArray, isCheckItems)`: takes the items of `new_array`.
            pub fn change_array(&self, new_array: &Handle<HArray1<$item>>, check_items: bool) -> Result<(), Failure> {
                change_array(self, new_array, check_items)
            }
            /// The inner array, shared with the attribute (null without array).
            pub fn $array_fn(&self) -> Handle<HArray1<$item>> {
                array(self)
            }
            /// `GetDelta`.
            pub fn get_delta(&self) -> bool {
                self.state.read().is_delta
            }
            /// `SetDelta`: no backup, as natively.
            pub fn set_delta(&self, is_delta: bool) {
                self.state.write().is_delta = is_delta;
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
            fn paste(&self, into: &Handle<dyn TdfAttribute>, _table: &Handle<TdfRelocationTable>) -> Result<(), Failure> {
                paste(self, into)
            }
            fn dump(&self, stream: &mut dyn io::Write) -> io::Result<()> {
                $dump(self, stream)
            }
            fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
                dump_json(self, stream, depth)
            }
            /// `DeltaOnModification(OldAttribute)`: the array delta when the delta flag is
            /// on, else the default delta.
            fn delta_on_modification(
                &self,
                old_attribute: &Handle<dyn TdfAttribute>,
            ) -> Result<Handle<dyn TdfDeltaOnModification>, Failure> {
                if !self.get_delta() {
                    return Ok(TdfDefaultDeltaOnModification::new(old_attribute).cast());
                }
                Ok($delta::new(&old_attribute.cast::<$name>())?.cast())
            }
        }
    };
}
pub(crate) use array_attribute;
