// Rust port of DataExchange/TKXSBase/Interface/Interface_CheckIterator.cxx (Print)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The printing of an `Interface_CheckIterator`.
use super::{
    InterfaceCheck, InterfaceCheckIterator, InterfaceInterfaceModel, InterfaceModelEntities,
};
use rustycad_tkernel::standard::Handle;
use std::io::{self, Write};

impl InterfaceCheckIterator {
    /// `Print(S, failsonly, final = 0)`: with the model of the iterator.
    pub fn print(&self, s: &mut dyn Write, failsonly: bool, final_form: i32) -> io::Result<()> {
        self.print_with_model(s, &self.model(), failsonly, final_form)
    }
    /// `Print(S, model, failsonly, final = 0)`: a title, then each check with warnings
    /// (unless `failsonly`) or fails, with its entity number and type.
    pub fn print_with_model(
        &self,
        s: &mut dyn Write,
        model: &Handle<dyn InterfaceInterfaceModel>,
        failsonly: bool,
        _final_form: i32,
    ) -> io::Result<()> {
        let (checks, nums) = self.snapshot();
        let nb = checks.len();
        let mut titre = false;
        for (i, (ach, nm0)) in checks.iter().zip(nums).enumerate() {
            let nbw = if failsonly { 0 } else { ach.nb_warnings() };
            if ach.nb_fails() + nbw == 0 {
                continue;
            }
            if !titre {
                s.write_all(b" **  ")?;
                s.write_all(self.name().as_bytes())?;
                s.write_all(b"  **\n")?;
            }
            titre = true;
            print_check_head(s, model, ach, nm0, i + 1, nb)?;
            ach.print(s, if failsonly { 1 } else { 3 }, 1)?;
        }
        Ok(())
    }
}

/// The "Check:<i> -- ..." lines of one check.
fn print_check_head(
    s: &mut dyn Write,
    model: &Handle<dyn InterfaceInterfaceModel>,
    ach: &InterfaceCheck,
    nm0: i32,
    i: usize,
    nb: usize,
) -> io::Result<()> {
    let ent = ach.entity();
    let entnul = ent.is_null();
    let num = match (nm0 <= 0, entnul, model.get()) {
        (true, true, _) => -1, // Global
        (true, false, Some(live)) => live.number(&ent),
        _ => nm0,
    };
    s.write_all(b"Check:")?;
    if nb > 9 && i < 10 {
        s.write_all(b" ")?;
    }
    if nb > 99 && i < 100 {
        s.write_all(b" ")?;
    }
    write!(s, "{i}")?;
    if num < 0 {
        return s.write_all(b" -- Global Check\n");
    }
    match (num, model.get()) {
        (0, _) => s.write_all(b" -- Entity n0 ??:")?,
        (_, Some(live)) => {
            s.write_all(b" -- Entity (n0:id) ")?;
            live.print(&ent, s, 0)?;
        }
        (_, None) => write!(s, " -- Entity n0 {num}")?,
    }
    if entnul {
        return s.write_all(b" (unknown Type)\n");
    }
    let type_name = match model.get() {
        Some(live) => live.type_name(&ent, true),
        None => ent
            .dynamic_type()
            .map_or_else(String::new, |t| t.name().to_owned()),
    };
    writeln!(s, "   Type:{type_name}")
}
