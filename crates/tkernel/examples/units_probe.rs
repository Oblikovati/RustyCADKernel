//! Unit sentence and conversion probe using byte-exact legacy unit symbols.
//! One context lives for the whole run, as the native globals do.
use rustycad_tkernel::{
    resource::ResourceHost,
    standard::Failure,
    units::{UnitSentence, Units, UnitsDiagnostics},
    units_api::{SystemUnits, UnitsApi},
};
use std::{
    io::{self, BufRead},
    sync::Arc,
};
/// The native probe swallows the conversion warnings; so does this one.
struct SilentDiagnostics;
impl UnitsDiagnostics for SilentDiagnostics {
    fn print(&self, _: &[u8]) {}
    fn warning(&self, _: &[u8]) {}
    fn info(&self, _: &[u8]) {}
}
fn unhex(text: &str) -> Vec<u8> {
    (0..text.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&text[i..i + 2], 16).unwrap())
        .collect()
}
fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}
fn probe(units: &mut UnitsApi, line: &str) -> Result<String, Failure> {
    let fields: Vec<_> = line.split_whitespace().collect();
    let expression = unhex(fields[0]);
    let value: f64 = fields[1].parse().unwrap();
    let target = unhex(fields[2]);
    units.set_local_system(SystemUnits::Si)?;
    let mut sentence = UnitSentence::new(units.units(), &expression)?;
    let mut output = vec![usize::from(sentence.is_done()?).to_string()];
    if sentence.is_done()? {
        let token = sentence.evaluate()?;
        let Some(token) = token.get() else {
            return Ok("null".into());
        };
        output.push(hex(token.word().as_bytes()));
        output.push(token.value().to_bits().to_string());
        let dimensions = token.dimensions();
        output.extend(
            dimensions
                .get()
                .unwrap()
                .exponents()
                .iter()
                .map(|value| value.to_bits().to_string()),
        );
    }
    output.push(units.any_to_si(value, &expression)?.to_bits().to_string());
    output.push(units.any_from_si(value, &expression)?.to_bits().to_string());
    output.push(
        units
            .any_to_any(value, &expression, &target)?
            .to_bits()
            .to_string(),
    );
    units.set_local_system(SystemUnits::Mdtv)?;
    output.push(units.any_to_ls(value, &expression)?.to_bits().to_string());
    Ok(output.join(" "))
}
fn main() {
    let silent = Units::with_diagnostics(Box::new(SilentDiagnostics));
    let mut units = UnitsApi::new(Arc::new(silent), ResourceHost::native());
    for line in io::stdin().lock().lines() {
        match probe(&mut units, &line.unwrap()) {
            Ok(result) => println!("{result}"),
            Err(error) => println!(
                "error {} {}",
                error.exception_type(),
                hex(error.message().as_bytes())
            ),
        }
    }
}
