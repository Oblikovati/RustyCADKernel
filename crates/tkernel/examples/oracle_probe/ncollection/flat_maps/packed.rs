//! Packed integer map scripts and the deprecated member set algebra.
#![allow(deprecated)] // the deprecated PackedMap members are probed on purpose

use super::{command, flag, int};
use rustycad_tkernel::ncollection::{PackedInteger, PackedMap, PackedMapIter, packed_map_algo};
use std::fmt::{Display, Write};
use std::str::{FromStr, SplitWhitespace};

pub fn packed_script(input: &mut SplitWhitespace<'_>, out: &mut String) {
    let mut map = PackedMap::<i32>::from_int(int(input).expect("bucket count"));
    write!(out, " nb {}", map.bucket_count()).unwrap();
    while let Some((name, key)) = command(input) {
        packed_command(&mut map, name, key, out);
    }
    dump(&map, out);
    let mut cursor = map.iter();
    if cursor.more() {
        cursor.advance();
    }
    cursor.reset();
    let empty = PackedMapIter::<i32>::default();
    write!(out, " reset {}", cursor.key().copied().unwrap_or(-1)).unwrap();
    flag(out, empty.more());
}

fn packed_command(map: &mut PackedMap<i32>, name: char, key: i32, out: &mut String) {
    match name {
        'a' => flag(out, map.add(key)),
        'r' => flag(out, map.remove(key)),
        'h' => flag(out, map.contains(key)),
        'R' => map.resize_int(key),
        'c' => map.clear(),
        'm' => {
            let mut target = PackedMap::<i32>::new(7);
            target.add(1000);
            target.move_assign(map);
            write!(out, " moved {} {}", map.bucket_count(), map.extent()).unwrap();
            map.assign(&target);
        }
        _ => panic!("unknown packed command {name}"),
    }
}

fn dump<T: PackedInteger + Display>(map: &PackedMap<T>, out: &mut String) {
    write!(
        out,
        " [{} {} {} {} {} {} {} :",
        map.extent(),
        map.extent(),
        map.len(),
        map.bucket_count(),
        u8::from(map.is_empty()),
        map.minimal(),
        map.maximal()
    )
    .unwrap();
    for key in map.iter() {
        write!(out, " {key}").unwrap();
    }
    out.push(']');
}

fn keys<T: PackedInteger + FromStr>(input: &mut SplitWhitespace<'_>) -> PackedMap<T> {
    let count: usize = int(input).expect("key count");
    let mut map = PackedMap::<T>::default();
    for _ in 0..count {
        map.add(int(input).expect("key"));
    }
    map
}

/// One in-place deprecated member (`Unite`, `Intersect`, `Subtract`, `Differ`).
type InPlaceStep<T> = fn(&mut PackedMap<T>, &PackedMap<T>) -> bool;

pub fn algebra<T: PackedInteger + FromStr + Display>(
    input: &mut SplitWhitespace<'_>,
    out: &mut String,
) {
    let left = keys::<T>(input);
    let right = keys::<T>(input);
    let mut result = PackedMap::<T>::default();
    result.union(&left, &right);
    dump(&result, out);
    result.intersection(&left, &right);
    dump(&result, out);
    result.subtraction(&left, &right);
    dump(&result, out);
    result.difference(&left, &right);
    dump(&result, out);
    let steps: [InPlaceStep<T>; 4] = [
        PackedMap::unite,
        PackedMap::intersect,
        PackedMap::subtract,
        PackedMap::differ,
    ];
    let mut work = left.clone();
    for step in steps {
        work.assign(&left);
        flag(out, step(&mut work, &right));
        dump(&work, out);
    }
    let checks = [
        left.is_equal(&right),
        left.is_subset(&right),
        left.has_intersection(&right),
        left.contains_map(&right),
        packed_map_algo::is_equal(&left, &left),
    ];
    out.push(' ');
    for check in checks {
        write!(out, "{}", u8::from(check)).unwrap();
    }
}
