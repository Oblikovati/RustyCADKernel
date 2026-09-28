//! `std::ostream` text for doubles in the pinned C locale, shared by the Dump/Write ports.
use rustycad_tkernel::tcollection::AsciiString;
/// Default `operator<<(double)`: `%g` with six significant digits; `general(0.1) == "0.1"`.
pub(crate) fn general(value: f64) -> String {
    String::from_utf8_lossy(AsciiString::from_real(value).as_bytes()).into_owned()
}
/// `std::setw(width) << value`: right-aligned in at least `width` columns; `padded(1.5, 10)`.
pub(crate) fn padded(value: f64, width: usize) -> String {
    format!("{:>width$}", general(value))
}
