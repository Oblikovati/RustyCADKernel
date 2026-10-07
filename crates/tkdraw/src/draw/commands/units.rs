//! `Draw_UnitCommands.cxx`: unit parsing and conversion. The C++ uses the process-wide
//! `Units`/`UnitsAPI` state; here the session owns one `UnitsApi` (and its `Units`).
use crate::draw::{DrawInterpretor, DrawSession};
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::units::UnitSentence;
use rustycad_tkernel::units_api::SystemUnits;

/// `unitparsing string [nbiter]`: analyses a unit sentence `nbiter` times and prints the
/// word of its evaluated token.
pub(super) fn parsing(
    session: &DrawSession,
    di: &DrawInterpretor,
    a: &[&str],
) -> Result<i32, Failure> {
    if a.len() < 2 {
        di.append(format!("Usage : {} string [nbiter]\n", a[0]))?;
        return Ok(1);
    }
    let iterations = a.get(2).map_or(1, |text| session.atoi(text));
    let mut api = session.units_api.borrow_mut();
    api.set_local_system(SystemUnits::Si)?;
    let mut sentence = UnitSentence::new(api.units(), a[1].as_bytes())?;
    if !sentence.is_done()? {
        di.append("can not create a sentence\n")?;
        return Ok(1);
    }
    for _ in 1..=iterations {
        sentence.analyse()?;
        let _ = sentence.sequence();
    }
    let token = sentence.evaluate()?;
    let token = token.get().ok_or_else(|| {
        Failure::with_message(
            "Standard_NullObject",
            "unitparsing: the sentence evaluates to no token",
        )
    })?;
    di.append(format!(
        "Token word : {}\n",
        String::from_utf8_lossy(token.word().as_bytes())
    ))?;
    Ok(0)
}

/// `unitsdico`: prints the dictionary of units (level 2) to the console.
pub(super) fn unitsdico(
    session: &DrawSession,
    _di: &DrawInterpretor,
    _a: &[&str],
) -> Result<i32, Failure> {
    let mut api = session.units_api.borrow_mut();
    api.set_local_system(SystemUnits::Si)?;
    let dictionary = api.units().dictionary_of_units(false)?;
    let mut text: Vec<u8> = Vec::new();
    if let Some(dictionary) = dictionary.get() {
        dictionary.dump(2, &mut text)?;
    }
    session.console().write_out(&String::from_utf8_lossy(&text));
    Ok(0)
}

/// `unitconvtoSI value unit`: the value in SI units.
pub(super) fn convert_to_si(
    session: &DrawSession,
    di: &DrawInterpretor,
    a: &[&str],
) -> Result<i32, Failure> {
    if a.len() < 3 {
        di.append("Invalid number of parameter, use: unitconvtoSI real string\n")?;
        return Ok(1);
    }
    let value = session.atof(a[1]);
    let result = session
        .units_api
        .borrow_mut()
        .any_to_si(value, a[2].as_bytes())?;
    di.append("result: ")?.append(result)?.append("\n")?;
    Ok(0)
}

/// `unitconvtoMDTV value unit`: the value in the MDTV local system.
pub(super) fn convert_to_mdtv(
    session: &DrawSession,
    di: &DrawInterpretor,
    a: &[&str],
) -> Result<i32, Failure> {
    if a.len() < 3 {
        di.append("Invalid number of parameter, use: unitconvtoMDTV real string\n")?;
        return Ok(1);
    }
    let value = session.atof(a[1]);
    let mut api = session.units_api.borrow_mut();
    api.set_local_system(SystemUnits::Mdtv)?;
    let result = api.any_to_ls(value, a[2].as_bytes())?;
    di.append("result: ")?.append(result)?.append("\n")?;
    Ok(0)
}

/// `unit value from to`: `Units::Convert`.
pub(super) fn unit(
    session: &DrawSession,
    di: &DrawInterpretor,
    a: &[&str],
) -> Result<i32, Failure> {
    if a.len() != 4 {
        return Ok(1);
    }
    let value = session.atof(a[1]);
    let result =
        session
            .units_api
            .borrow()
            .units()
            .convert(value, a[2].as_bytes(), a[3].as_bytes())?;
    di.append(result)?;
    Ok(0)
}
