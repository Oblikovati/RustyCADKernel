//! Insertion-ordered map scripts: bucket selection, order links, reset and move-swap.
use super::{command, flag, guarded, int};
use rustycad_tkernel::ncollection::{BaseAllocator, DefaultHashPolicy, OrderedDataMap, OrderedMap};
use rustycad_tkernel::standard::Failure;
use std::fmt::Write;
use std::str::SplitWhitespace;

type Keys = OrderedMap<i32>;
type Pairs = OrderedDataMap<i32, i32>;

pub fn ordered_keys(input: &mut SplitWhitespace<'_>, out: &mut String) {
    let buckets: i32 = int(input).expect("bucket count");
    let mut map = match Keys::with_int_buckets(DefaultHashPolicy, buckets, BaseAllocator::common())
    {
        Ok(map) => map,
        Err(error) => return refused(input, out, &error),
    };
    write!(out, " nb {}", map.bucket_count()).unwrap();
    while let Some((name, key)) = command(input) {
        keys_command(&mut map, name, key, out);
    }
    write!(out, " | {} nb {} :", map.extent(), map.bucket_count()).unwrap();
    let mut node = map.first_node();
    while let Some(current) = node {
        write!(out, " {}", current.key()).unwrap();
        node = current.order_next();
    }
    let mut cursor = map.iter();
    if cursor.more() {
        cursor.advance();
    }
    cursor.reset();
    write!(out, " reset {}", cursor.key().map_or(-1, |key| *key)).unwrap();
    guarded(out, " first ", map.first());
    guarded(out, " last ", map.last());
}

fn keys_command(map: &mut Keys, name: char, key: i32, out: &mut String) {
    match name {
        'a' => flag(out, map.add(key)),
        'd' => write!(out, " {}", map.added(key)).unwrap(),
        'e' => write!(out, " {}", map.emplaced(key)).unwrap(),
        't' => write!(out, " {}", map.try_emplaced(key)).unwrap(),
        'r' => flag(out, map.remove(&key)),
        'q' => write!(out, " {}", map.contained(&key).copied().unwrap_or(-1)).unwrap(),
        'R' => {
            if let Err(error) = map.resize_int(key) {
                write!(out, " throw {}", error.exception_type()).unwrap();
            }
        }
        'c' => map.clear(key != 0),
        'C' => map.clear_with_allocator(None),
        'N' => write!(out, " nb {}", map.bucket_count()).unwrap(),
        'm' => keys_move(map, out),
        _ => panic!("unknown ordered command {name}"),
    }
}

// Native ordered move assignment swaps: the source receives the target's former keys.
fn keys_move(map: &mut Keys, out: &mut String) {
    let mut target = Keys::new(3);
    target.add(1000);
    target.move_assign(map);
    write!(out, " moved {} source", target.extent()).unwrap();
    for key in map.iter() {
        write!(out, " {key}").unwrap();
    }
    map.assign(&target);
}

pub fn ordered_data(input: &mut SplitWhitespace<'_>, out: &mut String) {
    let buckets: i32 = int(input).expect("bucket count");
    let mut map = match Pairs::with_int_buckets(DefaultHashPolicy, buckets, BaseAllocator::common())
    {
        Ok(map) => map,
        Err(error) => return refused(input, out, &error),
    };
    write!(out, " nb {}", map.bucket_count()).unwrap();
    while let Some((name, key)) = command(input) {
        let value = if "bBtTeE".contains(name) {
            int(input).expect("bound value")
        } else {
            0
        };
        pairs_command(&mut map, name, key, value, out);
    }
    pairs_summary(&map, out);
}

fn pairs_summary(map: &Pairs, out: &mut String) {
    write!(out, " | {} nb {} :", map.extent(), map.bucket_count()).unwrap();
    let mut node = map.first_node();
    while let Some(current) = node {
        write!(out, " {}:{}", current.key(), current.value()).unwrap();
        node = current.order_next();
    }
    let mut cursor = map.cursor();
    if cursor.more() {
        cursor.advance(map);
    }
    cursor.reset(map);
    write!(out, " reset {}", cursor.key(map).map_or(-1, |key| *key)).unwrap();
    guarded(out, " first ", map.first());
    guarded(out, " last ", map.last());
    // FirstValue and LastValue fail together, on an empty map.
    let values = map
        .first_value()
        .and_then(|first| Ok(format!("{first} {}", map.last_value()?)));
    guarded(out, " values ", values);
}

fn pairs_command(map: &mut Pairs, name: char, key: i32, value: i32, out: &mut String) {
    match name {
        'b' => flag(out, map.bind(key, value)),
        'B' => write!(out, " {}", map.bound(key, value)).unwrap(),
        't' => flag(out, map.try_bind(key, value)),
        'T' => write!(out, " {}", map.try_bound(key, value)).unwrap(),
        'e' => write!(out, " {}", map.emplaced(key, || value)).unwrap(),
        'E' => write!(out, " {}", map.try_emplaced(key, || value)).unwrap(),
        'u' => flag(out, map.unbind(&key)),
        'f' => guarded(out, " ", map.find(&key)),
        'F' => {
            let mut copy = -1;
            let found = map.find_copy(&key, &mut copy);
            write!(out, " {}:{copy}", u8::from(found)).unwrap();
        }
        'R' => {
            if let Err(error) = map.resize_int(key) {
                write!(out, " throw {}", error.exception_type()).unwrap();
            }
        }
        'c' => map.clear(key != 0),
        'C' => map.clear_with_allocator(None),
        'n' => {
            if let Some(mut node) = map.first_node_mut() {
                *node.change_value() = key;
            }
        }
        'm' => pairs_move(map, out),
        _ => panic!("unknown ordered data command {name}"),
    }
}

fn pairs_move(map: &mut Pairs, out: &mut String) {
    let mut target = Pairs::new(3);
    target.bind(1000, 1);
    target.move_assign(map);
    write!(out, " moved {} source", target.extent()).unwrap();
    for (key, value) in map.items() {
        write!(out, " {key}:{value}").unwrap();
    }
    map.assign(&target);
}

// A throwing native constructor abandons the rest of the script unread.
fn refused(input: &mut SplitWhitespace<'_>, out: &mut String, error: &Failure) {
    input.for_each(drop);
    write!(out, " throw {}", error.exception_type()).unwrap();
}
