// Rust port of ApplicationFramework/TKLCAF/TDataStd/TDataStd.hxx and .cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The `TDataStd` package class.
use super::{TDataStdInteger, TDataStdName, TDataStdReal, TDataStdRealEnum};
use crate::tdf::{TdfReference, TdfTagSource};
use rustycad_tkernel::standard::Guid;
use std::io;

/// `TDataStd`.
pub struct TDataStd;
impl TDataStd {
    /// `IDList`: appends the IDs of the standard attributes, in native order.
    pub fn id_list(ids: &mut Vec<Guid>) {
        ids.push(TdfTagSource::get_id());
        ids.push(TdfReference::get_id());
        ids.push(TDataStdInteger::get_id());
        ids.push(TDataStdName::get_id());
        ids.push(TDataStdReal::get_id());
        ids.extend(super::array_ids());
    }
    /// `Print(DIM, S)`: `SCALAR`, `LENGTH` or `ANGULAR`.
    ///
    /// ```
    /// use rustycad_tklcaf::tdata_std::{TDataStd, TDataStdRealEnum};
    /// let mut text = Vec::new();
    /// TDataStd::print(TDataStdRealEnum::Length, &mut text).unwrap();
    /// assert_eq!(text, b"LENGTH");
    /// ```
    pub fn print(dimension: TDataStdRealEnum, stream: &mut dyn io::Write) -> io::Result<()> {
        let text = match dimension {
            TDataStdRealEnum::Scalar => "SCALAR",
            TDataStdRealEnum::Length => "LENGTH",
            TDataStdRealEnum::Angular => "ANGULAR",
        };
        write!(stream, "{text}")
    }
}
