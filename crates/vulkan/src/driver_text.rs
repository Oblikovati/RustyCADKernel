//! Text measurements reuse the ported FreeType adapter, independently of text rasterization.
use rustycad_tkernel::{standard::Handle, tcollection::AsciiString};
use rustycad_tkservice::{font::*, graphic3d::*};
pub(crate) fn text_size(view: &Handle<dyn Graphic3dCView>, text: &str, height: f32) -> [f32; 3] {
    let Some(view) = view.get() else {
        return [0.; 3];
    };
    let params = font_params(&view.rendering_params(), height);
    let font = FontFTFont::default();
    if !font.find_and_init(
        &AsciiString::from("Courier"),
        FontFontAspect::Regular,
        &params,
        FontStrictLevel::Any,
    ) {
        return [0.; 3];
    }
    [
        string_width(text, &|a, b| font.advance_x_of(a, b)),
        font.ascender(),
        font.descender(),
    ]
}
fn string_width(text: &str, advance: &impl Fn(u32, u32) -> f32) -> f32 {
    let mut chars = text.chars().take_while(|c| *c != '\0').peekable();
    let (mut width, mut maximum) = (0f32, 0f32);
    while let Some(character) = chars.next() {
        let next = chars.peek().copied().unwrap_or('\0') as u32;
        match character {
            '\r' | '\x07' | '\x0C' | '\x08' | '\x0B' => {}
            '\n' => {
                maximum = maximum.max(width);
                width = 0.;
            }
            '\t' => width += advance(' ' as u32, next) * 8.,
            character => width += advance(character as u32, next),
        }
    }
    maximum.max(width)
}

fn font_params(rendering: &Graphic3dRenderingParams, height: f32) -> FontFTFontParams {
    FontFTFontParams {
        point_size: if height < 2. { 16 } else { height as u32 },
        resolution: rendering.resolution,
        font_hinting: rendering.font_hinting,
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    struct FixedFontMetrics;
    impl FixedFontMetrics {
        fn advance(&self, current: u32, next: u32) -> f32 {
            if current == 'A' as u32 && next == 'V' as u32 {
                0.5
            } else {
                1.
            }
        }
    }
    #[test]
    fn multiline_tabs_controls_and_kerning_follow_native_width_rules() {
        let metrics = FixedFontMetrics;
        let advance = |a, b| metrics.advance(a, b);
        assert_eq!(string_width("AV", &advance), 1.5);
        assert_eq!(string_width("AV\nabc", &advance), 3.);
        assert_eq!(string_width("a\tb", &advance), 10.);
        assert_eq!(string_width("a\r\x07\x0C\x08\x0Bb\0ignored", &advance), 2.);
        assert_eq!(text_size(&Handle::default(), "abc", 16.), [0.; 3]);
    }
    #[test]
    fn font_parameters_use_view_resolution_and_native_minimum_height() {
        let mut rendering = Graphic3dRenderingParams::new().unwrap();
        rendering.resolution = 144;
        assert_eq!(font_params(&rendering, 1.).point_size, 16);
        assert_eq!(font_params(&rendering, 17.9).point_size, 17);
        assert_eq!(font_params(&rendering, 16.).resolution, 144);
        assert_eq!(
            font_params(&rendering, 16.).font_hinting,
            rendering.font_hinting
        );
    }
}
