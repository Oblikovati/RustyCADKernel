// Rust port of DataExchange/TKXSBase/Transfer/Transfer_TransientMapper (.hxx, _0.cxx) and
// Transfer_DataInfo (.hxx, .cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Transfer_TransientMapper`: a finder on a transient, hashed by its address;
//! `Transfer_DataInfo`: the type of a transient value.
use super::{TransferFinder, TransferFinderBase};
use rustycad_tkernel::{
    impl_transient,
    standard::{Handle, RuntimeType, RuntimeTyped, StandardTransient},
};

/// `Transfer_TransientMapper`.
///
/// ```
/// use rustycad_tkxsbase::transfer::{TransferFinder, TransferTransientMapper};
/// use rustycad_tkernel::standard::{Handle, StandardTransient};
/// let key = Handle::new(StandardTransient::default());
/// let first: Handle<dyn TransferFinder> = Handle::new(TransferTransientMapper::new(&key)).cast();
/// let second: Handle<dyn TransferFinder> = Handle::new(TransferTransientMapper::new(&key)).cast();
/// assert!(first.equates(&second));
/// ```
pub struct TransferTransientMapper {
    transient: StandardTransient,
    base: TransferFinderBase,
    theval: Handle<StandardTransient>,
}
impl_transient!(TransferTransientMapper, "Transfer_TransientMapper", dyn TransferFinder, transient, interfaces: [dyn TransferFinder]);

impl TransferTransientMapper {
    /// `Transfer_TransientMapper(akey)`: the hash code is the key's address hash.
    pub fn new(akey: &Handle<StandardTransient>) -> Self {
        let mapper = Self {
            transient: StandardTransient::default(),
            base: TransferFinderBase::default(),
            theval: akey.clone(),
        };
        mapper.set_hash_code(akey.object_address() as usize);
        mapper
    }
    /// `Value`.
    pub fn value(&self) -> &Handle<StandardTransient> {
        &self.theval
    }
}

impl TransferFinder for TransferTransientMapper {
    fn finder_base(&self) -> &TransferFinderBase {
        &self.base
    }
    /// `Equates(other)`: same hash code, same dynamic type, same value.
    fn equates(&self, other: &Handle<dyn TransferFinder>) -> bool {
        let Some(live) = other.get() else {
            return false;
        };
        if self.get_hash_code() != live.get_hash_code() {
            return false;
        }
        if !std::ptr::eq(live.dynamic_type(), Self::static_type()) {
            return false;
        }
        other
            .cast::<TransferTransientMapper>()
            .get()
            .is_some_and(|m| m.theval == self.theval)
    }
    /// `ValueType`: `Transfer_DataInfo::Type` of the value.
    fn value_type(&self) -> &'static RuntimeType {
        TransferDataInfo::type_of(&self.theval)
    }
    /// `ValueTypeName`: `Transfer_DataInfo::TypeName` of the value.
    fn value_type_name(&self) -> String {
        TransferDataInfo::type_name(&self.theval)
    }
}

/// `Transfer_DataInfo`: the type of a transient value of a transfer.
pub struct TransferDataInfo;

impl TransferDataInfo {
    /// `Type(ent)`: its dynamic type (`Standard_Transient` for null, which the native
    /// code dereferences).
    pub fn type_of(ent: &Handle<StandardTransient>) -> &'static RuntimeType {
        ent.dynamic_type()
            .unwrap_or_else(StandardTransient::static_type)
    }
    /// `TypeName(ent)`.
    pub fn type_name(ent: &Handle<StandardTransient>) -> String {
        Self::type_of(ent).name().to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rustycad_tkernel::tcollection::HAsciiString;

    #[test]
    fn mapper_identity_and_types() {
        let text: Handle<StandardTransient> = Handle::new(HAsciiString::new("x")).cast();
        let mapper = TransferTransientMapper::new(&text);
        assert!(*mapper.value() == text);
        assert_eq!(mapper.value_type().name(), "TCollection_HAsciiString");
        assert_eq!(mapper.value_type_name(), "TCollection_HAsciiString");
        let other: Handle<dyn TransferFinder> = Handle::new(TransferTransientMapper::new(
            &Handle::new(StandardTransient::default()),
        ))
        .cast();
        assert!(!mapper.equates(&other) && !mapper.equates(&Handle::default()));
        assert_eq!(
            TransferDataInfo::type_name(&Handle::default()),
            "Standard_Transient"
        );
        mapper.set_real_attribute("r", 2.5);
        assert_eq!(
            mapper.attribute_type("r"),
            crate::interface::InterfaceParamType::ParamReal
        );
        assert_eq!(mapper.real_attribute("r"), 2.5);
        mapper.set_string_attribute("s", "t");
        let copy = TransferTransientMapper::new(&text);
        copy.get_attributes(
            &Handle::new(TransferTransientMapper::new(&text)).cast(),
            "",
            true,
        );
        let source: Handle<dyn TransferFinder> =
            Handle::new(TransferTransientMapper::new(&text)).cast();
        source.set_integer_attribute("i", 3);
        copy.same_attributes(&source);
        assert_eq!(copy.integer_attribute("i"), 3);
        assert_eq!(copy.attr_list().len(), 1);
        assert!(copy.remove_attribute("i") && !copy.remove_attribute("i"));
        assert_eq!(mapper.string_attribute("s").as_bytes(), b"t");
    }
}
