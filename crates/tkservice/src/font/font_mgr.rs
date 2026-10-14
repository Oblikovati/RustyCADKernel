//! `Font_FontMgr`: the process-wide registry of the system fonts and font aliases.
use super::system_font::FONT_ASPECTS;
use super::{FontFontAspect, FontStrictLevel, FontSystemFont, FontUnicodeSubset};
use rustycad_tkernel::message::Message;
use rustycad_tkernel::ncollection::DataMap;
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkernel::tcollection::{AsciiString, HAsciiString};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, OnceLock};

/// `Font_FontAlias`: a font name (lower-cased) and the aspect it stands for.
#[derive(Clone, Debug)]
pub(crate) struct FontAlias {
    pub(crate) font_name: AsciiString,
    pub(crate) font_aspect: FontFontAspect,
}

impl FontAlias {
    fn new(name: &str) -> Self {
        Self::with_aspect(AsciiString::from(name), FontFontAspect::Undefined)
    }
    fn with_aspect(font_name: AsciiString, font_aspect: FontFontAspect) -> Self {
        Self {
            font_name,
            font_aspect,
        }
    }
}

/// `Font_FontAliasSequence` (shared: several alias names bind the same sequence).
pub(crate) type FontAliasSequence = Arc<SharedState<Vec<FontAlias>>>;

/// The registry fields, shared by every handle.
pub(crate) struct FontMgrState {
    /// `myFontMap`: the fonts in registration order.
    pub(crate) fonts: Vec<Handle<FontSystemFont>>,
    pub(crate) aliases: DataMap<AsciiString, FontAliasSequence>,
    pub(crate) fallback_alias: FontAliasSequence,
    pub(crate) to_trace_aliases: bool,
    pub(crate) to_print_errors: bool,
}

/// The font manager; `FontFontMgr::get_instance().get_available_fonts()`.
pub struct FontFontMgr {
    transient: StandardTransient,
    pub(crate) state: SharedState<FontMgrState>,
}
rustycad_tkernel::impl_transient!(FontFontMgr, "Font_FontMgr", StandardTransient, transient);

/// The process-wide `ToUseUnicodeSubsetFallback` flag (on by default).
static UNICODE_SUBSET_FALLBACK: AtomicBool = AtomicBool::new(true);

impl FontFontMgr {
    /// The process-wide instance, built (and its database scanned) on first use.
    /// Example: `FontFontMgr::get_instance()`.
    pub fn get_instance() -> Handle<FontFontMgr> {
        static INSTANCE: OnceLock<Handle<FontFontMgr>> = OnceLock::new();
        INSTANCE.get_or_init(|| Handle::new(Self::new())).clone()
    }
    /// The private constructor: the default aliases, then `InitFontDataBase`.
    fn new() -> Self {
        let sans = alias_sequence(&["arial", "helvetica", "freesans", "dejavu sans"]);
        let manager = Self {
            transient: StandardTransient::default(),
            state: SharedState::new(FontMgrState {
                fonts: Vec::new(),
                aliases: DataMap::default(),
                fallback_alias: Arc::clone(&sans),
                to_trace_aliases: false,
                to_print_errors: true,
            }),
        };
        manager.add_default_aliases(&sans);
        if let Err(error) = manager.init_font_data_base() {
            // Natively the exception escapes the singleton construction; here the fonts
            // registered before it stay and the failure is reported.
            Message::send_fail_text(&AsciiString::from_view(error.message().as_bytes()));
        }
        manager
    }
    /// `FontAspectToString`; `FontFontMgr::font_aspect_to_string(FontFontAspect::Bold) == "bold"`.
    pub fn font_aspect_to_string(aspect: FontFontAspect) -> &'static str {
        match aspect {
            FontFontAspect::Undefined => "undefined",
            FontFontAspect::Regular => "regular",
            FontFontAspect::Bold => "bold",
            FontFontAspect::Italic => "italic",
            FontFontAspect::BoldItalic => "bold-italic",
            // Pinned Font_FontMgr.hxx preserves its post-switch invalid diagnostic.
            _ => "invalid",
        }
    }
    /// Whether fonts fall back to other fonts for unsupported Unicode subsets.
    /// Example: `FontFontMgr::to_use_unicode_subset_fallback()`.
    pub fn to_use_unicode_subset_fallback() -> bool {
        UNICODE_SUBSET_FALLBACK.load(Ordering::Relaxed)
    }
    /// Sets the process-wide fallback flag; `FontFontMgr::set_use_unicode_subset_fallback(false)`.
    pub fn set_use_unicode_subset_fallback(to_use: bool) {
        UNICODE_SUBSET_FALLBACK.store(to_use, Ordering::Relaxed);
    }
}

