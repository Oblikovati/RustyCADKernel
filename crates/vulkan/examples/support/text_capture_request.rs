//! Retained-text probes share strict argument shape and explicit fixture mode selection.
use martensite_vulkan::RenderError;
/// Select a known fixture and output prefix; `parse_text_capture(&args, &["subtitle-basic"])?`.
pub fn parse_text_capture<'a>(
    args: &'a [String],
    modes: &[&str],
) -> Result<(&'a str, &'a str), RenderError> {
    if args.len() != 2 {
        return Err(RenderError::rejected(
            format!("arguments {args:?}"),
            "mode prefix",
        ));
    }
    if !modes.contains(&args[0].as_str()) {
        return Err(RenderError::rejected(
            format!("mode {:?}", args[0]),
            &format!("one of {modes:?}"),
        ));
    }
    Ok((&args[0], &args[1]))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn known_mode_and_argument_shape_are_explicit() {
        let valid = vec!["subtitle-basic".to_owned(), "capture".to_owned()];
        assert_eq!(
            parse_text_capture(&valid, &["subtitle-basic"]).unwrap(),
            ("subtitle-basic", "capture")
        );
        assert!(parse_text_capture(&[], &["subtitle-basic"]).is_err());
        let error = parse_text_capture(&valid, &["unknown"])
            .unwrap_err()
            .to_string();
        assert!(error.contains("subtitle-basic") && error.contains("unknown"));
    }
}
