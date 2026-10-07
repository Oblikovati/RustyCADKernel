// Rust port of DataExchange/TKXSBase/Interface/Interface_Static.cxx (the static functions
// on the map of static values) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The static functions of `Interface_Static`: creation, lookup, definitions and values
//! of the static values by name, in the map `MoniTool_TypedValue::Stats`.
use super::{InterfaceParamType, InterfaceStatic};
use crate::c_text::atoi;
use crate::moni_tool::MoniToolTypedValue;
use rustycad_tkernel::{
    ncollection::DataMap,
    standard::{Failure, Handle, c_string::atof_bytes},
    tcollection::{AsciiString, HAsciiString},
};

/// The parameter type of a type letter of `Init`; `None` for '&' and unknown letters.
fn letter_type(letter: u8) -> Option<InterfaceParamType> {
    match letter {
        b'e' => Some(InterfaceParamType::ParamEnum),
        b'i' => Some(InterfaceParamType::ParamInteger),
        b'o' => Some(InterfaceParamType::ParamIdent),
        b'p' | b't' => Some(InterfaceParamType::ParamText),
        b'r' => Some(InterfaceParamType::ParamReal),
        b'=' => Some(InterfaceParamType::ParamMisc),
        _ => None,
    }
}

/// `sscanf(part, "%30s %d")`: the integer after the first word, 0 when none.
fn second_integer(part: &str) -> i32 {
    let rest = part.split_whitespace().nth(1).unwrap_or("");
    let digits = rest.trim_start_matches(['+', '-']);
    if !digits.starts_with(|c: char| c.is_ascii_digit()) {
        return 0;
    }
    atoi(rest.as_bytes())
}

