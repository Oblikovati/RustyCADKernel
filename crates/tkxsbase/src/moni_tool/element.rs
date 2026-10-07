// Rust port of DataExchange/TKXSBase/MoniTool/MoniTool_Element (.hxx, .cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `MoniTool_Element`: maps any kind of object as a key of a map, with the hash code of
//! the real key computed when the element is created.
use super::MoniToolAttrList;
use rustycad_tkernel::{
    impl_transient_interface,
    standard::{Handle, RuntimeType, SharedState, StandardTransient, Transient},
};
use std::sync::{RwLockReadGuard, RwLockWriteGuard};

/// The private fields of `MoniTool_Element`: the stored hash code and the attributes.
#[derive(Default)]
pub struct MoniToolElementBase {
    thecode: SharedState<usize>,
    theattrib: SharedState<MoniToolAttrList>,
}

/// `MoniTool_Element`, abstract: an element compares its value with another one
/// (`Equates`) and gives the hash code stored at creation (`GetHashCode`).
///
/// ```
/// use rustycad_tkernel::{impl_transient, standard::{Handle, StandardTransient}};
/// use rustycad_tkxsbase::moni_tool::{MoniToolElement, MoniToolElementBase};
/// #[derive(Default)]
/// struct Key { transient: StandardTransient, base: MoniToolElementBase }
/// impl_transient!(Key, "Doc_Key", dyn MoniToolElement, transient, interfaces: [dyn MoniToolElement]);
/// impl MoniToolElement for Key {
///     fn element_base(&self) -> &MoniToolElementBase { &self.base }
///     fn equates(&self, other: &Handle<dyn MoniToolElement>) -> bool { other.cast::<Key>().is_some() }
/// }
/// let key = Key::default();
/// key.set_hash_code(17);
/// assert_eq!(key.get_hash_code(), 17);
/// assert_eq!(key.value_type_name(), "(finder)");
/// ```
pub trait MoniToolElement: Transient {
    /// The embedded private fields.
    fn element_base(&self) -> &MoniToolElementBase;
    /// `Equates`: false for another true type, else compares the contents.
    fn equates(&self, other: &Handle<dyn MoniToolElement>) -> bool;
    /// `ValueType`: by default the dynamic type of the element.
    fn value_type(&self) -> &'static RuntimeType {
        self.dynamic_type()
    }
    /// `ValueTypeName`: by default "(finder)".
    fn value_type_name(&self) -> String {
        String::from("(finder)")
    }
    /// `GetHashCode`: the code stored by `SetHashCode` (0 by default).
    fn get_hash_code(&self) -> usize {
        *self.element_base().thecode.read()
    }
    /// `SetHashCode` (protected): stores the hash code of the value.
    fn set_hash_code(&self, code: usize) {
        *self.element_base().thecode.write() = code;
    }
    /// `ListAttr`: the attribute list, read only (a guard to drop before editing it).
    fn list_attr(&self) -> RwLockReadGuard<'_, MoniToolAttrList> {
        self.element_base().theattrib.read()
    }
    /// `ChangeAttr`: the attribute list, modifiable.
    fn change_attr(&self) -> RwLockWriteGuard<'_, MoniToolAttrList> {
        self.element_base().theattrib.write()
    }
}
impl_transient_interface!(dyn MoniToolElement, "MoniTool_Element", StandardTransient);

#[cfg(test)]
mod tests {
    use super::*;
    use rustycad_tkernel::{impl_transient, standard::RuntimeTyped};

    #[derive(Default)]
    struct IntElement {
        transient: StandardTransient,
        base: MoniToolElementBase,
        value: i32,
    }
    impl_transient!(IntElement, "Test_IntElement", dyn MoniToolElement, transient, interfaces: [dyn MoniToolElement]);
    impl MoniToolElement for IntElement {
        fn element_base(&self) -> &MoniToolElementBase {
            &self.base
        }
        fn equates(&self, other: &Handle<dyn MoniToolElement>) -> bool {
            other
                .cast::<IntElement>()
                .get()
                .is_some_and(|o| o.value == self.value)
        }
    }

    #[test]
    fn hash_code_attributes_and_rtti() {
        let element = Handle::new(IntElement {
            value: 3,
            ..Default::default()
        });
        assert_eq!(element.get_hash_code(), 0);
        element.set_hash_code(3);
        assert_eq!(element.get_hash_code(), 3);
        element.change_attr().set_integer_attribute("n", 9);
        assert_eq!(element.list_attr().integer_attribute("n"), 9);
        let as_element: Handle<dyn MoniToolElement> = element.cast();
        assert!(element.equates(&as_element));
        assert_eq!(as_element.value_type().name(), "Test_IntElement");
        assert!(element.is_kind(<dyn MoniToolElement>::static_type()));
    }
}
