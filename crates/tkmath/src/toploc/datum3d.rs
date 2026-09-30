// Rust translation of FoundationClasses/TKMath/TopLoc/TopLoc_Datum3D.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

use crate::gp::{Transform3, TransformForm};
use rustycad_tkernel::standard::{
    StandardTransient,
    dump::{self, DumpOutput, DumpStream},
};
use std::io::{self, Write};
/// `TopLoc_Datum3D`: an elementary coordinate system whose identity is its handle; `Datum3::new(transform)`.
#[derive(Default)]
pub struct Datum3 {
    base: StandardTransient,
    transform: Transform3,
}
rustycad_tkernel::impl_transient!(Datum3, "TopLoc_Datum3D", StandardTransient, base);
impl std::fmt::Debug for Datum3 {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("Datum3")
            .field("transform", &self.transform)
            .finish()
    }
}
impl Datum3 {
    /// Stores the transformation; like OCCT, a non-rigid one is not rejected; `Datum3::new(transform)`.
    pub fn new(transform: Transform3) -> Self {
        Self {
            base: StandardTransient::default(),
            transform,
        }
    }
    /// Borrows the transformation that maps this datum to the default one; `datum.transformation()`.
    pub const fn transformation(&self) -> &Transform3 {
        &self.transform
    }
    /// Alias of [`Datum3::transformation`], as `Trsf`; `datum.trsf()`.
    pub const fn trsf(&self) -> &Transform3 {
        &self.transform
    }
    /// Returns the transformation form; `datum.form()`.
    pub const fn form(&self) -> TransformForm {
        self.transform.form()
    }
    /// Writes the class name and the dumped transformation; `datum.dump_json(&mut stream, -1)?`.
    pub fn dump_json(&self, output: &mut impl DumpOutput, depth: i32) -> io::Result<()> {
        dump::dump_class_begin(output, "TopLoc_Datum3D")?;
        if depth != 0 {
            let mut field = DumpStream::default();
            self.transform.dump_json(&mut field)?;
            dump::dump_key_to_class(output, "Trsf", field.text())?;
        }
        Ok(())
    }
    /// Prints the address and three matrix rows with `setw(10)` cells; `datum.shallow_dump(&mut out)?`.
    pub fn shallow_dump(&self, output: &mut impl Write) -> io::Result<()> {
        writeln!(output, " TopLoc_Datum3D {:p}", std::ptr::from_ref(self))?;
        for row in 1..=3 {
            let cells: Vec<String> = (1..=4).map(|column| self.cell(row, column)).collect();
            writeln!(output, "  ( {})", cells.join(","))?;
        }
        writeln!(output)
    }
    fn cell(&self, row: i32, column: i32) -> String {
        // Rows and columns 1..3 x 1..4 are always inside gp_Trsf::Value's range.
        let value = self.transform.value(row, column).unwrap_or_default();
        crate::stream_format::padded(value, 10)
    }
}