/// A new alias sequence of plain names.
fn alias_sequence(names: &[&str]) -> FontAliasSequence {
    Arc::new(SharedState::new(
        names.iter().map(|name| FontAlias::new(name)).collect(),
    ))
}

/// The lower-cased copy of a name.
pub(crate) fn lower(name: &AsciiString) -> AsciiString {
    let mut lowered = name.clone();
    lowered.lower_case();
    lowered
}

impl FontFontMgr {
    /// The alias table of the constructor (the X11 platform branch of the native code).
    fn add_default_aliases(&self, sans: &FontAliasSequence) {
        let mono = alias_sequence(&["courier new", "freemono", "dejavu sans mono"]);
        let serif = alias_sequence(&["times new roman", "times", "freeserif", "dejavu serif"]);
        let cjk = alias_sequence(&[
            "simsun",
            "droid sans fallback",
            "noto sans sc",
            "noto sans cjk jp",
        ]);
        let korean = alias_sequence(&[
            "nanumgothic",
            "noto sans kr",
            "nanummyeongjo",
            "noto serif cjk jp",
            "noto sans cjk jp",
            "droid sans fallback",
        ]);
        let (symbol, script) = (alias_sequence(&["symbol"]), alias_sequence(&["script"]));
        let (windin, iris) = (
            alias_sequence(&["wingdings"]),
            alias_sequence(&["lucida console"]),
        );
        let arabic = alias_sequence(&[]);
        let undefined = FontFontAspect::Undefined;
        let table: [(&str, &FontAliasSequence, FontFontAspect); 22] = [
            ("mono", &mono, undefined),
            ("courier", &mono, undefined),
            ("monospace", &mono, undefined),
            ("rock", sans, undefined),
            ("sansserif", sans, undefined),
            ("sans-serif", sans, undefined),
            ("sans", sans, undefined),
            ("arial", sans, undefined),
            ("times", &serif, undefined),
            ("serif", &serif, undefined),
            ("times-roman", &serif, undefined),
            ("times-bold", &serif, FontFontAspect::Bold),
            ("times-italic", &serif, FontFontAspect::Italic),
            ("times-bolditalic", &serif, FontFontAspect::BoldItalic),
            ("symbol", &symbol, undefined),
            ("iris", &iris, undefined),
            ("korean", &korean, undefined),
            ("cjk", &cjk, undefined),
            ("nsimsun", &cjk, undefined),
            ("arabic", &arabic, undefined),
            ("ZapfDingbats", &windin, undefined),
            ("ZapfChancery-MediumItalic", &script, undefined),
        ];
        for (name, aliases, aspect) in table {
            self.add_font_alias_seq(&AsciiString::from(name), aliases, aspect);
        }
    }
    /// `addFontAlias`: binds a lower-cased alias to the sequence (a copy with `aspect` set on
    /// every entry when the aspect is defined); empty sequences are skipped.
    fn add_font_alias_seq(
        &self,
        alias: &AsciiString,
        aliases: &FontAliasSequence,
        aspect: FontFontAspect,
    ) {
        if aliases.read().is_empty() {
            return;
        }
        let bound = if aspect == FontFontAspect::Undefined {
            Arc::clone(aliases)
        } else {
            let entries = aliases
                .read()
                .iter()
                .map(|a| FontAlias::with_aspect(a.font_name.clone(), aspect))
                .collect();
            Arc::new(SharedState::new(entries))
        };
        self.state.write().aliases.bind(lower(alias), bound);
    }
}

