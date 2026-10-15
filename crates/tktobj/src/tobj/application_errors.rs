// Rust port of ApplicationFramework/TKTObj/TObj/TObj_Application.cxx (the two SetError)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The message keys of `TObj_Application::SetError` for the store and reader statuses.
use rustycad_tkcdf::pcdm::{PcdmReaderStatus, PcdmStoreStatus};

/// The key of `SetError(PCDM_StoreStatus, ...)`.
pub(crate) fn store_error_key(status: PcdmStoreStatus) -> &'static str {
    match status {
        PcdmStoreStatus::DriverFailure => "TObj_Appl_SDriverFailure",
        PcdmStoreStatus::WriteFailure => "TObj_Appl_SWriteFailure",
        PcdmStoreStatus::Failure => "TObj_Appl_SFailure",
        PcdmStoreStatus::DocIsNull => "TObj_Appl_SDocIsNull",
        PcdmStoreStatus::NoObj => "TObj_Appl_SNoObj",
        PcdmStoreStatus::InfoSectionError => "TObj_Appl_SInfoSectionError",
        _ => "TObj_Appl_SUnknownFailure",
    }
}

/// The key of `SetError(PCDM_ReaderStatus, ...)`.
pub(crate) fn reader_error_key(status: PcdmReaderStatus) -> &'static str {
    match status {
        PcdmReaderStatus::UnknownDocument => "TObj_Appl_RUnknownDocument",
        PcdmReaderStatus::AlreadyRetrieved => "TObj_Appl_RAlreadyRetrieved",
        PcdmReaderStatus::AlreadyRetrievedAndModified => "TObj_Appl_RAlreadyRetrievedAndModified",
        PcdmReaderStatus::NoDriver | PcdmReaderStatus::UnknownFileDriver => "TObj_Appl_RNoDriver",
        PcdmReaderStatus::OpenError => "TObj_Appl_ROpenError",
        PcdmReaderStatus::NoVersion => "TObj_Appl_RNoVersion",
        PcdmReaderStatus::NoModel => "TObj_Appl_RNoModel",
        PcdmReaderStatus::NoDocument => "TObj_Appl_RNoDocument",
        PcdmReaderStatus::FormatFailure => "TObj_Appl_RFormatFailure",
        PcdmReaderStatus::TypeNotFoundInSchema => "TObj_Appl_RTypeNotFound",
        PcdmReaderStatus::UnrecognizedFileFormat => "TObj_Appl_RBadFileFormat",
        PcdmReaderStatus::MakeFailure => "TObj_Appl_RMakeFailure",
        PcdmReaderStatus::PermissionDenied => "TObj_Appl_RPermissionDenied",
        PcdmReaderStatus::DriverFailure => "TObj_Appl_RDriverFailure",
        PcdmReaderStatus::ReaderException => "TObj_Appl_RException",
        _ => "TObj_Appl_RUnknownFail",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_status_has_the_native_key() {
        assert_eq!(
            store_error_key(PcdmStoreStatus::UserBreak),
            "TObj_Appl_SUnknownFailure"
        );
        assert_eq!(store_error_key(PcdmStoreStatus::NoObj), "TObj_Appl_SNoObj");
        assert_eq!(
            reader_error_key(PcdmReaderStatus::UnknownFileDriver),
            "TObj_Appl_RNoDriver"
        );
        assert_eq!(
            reader_error_key(PcdmReaderStatus::NoSchema),
            "TObj_Appl_RUnknownFail"
        );
        assert_eq!(
            reader_error_key(PcdmReaderStatus::ReaderException),
            "TObj_Appl_RException"
        );
    }
}
