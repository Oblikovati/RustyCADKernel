// Rust port of DataExchange/TKXSBase/MoniTool/MoniTool_TypedValue.cxx (Definition, Print,
// PrintValue) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The texts of a `MoniTool_TypedValue`: its computed definition and its printing.
use super::typed_value::TypedValueState;
use super::{MoniToolTypedValue, MoniToolValueType};
use rustycad_tkernel::{standard::Handle, tcollection::AsciiString};
use std::io::{self, Write};

impl MoniToolTypedValue {
    /// `Print`: name, label, definition, value and the name of the satisfies function.
    pub fn print(&self, s: &mut dyn Write) -> io::Result<()> {
        let (name, label, satisf) = {
            let state = self.state.read();
            let satisf = state.thesatisf.map(|_| state.thesatisn.clone());
            (state.thename.clone(), state.thelabel.clone(), satisf)
        };
        s.write_all(b"--- Typed Value : ")?;
        s.write_all(name.as_bytes())?;
        if label.length() > 0 {
            s.write_all(b"  Label : ")?;
            s.write_all(label.as_bytes())?;
        }
        s.write_all(b"\n--- Type : ")?;
        s.write_all(self.definition().as_bytes())?;
        s.write_all(b"\n--- Value : ")?;
        self.print_value(s)?;
        s.write_all(b"\n")?;
        if let Some(satisn) = satisf {
            s.write_all(b" -- Specific Function for Satisfies : ")?;
            s.write_all(satisn.as_bytes())?;
            s.write_all(b"\n")?;
        }
        Ok(())
    }
    /// `PrintValue`: the object type, the text and its native and coded forms, or
    /// "(not set)".
    pub fn print_value(&self, s: &mut dyn Write) -> io::Result<()> {
        if !self.is_set_value() {
            return s.write_all(b"(not set)");
        }
        let (kind, hval, oval) = {
            let state = self.state.read();
            (state.thetype, state.thehval.clone(), state.theoval.clone())
        };
        let ident = kind == MoniToolValueType::ValueIdent;
        if let Some(typ) = oval.dynamic_type().filter(|_| ident) {
            write!(s, " (type) {}", typ.name())?;
        }
        if let Some(text) = hval.get() {
            s.write_all(if ident { b" : " } else { b"" })?;
            s.write_all(text.string().as_bytes())?;
        }
        if self.has_interpret() {
            self.print_interpreted(s, &hval)?;
        }
        Ok(())
    }
    /// The "  (Native:...  Coded:...)" part of `PrintValue`.
    fn print_interpreted(
        &self,
        s: &mut dyn Write,
        hval: &Handle<rustycad_tkernel::tcollection::HAsciiString>,
    ) -> io::Result<()> {
        s.write_all(b"  (")?;
        let native = self.interpret(hval, true);
        if native.is_some() && native != *hval {
            s.write_all(b"Native:")?;
            s.write_all(native.string().as_bytes())?;
        }
        let coded = self.interpret(hval, false);
        if coded.is_some() && coded != *hval {
            s.write_all(b"  Coded:")?;
            s.write_all(coded.string().as_bytes())?;
        }
        s.write_all(b")")
    }
}

impl TypedValueState {
    /// `Definition`: the enforced definition, else the one of the type.
    pub(super) fn definition(&self) -> AsciiString {
        if self.thedef.length() > 0 {
            return self.thedef.clone();
        }
        let text = match self.thetype {
            MoniToolValueType::ValueInteger => self.integer_definition(),
            MoniToolValueType::ValueReal => self.real_definition(),
            MoniToolValueType::ValueEnum => self.enum_definition(),
            MoniToolValueType::ValueIdent => self.ident_definition(),
            MoniToolValueType::ValueText if self.themaxlen > 0 => {
                format!("Text <= {} C.", self.themaxlen)
            }
            MoniToolValueType::ValueText => String::from("Text"),
            _ => String::from("(undefined)"),
        };
        AsciiString::new(&text)
    }
    fn integer_definition(&self) -> String {
        let mut def = String::from("Integer");
        if let (true, low) = self.integer_limit(false) {
            def += &format!("  >= {low}");
        }
        if let (true, up) = self.integer_limit(true) {
            def += &format!("  <= {up}");
        }
        def
    }
    fn real_definition(&self) -> String {
        let mut def = String::from("Real");
        if let (true, low) = self.real_limit(false) {
            def += &format!("  >= {low:.6}");
        }
        if let (true, up) = self.real_limit(true) {
            def += &format!("  <= {up:.6}");
        }
        if self.theunidef.length() > 0 {
            def += "  Unit:";
            def += &String::from_utf8_lossy(self.theunidef.as_bytes());
        }
        def
    }
    fn enum_definition(&self) -> String {
        let (start, end, match_required) = self.enum_def().unwrap_or((0, 0, false));
        let mut def = format!(
            "Enum [{}{start}-{end}]",
            if match_required { "in " } else { "" }
        );
        for case in start..=end {
            let text = self.enum_val(case);
            if matches!(text.as_bytes().first(), None | Some(b'?')) {
                continue;
            }
            def += &format!(" {case}:{}", String::from_utf8_lossy(text.as_bytes()));
        }
        if !self.theeadds.is_empty() {
            def += " , alpha: ";
            for (name, case) in self.theeadds.items() {
                if name.as_bytes().first() == Some(&b'?') {
                    continue;
                }
                def += &format!("{}:{case} ", String::from_utf8_lossy(name.as_bytes()));
            }
        }
        def
    }
    fn ident_definition(&self) -> String {
        match self.theotyp.get() {
            Some(typ) => format!("Object(Entity):{}", typ.name()),
            None => String::from("Object(Entity)"),
        }
    }
}
