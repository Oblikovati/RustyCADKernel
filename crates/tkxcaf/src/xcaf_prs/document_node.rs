// Rust port of DataExchange/TKXCAF/XCAFPrs/XCAFPrs_DocumentNode.hxx and
// XCAFPrs_DocumentIdIterator.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The nodes of the document explorer (a label with its reference, style, locations and
//! path identifier) and the iterator over the entries of a path identifier.
use super::XCAFPrsStyle;
use rustycad_tkernel::ncollection::HashKey;
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::tcollection::AsciiString;
use rustycad_tklcaf::tdf::{TdfChildIterator, TdfLabel};
use rustycad_tkmath::toploc::Location;

/// `XCAFPrs_DocumentNode`: equal (and hashed) by identifier.
#[derive(Clone, Default)]
pub struct XCAFPrsDocumentNode {
    /// `Id`: the path identifier (`0:1:1:1.` entries joined by `/`).
    pub id: AsciiString,
    /// `Label`: the label in the document.
    pub label: TdfLabel,
    /// `RefLabel`: the referred label (the label itself when not a reference).
    pub ref_label: TdfLabel,
    /// `Style`: the merged style.
    pub style: XCAFPrsStyle,
    /// `Location`: the global location.
    pub location: Location,
    /// `LocalTrsf`: the location relative to the parent.
    pub local_trsf: Location,
    /// `ChildIter`: the iterator over the children of the referred assembly.
    pub child_iter: TdfChildIterator,
    /// `IsAssembly`: the node is an assembly.
    pub is_assembly: bool,
}

impl PartialEq for XCAFPrsDocumentNode {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl XCAFPrsDocumentNode {
    /// `std::hash<XCAFPrs_DocumentNode>`: the hash of the identifier.
    pub fn hash_code(&self) -> usize {
        self.id.occt_hash()
    }
}

/// `XCAFPrs_DocumentIdIterator`: the entries of a path identifier, without their trailing
/// `.`.
///
/// ```
/// use rustycad_tkxcaf::xcaf_prs::XCAFPrsDocumentIdIterator;
/// let path = "0:1:1:1./0:1:1:2.".into();
/// let mut iterator = XCAFPrsDocumentIdIterator::new(&path)?;
/// assert_eq!(iterator.value().as_bytes(), b"0:1:1:1");
/// iterator.next()?;
/// assert_eq!(iterator.value().as_bytes(), b"0:1:1:2");
/// iterator.next()?;
/// assert!(!iterator.more());
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
pub struct XCAFPrsDocumentIdIterator<'a> {
    path: &'a AsciiString,
    sub_id: AsciiString,
    position: i32,
}

impl<'a> XCAFPrsDocumentIdIterator<'a> {
    /// `XCAFPrs_DocumentIdIterator(thePath)`: positioned on the first entry.
    pub fn new(path: &'a AsciiString) -> Result<Self, Failure> {
        let mut iterator = Self {
            path,
            sub_id: AsciiString::default(),
            position: 0,
        };
        iterator.next()?;
        Ok(iterator)
    }
    /// `More`: a current entry.
    pub fn more(&self) -> bool {
        !self.sub_id.is_empty()
    }
    /// `Value`.
    pub fn value(&self) -> &AsciiString {
        &self.sub_id
    }
    /// `Next`: the entry up to the next `/` (a too short one stops the iteration in place,
    /// as natively), else the rest of the path.
    #[allow(clippy::should_implement_trait)] // OCCT's More/Next protocol, fallible.
    pub fn next(&mut self) -> Result<(), Failure> {
        let length = self.path.length();
        for index in self.position + 1..=length {
            if self.path.value(index)? == b'/' {
                if index - self.position - 2 < 1 {
                    return Ok(()); // assert - should never happen for valid IDs!
                }
                self.sub_id = self.path.sub_string(self.position + 1, index - 2)?;
                self.position = index;
                return Ok(());
            }
        }
        if self.position < length {
            self.sub_id = self.path.sub_string(self.position + 1, length - 1)?;
        } else {
            self.sub_id = AsciiString::default();
        }
        self.position = length;
        Ok(())
    }
}
