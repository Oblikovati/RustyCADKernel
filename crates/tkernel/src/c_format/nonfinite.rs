//! Host C spellings matter for indeterminate NaNs (Windows UCRT uses nan(ind)).
#[cfg(windows)]
#[allow(unsafe_code)]
pub(crate) fn nonfinite_text(value: f64) -> String {
    let mut buffer = [0_i8; 64];
    // Only nonfinite values enter this fixed %g conversion; its output fits 64 bytes.
    // SAFETY: bounded destination and the double argument match the C format.
    let count = unsafe { libc::snprintf(buffer.as_mut_ptr(), buffer.len(), c"%g".as_ptr(), value) };
    assert!(
        (1..64).contains(&count),
        "nonfinite formatter returned {count}; expected 1..64 bytes"
    );
    let bytes: Vec<u8> = buffer[..count as usize]
        .iter()
        .map(|byte| *byte as u8)
        .collect();
    String::from_utf8(bytes).expect("C nonfinite spellings are ASCII")
}
#[cfg(not(windows))]
pub(crate) fn nonfinite_text(value: f64) -> String {
    let word = if value.is_nan() { "nan" } else { "inf" };
    format!("{}{word}", if value.is_sign_negative() { "-" } else { "" })
}

#[cfg(test)]
mod tests {
    use super::nonfinite_text;
    #[test]
    fn signed_special_values_and_platform_indeterminacy_are_preserved() {
        assert_eq!(nonfinite_text(f64::INFINITY), "inf");
        assert_eq!(nonfinite_text(-f64::INFINITY), "-inf");
        assert_eq!(nonfinite_text(f64::NAN), "nan");
        let zero = std::hint::black_box(0.0_f64);
        let expected = if cfg!(windows) {
            "-nan(ind)"
        } else if cfg!(target_arch = "aarch64") {
            "nan"
        } else {
            "-nan"
        };
        assert_eq!(nonfinite_text(zero / zero), expected);
    }
}
