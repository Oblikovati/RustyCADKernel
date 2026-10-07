// Rust port of ApplicationFramework/TKLCAF/TDF/TDF_DeltaOnAddition, TDF_DeltaOnForget,
// TDF_DeltaOnResume, TDF_DeltaOnRemoval, TDF_DeltaOnModification,
// TDF_DefaultDeltaOnRemoval and TDF_DefaultDeltaOnModification (.hxx and .cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The attribute deltas the framework makes on commit (ocaf-design.md section 5).
use super::{
    TdfAttribute, TdfAttributeDelta, TdfAttributeDeltaBase, attribute_delta::dump_attribute_delta,
};
use crate::live;
use rustycad_tkcdf::json_dump::{base_class, class_begin};
use rustycad_tkernel::{
    impl_transient, impl_transient_interface,
    standard::{Failure, Handle, StandardTransient, dump::DumpStream},
};
use std::io;

/// Declares a concrete delta struct over the attribute-delta base.
macro_rules! concrete_delta {
    ($(#[$doc:meta])* $name:ident, $occt:literal, $parent:ty, interfaces: [$($interface:ty),*]) => {
        $(#[$doc])*
        pub struct $name {
            transient: StandardTransient,
            base: TdfAttributeDeltaBase,
        }
        impl_transient!($name, $occt, $parent, transient, interfaces: [dyn TdfAttributeDelta $(, $interface)*]);
        impl $name {
            /// The constructor `(anAttribute)`, already in its bound handle.
            pub fn new(attribute: &Handle<dyn TdfAttribute>) -> Handle<Self> {
                TdfAttributeDeltaBase::share(Self {
                    transient: StandardTransient::default(),
                    base: TdfAttributeDeltaBase::new(attribute),
                })
            }
        }
    };
}

concrete_delta!(
    /// `TDF_DeltaOnAddition`: undoing an addition forgets the attribute.
    TdfDeltaOnAddition, "TDF_DeltaOnAddition", dyn TdfAttributeDelta, interfaces: []
);
impl TdfAttributeDelta for TdfDeltaOnAddition {
    fn attribute_delta_base(&self) -> &TdfAttributeDeltaBase {
        &self.base
    }
    /// `Apply`: forgets the attribute of the same ID on the label, if any.
    fn apply(&self) -> Result<(), Failure> {
        let mut current = Handle::default();
        if self.label().find_attribute(&self.id()?, &mut current)? {
            self.label().forget_attribute(&current)?;
        }
        Ok(())
    }
}

concrete_delta!(
    /// `TDF_DeltaOnForget`: undoing a forget resumes the attribute.
    TdfDeltaOnForget, "TDF_DeltaOnForget", dyn TdfAttributeDelta, interfaces: []
);
impl TdfAttributeDelta for TdfDeltaOnForget {
    fn attribute_delta_base(&self) -> &TdfAttributeDeltaBase {
        &self.base
    }
    /// `Apply`: resumes the attribute and clears its saved transaction.
    fn apply(&self) -> Result<(), Failure> {
        let attribute = self.attribute();
        self.label().resume_attribute(&attribute)?;
        live(&attribute, "TDF_DeltaOnForget::Apply")?
            .attribute_base()
            .set_saved_transaction(0);
        Ok(())
    }
}

concrete_delta!(
    /// `TDF_DeltaOnResume`: undoing a resume forgets the attribute.
    TdfDeltaOnResume, "TDF_DeltaOnResume", dyn TdfAttributeDelta, interfaces: []
);
impl TdfAttributeDelta for TdfDeltaOnResume {
    fn attribute_delta_base(&self) -> &TdfAttributeDeltaBase {
        &self.base
    }
    /// `Apply`: forgets the attribute.
    fn apply(&self) -> Result<(), Failure> {
        self.label().forget_attribute(&self.attribute())
    }
    /// `DumpJson`: the class name and the base-class dump.
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        class_begin(stream, "TDF_DeltaOnResume")?;
        base_class(stream, depth, &|s, d| dump_attribute_delta(self, s, d))
    }
}

/// `TDF_DeltaOnRemoval`: the abstract delta of a removed attribute (its constructor is
/// protected; `TDF_DefaultDeltaOnRemoval` and TNaming derive from it).
pub trait TdfDeltaOnRemoval: TdfAttributeDelta {}
impl_transient_interface!(
    dyn TdfDeltaOnRemoval,
    "TDF_DeltaOnRemoval",
    dyn TdfAttributeDelta
);

/// `TDF_DeltaOnModification`: the delta of a modified attribute (its constructor is
/// protected; TDataStd and TNaming derive from it). Its `Apply` is
/// [`TdfDeltaOnModification::apply_modification`], which an implementation's `apply` calls.
pub trait TdfDeltaOnModification: TdfAttributeDelta {
    /// `TDF_DeltaOnModification::Apply`: the attribute applies this delta to itself.
    fn apply_modification(&self) -> Result<(), Failure> {
        let this = self
            .attribute_delta_base()
            .this()?
            .cast::<dyn TdfDeltaOnModification>();
        live(&self.attribute(), "TDF_DeltaOnModification::Apply")?
            .apply_delta_on_modification(&this)
    }
}
impl_transient_interface!(
    dyn TdfDeltaOnModification,
    "TDF_DeltaOnModification",
    dyn TdfAttributeDelta
);

concrete_delta!(
    /// `TDF_DefaultDeltaOnRemoval`: undoing a removal adds the attribute back at the end.
    TdfDefaultDeltaOnRemoval, "TDF_DefaultDeltaOnRemoval", dyn TdfDeltaOnRemoval, interfaces: [dyn TdfDeltaOnRemoval]
);
impl TdfAttributeDelta for TdfDefaultDeltaOnRemoval {
    fn attribute_delta_base(&self) -> &TdfAttributeDeltaBase {
        &self.base
    }
    /// `Apply`: adds the attribute to the label again (appended).
    fn apply(&self) -> Result<(), Failure> {
        self.label().add_attribute(&self.attribute(), true)
    }
}
impl TdfDeltaOnRemoval for TdfDefaultDeltaOnRemoval {}

concrete_delta!(
    /// `TDF_DefaultDeltaOnModification`: undoing a modification restores the old copy into
    /// the current attribute of the same ID.
    TdfDefaultDeltaOnModification, "TDF_DefaultDeltaOnModification", dyn TdfDeltaOnModification,
    interfaces: [dyn TdfDeltaOnModification]
);
impl TdfAttributeDelta for TdfDefaultDeltaOnModification {
    fn attribute_delta_base(&self) -> &TdfAttributeDeltaBase {
        &self.base
    }
    /// `Apply`: the current attribute of the saved one's ID applies this delta.
    fn apply(&self) -> Result<(), Failure> {
        let saved = self.attribute();
        let id = live(&saved, "TDF_DefaultDeltaOnModification::Apply")?.id();
        let mut current = Handle::default();
        if !self.label().find_attribute(&id, &mut current)? {
            return Ok(());
        }
        let this = self
            .attribute_delta_base()
            .this()?
            .cast::<dyn TdfDeltaOnModification>();
        live(&current, "TDF_DefaultDeltaOnModification::Apply")?.apply_delta_on_modification(&this)
    }
}
impl TdfDeltaOnModification for TdfDefaultDeltaOnModification {}
