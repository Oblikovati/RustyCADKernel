//! `units_system` requests on the probe's single units system.
use super::super::common::{arg, bits, flag, hex, real, resource_host, word};
use super::super::{UnitsProbeState, run};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkernel::units::UnitsSystem;
use std::str::SplitWhitespace;

fn state_text(state: &UnitsProbeState, res: &mut String) -> Result<(), Failure> {
    let system = state.system.get().unwrap();
    let active = system.active_units_sequence();
    let values: Vec<i32> = active
        .get()
        .unwrap()
        .sequence(|s| s.iter().copied().collect());
    res.push_str(&format!("{} {}", flag(system.is_empty()?), values.len()));
    for value in values {
        res.push_str(&format!(",{value}"));
    }
    system.dump(&mut state.capture.writer())
}

pub fn system(state: &mut UnitsProbeState, input: &mut SplitWhitespace<'_>) -> String {
    let op = word(input);
    let quantity = if matches!(op.as_str(), "new" | "named" | "activates" | "state") {
        Vec::new()
    } else {
        arg(input)
    };
    let value = if op == "convert" { real(input) } else { 0.0 };
    let unit = if matches!(op.as_str(), "specify" | "remove" | "activate" | "convert") {
        arg(input)
    } else {
        Vec::new()
    };
    run(state, |state, res| {
        let units = state.units.clone();
        match op.as_str() {
            "new" => state.system = Handle::new(UnitsSystem::new()),
            "named" => {
                let host = resource_host(&state.capture);
                let named = UnitsSystem::with_resource("Units", false, host)?;
                state.system = Handle::new(named);
            }
            "specify" => state
                .system
                .get()
                .unwrap()
                .specify(&units, &quantity, &unit)?,
            "remove" => state.system.get().unwrap().remove(&quantity, &unit)?,
            "activate" => state.system.get().unwrap().activate(&quantity, &unit)?,
            "activates" => state.system.get().unwrap().activates()?,
            "active" => res.push_str(&hex(state
                .system
                .get()
                .unwrap()
                .active_unit(&quantity)?
                .as_bytes())),
            "convert" => {
                let system = state.system.get().unwrap();
                let to_user =
                    system.convert_value_to_user_system(&units, &quantity, value, &unit)?;
                res.push_str(&format!("{} ", bits(to_user)));
                let from_si = system.convert_si_value_to_user_system(&units, &quantity, value)?;
                res.push_str(&format!("{} ", bits(from_si)));
                res.push_str(
                    &bits(system.convert_user_system_value_to_si(&units, &quantity, value)?)
                        .to_string(),
                );
            }
            _ => state_text(state, res)?,
        }
        Ok(())
    })
}
