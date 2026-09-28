//! Tokens, measurements, units, explorers and units systems (twin of
//! `tkernel_units_probe_objects.cxx` and the system part of `tkernel_units_probe_api.cxx`).
use super::common::{arg, bits, flag, hex, real, token, word};
use super::{UnitsProbeState, run};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkernel::units::{
    Dimensions, Explorer, Measurement, Quantity, ShiftedToken, ShiftedUnit, Token, TokenHandle,
    Unit, UnitHandle, UnitSentence, Units, UnitsSystem, operators,
};
use std::str::SplitWhitespace;

mod system;

pub fn evaluate(
    op: &str,
    input: &mut SplitWhitespace<'_>,
    state: &mut UnitsProbeState,
) -> Option<String> {
    Some(match op {
        "units_token" => token_op(state, input),
        "units_measure" => measure(state, input),
        "units_unit" => unit(state, input),
        "units_explore" => explore(state, input),
        "units_system" => system::system(state, input),
        _ => return None,
    })
}

fn evaluated(units: &Units, expression: &[u8]) -> Result<Handle<Token>, Failure> {
    let mut sentence = UnitSentence::new(units, expression)?;
    if sentence.is_done()? {
        sentence.evaluate()
    } else {
        Ok(Handle::default())
    }
}

fn arith(
    res: &mut String,
    a: &Handle<Token>,
    b: &Handle<Token>,
    value: f64,
) -> Result<(), Failure> {
    let object = a.get().unwrap();
    let results = [
        object.add(b)?,
        object.subtract(b)?,
        object.multiply(b)?,
        a.divide(b)?,
        object.power_token(b)?,
        object.power(value)?,
        object.add_integer(3),
        operators::token_add(a, b)?,
        operators::token_subtract(a, b)?,
        operators::token_multiply(a, b)?,
        operators::token_divide(a, b)?,
        operators::token_pow_token(a, b)?,
        operators::token_pow(a, value)?,
        operators::token_add_integer(a, 7)?,
    ];
    for result in &results {
        let same = if std::ptr::eq(result.as_ptr(), a.as_ptr()) {
            " same"
        } else {
            ""
        };
        res.push_str(&format!("{}{same} | ", token(result)));
    }
    res.push_str(&format!(
        "{} {} ",
        bits(a.multiplied(value)?),
        bits(a.divided(value)?)
    ));
    Ok(())
}

fn compare(
    res: &mut String,
    a: &Handle<Token>,
    b: &Handle<Token>,
    text: &[u8],
) -> Result<(), Failure> {
    let o = a.get().unwrap();
    let flags = [
        o.is_equal(text),
        o.is_not_equal(text),
        o.is_less_or_equal(text),
        o.is_greater(text),
        o.is_greater_token(b)?,
        o.is_greater_or_equal(b)?,
        o.is_equal_token(b)?,
        o.is_not_equal_token(b)?,
        operators::token_equals(a, text)?,
        operators::token_not_equals(a, text)?,
        operators::token_less_or_equal(a, text)?,
        operators::token_greater(a, text)?,
        operators::token_greater_token(a, b)?,
        operators::token_greater_or_equal(a, b)?,
    ];
    let flags: String = flags.iter().map(|f| flag(*f).to_string()).collect();
    res.push_str(&format!("{flags} {} ", o.length()));
    Ok(())
}

fn copies(
    state: &UnitsProbeState,
    res: &mut String,
    a: &Handle<Token>,
    mean: &[u8],
    value: f64,
) -> Result<(), Failure> {
    let created = a.creates()?;
    let copy = Token::from_token(a)?;
    let object = a.get().unwrap();
    res.push_str(&token(&created));
    let shares = std::ptr::eq(copy.dimensions().as_ptr(), object.dimensions().as_ptr());
    let created_shares = std::ptr::eq(
        created.get().unwrap().dimensions().as_ptr(),
        object.dimensions().as_ptr(),
    );
    res.push_str(&format!(" {}{} ", flag(shares), flag(created_shares)));
    let mut out = state.capture.writer();
    created.get().unwrap().update(mean, &mut out).unwrap();
    created.get().unwrap().update(mean, &mut out).unwrap();
    res.push_str(&token(&created));
    let shifted: Handle<Token> = Handle::new(ShiftedToken::new(
        b"sh",
        mean,
        value,
        2.5,
        &object.dimensions(),
    ))
    .cast();
    res.push_str(&format!(
        " {} {} ",
        bits(shifted.multiplied(value)?),
        bits(shifted.divided(value)?)
    ));
    res.push_str(&token(&shifted.creates()?));
    shifted.dump(1, 1, &mut out)?;
    a.dump(0, 1, &mut out)
}

