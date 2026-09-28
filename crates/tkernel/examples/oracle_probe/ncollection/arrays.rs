//! NCollection arrays probe operations (twin of `tkernel_ncollection_arrays_probe.cxx`).
//!
//! Each operation writes the text its native twin prints; a native exception becomes
//! `throw <class>` appended after whatever the operation had already printed.
use rustycad_tkernel::ncollection::{
    AliasedArray, Array1, Array2, ArrayError, DynamicArray, LinearVector, SparseArray,
};
use std::fmt::Write;
use std::str::SplitWhitespace;

type Input<'a, 'b> = &'a mut SplitWhitespace<'b>;
type ProbeResult = Result<(), &'static str>;
const OUT_OF_RANGE: &str = "Standard_OutOfRange";

/// The output of an arrays operation, or `None` when `op` belongs to another family.
pub fn evaluate(op: &str, input: &mut SplitWhitespace<'_>) -> Option<String> {
    let probe: fn(Input, &mut String) -> ProbeResult = match op {
        "ncoll_arrays_array1" => array1,
        "ncoll_arrays_range" => range,
        "ncoll_arrays_layout" => layout,
        "ncoll_arrays_aliased" => aliased,
        "ncoll_arrays_linear" => linear,
        "ncoll_arrays_sparse" => sparse,
        "ncoll_arrays_sparse_assign" => sparse_assign,
        "ncoll_arrays_dynamic" => dynamic,
        _ => return None,
    };
    let mut out = String::new();
    if let Err(exception) = probe(input, &mut out) {
        write!(out, "throw {exception}").unwrap();
    }
    Some(out)
}

fn number<T: std::str::FromStr>(input: Input) -> T {
    let text = input.next().expect("numeric field");
    text.parse()
        .unwrap_or_else(|_| panic!("field {text:?}; expected a number"))
}

fn flag(value: bool) -> u8 {
    u8::from(value)
}

// Splits "x12" or "x3,4" into the operation letter and its numeric arguments.
fn op_arguments(token: &str) -> (char, i64, i64) {
    let letter = token.chars().next().expect("operation letter");
    let rest = &token[letter.len_utf8()..];
    let mut parts = rest.split(',').filter(|part| !part.is_empty());
    let first = parts
        .next()
        .map_or(0, |part| part.parse().expect("integer"));
    let second = parts
        .next()
        .map_or(0, |part| part.parse().expect("integer"));
    (letter, first, second)
}

fn array1(input: Input, out: &mut String) -> ProbeResult {
    let (lower, upper, new_upper): (i32, i32, i32) = (number(input), number(input), number(input));
    let mut array = Array1::<i32>::with_bounds(lower, upper).expect("valid bounds in corpus");
    let range = Array1::<i32>::range_size(lower, upper);
    write!(
        out,
        "{} {} {} {range}",
        array.length(),
        array.lower(),
        array.upper()
    )
    .unwrap();
    array.update_upper_bound(new_upper);
    write!(out, " {} {}", array.lower(), array.upper()).unwrap();
    Ok(())
}

fn range(input: Input, out: &mut String) -> ProbeResult {
    let (lower, upper, size): (i32, i32, usize) = (number(input), number(input), number(input));
    write!(
        out,
        "{} {} {} {}",
        Array1::<i32>::range_size(lower, upper),
        Array2::<i32>::range_size(lower, upper),
        Array2::<i32>::upper_for_size(lower, size),
        Array2::<i32>::array_size(size, 3)
    )
    .unwrap();
    Ok(())
}

fn layout(input: Input, out: &mut String) -> ProbeResult {
    let bounds: [i32; 4] = [number(input), number(input), number(input), number(input)];
    let [rl, ru, cl, cu] = bounds;
    let begin = Array2::<i32>::begin_position(rl, ru, cl, cu);
    write!(
        out,
        "{begin} {}",
        Array2::<i32>::last_position(rl, ru, cl, cu)
    )
    .unwrap();
    let mut grid = Array2::<i32>::with_bounds(rl, ru, cl, cu).expect("valid bounds in corpus");
    write_grid_shape(&grid, out);
    grid.update_upper_row(ru.wrapping_add(7));
    grid.update_upper_col(cu.wrapping_sub(3));
    write!(out, " {} {}", grid.lower_row(), grid.lower_col()).unwrap();
    Ok(())
}

fn write_grid_shape(grid: &Array2<'_, i32>, out: &mut String) {
    write!(
        out,
        " {} {} {} {} {} {} {} {} {} {}",
        grid.length(),
        grid.nb_rows(),
        grid.nb_columns(),
        grid.row_length(),
        grid.col_length(),
        flag(grid.is_empty()),
        grid.upper_row(),
        grid.upper_col(),
        grid.array1().lower(),
        grid.array1().upper()
    )
    .unwrap();
}

