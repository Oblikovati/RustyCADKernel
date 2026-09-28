//! UnitsAPI and UnitsMethods requests (twin of `tkernel_units_probe_api.cxx`).
use super::common::{arg, bits, dims, hex, real, word};
use super::{UnitsProbeState, run};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkernel::units::Dimensions;
use rustycad_tkernel::units_api::{SystemUnits, UnitsApi};
use rustycad_tkernel::units_methods::{LengthUnit, UnitsMethods};
use std::str::SplitWhitespace;

pub fn evaluate(
    op: &str,
    input: &mut SplitWhitespace<'_>,
    state: &mut UnitsProbeState,
) -> Option<String> {
    Some(match op {
        "units_api" => api(state, input),
        "units_methods" => methods(state, input),
        _ => return None,
    })
}

fn system_code(system: SystemUnits) -> i32 {
    match system {
        SystemUnits::Default => 0,
        SystemUnits::Si => 1,
        SystemUnits::Mdtv => 2,
    }
}

fn system_of(code: i32) -> SystemUnits {
    match code {
        0 => SystemUnits::Default,
        1 => SystemUnits::Si,
        _ => SystemUnits::Mdtv,
    }
}

type Single = fn(&mut UnitsApi, f64, &[u8]) -> Result<f64, Failure>;

fn single(op: &str) -> Option<Single> {
    Some(match op {
        "c2ls" => UnitsApi::current_to_ls,
        "c2si" => UnitsApi::current_to_si,
        "cfls" => UnitsApi::current_from_ls,
        "cfsi" => UnitsApi::current_from_si,
        "a2ls" => UnitsApi::any_to_ls,
        "a2si" => UnitsApi::any_to_si,
        "afls" => UnitsApi::any_from_ls,
        "afsi" => UnitsApi::any_from_si,
        "ls2si" => UnitsApi::ls_to_si,
        "si2ls" => UnitsApi::si_to_ls,
        _ => return None,
    })
}

fn values(
    api: &mut UnitsApi,
    op: &str,
    value: f64,
    a: &[u8],
    b: &[u8],
    res: &mut String,
) -> Result<(), Failure> {
    if let Some(convert) = single(op) {
        res.push_str(&bits(convert(api, value, a)?).to_string());
        return Ok(());
    }
    if op == "a2lsd" || op == "a2sid" {
        let mut dimensions = Dimensions::a_less();
        let result = if op == "a2lsd" {
            api.any_to_ls_dimensions(value, a, &mut dimensions)?
        } else {
            api.any_to_si_dimensions(value, a, &mut dimensions)?
        };
        res.push_str(&format!("{} {}", bits(result), dims(&dimensions)));
        return Ok(());
    }
    let result = match op {
        "c2any" => api.current_to_any(value, a, b)?,
        "cfany" => api.current_from_any(value, a, b)?,
        _ => api.any_to_any(value, a, b)?,
    };
    res.push_str(&bits(result).to_string());
    Ok(())
}

fn basic_dimensions() -> String {
    let all: [Handle<Dimensions>; 10] = [
        UnitsApi::dimension_less(),
        UnitsApi::dimension_mass(),
        UnitsApi::dimension_length(),
        UnitsApi::dimension_time(),
        UnitsApi::dimension_electric_current(),
        UnitsApi::dimension_thermodynamic_temperature(),
        UnitsApi::dimension_amount_of_substance(),
        UnitsApi::dimension_luminous_intensity(),
        UnitsApi::dimension_plane_angle(),
        UnitsApi::dimension_solid_angle(),
    ];
    all.iter().map(|d| format!("{} ", dims(d))).collect()
}

