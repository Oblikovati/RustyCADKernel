//! Validate capture requests before constructing GPU/native resources.
pub fn capture_arguments(arguments: Vec<String>) -> Result<[String; 2], String> {
    if arguments.len() != 3 {
        return Err(format!(
            "received arguments={arguments:?}; expected executable, mode and capture prefix"
        ));
    }
    let mut values = arguments.into_iter().skip(1);
    Ok([values.next().unwrap(), values.next().unwrap()])
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn capture_shape_preserves_mode_and_prefix_and_reports_offending_values() {
        assert_eq!(
            capture_arguments(vec![
                "capture".into(),
                "lit-basic".into(),
                "/tmp/fixed".into()
            ])
            .unwrap(),
            ["lit-basic", "/tmp/fixed"]
        );
        let message = capture_arguments(vec!["capture".into(), "bad-mode".into()]).unwrap_err();
        assert!(message.contains("bad-mode"));
        assert!(message.contains("expected executable, mode and capture prefix"));
    }
}
