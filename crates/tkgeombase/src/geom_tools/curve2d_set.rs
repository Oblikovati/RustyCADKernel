// Rust translation of ModelingData/TKGeomBase/GeomTools/GeomTools_Curve2dSet.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomTools_Curve2dSet`: the indexed `Curve2ds` table of the BRep ASCII format.
use super::console::{ConsoleOutput, stdout_console};
use super::geometry_table::{GeometryTable, TableLabels};
use super::{curve2d_print, curve2d_read};
use rustycad_tkernel::message::ProgressRange;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkernel::storage::IoStream;
use rustycad_tkg2d::geom2d::Geom2dCurve;
use std::sync::Arc;

static LABELS: TableLabels = TableLabels {
    dump_plural: b"Curve2ds",
    tag: b"Curve2ds",
    scope: "2D Curves",
    not_a_table: b"Not a Curve2d table",
};

/// Stores a set of `Geom2d_Curve`s, each at the index it was first added with.
/// Unlike [`super::CurveSet`], a null handle is stored like any curve.
///
/// ```
/// use rustycad_tkernel::standard::Handle;
/// use rustycad_tkgeombase::geom_tools::Curve2dSet;
/// let mut set = Curve2dSet::new();
/// assert_eq!(set.add(&Handle::default()), 1);
/// assert!(set.curve2d(1).is_null());
/// ```
pub struct Curve2dSet {
    table: GeometryTable<dyn Geom2dCurve>,
}

impl Default for Curve2dSet {
    fn default() -> Self {
        Self::new()
    }
}

impl Curve2dSet {
    /// `GeomTools_Curve2dSet()`: an empty set whose `Read` reports on standard
    /// output; `Curve2dSet::new()`.
    pub fn new() -> Self {
        Self::with_console(stdout_console())
    }

    /// An empty set whose `Read` writes its "Not a Curve2d table" line to `console`;
    /// `Curve2dSet::with_console(Arc::new(StdoutConsole))`.
    pub fn with_console(console: Arc<dyn ConsoleOutput>) -> Self {
        Self {
            table: GeometryTable::new(&LABELS, console),
        }
    }

    /// `Clear()`: removes every curve; `set.clear()`.
    pub fn clear(&mut self) {
        self.table.clear();
    }

    /// `Add(C)`: the index of `curve`, added when new; a null handle is stored too;
    /// `set.add(&curve)`.
    pub fn add(&mut self, curve: &Handle<dyn Geom2dCurve>) -> i32 {
        self.table.add(curve)
    }

    /// `Curve2d(I)`: the curve of index `index`, null outside the table; `set.curve2d(1)`.
    pub fn curve2d(&self, index: i32) -> Handle<dyn Geom2dCurve> {
        self.table.get(index)
    }

    /// `Index(C)`: the index of `curve`, 0 when absent; a stored null handle has an
    /// index; `set.index(&curve)`.
    pub fn index(&self, curve: &Handle<dyn Geom2dCurve>) -> i32 {
        self.table.index(curve)
    }

    /// `Dump(OS)`: every curve in the dump layout; Standard_NullObject for a null
    /// element, which the native code dereferences; `set.dump(&mut stream)?`.
    pub fn dump(&self, stream: &mut IoStream) -> Result<(), Failure> {
        self.table.dump(stream, curve2d_print::print_curve2d)
    }

    /// `Write(OS, theProgress)`: the `Curve2ds n` table in the compact layout at
    /// precision 17; `set.write(&mut stream, &ProgressRange::default())?`.
    pub fn write(
        &self,
        stream: &mut IoStream,
        progress: &ProgressRange<'_>,
    ) -> Result<(), Failure> {
        self.table
            .write(stream, progress, curve2d_print::print_curve2d)
    }

    /// `Read(IS, theProgress)`: appends the curves of a `Curve2ds n` table; another
    /// tag writes "Not a Curve2d table" to the console;
    /// `set.read(&mut stream, &ProgressRange::default())`.
    pub fn read(&mut self, stream: &mut IoStream, progress: &ProgressRange<'_>) {
        self.table
            .read(stream, progress, curve2d_read::read_curve2d);
    }

    /// `PrintCurve2d(C, OS, compact)`; `Curve2dSet::print_curve2d(&curve, &mut stream, true)?`.
    pub fn print_curve2d(
        curve: &Handle<dyn Geom2dCurve>,
        stream: &mut IoStream,
        compact: bool,
    ) -> Result<(), Failure> {
        curve2d_print::print_curve2d(curve, stream, compact)
    }

    /// `ReadCurve2d(IS)`: one compact record; null when it cannot be read or built;
    /// `Curve2dSet::read_curve2d(&mut stream)`.
    pub fn read_curve2d(stream: &mut IoStream) -> Handle<dyn Geom2dCurve> {
        curve2d_read::read_curve2d(stream)
    }
}
