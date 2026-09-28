//! NCollection lists probe operations (twin of `tkernel_ncollection_lists_probe.cxx`).
//!
//! A request scripts commands on one list or sequence of ints and one cursor; each command
//! prints its result token or `throw <exception class>`, and the final state follows `|`.
use rustycad_tkernel::{
    ncollection::{HSequence, List, ListCursor, Sequence, SequenceCursor},
    standard::{Failure, Handle},
};
use std::str::SplitWhitespace;

/// The output of a lists operation, or `None` when `op` belongs to another family.
pub fn evaluate(op: &str, input: &mut SplitWhitespace<'_>) -> Option<String> {
    match op {
        "ncoll_lists_list" => Some(list_script(input)),
        "ncoll_lists_seq" => Some(sequence_script(input)),
        "ncoll_lists_hseq" => Some(hsequence(input)),
        _ => None,
    }
}

type Outcome = Result<String, Failure>;

fn integer<N: std::str::FromStr>(input: &mut SplitWhitespace<'_>) -> N {
    let token = input.next().expect("lists probe integer");
    token
        .parse()
        .unwrap_or_else(|_| panic!("lists probe token {token:?}; expected an integer"))
}
fn read_values(input: &mut SplitWhitespace<'_>) -> Vec<i32> {
    let count: usize = integer(input);
    (0..count).map(|_| integer(input)).collect()
}
fn bracketed<'a>(values: impl Iterator<Item = &'a i32>) -> String {
    let body: String = values.map(|value| format!(" {value}")).collect();
    format!("[{body} ]")
}
fn token(outcome: Outcome) -> String {
    outcome.unwrap_or_else(|failure| format!("throw {}", failure.exception_type()))
}
fn cursor_text(value: Option<&i32>) -> String {
    value.map_or_else(|| " it=end".to_owned(), |value| format!(" it={value}"))
}

fn list_script(input: &mut SplitWhitespace<'_>) -> String {
    let mut list: List<i32> = read_values(input).into_iter().collect();
    let mut cursor = list.cursor();
    let mut out = String::new();
    while let Some(command) = input.next() {
        let result = list_command(command, input, &mut list, &mut cursor);
        out.push_str(&format!("{command}={} ", token(result)));
    }
    let state = bracketed(list.iter());
    let at = cursor_text(cursor.value(&list));
    format!("{out}|{state} n={}{at}", list.len())
}
fn list_command(
    command: &str,
    input: &mut SplitWhitespace<'_>,
    list: &mut List<i32>,
    cursor: &mut ListCursor,
) -> Outcome {
    match command {
        "ap" | "pp" | "aa" | "ib" | "ia" | "rx" | "ct" => {
            list_value_command(command, integer(input), list, cursor)
        }
        "lb" | "la" | "al" | "pl" => list_splice(command, read_values(input), list, cursor),
        _ => list_position_command(command, list, cursor),
    }
}
fn list_value_command(
    command: &str,
    value: i32,
    list: &mut List<i32>,
    cursor: &mut ListCursor,
) -> Outcome {
    let text = match command {
        "ap" => list.append(value).to_string(),
        "pp" => {
            let text = list.prepend(value).to_string();
            cursor.init(list);
            text
        }
        "aa" => {
            list.append_at(value, cursor);
            ".".to_owned()
        }
        "ib" => list.insert_before(value, cursor)?.to_string(),
        "ia" => list.insert_after(value, cursor)?.to_string(),
        "rx" => {
            let removed = u8::from(list.remove_value(&value));
            cursor.init(list);
            removed.to_string()
        }
        _ => u8::from(list.contains(&value)).to_string(),
    };
    Ok(text)
}
fn list_position_command(command: &str, list: &mut List<i32>, cursor: &mut ListCursor) -> Outcome {
    match command {
        "rf" => {
            list.remove_first()?;
            cursor.init(list);
        }
        "rm" => list.remove(cursor)?,
        "nx" => cursor.next(list),
        "it" => cursor.init(list),
        "rv" => {
            list.reverse();
            cursor.init(list);
        }
        "fl" => return list_ends(list),
        _ => panic!("unknown lists probe command {command}"),
    }
    Ok(".".to_owned())
}
fn list_ends(list: &List<i32>) -> Outcome {
    let missing = || Failure::with_type("Standard_NoSuchObject", "NCollection_List::First", "");
    let first = list.first().ok_or_else(missing)?;
    Ok(format!("{first},{}", list.last().ok_or_else(missing)?))
}
fn list_splice(
    command: &str,
    values: Vec<i32>,
    list: &mut List<i32>,
    cursor: &mut ListCursor,
) -> Outcome {
    let mut source: List<i32> = values.into_iter().collect();
    match command {
        "lb" => list.insert_list_before(&mut source, cursor)?,
        "la" => list.insert_list_after(&mut source, cursor)?,
        "al" => list.append_list(&mut source),
        _ => {
            list.prepend_list(&mut source);
            cursor.init(list);
        }
    }
    Ok(source.extent().to_string())
}

