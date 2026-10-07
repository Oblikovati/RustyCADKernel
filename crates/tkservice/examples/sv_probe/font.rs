//! Font_FontMgr, Font_FTFont and Font_TextFormatter, the twin of
//! tools/oracle/tkservice_probe_font.cxx.
use super::common::{Reader, end, flag, int, real, text, xy};
use super::print::show;
use super::tried;
use rustycad_tkernel::message::Message;
use rustycad_tkernel::ncollection::String8;
use rustycad_tkernel::tcollection::AsciiString;
use rustycad_tkservice::font::{
    FontFTFont, FontFTFontParams, FontFontAspect, FontFontMgr, FontGlyphOutline, FontHinting,
    FontRect, FontStrictLevel, FontTextFormatter, FontTextFormatterIterationFilter,
    FontTextFormatterIterator, FontUnicodeSubset,
};
use rustycad_tkservice::graphic3d::{
    Graphic3dHorizontalTextAlignment, Graphic3dVerticalTextAlignment,
};
use rustycad_tkservice::image::ImagePixMap;

const NAMES: [&str; 12] = [
    "sans-serif",
    "monospace",
    "serif",
    "DejaVu Sans",
    "Courier",
    "cjk",
    "korean",
    "no-such-font",
    "Times-Bold",
    "arial",
    "",
    "",
];

fn quiet() {
    Message::default_messenger().printers_mut().clear();
}

fn aspect(value: i32) -> FontFontAspect {
    use FontFontAspect as A;
    [A::Undefined, A::Regular, A::Bold, A::Italic, A::BoldItalic][(value + 1) as usize]
}
fn strict(value: i32) -> FontStrictLevel {
    [
        FontStrictLevel::Strict,
        FontStrictLevel::Aliases,
        FontStrictLevel::Any,
    ][value as usize]
}
fn align_x(value: i32) -> Graphic3dHorizontalTextAlignment {
    use Graphic3dHorizontalTextAlignment as H;
    [H::Left, H::Center, H::Right][value as usize]
}
fn align_y(value: i32) -> Graphic3dVerticalTextAlignment {
    use Graphic3dVerticalTextAlignment as V;
    [V::Bottom, V::Center, V::Top, V::Topfirstline][value as usize]
}

/// `N (C)*`: the UTF-32 code points as a UTF-8 string.
fn read_text(r: &mut Reader<'_>) -> (String8, Vec<u32>) {
    let chars: Vec<u32> = (0..r.integer()).map(|_| r.integer() as u32).collect();
    let mut units = chars.clone();
    units.push(0);
    (String8::from_unicode_units(&units, -1), chars)
}

fn rect(b: &FontRect) {
    [b.left, b.right, b.top, b.bottom]
        .map(f64::from)
        .into_iter()
        .for_each(real);
}

/// The size, format and FNV-1a hash of the image pixels, as `sv_font_image`.
fn image(img: &ImagePixMap) {
    let (w, h) = (img.width(), img.height());
    int(w as i32);
    int(h as i32);
    int(img.format() as i32);
    flag(img.is_top_down());
    let row_bytes = w * img.size_pixel_bytes();
    let data = img.data();
    let mut hash: u32 = 2_166_136_261;
    for y in 0..h {
        for &b in data.data.row(y).map_or(&[][..], |row| &row[..row_bytes]) {
            hash = (hash ^ u32::from(b)).wrapping_mul(16_777_619);
        }
    }
    int(hash as i32);
}

fn find_fonts(manager: &FontFontMgr) {
    for name in NAMES {
        let name = AsciiString::from(name);
        for level in 0..3 {
            for a in -1..4 {
                let mut found_aspect = aspect(a);
                let font = manager.find_font(&name, strict(level), &mut found_aspect, true);
                flag(font.is_some());
                if let Some(font) = font.get() {
                    text(font.font_name().as_bytes());
                }
                int(found_aspect as i32);
            }
        }
        for subset in [
            FontUnicodeSubset::Western,
            FontUnicodeSubset::Korean,
            FontUnicodeSubset::Cjk,
            FontUnicodeSubset::Arabic,
        ] {
            let font = manager.find_fallback_font(subset, FontFontAspect::Regular);
            flag(font.is_some());
            if let Some(font) = font.get() {
                text(font.font_name().as_bytes());
            }
        }
    }
}