fn constructors(
    state: &UnitsProbeState,
    res: &mut String,
    text: &[u8],
    value: f64,
) -> Result<(), Failure> {
    let tokens = [
        Handle::new(Token::new()),
        Handle::new(Token::with_word(text)),
        Handle::new(Token::with_mean(text, b"U")),
        Handle::new(Token::with_value(text, b"M", value)),
        Handle::new(Token::with_dimensions(
            text,
            b"K",
            value,
            &Handle::default(),
        )),
    ];
    for item in &tokens {
        res.push_str(&format!("{} | ", token(item)));
    }
    let first = tokens[0].get().unwrap();
    first.set_word(b"w");
    first.set_mean(b"m");
    first.set_value(value);
    first.set_dimensions(&Dimensions::a_length());
    let same = std::ptr::eq(first.dimensions().as_ptr(), Dimensions::a_length().as_ptr());
    res.push_str(&format!("{} {}", token(&tokens[0]), flag(same)));
    first.set_dimensions(&Handle::default());
    res.push_str(&format!(" {}", token(&tokens[0])));
    tokens[1].dump(1, 0, &mut state.capture.writer())
}

fn token_op(state: &mut UnitsProbeState, input: &mut SplitWhitespace<'_>) -> String {
    let (left, right) = (arg(input), arg(input));
    let value = real(input);
    let (text, mean) = (arg(input), arg(input));
    run(state, |state, res| {
        constructors(state, res, &text, value)?;
        res.push_str(" || ");
        let (a, b) = (
            evaluated(&state.units, &left)?,
            evaluated(&state.units, &right)?,
        );
        if a.is_null() || b.is_null() {
            res.push_str("null");
            return Ok(());
        }
        compare(res, &a, &b, &text)?;
        arith(res, &a, &b, value)?;
        copies(state, res, &a, &mean, value)
    })
}

fn print_measure(res: &mut String, measure: &Measurement) {
    let text = format!(
        "{} {} {} | ",
        flag(measure.has_token()),
        bits(measure.measurement()),
        token(&measure.token())
    );
    res.push_str(&text);
}

fn measure(state: &mut UnitsProbeState, input: &mut SplitWhitespace<'_>) -> String {
    let (v1, u1) = (real(input), arg(input));
    let (v2, u2, u3) = (real(input), arg(input), arg(input));
    let exponent = real(input);
    run(state, |state, res| {
        let units = &state.units;
        let first = Measurement::with_unit(units, v1, &u1)?;
        let second = Measurement::with_unit(units, v2, &u2)?;
        for m in [
            &Measurement::new(),
            &first,
            &second,
            &first.integer(),
            &first.fractional(),
        ] {
            print_measure(res, m);
        }
        let mut converted = first.clone();
        converted.convert(units, &u3)?;
        print_measure(res, &converted);
        if !first.has_token() || !second.has_token() {
            return Ok(());
        }
        let results = [
            first.add(&second)?,
            first.subtract(&second)?,
            first.multiply(&second)?,
            first.divide(&second)?,
            (&first + &second)?,
            (&first - &second)?,
            (&first * &second)?,
            (&first / &second)?,
            first.multiply_real(exponent)?,
            first.divide_real(exponent)?,
            (&first * exponent)?,
            (&first / exponent)?,
            first.power(exponent)?,
            Measurement::with_token(v2, &first.token()),
        ];
        for m in &results {
            print_measure(res, m);
        }
        first.dump(units, &mut state.capture.writer())
    })
}

fn unit(state: &mut UnitsProbeState, input: &mut SplitWhitespace<'_>) -> String {
    let (name, symbol) = (arg(input), arg(input));
    let (value, movement) = (real(input), real(input));
    run(state, |state, res| {
        unit_body(state, res, &name, &symbol, value, movement)
    })
}

