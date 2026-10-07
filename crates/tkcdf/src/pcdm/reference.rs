// Rust port of ApplicationFramework/TKCDF/PCDM/PCDM_Reference.hxx and .cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

use rustycad_tkernel::tcollection::ExtendedString;

/// `PCDM_Reference`: a stored reference (identifier, referenced file, its version);
/// `PcdmReference::new(1, &ExtendedString::new("a.std"), 3)`.
///
/// ```
/// use rustycad_tkcdf::pcdm::PcdmReference;
/// use rustycad_tkernel::tcollection::ExtendedString;
/// let reference = PcdmReference::new(4, &ExtendedString::new("f"), 2);
/// assert_eq!((reference.reference_identifier(), reference.document_version()), (4, 2));
/// assert_eq!(PcdmReference::default().reference_identifier(), 0);
/// ```
#[derive(Clone, Debug, Default)]
pub struct PcdmReference {
    reference_identifier: i32,
    file_name: ExtendedString,
    document_version: i32,
}
impl PcdmReference {
    /// `PCDM_Reference(id, fileName, version)`.
    pub fn new(
        reference_identifier: i32,
        file_name: &ExtendedString,
        document_version: i32,
    ) -> Self {
        Self {
            reference_identifier,
            file_name: file_name.clone(),
            document_version,
        }
    }
    /// `ReferenceIdentifier`; `reference.reference_identifier()`.
    pub fn reference_identifier(&self) -> i32 {
        self.reference_identifier
    }
    /// `FileName`; `reference.file_name()`.
    pub fn file_name(&self) -> ExtendedString {
        self.file_name.clone()
    }
    /// `DocumentVersion`; `reference.document_version()`.
    pub fn document_version(&self) -> i32 {
        self.document_version
    }
}
