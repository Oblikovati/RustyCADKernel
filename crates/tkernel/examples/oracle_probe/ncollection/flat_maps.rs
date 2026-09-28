//! NCollection flat maps probe operations (twin of `tkernel_ncollection_flat_maps_probe.cxx`).
//!
//! Each request is a script of one-letter commands with integer arguments applied to one
//! map; every command prints its observable result, then the map's final state is dumped.
use rustycad_tkernel::ncollection::{FlatDataMap, FlatMap, FlatMapIter};
use rustycad_tkernel::standard::Failure;
use std::fmt::Write;
use std::str::SplitWhitespace;

mod ordered;
mod packed;

/// The output of a flat maps operation, or `None` when `op` belongs to another family.
pub fn evaluate(op: &str, input: &mut SplitWhitespace<'_>) -> Option<String> {
    let mut out = String::new();
    match op {
        "ncoll_flat_maps_set" => flat_set(input, &mut out),
        "ncoll_flat_maps_data" => flat_data(input, &mut out),
        "ncoll_flat_maps_ordered" => ordered::ordered_keys(input, &mut out),
        "ncoll_flat_maps_ordered_data" => ordered::ordered_data(input, &mut out),
        "ncoll_flat_maps_packed" => packed::packed_script(input, &mut out),
        "ncoll_flat_maps_algebra32" => packed::algebra::<i32>(input, &mut out),
        "ncoll_flat_maps_algebra64" => packed::algebra::<i64>(input, &mut out),
        _ => return None,
    }
    Some(out)
}

/// The next integer argument, `None` at the end of the request.
fn int<T: std::str::FromStr>(input: &mut SplitWhitespace<'_>) -> Option<T> {
    input.next().map(|text| {
        text.parse()
            .unwrap_or_else(|_| panic!("integer field expected, got {text:?}"))
    })
}

/// The next `command key` pair of a script.
fn command(input: &mut SplitWhitespace<'_>) -> Option<(char, i32)> {
    let name = input.next()?.chars().next()?;
    Some((name, int(input)?))
}

/// Appends a native bool the way `std::ostream` prints it (`1`/`0`).
fn flag(out: &mut String, value: bool) {
    write!(out, " {}", u8::from(value)).unwrap();
}

/// Appends `prefix`, then a value or the class of the native exception replacing it.
///
/// C++17 sequences `out << prefix << f()` left to right, so the native probe has already
/// printed the prefix when `f()` throws.
fn guarded<T: std::fmt::Display>(out: &mut String, prefix: &str, value: Result<T, Failure>) {
    out.push_str(prefix);
    match value {
        Ok(value) => write!(out, "{value}").unwrap(),
        Err(error) => write!(out, " throw {}", error.exception_type()).unwrap(),
    }
}

fn flat_set(input: &mut SplitWhitespace<'_>, out: &mut String) {
    let mut map = FlatMap::<i32>::default();
    while let Some((name, key)) = command(input) {
        flat_set_command(&mut map, name, key, out);
    }
    write!(out, " | {} {} :", map.len(), map.capacity()).unwrap();
    for key in map.iter() {
        write!(out, " {key}").unwrap();
    }
    // A default native Iterator is exhausted and equals an iterator over an empty map only.
    let end = FlatMapIter::<i32>::default();
    out.push_str(" end");
    flag(out, end.clone().next().is_some());
    flag(out, end == map.iter());
}

fn flat_set_command(map: &mut FlatMap<i32>, name: char, key: i32, out: &mut String) {
    match name {
        'a' => flag(out, map.add(key)),
        'd' => write!(out, " {}", map.added(key)).unwrap(),
        'e' => write!(out, " {}", map.emplaced(key)).unwrap(),
        'E' => flag(out, map.emplace(key)),
        't' => write!(out, " {}", map.try_emplaced(key)).unwrap(),
        'T' => flag(out, map.try_emplace(key)),
        'r' => flag(out, map.remove(&key)),
        'q' => write!(out, " {}", map.contained(&key).copied().unwrap_or(-1)).unwrap(),
        'R' => map.reserve(key as usize),
        'c' => map.clear(key != 0),
        'm' => flat_set_move(map, out),
        _ => panic!("unknown flat set command {name}"),
    }
}

// Native `aTarget = std::move(aMap)` releases the target and empties the source.
fn flat_set_move(map: &mut FlatMap<i32>, out: &mut String) {
    let mut target = FlatMap::<i32>::new(40);
    target.add(1000);
    target.move_assign(map);
    write!(
        out,
        " moved {} {} {}",
        map.len(),
        map.capacity(),
        target.len()
    )
    .unwrap();
    map.clone_from(&target);
}

fn flat_data(input: &mut SplitWhitespace<'_>, out: &mut String) {
    let mut map = FlatDataMap::<i32, i32>::default();
    while let Some((name, key)) = command(input) {
        let value = if "bBtTeE".contains(name) {
            int(input).expect("bound value")
        } else {
            0
        };
        flat_data_command(&mut map, name, key, value, out);
    }
    write!(
        out,
        " | {} {} {} :",
        map.len(),
        map.extent(),
        map.capacity()
    )
    .unwrap();
    for (key, value) in map.items() {
        write!(out, " {key}:{value}").unwrap();
    }
}

fn flat_data_command(
    map: &mut FlatDataMap<i32, i32>,
    name: char,
    key: i32,
    value: i32,
    out: &mut String,
) {
    match name {
        'b' => flag(out, map.bind(key, value)),
        'B' => write!(out, " {}", map.bound(key, value)).unwrap(),
        't' => flag(out, map.try_bind(key, value)),
        'T' => write!(out, " {}", map.try_bound(key, value)).unwrap(),
        'e' => write!(out, " {}", map.emplaced(key, || value)).unwrap(),
        'E' => write!(out, " {}", map.try_emplaced(key, || value)).unwrap(),
        'u' => flag(out, map.unbind(&key)),
        'f' => guarded(out, " ", map.find(&key)),
        'q' => match map.contained(&key) {
            Some((key, value)) => write!(out, " {key}:{value}").unwrap(),
            None => out.push_str(" none"),
        },
        'R' => map.reserve(key as usize),
        'c' => map.clear(key != 0),
        _ => panic!("unknown flat data command {name}"),
    }
}