/// `sv_fontmgr`: the font database, the aliases and the font lookups.
pub fn fontmgr_case(_values: &[f64]) {
    quiet();
    let manager = FontFontMgr::get_instance();
    let fonts = manager.get_available_fonts();
    int(fonts.len() as i32);
    for font in &fonts {
        text(font.to_string_desc().as_bytes());
        flag(font.is_single_stroke_font());
    }
    let aliases = manager.get_all_aliases();
    int(aliases.len() as i32);
    for alias in &aliases {
        text(alias.string().as_bytes());
        let names = manager.get_font_aliases(&alias.string());
        int(names.len() as i32);
        names.iter().for_each(|n| text(n.string().as_bytes()));
    }
    find_fonts(&manager);
    end();
}

fn metrics(f: &FontFTFont) {
    flag(f.is_valid());
    [f.ascender(), f.descender(), f.line_spacing()]
        .map(f64::from)
        .into_iter()
        .for_each(real);
    int(f.glyph_max_size_x(false) as i32);
    int(f.glyph_max_size_y(false) as i32);
    int(f.glyphs_number(false));
    int(f.point_size() as i32);
    flag(f.to_synthesize_italic());
    flag(f.is_single_stroke_font());
    real(f64::from(f.width_scaling()));
}

fn outline(o: &Option<FontGlyphOutline>) {
    flag(o.is_some());
    let Some(o) = o else { return };
    real(o.units_per_em());
    int(o.rule() as i32);
    flag(o.has_possible_overlaps());
    flag(o.is_single_stroke());
    int(o.segments().len() as i32);
    int(o.contours().len() as i32);
    for s in o.segments() {
        int(s.kind() as i32);
        [s.start(), s.control1(), s.control2(), s.end()]
            .into_iter()
            .for_each(xy);
    }
    for c in o.contours() {
        int(c.first_segment() as i32);
        int(c.nb_segments() as i32);
    }
}

fn character(f: &FontFTFont, c: u32, next: u32) {
    flag(f.has_symbol(c));
    flag(f.render_glyph(c));
    let mut bounds = FontRect::default();
    f.glyph_rect(&mut bounds);
    rect(&bounds);
    image(f.glyph_image());
    real(f64::from(f.advance_x_of(c, next)));
    real(f64::from(f.advance_y_of(c, next)));
    let glyph = f.resolve_glyph(c);
    flag(glyph.is_some());
    if let Some(glyph) = &glyph {
        int(glyph.face() as i32);
        int(glyph.index() as i32);
        flag(f.is_glyph_valid(glyph));
    }
    outline(&f.load_glyph_outline_char(c));
}

fn read_params(r: &mut Reader<'_>) -> FontFTFontParams {
    let (point_size, resolution, hinting) = (
        r.integer() as u32,
        r.integer() as u32,
        FontHinting(r.integer()),
    );
    let (to_synthesize_italic, is_single_stroke_font) = (r.flag(), r.flag());
    FontFTFontParams {
        point_size,
        resolution,
        font_hinting: hinting,
        to_synthesize_italic,
        is_single_stroke_font,
    }
}

/// The embedded fallback font, or null.
fn embedded_font(params: &FontFTFontParams) -> rustycad_tkernel::standard::Handle<FontFTFont> {
    let font = FontFTFont::default();
    let name = AsciiString::from("Embedded fallback");
    if font.init_from(&FontFontMgr::embed_fallback_font(), &name, params, 0) {
        rustycad_tkernel::standard::Handle::new(font)
    } else {
        rustycad_tkernel::standard::Handle::default()
    }
}

