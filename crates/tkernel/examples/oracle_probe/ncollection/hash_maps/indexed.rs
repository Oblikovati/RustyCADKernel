//! IndexedMap, IndexedDataMap and DoubleMap scripts (twins of `hm_indexed`,
//! `hm_indexed_data` and `hm_double`).
use super::{flag, guard, int, size};
use rustycad_tkernel::ncollection::{DefaultHashPolicy, DoubleMap, IndexedDataMap, IndexedMap};
use std::str::SplitWhitespace;

type IntIndexedMap = IndexedMap<i32>;
type IntIndexedDataMap = IndexedDataMap<i32, i32>;
type IntDoubleMap = DoubleMap<i32, i32>;

/// Runs every remaining script code through `step`, separating results by spaces.
fn run_script<M>(
    map: &mut M,
    input: &mut SplitWhitespace<'_>,
    step: fn(&mut M, &str, &mut SplitWhitespace<'_>) -> String,
) -> String {
    let mut out = String::new();
    while let Some(code) = input.next() {
        out += &step(map, code, input);
        out.push(' ');
    }
    out
}

/// `hm_indexed`: a `NCollection_IndexedMap<int>(size_t)` driven by a script.
pub fn indexed_map(input: &mut SplitWhitespace<'_>) -> String {
    let mut map = IntIndexedMap::with_hasher(DefaultHashPolicy, size(input));
    let mut out = run_script(&mut map, input, indexed_op);
    out += &format!("| n={} e={} :", map.bucket_count(), map.extent());
    let mut cursor = map.cursor();
    while cursor.more() {
        let key = cursor.value().unwrap();
        out += &format!(" {}={key}@{}", cursor.index(), map.find_index(key));
        cursor.next();
    }
    out
}

fn indexed_op(map: &mut IntIndexedMap, code: &str, input: &mut SplitWhitespace<'_>) -> String {
    match code {
        "a" => map.add(int(input)).to_string(),
        "A" => map.added(int(input)).to_string(),
        "e" => map.emplace(int(input)).to_string(),
        "y" => map.try_emplace(int(input)).to_string(),
        "m" => flag(map.remove_key(&int(input))),
        "f" => guard(map.find_key(size(input)).copied()),
        "n" => map.find_index(&int(input)).to_string(),
        "x" => map
            .contained(&int(input))
            .map_or("none".into(), i32::to_string),
        _ => indexed_edit(map, code, input),
    }
}

fn indexed_edit(map: &mut IntIndexedMap, code: &str, input: &mut SplitWhitespace<'_>) -> String {
    let done = |result: Result<(), _>| guard(result.map(|()| "-"));
    match code {
        "s" => {
            let index = size(input);
            done(map.substitute(index, int(input)))
        }
        "w" => done(map.swap(size(input), size(input))),
        "l" => done(map.remove_last()),
        "R" => done(map.remove_from_index(size(input))),
        "r" => {
            map.resize(size(input));
            "-".into()
        }
        "c" => {
            map.clear(int(input) != 0);
            "-".into()
        }
        "C" => {
            let copy = map.clone();
            map.assign(&copy);
            "-".into()
        }
        other => panic!("unknown IndexedMap code {other}"),
    }
}

/// `hm_indexed_data`: a `NCollection_IndexedDataMap<int,int>(size_t)` driven by a script.
pub fn indexed_data_map(input: &mut SplitWhitespace<'_>) -> String {
    let mut map = IntIndexedDataMap::with_hasher(DefaultHashPolicy, size(input));
    let mut out = run_script(&mut map, input, indexed_data_op);
    out += &format!("| n={} e={} :", map.bucket_count(), map.extent());
    let mut cursor = map.cursor();
    while cursor.more() {
        let (key, value) = (cursor.key().unwrap(), cursor.value().unwrap());
        out += &format!(" {}={key}:{value}@{}", cursor.index(), map.find_index(key));
        cursor.next();
    }
    out
}

fn indexed_data_op(
    map: &mut IntIndexedDataMap,
    code: &str,
    input: &mut SplitWhitespace<'_>,
) -> String {
    match code {
        "a" | "b" | "t" | "B" | "T" | "e" | "y" => {
            let (key, value) = (int(input), int(input));
            indexed_data_insert(map, code, key, value)
        }
        "m" => {
            // Native IndexedDataMap::RemoveKey returns void; the Rust bool is not printed.
            map.remove_key(&int(input));
            "-".into()
        }
        "f" => guard(map.find_from_index(size(input)).copied()),
        "F" => guard(map.find_from_key(&int(input)).copied()),
        "G" => {
            let mut value = -1;
            let found = map.find_copy(&int(input), &mut value);
            format!("{}:{value}", flag(found))
        }
        "q" => map.seek(&int(input)).map_or("null".into(), i32::to_string),
        _ => indexed_data_edit(map, code, input),
    }
}

fn indexed_data_insert(map: &mut IntIndexedDataMap, code: &str, key: i32, value: i32) -> String {
    match code {
        "a" => map.add(key, value).to_string(),
        "b" => flag(map.bind(key, value)),
        "t" => flag(map.try_bind(key, value)),
        "B" => map.bound(key, value).to_string(),
        "T" => map.try_bound(key, value).to_string(),
        "e" => map.emplace(key, || value).to_string(),
        _ => map.try_emplace(key, || value).to_string(),
    }
}

fn indexed_data_edit(
    map: &mut IntIndexedDataMap,
    code: &str,
    input: &mut SplitWhitespace<'_>,
) -> String {
    let done = |result: Result<(), _>| guard(result.map(|()| "-"));
    match code {
        "s" => {
            let index = size(input);
            let (key, value) = (int(input), int(input));
            done(map.substitute(index, key, value))
        }
        "w" => done(map.swap(size(input), size(input))),
        "l" => done(map.remove_last()),
        "R" => done(map.remove_from_index(size(input))),
        "r" => {
            map.resize(size(input));
            "-".into()
        }
        "c" => {
            map.clear(int(input) != 0);
            "-".into()
        }
        "C" => {
            let copy = map.clone();
            map.assign(&copy);
            "-".into()
        }
        other => panic!("unknown IndexedDataMap code {other}"),
    }
}

/// `hm_double`: a `NCollection_DoubleMap<int,int>(size_t)` driven by a script.
pub fn double_map(input: &mut SplitWhitespace<'_>) -> String {
    let mut map = IntDoubleMap::with_allocator(
        DefaultHashPolicy,
        DefaultHashPolicy,
        size(input),
        Default::default(),
    );
    let mut out = run_script(&mut map, input, double_op);
    out += &format!("| n={} e={} :", map.bucket_count(), map.extent());
    let mut cursor = map.cursor();
    while cursor.more() {
        out += &format!(" {}:{}", cursor.key1().unwrap(), cursor.key2().unwrap());
        cursor.next();
    }
    out
}

fn double_op(map: &mut IntDoubleMap, code: &str, input: &mut SplitWhitespace<'_>) -> String {
    match code {
        "b" => {
            let (first, second) = (int(input), int(input));
            guard(map.bind(first, second).map(|()| "-"))
        }
        "t" => {
            let (first, second) = (int(input), int(input));
            flag(map.try_bind(first, second))
        }
        "q" => {
            let (first, second) = (int(input), int(input));
            flag(map.are_bound(&first, &second))
        }
        "u" => flag(map.unbind1(&int(input))),
        "U" => flag(map.unbind2(&int(input))),
        "i" => flag(map.is_bound1(&int(input))),
        "I" => flag(map.is_bound2(&int(input))),
        _ => double_query(map, code, input),
    }
}

fn double_query(map: &mut IntDoubleMap, code: &str, input: &mut SplitWhitespace<'_>) -> String {
    let mut partner = -1;
    match code {
        "f" => guard(map.find1(&int(input)).copied()),
        "g" => guard(map.find2(&int(input)).copied()),
        "F" => {
            let found = map.find1_into(&int(input), &mut partner);
            format!("{}:{partner}", flag(found))
        }
        "G" => {
            let found = map.find2_into(&int(input), &mut partner);
            format!("{}:{partner}", flag(found))
        }
        "r" => {
            map.resize(size(input));
            "-".into()
        }
        "c" => {
            map.clear(int(input) != 0);
            "-".into()
        }
        "C" => {
            let copy = map.clone();
            map.assign(&copy);
            "-".into()
        }
        other => panic!("unknown DoubleMap code {other}"),
    }
}
