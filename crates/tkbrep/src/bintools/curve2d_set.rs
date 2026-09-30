// Rust translation of ModelingData/TKBRep/BinTools/BinTools_Curve2dSet.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BinTools_Curve2dSet`: the indexed `Curve2ds` table of the binary shape set.
use super::handle_table::HandleTable;
use super::ostream::BinOStream;
use super::table_io::{TableLabels, read_table, write_table};
use super::{curve2d_read, curve2d_write};
use rustycad_tkernel::message::ProgressRange;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkernel::storage::IoStream;
use rustycad_tkg2d::geom2d::Geom2dCurve;

static LABELS: TableLabels = TableLabels {
    tag: b"Curve2ds",
    write_scope: "Writing 2D curves",
    read_scope: "Reading curves 2d",
    not_a_table: "BinTools_Curve2dSet::Read:  Not a Curve2d table\n",
};

/// Stores a set of `Geom2d_Curve`s, each at the index it was first added with. Unlike
/// the 3D curve set, `Add` and `Index` do not test for a null handle (a null handle is
/// stored like any other key) and `Curve2d(0)` is out of range, as natively.
///
/// ```
/// use rustycad_tkbrep::bintools::BinCurve2dSet;
/// use rustycad_tkernel::standard::Handle;
/// let mut set = BinCurve2dSet::new();
/// assert_eq!(set.add(&Handle::default()), 1);
/// assert!(set.curve2d(0).is_err());
/// ```
pub struct BinCurve2dSet {
    table: HandleTable<dyn Geom2dCurve>,
}

impl Default for BinCurve2dSet {
    fn default() -> Self {
        Self::new()
    }
}

impl BinCurve2dSet {
    /// `BinTools_Curve2dSet()`: an empty set; `BinCurve2dSet::new()`.
    pub fn new() -> Self {
        Self {
            table: HandleTable::new("BinTools_Curve2dSet"),
        }
    }

    /// `Clear()`; `set.clear()`.
    pub fn clear(&mut self) {
        self.table.clear();
    }

    /// `Add(C)`: the index of `curve`, added when new (a null handle too);
    /// `set.add(&curve)`.
    pub fn add(&mut self, curve: &Handle<dyn Geom2dCurve>) -> i32 {
        self.table.add(curve)
    }

    /// `Curve2d(I)`: the curve of index `index`; Standard_OutOfRange outside
    /// `1..=Extent`; `set.curve2d(1)?`.
    pub fn curve2d(&self, index: i32) -> Result<Handle<dyn Geom2dCurve>, Failure> {
        self.table.get(index)
    }

    /// `Index(C)`: the index of `curve`, 0 when absent; `set.index(&curve)`.
    pub fn index(&self, curve: &Handle<dyn Geom2dCurve>) -> i32 {
        self.table.index(curve)
    }

    /// The number of stored curves (`myMap.Extent()`); `set.extent()`.
    pub fn extent(&self) -> i32 {
        self.table.extent()
    }

    /// `Write(OS, theRange)`: `Curve2ds n\n`, then each curve record; a stored null
    /// curve raises Standard_NullObject (a crash natively);
    /// `set.write(&mut stream, &ProgressRange::default())?`.
    pub fn write(
        &self,
        stream: &mut IoStream,
        progress: &ProgressRange<'_>,
    ) -> Result<(), Failure> {
        write_table(stream, progress, &LABELS, self.extent(), |index, out| {
            Self::write_curve2d(&self.table.get(index)?, out)
        })
    }

    /// `Read(IS, theRange)`: appends the curves of a `Curve2ds n` table; another tag
    /// raises Standard_Failure ("Not a Curve2d table");
    /// `set.read(&mut stream, &ProgressRange::default())?`.
    pub fn read(
        &mut self,
        stream: &mut IoStream,
        progress: &ProgressRange<'_>,
    ) -> Result<(), Failure> {
        let table = &mut self.table;
        read_table(stream, progress, &LABELS, |stream| {
            table.add(&Self::read_curve2d(stream)?);
            Ok(())
        })
    }

    /// `WriteCurve2d(C, OS)`; `BinCurve2dSet::write_curve2d(&curve, &mut out)?`.
    pub fn write_curve2d(
        curve: &Handle<dyn Geom2dCurve>,
        out: &mut BinOStream<'_>,
    ) -> Result<(), Failure> {
        curve2d_write::write_curve2d(curve, out)
    }

    /// `ReadCurve2d(IS, C)`; `BinCurve2dSet::read_curve2d(&mut stream)?`.
    pub fn read_curve2d(stream: &mut IoStream) -> Result<Handle<dyn Geom2dCurve>, Failure> {
        curve2d_read::read_curve2d(stream)
    }
}
