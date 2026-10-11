//! Native-facing family selection stays explicit for Decal and shared-state regressions.
use martensite_vulkan::RenderError;
use rustycad_tkservice::aspect::AspectTypeOfDisplayText;
/// Validate actual display mode; `text_family(Some("shadow"))?` selects the native Shadow family.
pub fn text_family(name: Option<&str>) -> Result<AspectTypeOfDisplayText, RenderError> {
    match name.unwrap_or("decal") {
        "decal" => Ok(AspectTypeOfDisplayText::Dekale),
        "shadow" => Ok(AspectTypeOfDisplayText::Shadow),
        "subtitle" => Ok(AspectTypeOfDisplayText::Subtitle),
        actual => Err(RenderError::rejected(actual, "decal, shadow or subtitle")),
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_text_family_shape_is_explicit() {
        assert_eq!(text_family(None).unwrap(), AspectTypeOfDisplayText::Dekale);
        assert_eq!(
            text_family(Some("shadow")).unwrap(),
            AspectTypeOfDisplayText::Shadow
        );
        assert_eq!(
            text_family(Some("subtitle")).unwrap(),
            AspectTypeOfDisplayText::Subtitle
        );
        assert!(
            text_family(Some("wrong"))
                .unwrap_err()
                .to_string()
                .contains("wrong")
        );
    }
}
