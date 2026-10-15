// Rust port of DataExchange/TKXCAF/XCAFPrs/XCAFPrs_Driver.hxx and .cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `XCAFPrs_Driver`: the presentation driver of the XDE shape labels.
use super::XCAFPrsAISObject;
use crate::xcaf_doc::XCAFDocShapeTool;
use rustycad_tkernel::standard::{Failure, Guid, Handle, StandardTransient};
use rustycad_tklcaf::guid_from_text;
use rustycad_tklcaf::tdf::TdfLabel;
use rustycad_tkv3d::ais::AisInteractiveObject;
use rustycad_tkvcaf::tprs_std::TPrsStdDriver;

/// `XCAFPrs_Driver`.
///
/// ```
/// use rustycad_tklcaf::tdf::TdfData;
/// use rustycad_tkvcaf::tprs_std::TPrsStdDriver;
/// use rustycad_tkxcaf::xcaf_prs::XCAFPrsDriver;
/// let data = TdfData::new();
/// let mut ais = Default::default();
/// assert!(!XCAFPrsDriver::new().update(&data.root(), &mut ais)?);
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
#[derive(Default)]
pub struct XCAFPrsDriver {
    transient: StandardTransient,
}
rustycad_tkernel::impl_transient!(XCAFPrsDriver, "XCAFPrs_Driver", dyn TPrsStdDriver, transient,
    interfaces: [dyn TPrsStdDriver]);

impl XCAFPrsDriver {
    /// `XCAFPrs_Driver()`.
    pub fn new() -> Self {
        Self::default()
    }
    /// `GetID`: the GUID the driver is registered with.
    pub fn get_id() -> Guid {
        const ID: Guid = guid_from_text("5b896afc-3adf-11d4-b9b7-0060b0ee281b");
        ID
    }
}

impl TPrsStdDriver for XCAFPrsDriver {
    /// `Update(L, ais)`: a new `XCAFPrs_AISObject` for a shape label.
    fn update(
        &self,
        label: &TdfLabel,
        ais: &mut Handle<dyn AisInteractiveObject>,
    ) -> Result<bool, Failure> {
        if !XCAFDocShapeTool::is_shape(label)? {
            return Ok(false);
        }
        *ais = XCAFPrsAISObject::new(label)?.cast();
        Ok(true)
    }
}
