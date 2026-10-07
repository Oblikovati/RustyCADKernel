// Rust port of ApplicationFramework/TKLCAF/TDF/TDF.hxx and .cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The `TDF` package class: GUID bounds and the GUID/ProgID registry.
use crate::guid_from_text;
use rustycad_tkernel::{standard::Guid, tcollection::ExtendedString};
use std::sync::{Mutex, PoisonError};

/// The file-static `guidprogidmap`, a double map GUID <-> ProgID.
static GUID_PROG_ID: Mutex<Vec<(Guid, ExtendedString)>> = Mutex::new(Vec::new());

/// `TDF`.
pub struct Tdf;
impl Tdf {
    /// `LowestID`: the all-zero GUID.
    pub fn lowest_id() -> Guid {
        guid_from_text("00000000-0000-0000-0000-000000000000")
    }
    /// `UppestID`: the all-`f` GUID.
    pub fn uppest_id() -> Guid {
        guid_from_text("ffffffff-ffff-ffff-ffff-ffffffffffff")
    }
    /// `AddLinkGUIDToProgID`: binds the pair, unbinding any previous use of either side.
    ///
    /// ```
    /// use rustycad_tklcaf::tdf::Tdf;
    /// use rustycad_tkernel::tcollection::ExtendedString;
    /// let id = Tdf::uppest_id();
    /// Tdf::add_link_guid_to_prog_id(&id, &ExtendedString::new("Prog.Doc"));
    /// let mut found = rustycad_tkernel::standard::Guid::new();
    /// assert!(Tdf::guid_from_prog_id(&ExtendedString::new("Prog.Doc"), &mut found) && found == id);
    /// ```
    pub fn add_link_guid_to_prog_id(id: &Guid, prog_id: &ExtendedString) {
        let mut map = GUID_PROG_ID.lock().unwrap_or_else(PoisonError::into_inner);
        map.retain(|(guid, prog)| guid != id && prog != prog_id);
        map.push((*id, prog_id.clone()));
    }
    /// `GUIDFromProgID`; false when the ProgID is not bound.
    pub fn guid_from_prog_id(prog_id: &ExtendedString, id: &mut Guid) -> bool {
        let map = GUID_PROG_ID.lock().unwrap_or_else(PoisonError::into_inner);
        let Some((guid, _)) = map.iter().find(|(_, prog)| prog == prog_id) else {
            return false;
        };
        *id = *guid;
        true
    }
    /// `ProgIDFromGUID`; false when the GUID is not bound.
    pub fn prog_id_from_guid(id: &Guid, prog_id: &mut ExtendedString) -> bool {
        let map = GUID_PROG_ID.lock().unwrap_or_else(PoisonError::into_inner);
        let Some((_, prog)) = map.iter().find(|(guid, _)| guid == id) else {
            return false;
        };
        *prog_id = prog.clone();
        true
    }
}
