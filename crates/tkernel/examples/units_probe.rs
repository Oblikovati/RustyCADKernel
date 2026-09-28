//! Unit sentence and conversion probe using byte-exact legacy unit symbols.
use rustycad_tkernel::{
    standard::Failure,
    units::{UnitSentence, UnitSystem, UnitsApi, UnitsDictionary},
};
use std::io::{self, BufRead};
fn unhex(text: &str) -> Vec<u8> {
    (0..text.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&text[i..i + 2], 16).unwrap())
        .collect()
}
fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}
fn probe(line: &str) -> Result<String, Failure> {
    let fields: Vec<_> = line.split_whitespace().collect();
    let expression = unhex(fields[0]);
    let value: f64 = fields[1].parse().unwrap();
    let target = unhex(fields[2]);
    let mut units = UnitsApi::new(UnitsDictionary::builtin()?);
    let mut sentence = UnitSentence::new(units.dictionary(), &expression)?;
    let mut output = vec![usize::from(sentence.is_done()).to_string()];
    if sentence.is_done() {
        let Some(token) = sentence.evaluate()? else {
            return Ok("null".into());
        };
        output.push(hex(token.word()));
        output.push(token.value().to_bits().to_string());
        output.extend(
            token
                .dimensions()
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
    units.set_local_system(UnitSystem::Mdtv)?;
    output.push(units.any_to_ls(value, &expression)?.to_bits().to_string());
    Ok(output.join(" "))
}
fn main() {
    for line in io::stdin().lock().lines() {
        match probe(&line.unwrap()) {
            Ok(result) => println!("{result}"),
            Err(error) => println!(
                "error {} {}",
                error.exception_type(),
                hex(error.message().as_bytes())
            ),
        }
    }
}