fn api(state: &mut UnitsProbeState, input: &mut SplitWhitespace<'_>) -> String {
    let op = word(input);
    let stateful = matches!(
        op.as_str(),
        "sls" | "ls" | "reload" | "dimbasic" | "save" | "scu" | "cu" | "check" | "dim"
    );
    let code = if op == "sls" { real(input) as i32 } else { 0 };
    let named = matches!(op.as_str(), "scu" | "cu" | "check" | "dim");
    let first = if named { arg(input) } else { Vec::new() };
    let second = if op == "scu" || op == "check" {
        arg(input)
    } else {
        Vec::new()
    };
    let value = if stateful { 0.0 } else { real(input) };
    let a = if stateful { Vec::new() } else { arg(input) };
    let b = if matches!(op.as_str(), "c2any" | "cfany" | "a2a") {
        arg(input)
    } else {
        Vec::new()
    };
    run(state, |state, res| {
        let api = &mut state.api;
        let outcome = match op.as_str() {
            "sls" => api.set_local_system(system_of(code)),
            "reload" => api.reload(),
            "save" => api.save().map(|_| ()),
            "scu" => api.set_current_unit(&first, &second),
            "cu" => api
                .current_unit(&first)
                .map(|unit| res.push_str(&hex(unit.as_bytes()))),
            "check" => api
                .check(&first, &second)
                .map(|ok| res.push_str(&u8::from(ok).to_string())),
            "dim" => api
                .dimensions(Some(&first))
                .map(|d| res.push_str(&dims(&d))),
            "dimbasic" => {
                res.push_str(&basic_dimensions());
                Ok(())
            }
            "ls" => Ok(()),
            _ => values(api, &op, value, &a, &b, res),
        };
        outcome?;
        res.push_str(&format!(" ls={}", system_code(api.local_system())));
        Ok(())
    })
}

fn unit(code: f64) -> LengthUnit {
    match code as i32 {
        1 => LengthUnit::Inch,
        2 => LengthUnit::Millimeter,
        4 => LengthUnit::Foot,
        5 => LengthUnit::Mile,
        6 => LengthUnit::Meter,
        7 => LengthUnit::Kilometer,
        8 => LengthUnit::Mil,
        9 => LengthUnit::Micron,
        10 => LengthUnit::Centimeter,
        11 => LengthUnit::Microinch,
        _ => LengthUnit::Undefined,
    }
}

fn methods(state: &mut UnitsProbeState, input: &mut SplitWhitespace<'_>) -> String {
    let op = word(input);
    let first = if op == "fromstr" { 0.0 } else { real(input) };
    let text = if op == "fromstr" {
        arg(input)
    } else {
        Vec::new()
    };
    let two = matches!(
        op.as_str(),
        "scale" | "byfactor" | "dumpf" | "fromstr" | "casset"
    );
    let second = if two { real(input) } else { 0.0 };
    run(state, |state, res| {
        let m = &state.methods;
        let text_out = match op.as_str() {
            "factor" => bits(UnitsMethods::get_length_factor_value(first as i32)).to_string(),
            "scale" => bits(UnitsMethods::get_length_unit_scale(
                unit(first),
                unit(second),
            ))
            .to_string(),
            "byfactor" => (UnitsMethods::get_length_unit_by_factor_value(first, unit(second))
                as i32)
                .to_string(),
            "dump" => hex(UnitsMethods::dump_length_unit(unit(first)).as_bytes()),
            "dumpf" => hex(UnitsMethods::dump_length_unit_factor(first, unit(second)).as_bytes()),
            "fromstr" => {
                (UnitsMethods::length_unit_from_string(&text, second != 0.0) as i32).to_string()
            }
            "casget" => bits(m.get_cascade_length_unit(unit(first))).to_string(),
            "casset" => {
                m.set_cascade_length_unit(first, unit(second));
                String::new()
            }
            _ => {
                m.set_cascade_length_unit_code(first as i32);
                String::new()
            }
        };
        res.push_str(&text_out);
        res.push_str(&format!(
            " cas={}",
            bits(m.get_cascade_length_unit(LengthUnit::Millimeter))
        ));
        Ok(())
    })
}