impl FontFontMgr {
    /// `AddFontAlias`: appends `font_name` to the alias (created when missing). Quirk kept:
    /// the duplicate check compares the entries with the alias name, not the font name.
    /// Example: `manager.add_font_alias(&"ui".into(), &"dejavu sans".into())`.
    pub fn add_font_alias(&self, alias: &AsciiString, font_name: &AsciiString) -> bool {
        let alias = lower(alias);
        let sequence = self.alias_or_new(&alias);
        let mut entries = sequence.write();
        if entries.iter().any(|entry| entry.font_name.is_equal(&alias)) {
            return false;
        }
        entries.push(FontAlias::with_aspect(
            font_name.clone(),
            FontFontAspect::Undefined,
        ));
        true
    }
    fn alias_or_new(&self, alias: &AsciiString) -> FontAliasSequence {
        let mut state = self.state.write();
        if let Some(found) = state.aliases.seek(alias) {
            return Arc::clone(found);
        }
        let created: FontAliasSequence = Arc::new(SharedState::new(Vec::new()));
        state.aliases.bind(alias.clone(), Arc::clone(&created));
        created
    }
    /// `RemoveFontAlias`: an empty alias clears every alias, an empty font name removes the
    /// alias, else the font is removed from it (and an emptied alias with it).
    /// Example: `manager.remove_font_alias(&"ui".into(), &"dejavu sans".into())`.
    pub fn remove_font_alias(&self, alias: &AsciiString, font_name: &AsciiString) -> bool {
        let mut state = self.state.write();
        if alias.is_empty() {
            let had = !state.aliases.is_empty();
            state.aliases.clear(true);
            return had;
        }
        let alias = lower(alias);
        let Some(sequence) = state.aliases.seek(&alias).cloned() else {
            return false;
        };
        if font_name.is_empty() {
            return state.aliases.unbind(&alias);
        }
        let mut entries = sequence.write();
        let Some(position) = entries.iter().position(|e| e.font_name.is_equal(font_name)) else {
            return false;
        };
        entries.remove(position);
        if entries.is_empty() {
            state.aliases.unbind(&alias);
        }
        true
    }
    /// `GetAllAliases`: the alias names in the native map order.
    /// Example: `manager.get_all_aliases()`.
    pub fn get_all_aliases(&self) -> Vec<Handle<HAsciiString>> {
        let state = self.state.read();
        state
            .aliases
            .items()
            .map(|(name, _)| Handle::new(HAsciiString::from_string(name)))
            .collect()
    }
    /// `GetFontAliases`: the font names of an alias; `manager.get_font_aliases(&"mono".into())`.
    pub fn get_font_aliases(&self, alias: &AsciiString) -> Vec<Handle<HAsciiString>> {
        let state = self.state.read();
        let Some(sequence) = state.aliases.seek(&lower(alias)) else {
            return Vec::new();
        };
        sequence
            .read()
            .iter()
            .map(|a| Handle::new(HAsciiString::from_string(&a.font_name)))
            .collect()
    }
    /// Whether the alias substitutions are traced; `manager.to_trace_aliases()`.
    pub fn to_trace_aliases(&self) -> bool {
        self.state.read().to_trace_aliases
    }
    /// Traces the alias substitutions; `manager.set_trace_aliases(true)`.
    pub fn set_trace_aliases(&self, to_trace: bool) {
        self.state.write().to_trace_aliases = to_trace;
    }
    /// Whether a missing font is reported; `manager.to_print_errors()`.
    pub fn to_print_errors(&self) -> bool {
        self.state.read().to_print_errors
    }
    /// Reports missing fonts; `manager.set_print_errors(false)`.
    pub fn set_print_errors(&self, to_print: bool) {
        self.state.write().to_print_errors = to_print;
    }
}

