//! `Font_SystemFont`: a font family with one file (and face id) per aspect.
use super::{FONT_FONT_ASPECT_NB, FontFontAspect, FontFontMgr};
use rustycad_tkernel::ncollection::HashKey;
use rustycad_tkernel::standard::exceptions::program_error;
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkernel::tcollection::AsciiString;

/// The aspects in index order (`Font_FontAspect` 0..`Font_FontAspect_NB`).
pub(crate) const FONT_ASPECTS: [FontFontAspect; FONT_FONT_ASPECT_NB as usize] = [
    FontFontAspect::Regular,
    FontFontAspect::Bold,
    FontFontAspect::Italic,
    FontFontAspect::BoldItalic,
];

/// The per-aspect files of a font.
#[derive(Clone, Default)]
struct FontFiles {
    paths: [AsciiString; FONT_FONT_ASPECT_NB as usize],
    face_ids: [i32; FONT_FONT_ASPECT_NB as usize],
    is_single_line: bool,
}

/// A system font family; `FontSystemFont::new(&"Arial".into())?`.
pub struct FontSystemFont {
    transient: StandardTransient,
    font_key: AsciiString,
    font_name: AsciiString,
    files: SharedState<FontFiles>,
}
rustycad_tkernel::impl_transient!(
    FontSystemFont,
    "Font_SystemFont",
    StandardTransient,
    transient
);

/// The array index of an aspect, `UNDEFINED` reading as `Regular`.
fn slot(aspect: FontFontAspect) -> usize {
    match aspect {
        FontFontAspect::Undefined => FontFontAspect::Regular as usize,
        other => other as usize,
    }
}

