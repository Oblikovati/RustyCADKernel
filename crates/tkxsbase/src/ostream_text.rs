//! The text that a default `std::ostream` writes for the values the native packages print.

use rustycad_tkernel::storage::format_general;

/// `std::ostream << double` at the default precision 6 (`%g` layout);
/// `ostream_real(0.5) == "0.5"`, `ostream_real(1e-7) == "1e-07"`.
pub(crate) fn ostream_real(value: f64) -> String {
    format_general(value, 6)
}

#[cfg(test)]
mod tests {
    use super::ostream_real;

    #[test]
    fn default_precision_layout() {
        assert_eq!(ostream_real(0.5), "0.5");
        assert_eq!(ostream_real(1e-7), "1e-07");
        assert_eq!(ostream_real(123456789.0), "1.23457e+08");
        assert_eq!(ostream_real(0.0), "0");
    }
}