impl FontFontMgr {
    /// `AvailableFonts`/`GetAvailableFonts`: the registered fonts in order.
    /// Example: `manager.get_available_fonts().len()`.
    pub fn get_available_fonts(&self) -> Vec<Handle<FontSystemFont>> {
        self.state.read().fonts.clone()
    }
    /// `GetAvailableFontsNames`: the family names in order; `manager.get_available_fonts_names()`.
    pub fn get_available_fonts_names(&self) -> Vec<Handle<HAsciiString>> {
        let state = self.state.read();
        state
            .fonts
            .iter()
            .map(|f| Handle::new(HAsciiString::from_string(f.font_name())))
            .collect()
    }
    /// `Font_FontMap::Find`: the font of a name (any font for an empty name), or null.
    pub(crate) fn find_in_map(
        fonts: &[Handle<FontSystemFont>],
        name: &AsciiString,
    ) -> Handle<FontSystemFont> {
        if fonts.is_empty() {
            return Handle::default();
        }
        if name.is_empty() {
            return fonts[0].clone();
        }
        let key = lower(name);
        fonts
            .iter()
            .find(|f| f.font_key().is_equal(&key))
            .cloned()
            .unwrap_or_default()
    }
    /// `GetFont(name)`: the registered font of a name, or null; `manager.get_font(&"arial".into())`.
    pub fn get_font(&self, name: &AsciiString) -> Handle<FontSystemFont> {
        Self::find_in_map(&self.state.read().fonts, name)
    }
    /// `GetFont(name, aspect, size)`: null for a size below 2 (except -1), a null name or a
    /// font without the requested aspect. Example: `manager.get_font_with(&name, aspect, 12)`.
    pub fn get_font_with(
        &self,
        name: &Handle<HAsciiString>,
        aspect: FontFontAspect,
        size: i32,
    ) -> Handle<FontSystemFont> {
        let Some(name) = name.get() else {
            return Handle::default();
        };
        if size < 2 && size != -1 {
            return Handle::default();
        }
        let font = self.get_font(&name.string());
        let keep = font
            .get()
            .is_none_or(|f| aspect == FontFontAspect::Undefined || f.has_font_aspect(aspect));
        if keep { font } else { Handle::default() }
    }
    /// `RegisterFont`: adds a new font, or merges the aspect files of a known one (`override`
    /// replaces existing files; a conflicting file without it refuses the merge).
    /// Example: `manager.register_font(&font, false)?`.
    pub fn register_font(
        &self,
        font: &Handle<FontSystemFont>,
        to_override: bool,
    ) -> Result<bool, Failure> {
        let Some(new_font) = font.get() else {
            return Ok(false);
        };
        let mut state = self.state.write();
        let Some(old) = state
            .fonts
            .iter()
            .find(|f| f.font_key().is_equal(new_font.font_key()))
            .cloned()
        else {
            state.fonts.push(font.clone());
            return Ok(true);
        };
        drop(state);
        merge_font(&old, new_font, to_override)
    }
    /// `RegisterFonts`: registers each font; true when any was registered.
    /// Example: `manager.register_fonts(&fonts, false)?`.
    pub fn register_fonts(
        &self,
        fonts: &[Handle<FontSystemFont>],
        to_override: bool,
    ) -> Result<bool, Failure> {
        let mut registered = false;
        for font in fonts {
            registered = self.register_font(font, to_override)? || registered;
        }
        Ok(registered)
    }
    /// `ClearFontDataBase`: forgets every font; `manager.clear_font_data_base()`.
    pub fn clear_font_data_base(&self) {
        self.state.write().fonts.clear();
    }
}

/// The aspect merge of `RegisterFont`.
fn merge_font(
    old: &FontSystemFont,
    new: &FontSystemFont,
    to_override: bool,
) -> Result<bool, Failure> {
    for aspect in FONT_ASPECTS {
        let same = old.font_path(aspect).is_equal(new.font_path(aspect))
            && old.font_face_id(aspect) == new.font_face_id(aspect);
        if same {
            continue;
        }
        if to_override || !old.has_font_aspect(aspect) {
            old.set_font_path(aspect, &new.font_path(aspect), new.font_face_id(aspect))?;
        } else if new.has_font_aspect(aspect) {
            return Ok(false);
        }
    }
    Ok(true)
}

/// The fallback alias names of the Unicode subsets (`Font_NOF_*`).
pub(crate) fn subset_alias(subset: FontUnicodeSubset) -> Option<(&'static str, &'static str)> {
    match subset {
        FontUnicodeSubset::Western => Some(("sans-serif", "Western")),
        FontUnicodeSubset::Korean => Some(("korean", "Korean")),
        FontUnicodeSubset::Cjk => Some(("cjk", "CJK")),
        FontUnicodeSubset::Arabic => Some(("arabic", "Arabic")),
        FontUnicodeSubset::Nb => None,
    }
}

impl FontFontMgr {
    /// `FindFallbackFont`: the font of the subset alias (aliases only), with a failure
    /// message when none. Example: `manager.find_fallback_font(FontUnicodeSubset::Cjk, aspect)`.
    pub fn find_fallback_font(
        &self,
        subset: FontUnicodeSubset,
        aspect: FontFontAspect,
    ) -> Handle<FontSystemFont> {
        let Some((alias, range)) = subset_alias(subset) else {
            return Handle::default();
        };
        let mut font_aspect = aspect;
        let font = self.find_font(
            &AsciiString::from(alias),
            FontStrictLevel::Aliases,
            &mut font_aspect,
            false,
        );
        if font.is_null() {
            let text = format!("Font_FontMgr, error: unable to find {range} fallback font!");
            Message::send_fail_text(&AsciiString::from(text.as_str()));
        }
        font
    }
}
