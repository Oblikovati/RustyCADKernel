//! Package functions, sentences and conversions (twin of `tkernel_units_probe_core.cxx`).
use super::common::{arg, bits, dims, flag, hex, real, token, tokens, word};
use super::{UnitsProbeState, run};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkernel::units::{
    Dimensions, Lexicon, LexiconHandle, MathSentence, Sentence, UnitSentence, UnitsDictionary,
    UnitsLexicon, operators,
};
use std::str::SplitWhitespace;

pub fn evaluate(
    op: &str,
    input: &mut SplitWhitespace<'_>,
    state: &mut UnitsProbeState,
) -> Option<String> {
    Some(match op {
        "units_sentence" => sentence(state, "unit", input),
        "units_math" => sentence(state, "math", input),
        "units_raw" => {
            let kind = word(input);
            sentence(state, &kind, input)
        }
        "units_tosi" => to_si(state, input),
        "units_convert" => convert(state, input),
        "units_quantity" => quantity(state, input),
        "units_first" => first(state, input),
        "units_dims" => dimensions(state, input),
        "units_basic" => basic(state),
        "units_lexicon" => lexicon(state, input),
        "units_lexicon_add" => lexicon_add(state, input),
        "units_dict" => dictionary(state, input),
        _ => return None,
    })
}

fn report(res: &mut String, sentence: &mut Sentence) -> Result<(), Failure> {
    res.push_str(&format!(
        "{} {}",
        flag(sentence.is_done()?),
        tokens(&sentence.sequence())?
    ));
    res.push_str(" = ");
    let evaluated = sentence.evaluate()?;
    res.push_str(&token(&evaluated));
    let left = sentence
        .sequence()
        .get()
        .unwrap()
        .sequence(|items| items.len());
    res.push_str(&format!(" left {left}"));
    Ok(())
}

fn sentence(state: &mut UnitsProbeState, kind: &str, input: &mut SplitWhitespace<'_>) -> String {
    let expression = arg(input);
    run(state, |state, res| match kind {
        "unit" => {
            let mut sentence = UnitSentence::new(&state.units, &expression)?;
            report(res, &mut sentence)
        }
        "math" => {
            let mut sentence = MathSentence::new(&state.units, &expression)?;
            report(res, &mut sentence)
        }
        _ => {
            let lexicon = match kind {
                "formula" => state.units.lexicon_formula()?,
                "units" => state.units.lexicon_units(false)?,
                _ => Handle::new(Lexicon::new()),
            };
            let mut sentence = Sentence::new(&lexicon, &expression)?;
            sentence.set_constants()?;
            report(res, &mut sentence)
        }
    })
}

fn to_si(state: &mut UnitsProbeState, input: &mut SplitWhitespace<'_>) -> String {
    let value = real(input);
    let unit = arg(input);
    run(state, |state, res| {
        let units = &state.units;
        let (mut to, mut from) = (Handle::default(), Handle::default());
        res.push_str(&format!(
            "{} ",
            bits(units.to_si_dimensions(value, &unit, &mut to)?)
        ));
        res.push_str(&dims(&to));
        res.push_str(&format!(
            " {} ",
            bits(units.from_si_dimensions(value, &unit, &mut from)?)
        ));
        res.push_str(&dims(&from));
        let plain = (units.to_si(value, &unit)?, units.from_si(value, &unit)?);
        res.push_str(&format!(" {} {}", bits(plain.0), bits(plain.1)));
        Ok(())
    })
}

fn convert(state: &mut UnitsProbeState, input: &mut SplitWhitespace<'_>) -> String {
    let value = real(input);
    let (from, to) = (arg(input), arg(input));
    run(state, |state, res| {
        res.push_str(&bits(state.units.convert(value, &from, &to)?).to_string());
        Ok(())
    })
}

fn quantity(state: &mut UnitsProbeState, input: &mut SplitWhitespace<'_>) -> String {
    let name = arg(input);
    run(state, |state, res| {
        let units = &state.units;
        let quantity = units.quantity(&name)?;
        match quantity.get() {
            None => res.push_str("null"),
            Some(q) => {
                let count = q.sequence().get().unwrap().sequence(|items| items.len());
                res.push_str(&format!(
                    "{} {count} {}",
                    hex(q.name().as_bytes()),
                    dims(&q.dimensions())
                ));
            }
        }
        let dictionary = units.dictionary_of_units(false)?;
        let active = dictionary
            .get()
            .unwrap()
            .active_unit(&name, &mut state.capture.writer())?;
        res.push_str(&format!(" active {}", hex(active.as_bytes())));
        let first = units.first_quantity(&name)?;
        let first = first.map_or("null".into(), |q| hex(q.as_bytes()));
        res.push_str(&format!(" first {first} dims "));
        res.push_str(&dims(&units.dimensions(Some(&name))?));
        Ok(())
    })
}

fn first(state: &mut UnitsProbeState, input: &mut SplitWhitespace<'_>) -> String {
    let unit = arg(input);
    run(state, |state, res| {
        let first = state.units.first_quantity(&unit)?;
        res.push_str(&first.map_or("null".into(), |q| hex(q.as_bytes())));
        Ok(())
    })
}