fn unit_body(
    state: &UnitsProbeState,
    res: &mut String,
    name: &[u8],
    symbol: &[u8],
    value: f64,
    movement: f64,
) -> Result<(), Failure> {
    let units = rustycad_tkernel::ncollection::HSequence::<Handle<Unit>>::default();
    let units = Handle::new(units);
    let quantity = Handle::new(Quantity::new(b"TESTQ", &Dimensions::a_length(), &units));
    let plain = Handle::new(Unit::new(name, symbol, value, &quantity));
    let o = plain.get().unwrap();
    o.symbol(b"alt");
    let flags = [
        o.is_equal(symbol)?,
        o.is_equal(b"alt")?,
        o.is_equal(b"zz")?,
        operators::unit_equals(&plain, symbol)?,
    ];
    let flags: String = flags.iter().map(|f| flag(*f).to_string()).collect();
    let count = o.symbols_sequence().get().unwrap().sequence(|s| s.len());
    let same = flag(std::ptr::eq(o.quantity().as_ptr(), quantity.as_ptr()));
    res.push_str(&format!(
        "{flags} {} {} {count} {same} ",
        hex(o.name().as_bytes()),
        bits(o.value())
    ));
    res.push_str(&token(&plain.token()?));
    let shifted = Handle::new(ShiftedUnit::new(name, symbol, value, movement, &quantity));
    let shifted_unit: Handle<Unit> = shifted.cast();
    res.push_str(&format!(
        " {} {}",
        bits(shifted.get().unwrap().movement()),
        token(&shifted_unit.token()?)
    ));
    let bare = ShiftedUnit::with_symbol(name, symbol);
    let named: Handle<Unit> = Handle::new(ShiftedUnit::with_name(name)).cast();
    bare.set_movement(movement);
    let named_moves = named.cast::<ShiftedUnit>().get().unwrap().movement();
    let named_count = named
        .get()
        .unwrap()
        .symbols_sequence()
        .get()
        .unwrap()
        .sequence(|s| s.len());
    let orphan = flag(Unit::with_symbol(name, symbol).quantity().is_null());
    let lone = Unit::with_name(name)
        .symbols_sequence()
        .get()
        .unwrap()
        .sequence(|s| s.len());
    res.push_str(&format!(
        " {} {} {named_count} {orphan} {lone}",
        bits(bare.movement()),
        bits(named_moves)
    ));
    units.get().unwrap().append(plain.clone());
    units.get().unwrap().append(shifted_unit.clone());
    o.set_value(value * 2.0);
    let mut out = state.capture.writer();
    plain.dump(0, 0, &mut out)?;
    shifted_unit.dump(0, 0, &mut out)?;
    let q = quantity.get().unwrap();
    q.dump(1, 1, &mut out)?;
    q.dump(0, 0, &mut out)?;
    let equal = [
        q.is_equal(b"TESTQ"),
        operators::quantity_equals(&quantity, b"TESTQ")?,
        q.is_equal(b"TEST"),
    ];
    let equal: String = equal.iter().map(|f| flag(*f).to_string()).collect();
    res.push_str(&format!(" {equal}"));
    named.token().map(|_| ())
}

fn walk(res: &mut String, explorer: &mut Explorer, all: bool) -> Result<(), Failure> {
    while explorer.more_quantity()? {
        res.push_str(&format!("{{{}", hex(explorer.quantity()?.as_bytes())));
        while explorer.more_unit()? {
            let active = if explorer.is_active()? { "*" } else { "" };
            res.push_str(&format!(" {}{active}", hex(explorer.unit()?.as_bytes())));
            explorer.next_unit();
        }
        res.push('}');
        if !all {
            return Ok(());
        }
        explorer.next_quantity()?;
    }
    Ok(())
}

fn explore(state: &mut UnitsProbeState, input: &mut SplitWhitespace<'_>) -> String {
    let (source, quantity, mode) = (word(input), word(input), word(input));
    let name = super::common::unhex(&quantity);
    run(state, |state, res| {
        let mut explorer = Explorer::default();
        let dictionary = state.units.dictionary_of_units(false)?;
        let dictionary = dictionary.get().unwrap();
        let system: &UnitsSystem = state.system.get().unwrap();
        match (source == "dict", quantity == "-") {
            (true, true) => explorer.init_dictionary(dictionary)?,
            (false, true) => explorer.init_system(system)?,
            (true, false) => explorer.init_dictionary_quantity(dictionary, &name)?,
            (false, false) => explorer.init_system_quantity(system, &name)?,
        }
        res.push_str(&format!("{} ", flag(explorer.more_quantity()?)));
        if mode == "head" {
            res.push_str(&hex(explorer.quantity()?.as_bytes()));
            return Ok(());
        }
        walk(res, &mut explorer, quantity == "-")
    })
}
