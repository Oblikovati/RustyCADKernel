//! `FSD_FileHeader`: the section offsets written after the binary magic number.

/// Byte offsets of every section of a binary container, plus the byte-order probe.
///
/// ```
/// use rustycad_tkernel::fsd::FileHeader;
/// let header = FileHeader::unset();
/// assert_eq!(header.fields(), [-1; 13]);
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FileHeader {
    /// `testindian`: the bytes 1, 2, 3, 4 read as a native `int`.
    pub test_indian: i32,
    /// `binfo`.
    pub begin_info: i32,
    /// `einfo`.
    pub end_info: i32,
    /// `bcomment`.
    pub begin_comment: i32,
    /// `ecomment`.
    pub end_comment: i32,
    /// `btype`.
    pub begin_type: i32,
    /// `etype`.
    pub end_type: i32,
    /// `broot`.
    pub begin_root: i32,
    /// `eroot`.
    pub end_root: i32,
    /// `bref`.
    pub begin_ref: i32,
    /// `eref`.
    pub end_ref: i32,
    /// `bdata`.
    pub begin_data: i32,
    /// `edata`.
    pub end_data: i32,
}

impl FileHeader {
    /// Every field at -1, as `FSD_BinaryFile()` initialises its header; `FileHeader::unset()`.
    pub fn unset() -> Self {
        Self::from_fields([-1; 13])
    }

    /// The fields in file order; `header.fields()[1] == header.begin_info`.
    pub fn fields(&self) -> [i32; 13] {
        [
            self.test_indian,
            self.begin_info,
            self.end_info,
            self.begin_comment,
            self.end_comment,
            self.begin_type,
            self.end_type,
            self.begin_root,
            self.end_root,
            self.begin_ref,
            self.end_ref,
            self.begin_data,
            self.end_data,
        ]
    }

    /// Builds a header from fields in file order; `FileHeader::from_fields([0; 13])`.
    pub fn from_fields(fields: [i32; 13]) -> Self {
        let [
            test_indian,
            begin_info,
            end_info,
            begin_comment,
            end_comment,
            begin_type,
            end_type,
            begin_root,
            end_root,
            begin_ref,
            end_ref,
            begin_data,
            end_data,
        ] = fields;
        Self {
            test_indian,
            begin_info,
            end_info,
            begin_comment,
            end_comment,
            begin_type,
            end_type,
            begin_root,
            end_root,
            begin_ref,
            end_ref,
            begin_data,
            end_data,
        }
    }

    /// Mutable references to the fields in file order, for field-by-field reads.
    pub(crate) fn fields_mut(&mut self) -> [&mut i32; 13] {
        [
            &mut self.test_indian,
            &mut self.begin_info,
            &mut self.end_info,
            &mut self.begin_comment,
            &mut self.end_comment,
            &mut self.begin_type,
            &mut self.end_type,
            &mut self.begin_root,
            &mut self.end_root,
            &mut self.begin_ref,
            &mut self.end_ref,
            &mut self.begin_data,
            &mut self.end_data,
        ]
    }
}
