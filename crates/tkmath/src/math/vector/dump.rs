use super::{VectorBase, VectorItem};
use rustycad_tkernel::ncollection::Array1;
use std::fmt;
impl<T: VectorItem> VectorBase<'_, T> {
    /// Writes the native `Dump` text, one declared index per line; `vector.dump(&mut text)?`.
    ///
    /// Both instantiations print the `math_Vector` label, as the shared template does.
    pub fn dump(&self, stream: &mut impl fmt::Write) -> fmt::Result {
        writeln!(stream, "math_Vector of Length = {}", self.length())?;
        for index in self.lower()..=self.upper() {
            writeln!(
                stream,
                "math_Vector({index}) = {}",
                self[index].stream_text()
            )?;
        }
        Ok(())
    }
    /// Views the elements as the underlying bounded array; `vector.array1().upper()`.
    pub fn array1(&self) -> Array1<'_, T> {
        Array1::from_shared(self.values(), self.lower())
    }
}
