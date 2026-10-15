//! The attributes of XCAFDoc derived from `TDataStd_GenericEmpty` (the tools, `XCAFDoc_Dimension`,
//! `XCAFDoc_GeomTolerance`, `XCAFDoc_View`): the RTTI, the empty `Restore` and `Paste` and the
//! derived-attribute registration their native `IMPLEMENT_DERIVED_ATTRIBUTE` expands to.

/// Declares the boilerplate of a `TDataStd_GenericEmpty`-derived XCAFDoc attribute: a struct
/// with the transient, the attribute base and the given state, its RTTI and `GetID`, and a
/// `TdfAttribute` implementation with the empty `Restore`/`Paste`, `NewEmpty`, the given
/// `DumpJson` body (the whole native body, class name included) and the given `Dump`. The constructor registers the derived attributes.
macro_rules! generic_empty_attribute {
    (
        $(#[$doc:meta])* $name:ident, $occt:literal, $id:literal, state: $state:ty,
        dump: $dump:expr $(, after_retrieval: $after:expr)?
    ) => {
        generic_empty_attribute!($(#[$doc])* $name, $occt, $id, state: $state, dump: $dump,
            text: |attribute: &$name, stream: &mut dyn std::io::Write| {
                rustycad_tklcaf::tdf::dump_attribute(attribute, stream)
            } $(, after_retrieval: $after)?);
    };
    (
        $(#[$doc:meta])* $name:ident, $occt:literal, $id:literal, state: $state:ty,
        dump: $dump:expr, text: $text:expr $(, after_retrieval: $after:expr)?
    ) => {
        $(#[$doc])*
        pub struct $name {
            transient: rustycad_tkernel::standard::StandardTransient,
            base: rustycad_tklcaf::tdf::TdfAttributeBase,
            // The tools without a cache (state `()`) never read it.
            #[allow(dead_code)]
            pub(super) state: rustycad_tkernel::standard::SharedState<$state>,
        }
        rustycad_tkernel::impl_transient!($name, $occt, dyn rustycad_tklcaf::tdata_std::TDataStdGenericEmpty,
            transient, interfaces: [dyn rustycad_tklcaf::tdf::TdfAttribute,
            dyn rustycad_tklcaf::tdata_std::TDataStdGenericEmpty]);
        impl rustycad_tklcaf::tdata_std::TDataStdGenericEmpty for $name {}

        impl Default for $name {
            fn default() -> Self {
                super::registration::register_derived_attributes();
                Self {
                    transient: rustycad_tkernel::standard::StandardTransient::default(),
                    base: rustycad_tklcaf::tdf::TdfAttributeBase::default(),
                    state: rustycad_tkernel::standard::SharedState::default(),
                }
            }
        }

        impl $name {
            #[doc = concat!("`", $occt, "()`.")]
            pub fn new() -> Self {
                Self::default()
            }
            /// `GetID`.
            pub fn get_id() -> rustycad_tkernel::standard::Guid {
                const ID: rustycad_tkernel::standard::Guid = rustycad_tklcaf::guid_from_text($id);
                ID
            }
        }

        impl rustycad_tklcaf::tdf::TdfAttribute for $name {
            fn attribute_base(&self) -> &rustycad_tklcaf::tdf::TdfAttributeBase {
                &self.base
            }
            fn id(&self) -> rustycad_tkernel::standard::Guid {
                Self::get_id()
            }
            /// `Restore` of `TDataStd_GenericEmpty`: nothing to restore.
            fn restore(
                &self,
                _with: &rustycad_tkernel::standard::Handle<dyn rustycad_tklcaf::tdf::TdfAttribute>,
            ) -> Result<(), rustycad_tkernel::standard::Failure> {
                Ok(())
            }
            /// `NewEmpty` (from `IMPLEMENT_DERIVED_ATTRIBUTE`).
            fn new_empty(&self) -> rustycad_tkernel::standard::Handle<dyn rustycad_tklcaf::tdf::TdfAttribute> {
                rustycad_tkernel::standard::Handle::new(Self::new()).cast()
            }
            /// `Paste` of `TDataStd_GenericEmpty`: nothing to paste.
            fn paste(
                &self,
                _into: &rustycad_tkernel::standard::Handle<dyn rustycad_tklcaf::tdf::TdfAttribute>,
                _table: &rustycad_tkernel::standard::Handle<rustycad_tklcaf::tdf::TdfRelocationTable>,
            ) -> Result<(), rustycad_tkernel::standard::Failure> {
                Ok(())
            }
            fn dump(&self, stream: &mut dyn std::io::Write) -> std::io::Result<()> {
                ($text)(self, stream)
            }
            $(
                fn after_retrieval(&self, force_it: bool) -> Result<bool, rustycad_tkernel::standard::Failure> {
                    ($after)(self, force_it)
                }
            )?
            fn dump_json(
                &self,
                stream: &mut rustycad_tkernel::standard::dump::DumpStream,
                depth: i32,
            ) -> std::io::Result<()> {
                ($dump)(self, stream, depth)
            }
        }
    };
}
pub(crate) use generic_empty_attribute;

/// `OCCT_DUMP_BASE_CLASS(theOStream, theDepth, TDataStd_GenericEmpty)`.
pub(crate) fn dump_generic_empty_base<
    A: rustycad_tklcaf::tdata_std::TDataStdGenericEmpty + ?Sized,
>(
    attribute: &A,
    stream: &mut rustycad_tkernel::standard::dump::DumpStream,
    depth: i32,
) -> std::io::Result<()> {
    rustycad_tkservice::json_dump::base_class(stream, depth, &|s, d| {
        rustycad_tklcaf::tdata_std::dump_generic_empty(attribute, s, d)
    })
}