pub fn read_dims(input: &mut SplitWhitespace<'_>) -> Handle<Dimensions> {
    let values: Vec<f64> = (0..9).map(|_| real(input)).collect();
    Dimensions::handle(values.try_into().unwrap())
}

fn dimensions(state: &mut UnitsProbeState, input: &mut SplitWhitespace<'_>) -> String {
    let (left, right) = (read_dims(input), read_dims(input));
    let power = real(input);
    run(state, |state, res| {
        res.push_str(&format!(
            "{} ",
            dims(&operators::dimensions_multiply(&left, &right)?)
        ));
        res.push_str(&format!(
            "{} ",
            dims(&operators::dimensions_divide(&left, &right)?)
        ));
        res.push_str(&dims(&operators::dimensions_pow(&left, power)?));
        let object = left.get().unwrap();
        let equal = (object.is_equal(&right)?, object.is_not_equal(&right)?);
        res.push_str(&format!(" {}{} ", flag(equal.0), flag(equal.1)));
        let quantity = object.quantity(&state.units)?;
        res.push_str(&quantity.map_or("null".into(), |q| hex(q.as_bytes())));
        object.dump(1, &mut state.capture.writer()).unwrap();
        Ok(())
    })
}

fn basic(state: &mut UnitsProbeState) -> String {
    run(state, |state, res| {
        let units = &state.units;
        let all = [
            Dimensions::a_less(),
            Dimensions::a_mass(),
            Dimensions::a_length(),
            Dimensions::a_time(),
            Dimensions::a_electric_current(),
            Dimensions::a_thermodynamic_temperature(),
            Dimensions::a_amount_of_substance(),
            Dimensions::a_luminous_intensity(),
            Dimensions::a_plane_angle(),
            Dimensions::a_solid_angle(),
            units.null_dimensions(),
            units.dimensions(None)?,
        ];
        for dimensions in &all {
            res.push_str(&format!("{} ", dims(dimensions)));
        }
        let same = |a: &Handle<Dimensions>, b: &Handle<Dimensions>| {
            flag(std::ptr::eq(a.as_ptr(), b.as_ptr()))
        };
        res.push_str(&format!(
            "{}{}{}",
            same(&Dimensions::a_less(), &Dimensions::a_less()),
            same(&units.null_dimensions(), &units.null_dimensions()),
            same(&units.dimensions(None)?, &Dimensions::a_less())
        ));
        Ok(())
    })
}

fn lexicon(state: &mut UnitsProbeState, input: &mut SplitWhitespace<'_>) -> String {
    let kind = word(input);
    run(state, |state, res| {
        let lexicon = if kind == "formula" {
            state.units.lexicon_formula()?
        } else {
            state.units.lexicon_units(false)?
        };
        res.push_str(&tokens(&lexicon.get().unwrap().sequence())?);
        lexicon.dump(&mut state.capture.writer())
    })
}

fn lexicon_add(state: &mut UnitsProbeState, input: &mut SplitWhitespace<'_>) -> String {
    let (word, mean) = (arg(input), arg(input));
    let value = real(input);
    run(state, |state, res| {
        let lexicon = Handle::new(Lexicon::new());
        let object = lexicon.get().unwrap();
        res.push_str(&format!("{} ", flag(object.sequence().is_null())));
        object.creates()?;
        object.add_token(&word, &mean, value, &mut state.capture.writer())?;
        object.add_token(&word, &mean, value, &mut state.capture.writer())?;
        res.push_str(&tokens(&object.sequence())?);
        let units_lexicon = Handle::new(UnitsLexicon::new());
        units_lexicon.get().unwrap().creates(false, &state.units)?;
        let count = units_lexicon
            .get()
            .unwrap()
            .lexicon()
            .sequence()
            .get()
            .unwrap()
            .sequence(|s| s.len());
        res.push_str(&format!(" {count}"));
        units_lexicon
            .cast::<Lexicon>()
            .dump(&mut state.capture.writer())
    })
}

fn dictionary(state: &mut UnitsProbeState, input: &mut SplitWhitespace<'_>) -> String {
    let kind = word(input);
    let dims_argument = (kind == "dumpdims").then(|| read_dims(input));
    run(state, |state, res| {
        let dictionary = state.units.dictionary_of_units(false)?;
        let object = dictionary.get().unwrap();
        let mut out = state.capture.writer();
        match kind.as_str() {
            "dump1" => object.dump(1, &mut out)?,
            "dump2" => object.dump(2, &mut out)?,
            "dumpdims" => object.dump_dimensions(dims_argument.as_ref().unwrap(), &mut out)?,
            "recreate" => {
                let again = state.units.dictionary_of_units(true)?;
                res.push_str(&flag(std::ptr::eq(again.as_ptr(), dictionary.as_ptr())).to_string());
            }
            "fresh" => {
                let fresh = UnitsDictionary::new();
                res.push_str(&format!("{} ", flag(fresh.sequence().is_null())));
                fresh.creates(&state.units)?;
                res.push_str(
                    &fresh
                        .sequence()
                        .get()
                        .unwrap()
                        .sequence(|s| s.len())
                        .to_string(),
                );
            }
            _ => {}
        }
        let count = object.sequence().get().unwrap().sequence(|s| s.len());
        res.push_str(&format!(" n={count}"));
        Ok(())
    })
}
