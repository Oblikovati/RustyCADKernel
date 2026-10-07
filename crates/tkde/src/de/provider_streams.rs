// Rust port of DE_Provider::ReadStreamNode, WriteStreamNode, ReadStreamList and
// WriteStreamList of DataExchange/TKDE/DE/DE_Provider.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The stream lists of the stream transfers: the first stream is the main one, the others
//! are referenced by relative path.
use rustycad_tkernel::{
    ncollection::List,
    osd::{IStream, OStream},
    tcollection::AsciiString,
};

/// `DE_Provider::ReadStreamNode`: an input stream and its relative path.
///
/// ```
/// use rustycad_tkde::de::{DeProviderReadStreamList, DeProviderReadStreamNode};
/// # use rustycad_tkernel::osd::FileStream;
/// let mut stream = FileStream::input();
/// let mut list = DeProviderReadStreamList::default();
/// list.append(DeProviderReadStreamNode::new(&"part.stp".into(), &mut stream));
/// assert_eq!(list.first().map(|node| node.path.as_bytes()), Some(&b"part.stp"[..]));
/// ```
pub struct DeProviderReadStreamNode<'a> {
    /// `Path`: the relative path of the input file.
    pub path: AsciiString,
    /// `Stream`: the input stream, borrowed for the transfer.
    pub stream: &'a mut dyn IStream,
}
impl<'a> DeProviderReadStreamNode<'a> {
    /// `ReadStreamNode(thePath, theStream)`.
    pub fn new(path: &AsciiString, stream: &'a mut dyn IStream) -> Self {
        Self {
            path: path.clone(),
            stream,
        }
    }
}

/// `DE_Provider::WriteStreamNode`: an output stream and its relative path.
pub struct DeProviderWriteStreamNode<'a> {
    /// `Path`: the relative path of the output file.
    pub path: AsciiString,
    /// `Stream`: the output stream, borrowed for the transfer.
    pub stream: &'a mut dyn OStream,
}
impl<'a> DeProviderWriteStreamNode<'a> {
    /// `WriteStreamNode(thePath, theStream)`;
    /// `DeProviderWriteStreamNode::new(&"part.stp".into(), &mut stream)`.
    pub fn new(path: &AsciiString, stream: &'a mut dyn OStream) -> Self {
        Self {
            path: path.clone(),
            stream,
        }
    }
}

/// `DE_Provider::ReadStreamList`: the main input stream first.
pub type DeProviderReadStreamList<'a> = List<DeProviderReadStreamNode<'a>>;

/// `DE_Provider::WriteStreamList`: the main output stream first.
pub type DeProviderWriteStreamList<'a> = List<DeProviderWriteStreamNode<'a>>;