fn sequence_script(input: &mut SplitWhitespace<'_>) -> String {
    let mut sequence: Sequence<i32> = read_values(input).into_iter().collect();
    let mut cursor = sequence.cursor();
    let mut out = String::new();
    while let Some(command) = input.next() {
        let result = sequence_command(command, input, &mut sequence, &mut cursor);
        out.push_str(&format!("{command}={} ", token(result)));
    }
    let state = bracketed(sequence.iter());
    let at = cursor_text(cursor.value(&sequence));
    format!("{out}|{state} n={}{at}", sequence.len())
}
fn sequence_command(
    command: &str,
    input: &mut SplitWhitespace<'_>,
    sequence: &mut Sequence<i32>,
    cursor: &mut SequenceCursor,
) -> Outcome {
    match command {
        "ap" | "pp" | "ic" | "ec" => {
            sequence_value_command(command, integer(input), sequence, cursor)
        }
        "ib" | "ia" | "ea" | "eb" | "sv" => {
            let index = integer(input);
            sequence_indexed_value(command, index, integer(input), sequence)
        }
        "rm" | "vl" | "at" | "sp" => {
            sequence_index_command(command, integer(input), sequence, cursor)
        }
        "rr" | "ex" => {
            let first = integer(input);
            sequence_pair_command(command, first, integer(input), sequence, cursor)
        }
        "qa" | "qb" => {
            let index = integer(input);
            sequence_splice(command, index, read_values(input), sequence)
        }
        "sa" | "sq" => sequence_splice(command, 0, read_values(input), sequence),
        _ => sequence_cursor_command(command, sequence, cursor),
    }
}
fn sequence_value_command(
    command: &str,
    value: i32,
    sequence: &mut Sequence<i32>,
    cursor: &SequenceCursor,
) -> Outcome {
    match command {
        "ap" => drop(sequence.append(value)),
        "pp" => drop(sequence.prepend(value)),
        "ic" => drop(sequence.insert_after_cursor(cursor, value)?),
        _ => return Ok(sequence.emplace_after_cursor(cursor, || value)?.to_string()),
    }
    Ok(".".to_owned())
}
fn sequence_indexed_value(
    command: &str,
    index: usize,
    value: i32,
    sequence: &mut Sequence<i32>,
) -> Outcome {
    match command {
        "ib" => drop(sequence.insert_before(index, value)?),
        "ia" => drop(sequence.insert_after(index, value)?),
        "ea" => return Ok(sequence.emplace_after(index, || value)?.to_string()),
        "eb" => return Ok(sequence.emplace_before(index, || value)?.to_string()),
        _ => sequence.set_value(index, value)?,
    }
    Ok(".".to_owned())
}
fn sequence_index_command(
    command: &str,
    index: usize,
    sequence: &mut Sequence<i32>,
    cursor: &mut SequenceCursor,
) -> Outcome {
    match command {
        "vl" => return Ok(sequence.value(index)?.to_string()),
        "at" => return Ok(sequence.at(index)?.to_string()),
        "sp" => return sequence_split(index, sequence, cursor),
        _ => sequence.remove(index)?,
    }
    cursor.init(sequence, true);
    Ok(".".to_owned())
}
fn sequence_split(
    index: usize,
    sequence: &mut Sequence<i32>,
    cursor: &mut SequenceCursor,
) -> Outcome {
    let mut tail: Sequence<i32> = [-7].into_iter().collect();
    if let Err(failure) = sequence.split(index, &mut tail) {
        return Ok(format!(
            "tail{} throw {}",
            tail.len(),
            failure.exception_type()
        ));
    }
    cursor.init(sequence, true);
    Ok(bracketed(tail.iter()))
}
fn sequence_pair_command(
    command: &str,
    first: usize,
    second: usize,
    sequence: &mut Sequence<i32>,
    cursor: &mut SequenceCursor,
) -> Outcome {
    if command == "ex" {
        sequence.exchange(first, second)?;
        return Ok(".".to_owned());
    }
    sequence.remove_range(first, second)?;
    cursor.init(sequence, true);
    Ok(".".to_owned())
}
fn sequence_splice(
    command: &str,
    index: usize,
    values: Vec<i32>,
    sequence: &mut Sequence<i32>,
) -> Outcome {
    let mut source: Sequence<i32> = values.into_iter().collect();
    match command {
        "qa" => sequence.insert_sequence_after(index, &mut source)?,
        "qb" => sequence.insert_sequence_before(index, &mut source)?,
        "sa" => sequence.append_sequence(&mut source),
        _ => sequence.prepend_sequence(&mut source),
    }
    Ok(source.length().to_string())
}
fn sequence_cursor_command(
    command: &str,
    sequence: &mut Sequence<i32>,
    cursor: &mut SequenceCursor,
) -> Outcome {
    match command {
        "rv" => sequence.reverse(),
        "fl" => return sequence_ends(sequence),
        "i0" => cursor.init(sequence, true),
        "i1" => cursor.init(sequence, false),
        "nx" => cursor.next(sequence),
        "pv" => cursor.previous(sequence),
        "ri" => sequence.remove_cursor(cursor)?,
        _ => panic!("unknown lists probe command {command}"),
    }
    Ok(".".to_owned())
}
fn sequence_ends(sequence: &Sequence<i32>) -> Outcome {
    let missing = || Failure::with_type("Standard_NoSuchObject", "NCollection_Sequence::First", "");
    let first = sequence.first().ok_or_else(missing)?;
    Ok(format!("{first},{}", sequence.last().ok_or_else(missing)?))
}

fn hsequence(input: &mut SplitWhitespace<'_>) -> String {
    let first: Sequence<i32> = read_values(input).into_iter().collect();
    let second: Sequence<i32> = read_values(input).into_iter().collect();
    let target = Handle::new(HSequence::from_sequence(&first));
    let source = Handle::new(HSequence::from_sequence(&second));
    let items = target.get().expect("fresh handle");
    items.append(42);
    items.append_handle(&source).expect("non-null handle");
    items.append_handle(&target).expect("non-null handle");
    let mut more: Sequence<i32> = [-1].into_iter().collect();
    items.append_sequence(&mut more);
    let state = items.sequence(|values| bracketed(values.iter()));
    let moved = source
        .get()
        .expect("fresh handle")
        .sequence(Sequence::length);
    let name = target.dynamic_type().expect("fresh handle").name();
    format!("{state} source={moved} more={} {name}", more.length())
}