impl InterfaceStatic {
    /// `Init(family, name, type, init = "")`: records a new static value (for
    /// `ParamMisc`, a copy of the static named `init`); false for an empty or known name.
    pub fn init(
        family: &str,
        name: &str,
        kind: InterfaceParamType,
        init: &str,
    ) -> Result<bool, Failure> {
        if name.is_empty() || MoniToolTypedValue::stats().is_bound(&AsciiString::new(name)) {
            return Ok(false);
        }
        let item = if kind == InterfaceParamType::ParamMisc {
            let other = Self::static_named(init);
            let Some(live) = other.get() else {
                return Ok(false);
            };
            InterfaceStatic::from_other(family, name, live)?
        } else {
            InterfaceStatic::new(family, name, kind, init)?
        };
        MoniToolTypedValue::stats().bind(AsciiString::new(name), Handle::new(item).cast());
        Ok(true)
    }
    /// `Init(family, name, type, init = "")` with a type letter: 'e', 'i', 'o', 'p', 'r',
    /// 't', '=' create a static ('p' clears a value that does not satisfy); '&' edits the
    /// definition of an existing static as `MoniTool_TypedValue::AddDef` ("imin 0"...).
    pub fn init_char(family: &str, name: &str, kind: u8, init: &str) -> Result<bool, Failure> {
        if kind == b'&' {
            return Self::edit_definition(name, init);
        }
        let Some(epyt) = letter_type(kind) else {
            return Ok(false);
        };
        if !Self::init(family, name, epyt, init)? {
            return Ok(false);
        }
        if kind != b'p' {
            return Ok(true);
        }
        let stat = Self::static_named(name);
        if let Some(live) = stat.get()
            && !live.satisfies(&live.h_string_value())
        {
            live.set_c_string_value("")?;
        }
        Ok(true)
    }
    /// The '&' form of `Init`: the edit commands of `AddDef` (without "tmax").
    fn edit_definition(name: &str, init: &str) -> Result<bool, Failure> {
        let unstat = Self::static_named(name);
        let Some(live) = unstat.get() else {
            return Ok(false);
        };
        let bytes = init.as_bytes();
        let iblc = bytes.iter().rposition(|b| *b == b' ').map_or(0, |i| i + 1);
        let arg = &init[iblc..];
        let at = |i: usize| bytes.get(i).copied().unwrap_or(0);
        match (at(0), at(1), at(2)) {
            (b'i', _, b'i') => live.set_integer_limit(false, atoi(arg.as_bytes()))?,
            (b'i', _, b'a') => live.set_integer_limit(true, atoi(arg.as_bytes()))?,
            (b'r', _, b'i') => live.set_real_limit(false, atof_bytes(arg.as_bytes()))?,
            (b'r', _, b'a') => live.set_real_limit(true, atof_bytes(arg.as_bytes()))?,
            (b'u', _, _) => live.set_unit_def(arg),
            (b'e', b'm', _) => live.start_enum(atoi(arg.as_bytes()), true)?,
            (b'e', b'n', _) => live.start_enum(atoi(arg.as_bytes()), false)?,
            (b'e', b'v', _) => live.add_enum(&[arg])?,
            _ => return Ok(false),
        }
        Ok(true)
    }
    /// `Static(name)`: the static value of a name, null when unknown.
    pub fn static_named(name: &str) -> Handle<InterfaceStatic> {
        let found = MoniToolTypedValue::stats()
            .seek(&AsciiString::new(name))
            .cloned();
        found.map(|h| h.cast()).unwrap_or_default()
    }
    /// `IsPresent(name)`.
    pub fn is_present(name: &str) -> bool {
        MoniToolTypedValue::stats().is_bound(&AsciiString::new(name))
    }
    /// `CDef(name, part)`: a part of the definition ("family", "label", "satis", "type",
    /// `"enum <n>"`, "imin"/"imax", "rmin"/"rmax", "unit"); empty when unknown.
    pub fn c_def(name: &str, part: &str) -> String {
        let stat = Self::static_named(name);
        let (Some(live), Some(first)) = (stat.get(), part.bytes().next()) else {
            return String::new();
        };
        let second = part.as_bytes().get(1).copied().unwrap_or(0);
        let lossy = |text: AsciiString| String::from_utf8_lossy(text.as_bytes()).into_owned();
        match (first, second) {
            (b'f', b'a') => lossy(live.family()),
            (b'l', b'a') => lossy(live.label()),
            (b's', b'a') => lossy(live.satisfies_name()),
            (b't', b'y') => type_word(live.param_type()).to_owned(),
            (b'e', _) => lossy(live.enum_val(second_integer(part))),
            (b'i', _) => match live.integer_limit(part.as_bytes().get(2) == Some(&b'a')) {
                (true, lim) => lim.to_string(),
                (false, _) => String::new(),
            },
            (b'r', _) => match live.real_limit(part.as_bytes().get(2) == Some(&b'a')) {
                (true, lim) => format!("{lim:.6}"),
                (false, _) => String::new(),
            },
            (b'u', _) => lossy(live.unit_def()),
            _ => String::new(),
        }
    }
    /// `IDef(name, part)`: an integer part of the definition ("imin"/"imax", "estart",
    /// "ecount", "ematch", `"eval <text>"`); 0 when unknown.
    pub fn i_def(name: &str, part: &str) -> i32 {
        let stat = Self::static_named(name);
        let (Some(live), Some(first)) = (stat.get(), part.bytes().next()) else {
            return 0;
        };
        if first == b'i' {
            let (set, lim) = live.integer_limit(part.as_bytes().get(2) == Some(&b'a'));
            return if set { lim } else { 0 };
        }
        if first != b'e' {
            return 0;
        }
        let (start, end, matched) = live.enum_def().unwrap_or((0, 0, false));
        match part.as_bytes().get(1) {
            Some(b's') => start,
            Some(b'c') => end - start + 1,
            Some(b'm') => i32::from(matched),
            Some(b'v') => live.enum_case(part.split_whitespace().nth(1).unwrap_or("")),
            _ => 0,
        }
    }
    /// `IsSet(name, proper = true)`: the value is set; without `proper`, or the value of
    /// its wild-card.
    pub fn is_set(name: &str, proper: bool) -> bool {
        let item = Self::static_named(name);
        let Some(live) = item.get() else { return false };
        if live.is_set_value() {
            return true;
        }
        if proper {
            return false;
        }
        live.wild().get().is_some_and(|w| w.is_set_value())
    }
    /// `CVal(name)`: the text value; empty with a warning on standard output when unknown.
    pub fn c_val(name: &str) -> AsciiString {
        let item = Self::static_named(name);
        match item.get() {
            Some(live) => live.c_string_value(),
            None => {
                println!("Warning: Interface_Static::CVal: incorrect parameter {name}");
                AsciiString::default()
            }
        }
    }
    /// `IVal(name)`: the integer value; 0 with a warning when unknown.
    pub fn i_val(name: &str) -> i32 {
        let item = Self::static_named(name);
        item.get().map_or_else(
            || {
                println!("Warning: Interface_Static::IVal: incorrect parameter {name}");
                0
            },
            |live| live.integer_value(),
        )
    }
    /// `RVal(name)`: the real value; 0.0 with a warning when unknown.
    pub fn r_val(name: &str) -> f64 {
        let item = Self::static_named(name);
        item.get().map_or_else(
            || {
                println!("Warning: Interface_Static::RVal: incorrect parameter {name}");
                0.0
            },
            |live| live.real_value(),
        )
    }
    /// `SetCVal(name, val)`: false when unknown or refused.
    pub fn set_c_val(name: &str, val: &str) -> Result<bool, Failure> {
        let item = Self::static_named(name);
        item.get()
            .map_or(Ok(false), |live| live.set_c_string_value(val))
    }
    /// `SetIVal(name, val)`.
    pub fn set_i_val(name: &str, val: i32) -> Result<bool, Failure> {
        let item = Self::static_named(name);
        item.get()
            .map_or(Ok(false), |live| live.set_integer_value(val))
    }
    /// `SetRVal(name, val)`.
    pub fn set_r_val(name: &str, val: f64) -> Result<bool, Failure> {
        let item = Self::static_named(name);
        item.get()
            .map_or(Ok(false), |live| live.set_real_value(val))
    }
    /// `Update(name)`: marks the static up to date; false when unknown.
    pub fn update(name: &str) -> bool {
        let item = Self::static_named(name);
        item.get().map(InterfaceStatic::set_uptodate).is_some()
    }
    /// `IsUpdated(name)`.
    pub fn is_updated(name: &str) -> bool {
        Self::static_named(name)
            .get()
            .is_some_and(InterfaceStatic::updated_status)
    }
    /// `Items(mode = 0, criter = "")`: the names of the statics of a family (all but the
    /// '$' families for "", the '$' ones for "$"), filtered by update status for modes
    /// 100 (not updated) and 200 (updated); in the native map order.
    pub fn items(mode: i32, criter: &str) -> Vec<Handle<HAsciiString>> {
        let modup = mode / 100; // 0 any, 1 non-update, 2 update
        let statics: Vec<(AsciiString, Handle<InterfaceStatic>)> = MoniToolTypedValue::stats()
            .items()
            .map(|(k, v)| (k.clone(), v.cast()))
            .collect();
        let mut list = Vec::new();
        for (key, item) in statics {
            let Some(live) = item.get() else { continue };
            let family = live.family();
            let dollar = family.as_bytes().first() == Some(&b'$');
            let ok = match criter {
                "$" => dollar,
                "" if dollar => continue,
                "" => true,
                other if other.as_bytes() != family.as_bytes() => continue,
                _ => true,
            };
            let ok = ok
                && (modup != 1 || !live.updated_status())
                && (modup != 2 || live.updated_status());
            if ok {
                list.push(Handle::new(HAsciiString::from_string(&key)));
            }
        }
        list
    }
    /// `FillMap(theMap)`: the text values of the statics (with a value), by name.
    pub fn fill_map(map: &mut DataMap<AsciiString, AsciiString>) {
        map.clear(false);
        let statics: Vec<(AsciiString, Handle<InterfaceStatic>)> = MoniToolTypedValue::stats()
            .items()
            .map(|(k, v)| (k.clone(), v.cast()))
            .collect();
        for (key, item) in statics {
            let Some(live) = item.get() else { continue };
            let hval = live.h_string_value();
            if let Some(text) = hval.get() {
                map.bind(key, text.string().clone());
            }
        }
    }
}

/// The type word of `CDef(name, "type")`.
fn type_word(typ: InterfaceParamType) -> &'static str {
    match typ {
        InterfaceParamType::ParamInteger => "integer",
        InterfaceParamType::ParamReal => "real",
        InterfaceParamType::ParamText => "text",
        InterfaceParamType::ParamEnum => "enum",
        _ => "?type?",
    }
}