impl FontSystemFont {
    /// A font named `name` (its key is the lower-cased name); an empty name raises
    /// `Standard_ProgramError`. Example: `FontSystemFont::new(&"DejaVu Sans".into())?`.
    pub fn new(name: &AsciiString) -> Result<Self, Failure> {
        if name.is_empty() {
            return Err(program_error(
                "Font_SystemFont constructor called with empty font name; expected a font name",
            ));
        }
        let mut font_key = name.clone();
        font_key.lower_case();
        Ok(Self {
            transient: StandardTransient::default(),
            font_key,
            font_name: name.clone(),
            files: SharedState::new(FontFiles::default()),
        })
    }
    /// The lower-cased family name; `font.font_key()`.
    pub fn font_key(&self) -> &AsciiString {
        &self.font_key
    }
    /// The family name; `font.font_name()`.
    pub fn font_name(&self) -> &AsciiString {
        &self.font_name
    }
    /// The file of an aspect (empty when missing); `font.font_path(FontFontAspect::Bold)`.
    pub fn font_path(&self, aspect: FontFontAspect) -> AsciiString {
        self.files.read().paths[slot(aspect)].clone()
    }
    /// The face id of an aspect; `font.font_face_id(FontFontAspect::Regular)`.
    pub fn font_face_id(&self, aspect: FontFontAspect) -> i32 {
        self.files.read().face_ids[slot(aspect)]
    }
    /// Sets the file and face id of an aspect; `UNDEFINED` raises `Standard_ProgramError`.
    /// Example: `font.set_font_path(FontFontAspect::Bold, &path, 0)?`.
    pub fn set_font_path(
        &self,
        aspect: FontFontAspect,
        path: &AsciiString,
        face_id: i32,
    ) -> Result<(), Failure> {
        if aspect == FontFontAspect::Undefined {
            return Err(program_error(
                "Font_SystemFont::SetFontPath() called with UNDEFINED aspect; expected a defined aspect",
            ));
        }
        let mut files = self.files.write();
        files.paths[aspect as usize] = path.clone();
        files.face_ids[aspect as usize] = face_id;
        Ok(())
    }
    /// Whether the aspect has a file; `font.has_font_aspect(FontFontAspect::Italic)`.
    pub fn has_font_aspect(&self, aspect: FontFontAspect) -> bool {
        !self.files.read().paths[slot(aspect)].is_empty()
    }
    /// `FontPathAny`: the file of the aspect, else (for italic aspects) the bold or regular
    /// file with `to_synthesize_italic` set, else the regular file, else any file; the face id
    /// is written to `face_id`. Example: `font.font_path_any(aspect, &mut italic, &mut face)`.
    pub fn font_path_any(
        &self,
        aspect: FontFontAspect,
        to_synthesize_italic: &mut bool,
        face_id: &mut i32,
    ) -> AsciiString {
        let files = self.files.read();
        let (index, synthesize) = path_any_slot(&files, aspect);
        *to_synthesize_italic |= synthesize;
        *face_id = files.face_ids[index];
        files.paths[index].clone()
    }
    /// Whether the fonts are the same object or have the same key; a null other raises
    /// `Standard_ProgramError` (natively a null dereference). Example: `font.is_equal(&other)?`.
    pub fn is_equal(&self, other: &Handle<FontSystemFont>) -> Result<bool, Failure> {
        let Some(other_font) = other.get() else {
            return Err(program_error(
                "Font_SystemFont::IsEqual() - null font; expected a font to compare",
            ));
        };
        Ok(std::ptr::eq(self, other_font) || self.font_key.is_equal(&other_font.font_key))
    }
    /// Whether the font draws single strokes; `font.is_single_stroke_font()`.
    pub fn is_single_stroke_font(&self) -> bool {
        self.files.read().is_single_line
    }
    /// Marks the font single-stroke; `font.set_single_stroke_font(true)`.
    pub fn set_single_stroke_font(&self, is_single_line: bool) {
        self.files.write().is_single_line = is_single_line;
    }
    /// `operator==`: same key; `font.same_key(&other)`.
    pub fn same_key(&self, other: &FontSystemFont) -> bool {
        self.font_key.is_equal(other.font_key())
    }
    /// `std::hash<Handle(Font_SystemFont)>`: the string hash of the key, 0 for null.
    /// Example: `FontSystemFont::handle_hash(&font)`.
    pub fn handle_hash(font: &Handle<FontSystemFont>) -> usize {
        font.get().map_or(0, |f| f.font_key.occt_hash())
    }
    /// `ToString`: `'Name' [aspects: regular,bold] [paths: a;b,1]`; `font.to_string_desc()`.
    pub fn to_string_desc(&self) -> AsciiString {
        let files = self.files.read();
        let present: Vec<usize> = (0..FONT_ASPECTS.len())
            .filter(|&k| !files.paths[k].is_empty())
            .collect();
        let aspects: Vec<&[u8]> = present
            .iter()
            .map(|&k| FontFontMgr::font_aspect_to_string(FONT_ASPECTS[k]).as_bytes())
            .collect();
        let paths: Vec<Vec<u8>> = present.iter().map(|&k| path_desc(&files, k)).collect();
        let mut text = [b"'", self.font_name.as_bytes(), b"' [aspects: "].concat();
        text.extend_from_slice(&aspects.join(&b","[..]));
        text.extend_from_slice(b"] [paths: ");
        text.extend_from_slice(&paths.join(&b";"[..]));
        text.push(b']');
        AsciiString::from_view(&text)
    }
}

/// One `path[,face]` item of `ToString`.
fn path_desc(files: &FontFiles, k: usize) -> Vec<u8> {
    let mut item = files.paths[k].as_bytes().to_vec();
    if files.face_ids[k] != 0 {
        item.extend_from_slice(format!(",{}", files.face_ids[k]).as_bytes());
    }
    item
}

/// The slot `FontPathAny` reads and whether it synthesizes italic.
fn path_any_slot(files: &FontFiles, aspect: FontFontAspect) -> (usize, bool) {
    let has = |k: usize| !files.paths[k].is_empty();
    let (regular, bold) = (
        FontFontAspect::Regular as usize,
        FontFontAspect::Bold as usize,
    );
    let index = slot(aspect);
    if has(index) {
        return (index, false);
    }
    let italic = matches!(aspect, FontFontAspect::Italic | FontFontAspect::BoldItalic);
    if italic && aspect == FontFontAspect::BoldItalic && has(bold) {
        return (bold, true);
    }
    if italic && has(regular) {
        return (regular, true);
    }
    if has(regular) {
        return (regular, false);
    }
    (
        (0..FONT_ASPECTS.len()).find(|&k| has(k)).unwrap_or(regular),
        false,
    )
}
