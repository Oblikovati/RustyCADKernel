// Rust port of ApplicationFramework/TKTObj/TObj/TObj_TXYZ.hxx and .cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TObj_TXYZ`: the OCAF attribute that stores a `gp_XYZ`.
use rustycad_tkernel::{
    impl_transient,
    standard::{Failure, Guid, Handle, SharedState, StandardTransient},
    tcollection::AsciiString,
};
use rustycad_tklcaf::{
    guid_from_text,
    tdata_std::support::{cast_attribute, find_or_add},
    tdf::{
        TdfAttribute, TdfAttributeBase, TdfAttributeState, TdfLabel, TdfRelocationTable,
        dump_attribute,
    },
};
use rustycad_tkmath::gp::Xyz;
use std::io;

/// `TObj_TXYZ::GetID()`.
const TXYZ_ID: Guid = guid_from_text("3bbefb50-e618-11d4-ba38-0060b0ee18ea");

/// `TObj_TXYZ`; `TObjTXyz::set(&label, &Xyz::new(1.0, 2.0, 3.0))`.
///
/// ```
/// use rustycad_tklcaf::tdf::TdfData;
/// use rustycad_tkmath::gp::Xyz;
/// use rustycad_tktobj::tobj::TObjTXyz;
/// let data = TdfData::new();
/// let attribute = TObjTXyz::set(&data.root(), &Xyz::new(1.0, 2.0, 3.0)).unwrap();
/// assert_eq!(attribute.get_xyz(), Xyz::new(1.0, 2.0, 3.0));
/// ```
#[derive(Default)]
pub struct TObjTXyz {
    transient: StandardTransient,
    base: TdfAttributeBase,
    xyz: SharedState<Xyz>,
}
impl_transient!(TObjTXyz, "TObj_TXYZ", dyn TdfAttribute, transient, interfaces: [dyn TdfAttribute]);

impl TObjTXyz {
    /// `GetID`.
    pub fn get_id() -> Guid {
        TXYZ_ID
    }
    /// `TObj_TXYZ()`: the origin.
    pub fn new() -> Self {
        Self::default()
    }
    /// `Set(theLabel, theXYZ)` (static): the attribute of the label, added when absent.
    pub fn set(label: &TdfLabel, xyz: &Xyz) -> Result<Handle<TObjTXyz>, Failure> {
        let attribute = find_or_add(label, &TXYZ_ID, false, Self::new)?;
        rustycad_tkcdf::live(&attribute, "TObj_TXYZ::Set")?.set_xyz(xyz)?;
        Ok(attribute)
    }
    /// `Set(theXYZ)`: backed up, then set.
    pub fn set_xyz(&self, xyz: &Xyz) -> Result<(), Failure> {
        self.backup()?;
        *self.xyz.write() = *xyz;
        Ok(())
    }
    /// `Get`.
    pub fn get_xyz(&self) -> Xyz {
        *self.xyz.read()
    }
}

/// A double as `operator<<` writes it (six significant digits).
fn stream_real(value: f64) -> String {
    String::from_utf8_lossy(AsciiString::from_real(value).as_bytes()).into_owned()
}

impl TdfAttribute for TObjTXyz {
    fn attribute_base(&self) -> &TdfAttributeBase {
        &self.base
    }
    fn id(&self) -> Guid {
        TXYZ_ID
    }
    fn restore(&self, with: &Handle<dyn TdfAttribute>) -> Result<(), Failure> {
        let other = cast_attribute::<TObjTXyz>(with, "TObj_TXYZ::Restore")?;
        *self.xyz.write() = other.get_xyz();
        Ok(())
    }
    fn new_empty(&self) -> Handle<dyn TdfAttribute> {
        Handle::new(Self::new()).cast()
    }
    fn paste(
        &self,
        into: &Handle<dyn TdfAttribute>,
        _relocation_table: &Handle<TdfRelocationTable>,
    ) -> Result<(), Failure> {
        let target = cast_attribute::<TObjTXyz>(into, "TObj_TXYZ::Paste")?;
        target.set_xyz(&self.get_xyz())
    }
    /// `Dump`: the `TDF_Attribute` dump, then `X: <x>\tY: <y>\tZ: <z>`.
    fn dump(&self, stream: &mut dyn io::Write) -> io::Result<()> {
        let xyz = self.get_xyz();
        dump_attribute(self, stream)?;
        write!(
            stream,
            "X: {}\tY: {}\tZ: {}",
            stream_real(xyz.x()),
            stream_real(xyz.y()),
            stream_real(xyz.z())
        )
    }
}
