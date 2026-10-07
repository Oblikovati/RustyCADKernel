//! `Font_FontMgr::FindFont`: the font of a name, through the aliases and the fallback
//! alias, with the native warnings.
use super::font_mgr::{FontAliasSequence, FontFontMgr, lower};
use super::{FontFontAspect, FontStrictLevel, FontSystemFont};
use rustycad_tkernel::message::Message;
use rustycad_tkernel::standard::Handle;
use rustycad_tkernel::tcollection::AsciiString;

/// The outcome of one alias pass.
struct AliasPass {
    font: Handle<FontSystemFont>,
    is_alias_used: bool,
    is_best_alias: bool,
}

impl FontFontMgr {
    /// `FindFont`: the exact font, else (unless strict) the first alias font, preferring one
    /// with the requested aspect (an alias aspect replaces `aspect`), else (level `Any`) the
    /// sans-serif fallback alias and finally any font; null with a failure message when there
    /// is no font at all. A substitution prints a warning.
    /// Example: `manager.find_font(&"courier".into(), FontStrictLevel::Any, &mut aspect, true)`.
    pub fn find_font(
        &self,
        name: &AsciiString,
        strict: FontStrictLevel,
        aspect: &mut FontFontAspect,
        to_fail_msg: bool,
    ) -> Handle<FontSystemFont> {
        let key = lower(name);
        let mut font = self.get_font(&key);
        if !font.is_null() || strict == FontStrictLevel::Strict {
            return font;
        }
        for pass in 0..2 {
            let Some(aliases) = self.pass_aliases(pass, &key, strict) else {
                continue;
            };
            let result = self.scan_aliases(&aliases, font.clone(), aspect);
            font = result.font;
            if pass == 0 {
                self.trace_alias(result.is_alias_used, &font, name);
                if result.is_best_alias {
                    return font;
                }
                if !font.is_null() {
                    break;
                }
            }
        }
        self.settle_font(font, name, &key, strict, *aspect, to_fail_msg)
    }
    /// `FindFont(name, aspect)`: level `Any`; `manager.find_font_any(&name, &mut aspect)`.
    pub fn find_font_any(
        &self,
        name: &AsciiString,
        aspect: &mut FontFontAspect,
    ) -> Handle<FontSystemFont> {
        self.find_font(name, FontStrictLevel::Any, aspect, true)
    }
    /// The aliases of a pass: the name's own aliases, then (level `Any`) the fallback alias.
    fn pass_aliases(
        &self,
        pass: i32,
        key: &AsciiString,
        strict: FontStrictLevel,
    ) -> Option<FontAliasSequence> {
        let state = self.state.read();
        let aliases = match pass {
            0 => state.aliases.seek(key).cloned(),
            _ if strict == FontStrictLevel::Any => Some(state.fallback_alias.clone()),
            _ => None,
        }?;
        let is_empty = aliases.read().is_empty();
        (!is_empty).then_some(aliases)
    }
    /// One pass over an alias sequence.
    fn scan_aliases(
        &self,
        aliases: &FontAliasSequence,
        mut font: Handle<FontSystemFont>,
        aspect: &mut FontFontAspect,
    ) -> AliasPass {
        let (mut is_alias_used, mut is_best_alias) = (false, false);
        for alias in aliases.read().iter() {
            let candidate = self.get_font(&alias.font_name);
            let Some(found) = candidate.get() else {
                continue;
            };
            if font.is_null() {
                (font, is_alias_used) = (candidate.clone(), true);
            }
            let defined = alias.font_aspect != FontFontAspect::Undefined;
            if defined && found.has_font_aspect(alias.font_aspect) {
                (is_best_alias, *aspect) = (true, alias.font_aspect);
                break;
            }
            if !defined && (*aspect == FontFontAspect::Undefined || found.has_font_aspect(*aspect))
            {
                is_best_alias = true;
                break;
            }
        }
        AliasPass {
            font,
            is_alias_used,
            is_best_alias,
        }
    }
    fn trace_alias(&self, is_alias_used: bool, font: &Handle<FontSystemFont>, name: &AsciiString) {
        let Some(font) = font
            .get()
            .filter(|_| is_alias_used && self.to_trace_aliases())
        else {
            return;
        };
        let text = [
            &b"Font_FontMgr, using font alias '"[..],
            font.font_name().as_bytes(),
            b"' instead of requested '",
            name.as_bytes(),
            b"'",
        ]
        .concat();
        Message::send_trace_text(&AsciiString::from_view(&text));
    }
    /// The end of `FindFont`: any font for level `Any`, the failure or substitution messages.
    fn settle_font(
        &self,
        mut font: Handle<FontSystemFont>,
        name: &AsciiString,
        key: &AsciiString,
        strict: FontStrictLevel,
        aspect: FontFontAspect,
        to_fail_msg: bool,
    ) -> Handle<FontSystemFont> {
        if font.is_null() && strict == FontStrictLevel::Any {
            font = self.get_font(&AsciiString::default());
        }
        let Some(found) = font.get() else {
            if to_fail_msg && self.to_print_errors() {
                Message::send_fail_text(&AsciiString::from(
                    "Font_FontMgr, error: unable to find any font!",
                ));
            }
            return Handle::default();
        };
        let wrong_aspect = aspect != FontFontAspect::Undefined && !found.has_font_aspect(aspect);
        if wrong_aspect || (!key.is_empty() && !key.is_equal(found.font_key())) {
            let aspect_name = Self::font_aspect_to_string(aspect).as_bytes();
            let text = [
                &b"Font_FontMgr, warning: unable to find font '"[..],
                name.as_bytes(),
                b"' [",
                aspect_name,
                b"]; ",
                found.to_string_desc().as_bytes(),
                b" is used instead",
            ]
            .concat();
            Message::send_warning_text(&AsciiString::from_view(&text));
        }
        font
    }
}