fn aliased(input: Input, out: &mut String) -> ProbeResult {
    let (stride, count): (usize, usize) = (number(input), number(input));
    let array = if count == 0 {
        AliasedArray::empty(stride)
    } else {
        AliasedArray::new(stride, count)
    }
    .expect("valid shape in corpus");
    let (lower, upper) = (array.lower(), array.upper());
    let allocated = flag(array.is_allocated());
    let bytes = array.size_bytes();
    write!(
        out,
        "{lower} {upper} {allocated} {bytes} {}",
        array.length()
    )
    .unwrap();
    Ok(())
}

fn linear(input: Input, out: &mut String) -> ProbeResult {
    let mut vector = LinearVector::<i32>::default();
    let mut counter = 0;
    for token in input.by_ref() {
        let (letter, argument, _) = op_arguments(token);
        linear_step(&mut vector, letter, argument as usize, &mut counter)?;
        let data = flag(vector.has_data());
        write!(out, "{}:{}:{data} ", vector.len(), vector.capacity()).unwrap();
    }
    for value in &vector {
        write!(out, "{value},").unwrap();
    }
    Ok(())
}

fn linear_step(
    vector: &mut LinearVector<i32>,
    letter: char,
    argument: usize,
    counter: &mut i32,
) -> ProbeResult {
    match letter {
        'a' => {
            vector.append(next(counter));
        }
        'r' => vector.reserve(argument),
        'R' => vector.resize(argument),
        'c' => vector.clear(false),
        'C' => vector.clear(true),
        'e' => vector.erase(argument).map_err(|_| OUT_OF_RANGE)?,
        'i' => {
            // The native call evaluates `++aCounter` before InsertBefore checks the index.
            let value = next(counter);
            vector
                .insert_before(argument, value)
                .map_err(|_| OUT_OF_RANGE)?;
        }
        other => panic!("linear op {other:?}; expected one of aRrcCei"),
    }
    Ok(())
}

fn next(counter: &mut i32) -> i32 {
    *counter += 1;
    *counter
}

fn sparse(input: Input, out: &mut String) -> ProbeResult {
    let mut array = SparseArray::new(number(input));
    for token in input.by_ref() {
        let (letter, index, _) = op_arguments(token);
        let index = index as usize;
        match letter {
            's' => {
                array.set_value(index, index as i32);
            }
            'u' => write!(out, "{}/", flag(array.unset_value(index))).unwrap(),
            _ => array.clear(),
        }
        write!(out, "{}:{} ", array.capacity(), array.len()).unwrap();
    }
    for (key, _) in array.entries() {
        write!(out, "{key},").unwrap();
    }
    Ok(())
}

fn filled_sparse(input: Input, side: i32) -> SparseArray<i32> {
    let mut array = SparseArray::new(number(input));
    let count: usize = number(input);
    for _ in 0..count {
        let key: usize = number(input);
        array.set_value(key, key as i32 * 2 + side);
    }
    array
}

fn sparse_assign(input: Input, out: &mut String) -> ProbeResult {
    let source = filled_sparse(input, 0);
    let mut destination = filled_sparse(input, 1);
    destination.assign(&source);
    write!(out, "{} {}", destination.capacity(), destination.len()).unwrap();
    for (key, value) in destination.entries() {
        write!(out, " {key}={value}").unwrap();
    }
    let mut empty = SparseArray::new(5);
    empty.exchange(&mut destination);
    write!(out, " {} {}", empty.capacity(), destination.capacity()).unwrap();
    Ok(())
}

fn dynamic(input: Input, out: &mut String) -> ProbeResult {
    let mut array = DynamicArray::<i32>::with_increment(number(input));
    for token in input.by_ref() {
        let (letter, first, second) = op_arguments(token);
        dynamic_step(&mut array, letter, first as i32, second as i32)?;
    }
    write!(out, "{} {}", array.length(), array.upper()).unwrap();
    for value in array.iter() {
        write!(out, " {value}").unwrap();
    }
    Ok(())
}

fn dynamic_step(
    array: &mut DynamicArray<i32>,
    letter: char,
    first: i32,
    second: i32,
) -> ProbeResult {
    // The native mutators return a reference; only their success matters here.
    let outcome: Result<(), ArrayError> = match letter {
        'a' => {
            array.append(first);
            Ok(())
        }
        's' => array.set_value(first, second).map(|_| ()),
        'b' => array.insert_before(first, second).map(|_| ()),
        'f' => array.insert_after(first, second).map(|_| ()),
        'e' => {
            array.erase_last();
            Ok(())
        }
        'c' => {
            array.clear(false);
            Ok(())
        }
        other => panic!("dynamic op {other:?}; expected one of asbfec"),
    };
    outcome.map_err(|_| OUT_OF_RANGE)
}
