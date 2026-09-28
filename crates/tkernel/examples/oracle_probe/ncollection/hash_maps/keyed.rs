//! DataMap, Map and MapAlgo scripts (twins of `hm_datamap`, `hm_map` and `hm_algo`).
use super::{flag, guard, int, size};
use rustycad_tkernel::ncollection::{DataMap, DefaultHashPolicy, Map, map_algo};
use std::str::SplitWhitespace;

type IntDataMap = DataMap<i32, i32>;
type IntMap = Map<i32>;

/// `hm_datamap`: a `NCollection_DataMap<int,int>(size_t)` driven by a script.
pub fn data_map(input: &mut SplitWhitespace<'_>) -> String {
    let mut map = IntDataMap::with_hasher(DefaultHashPolicy, size(input));
    let mut out = String::new();
    while let Some(code) = input.next() {
        out += &data_map_op(&mut map, code, input);
        out.push(' ');
    }
    out += &format!(
        "| n={} e={} s={} :",
        map.bucket_count(),
        map.extent(),
        map.len()
    );
    let mut cursor = map.cursor();
    while cursor.more() {
        out += &format!(" {}={}", cursor.key().unwrap(), cursor.value().unwrap());
        cursor.next();
    }
    out
}

fn data_map_op(map: &mut IntDataMap, code: &str, input: &mut SplitWhitespace<'_>) -> String {
    match code {
        "b" | "t" | "s" | "T" | "e" | "E" | "y" | "Y" => {
            let (key, value) = (int(input), int(input));
            data_map_insert(map, code, key, value)
        }
        "u" => flag(map.unbind(&int(input))),
        "i" => flag(map.is_bound(&int(input))),
        "r" => {
            map.resize(size(input));
            "-".into()
        }
        "c" => {
            map.clear(int(input) != 0);
            "-".into()
        }
        _ => data_map_query(map, code, input),
    }
}

fn data_map_insert(map: &mut IntDataMap, code: &str, key: i32, value: i32) -> String {
    match code {
        "b" => flag(map.bind(key, value)),
        "t" => flag(map.try_bind(key, value)),
        "s" => map.bound(key, value).to_string(),
        "T" => map.try_bound(key, value).to_string(),
        "e" => flag(map.emplace(key, || value)),
        "E" => map.emplaced(key, || value).to_string(),
        "y" => flag(map.try_emplace(key, || value)),
        _ => map.try_emplaced(key, || value).to_string(),
    }
}

fn data_map_query(map: &mut IntDataMap, code: &str, input: &mut SplitWhitespace<'_>) -> String {
    match code {
        "f" => guard(map.find(&int(input)).copied()),
        "F" => {
            let mut value = -1;
            let found = map.find_into(&int(input), &mut value);
            format!("{}:{value}", flag(found))
        }
        "q" => map.seek(&int(input)).map_or("null".into(), i32::to_string),
        "x" => map
            .contained(&int(input))
            .map_or("none".into(), |(key, value)| format!("{key}:{value}")),
        "A" => {
            let copy = map.clone();
            map.assign(&copy);
            "-".into()
        }
        other => panic!("unknown DataMap code {other}"),
    }
}

/// `hm_map`: a `NCollection_Map<int>(size_t)` driven by a script.
pub fn map(input: &mut SplitWhitespace<'_>) -> String {
    let mut map = IntMap::with_hasher(DefaultHashPolicy, size(input));
    let mut out = String::new();
    while let Some(code) = input.next() {
        out += &map_op(&mut map, code, input);
        out.push(' ');
    }
    out + &map_print(&map)
}

fn map_op(map: &mut IntMap, code: &str, input: &mut SplitWhitespace<'_>) -> String {
    match code {
        "a" => flag(map.add(int(input))),
        "A" => map.added(int(input)).to_string(),
        "e" => flag(map.emplace(int(input))),
        "E" => map.emplaced(int(input)).to_string(),
        "y" => flag(map.try_emplace(int(input))),
        "Y" => map.try_emplaced(int(input)).to_string(),
        "m" => flag(map.remove(&int(input))),
        "i" => flag(map.contains(&int(input))),
        "x" => map
            .contained(&int(input))
            .map_or("none".into(), i32::to_string),
        _ => map_maintenance(map, code, input),
    }
}

fn map_maintenance(map: &mut IntMap, code: &str, input: &mut SplitWhitespace<'_>) -> String {
    match code {
        "r" => map.resize(size(input)),
        "c" => map.clear(int(input) != 0),
        "C" => {
            let copy = map.clone();
            map.assign(&copy);
        }
        other => panic!("unknown Map code {other}"),
    }
    "-".into()
}

/// `hm_map_print`: bucket count, extent and the Iterator traversal.
fn map_print(map: &IntMap) -> String {
    let mut out = format!("| n={} e={} :", map.bucket_count(), map.extent());
    let mut cursor = map.cursor();
    while cursor.more() {
        out += &format!(" {}", cursor.key().unwrap());
        cursor.next();
    }
    out
}

fn read_keys(input: &mut SplitWhitespace<'_>) -> IntMap {
    let mut map = IntMap::default();
    for _ in 0..int(input) {
        map.add(int(input));
    }
    map
}

/// `hm_algo`: one NCollection_MapAlgo function over three prefilled maps.
pub fn algo(input: &mut SplitWhitespace<'_>) -> String {
    let name = input.next().expect("algorithm name");
    let (mut result, mut left, right) = (read_keys(input), read_keys(input), read_keys(input));
    // Void native calls print nothing before the separating space.
    let three_operand: Option<fn(&mut IntMap, &IntMap, &IntMap)> = match name {
        "union" => Some(map_algo::union),
        "intersection" => Some(map_algo::intersection),
        "subtraction" => Some(map_algo::subtraction),
        "difference" => Some(map_algo::difference),
        _ => None,
    };
    let answer = match (name, three_operand) {
        (_, Some(function)) => {
            function(&mut result, &left, &right);
            String::new()
        }
        _ => in_place(name, &mut left, &right),
    };
    format!("{answer} {} {}", map_print(&result), map_print(&left))
}

fn in_place(name: &str, left: &mut IntMap, right: &IntMap) -> String {
    match name {
        "unite" => flag(map_algo::unite(left, right)),
        "intersect" => flag(map_algo::intersect(left, right)),
        "subtract" => flag(map_algo::subtract(left, right)),
        "differ" => flag(map_algo::differ(left, right)),
        "has_intersection" => flag(map_algo::has_intersection(left, right)),
        "contains" => flag(map_algo::contains(left, right)),
        "is_equal" => flag(map_algo::is_equal(left, right)),
        other => panic!("unknown MapAlgo function {other}"),
    }
}
