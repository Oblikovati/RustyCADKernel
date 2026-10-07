// Rust port of ApplicationFramework/TKCDF/PCDM/PCDM_ReaderStatus.hxx, PCDM_StoreStatus.hxx,
// PCDM_TypeOfFileDriver.hxx and PCDM_DriverError.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The PCDM status enumerations and the `PCDM_DriverError` exception.
use rustycad_tkernel::standard::Failure;

/// `PCDM_ReaderStatus`: the outcome of a retrieval, in native order (the order matters:
/// the framework compares statuses with `>`); `PcdmReaderStatus::Ok as i32 == 0`.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, PartialOrd, Ord)]
pub enum PcdmReaderStatus {
    /// `PCDM_RS_OK`: success.
    #[default]
    Ok,
    /// `PCDM_RS_NoDriver`: no driver for the file format.
    NoDriver,
    /// `PCDM_RS_UnknownFileDriver`: the file is bad.
    UnknownFileDriver,
    /// `PCDM_RS_OpenError`: the file cannot be opened.
    OpenError,
    /// `PCDM_RS_NoVersion`: unknown document version.
    NoVersion,
    /// `PCDM_RS_NoSchema`: not used.
    NoSchema,
    /// `PCDM_RS_NoDocument`: the document is empty.
    NoDocument,
    /// `PCDM_RS_ExtensionFailure`: not used.
    ExtensionFailure,
    /// `PCDM_RS_WrongStreamMode`: the open mode is mistaken.
    WrongStreamMode,
    /// `PCDM_RS_FormatFailure`: the document data structure is wrong.
    FormatFailure,
    /// `PCDM_RS_TypeFailure`: a data type is unknown.
    TypeFailure,
    /// `PCDM_RS_TypeNotFoundInSchema`: a data type is not in the schema.
    TypeNotFoundInSchema,
    /// `PCDM_RS_UnrecognizedFileFormat`: the file format is not recognized.
    UnrecognizedFileFormat,
    /// `PCDM_RS_MakeFailure`: the conversion of data failed.
    MakeFailure,
    /// `PCDM_RS_PermissionDenied`: no permission to open the file.
    PermissionDenied,
    /// `PCDM_RS_DriverFailure`: a general reading error.
    DriverFailure,
    /// `PCDM_RS_AlreadyRetrievedAndModified`.
    AlreadyRetrievedAndModified,
    /// `PCDM_RS_AlreadyRetrieved`.
    AlreadyRetrieved,
    /// `PCDM_RS_UnknownDocument`: the file does not exist.
    UnknownDocument,
    /// `PCDM_RS_WrongResource`: wrong resource file.
    WrongResource,
    /// `PCDM_RS_ReaderException`: wrong data structure.
    ReaderException,
    /// `PCDM_RS_NoModel`: not used.
    NoModel,
    /// `PCDM_RS_UserBreak`: the user interrupted reading.
    UserBreak,
}

/// `PCDM_StoreStatus`: the outcome of a storage, in native order.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum PcdmStoreStatus {
    /// `PCDM_SS_OK`: saved.
    #[default]
    Ok,
    /// `PCDM_SS_DriverFailure`: no storage driver.
    DriverFailure,
    /// `PCDM_SS_WriteFailure`: writing the file failed.
    WriteFailure,
    /// `PCDM_SS_Failure`: an unexpected error.
    Failure,
    /// `PCDM_SS_Doc_IsNull`: a null document.
    DocIsNull,
    /// `PCDM_SS_No_Obj`: nothing to save.
    NoObj,
    /// `PCDM_SS_Info_Section_Error`: writing the information section failed.
    InfoSectionError,
    /// `PCDM_SS_UserBreak`: the user interrupted the storage.
    UserBreak,
    /// `PCDM_SS_UnrecognizedFormat`: no storage driver for the format.
    UnrecognizedFormat,
}

/// `PCDM_TypeOfFileDriver`: the native container kinds, in native order. The port reads no
/// native container (ocaf-design.md section 1), so it only answers `Unknown`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PcdmTypeOfFileDriver {
    /// `PCDM_TOFD_File`.
    File,
    /// `PCDM_TOFD_CmpFile`.
    CmpFile,
    /// `PCDM_TOFD_XmlFile`.
    XmlFile,
    /// `PCDM_TOFD_Unknown`.
    Unknown,
}

/// `PCDM_DriverError` (a `Standard_Failure`): a storage or retrieval driver failed;
/// `pcdm_driver_error("error during Make")`.
pub fn pcdm_driver_error(message: &str) -> Failure {
    Failure::with_type("PCDM_DriverError", message, "")
}