/// `sv_ftfont`: a font found by name (-1: the embedded fallback font), its metrics, characters and bounding boxes.
pub fn ftfont_case(values: &[f64]) {
    quiet();
    let mut r = Reader::new(values);
    let (name, a, level) = (r.integer(), r.integer(), r.integer());
    let params = read_params(&mut r);
    let (width, fallback) = (r.real() as f32, r.flag());
    let (text_value, chars) = read_text(&mut r);
    let font = if name < 0 {
        embedded_font(&params)
    } else {
        FontFTFont::find_and_create(
            &AsciiString::from(NAMES[name as usize]),
            aspect(a),
            &params,
            strict(level),
        )
    };
    flag(font.is_some());
    let Some(f) = font.get() else { return end() };
    flag(f.set_width_scaling(width));
    f.set_use_unicode_subset_fallback(fallback);
    metrics(f);
    for (i, &c) in chars.iter().enumerate() {
        character(f, c, chars.get(i + 1).copied().unwrap_or(0));
    }
    int(f.glyph_max_size_x(true) as i32);
    int(f.glyph_max_size_y(true) as i32);
    int(f.glyphs_number(true));
    for x in 0..3 {
        for y in 0..4 {
            show(f.bounding_box(&text_value, align_x(x), align_y(y)), |b| {
                rect(&b)
            });
        }
    }
    end();
}

fn formatter_queries(fmt: &FontTextFormatter, n: i32) {
    let lines = fmt.new_lines();
    for i in -1..=(lines.len() as i32 + 1) {
        real(f64::from(fmt.line_width(i)));
    }
    for i in -1..=n {
        let mut g = FontRect::default();
        let ok = fmt.glyph_bounding_box(i, &mut g).unwrap_or(false);
        flag(ok);
        if ok {
            rect(&g);
        }
    }
    for i in 0..n {
        show(fmt.is_lf_symbol(i), flag);
        show(fmt.line_position_index(i), int);
        show(fmt.line_index(i), int);
    }
    let mut it =
        FontTextFormatterIterator::new(fmt, FontTextFormatterIterationFilter::ExcludeInvisible);
    while it.more() {
        int(it.symbol() as i32);
        int(it.symbol_position());
        flag(it.has_next());
        int(it.symbol_next() as i32);
        it.next();
    }
}

fn formatter_results(fmt: &FontTextFormatter) -> i32 {
    let corners = fmt.corners();
    int(corners.len() as i32);
    corners.iter().for_each(|c| {
        real(f64::from(c.x));
        real(f64::from(c.y));
    });
    let lines = fmt.new_lines();
    int(lines.len() as i32);
    lines.iter().for_each(|l| real(f64::from(*l)));
    [
        fmt.result_width(),
        fmt.result_height(),
        fmt.maximum_symbol_width(),
        fmt.first_position(),
    ]
    .map(f64::from)
    .into_iter()
    .for_each(real);
    let mut b = FontRect::default();
    fmt.bnd_box(&mut b);
    rect(&b);
    real(f64::from(fmt.line_height(0)));
    real(f64::from(fmt.line_height(1)));
    corners.len() as i32
}

/// `sv_textfmt`: a formatted text, its corners, lines and glyph cells.
pub fn textfmt_case(values: &[f64]) {
    quiet();
    let mut r = Reader::new(values);
    let (name, x, y) = (r.integer(), r.integer(), r.integer());
    let (wrapping, word) = (r.real() as f32, r.flag());
    let params = FontFTFontParams {
        point_size: r.integer() as u32,
        ..Default::default()
    };
    let (text_value, _) = read_text(&mut r);
    let font = FontFTFont::find_and_create(
        &AsciiString::from(NAMES[name as usize]),
        FontFontAspect::Regular,
        &params,
        FontStrictLevel::Any,
    );
    flag(font.is_some());
    let Some(f) = font.get() else { return end() };
    let fmt = FontTextFormatter::default();
    fmt.setup_alignment(align_x(x), align_y(y));
    fmt.set_wrapping(wrapping);
    fmt.set_word_wrapping(word);
    tried(fmt.append(&text_value, f));
    tried(fmt.format());
    let n = formatter_results(&fmt);
    formatter_queries(&fmt, n);
    end();
}
