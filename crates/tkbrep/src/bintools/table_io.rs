// Rust translation of the Write and Read members BinTools_CurveSet.cxx,
// BinTools_Curve2dSet.cxx and BinTools_SurfaceSet.cxx repeat, at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The table layout the three geometry sets share: a text header `<Tag> <count>\n`,
//! then the binary records, each set reading back the header it writes.
use super::ostream::BinOStream;
use super::scalars::stream_get;
use rustycad_tkernel::message::{ProgressRange, ProgressScope};
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::storage::IoStream;

/// `char buffer[255]` of the readers, which extract the tag with `IS >> buffer`.
const TAG_BUFFER: usize = 255;

/// The texts of one set class.
pub(crate) struct TableLabels {
    /// The tag before the count (`Curves`).
    pub(crate) tag: &'static [u8],
    /// The Message_ProgressScope name of `Write`.
    pub(crate) write_scope: &'static str,
    /// The Message_ProgressScope name of `Read`.
    pub(crate) read_scope: &'static str,
    /// The Standard_Failure message of a missing tag.
    pub(crate) not_a_table: &'static str,
}

/// `Write(OS, theRange)`: the progress scope over `count` records, the header, then
/// each record through one `BinTools_OStream`, stopping when the indicator breaks.
pub(crate) fn write_table(
    stream: &mut IoStream,
    progress: &ProgressRange<'_>,
    labels: &TableLabels,
    count: i32,
    mut write_record: impl FnMut(i32, &mut BinOStream<'_>) -> Result<(), Failure>,
) -> Result<(), Failure> {
    let scope = ProgressScope::with_literal(progress, labels.write_scope, f64::from(count), false);
    stream.insert_bytes(labels.tag).insert_bytes(b" ");
    stream.insert_integer(i64::from(count)).insert_bytes(b"\n");
    let mut out = BinOStream::new(stream);
    let mut index = 1;
    while index <= count && scope.more() {
        write_record(index, &mut out)?;
        index += 1;
        drop(scope.next(1.0));
    }
    Ok(())
}

/// `Read(IS, theRange)`: checks the tag (Standard_Failure otherwise), reads the count
/// and the line feed after it, then reads and stores the announced records, stopping
/// when the indicator breaks. The set is not cleared first, although the native
/// documentation says so.
pub(crate) fn read_table(
    stream: &mut IoStream,
    progress: &ProgressRange<'_>,
    labels: &TableLabels,
    mut read_record: impl FnMut(&mut IoStream) -> Result<(), Failure>,
) -> Result<(), Failure> {
    if stream.extract_word(TAG_BUFFER) != labels.tag || stream.fail() {
        return Err(Failure::new(labels.not_a_table));
    }
    // Natively the count is uninitialised when the extraction cannot start (the tag
    // ended the stream); the port reads 0.
    let mut count = 0;
    stream.extract_int(&mut count);
    let scope = ProgressScope::with_literal(progress, labels.read_scope, f64::from(count), false);
    stream_get(stream);
    let mut index = 1;
    while index <= count && scope.more() {
        read_record(stream)?;
        index += 1;
        drop(scope.next(1.0));
    }
    Ok(())
}
