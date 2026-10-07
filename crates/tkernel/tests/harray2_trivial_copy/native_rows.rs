use super::TrivialItem;
use rustycad_tkernel::{
    ncollection::{Array1, Array2, HArray2},
    standard::{Handle, Transient},
};

fn flat_values(array: &Array2<'_, TrivialItem>) -> String {
    let result = array
        .as_slice()
        .iter()
        .map(|x| x.0.to_string())
        .collect::<Vec<_>>()
        .join(",");
    if result.is_empty() {
        "-".to_owned()
    } else {
        result
    }
}
fn metadata(array: &Array2<'_, TrivialItem>) -> String {
    format!(
        "{} {} {} {} {} {} {} {} {} {}",
        array.lower_row(),
        array.upper_row(),
        array.lower_col(),
        array.upper_col(),
        array.nb_rows(),
        array.nb_columns(),
        array.array1().lower(),
        array.array1().upper(),
        array.len(),
        array.length()
    )
}
fn snapshot(name: &str, source: &Array2<'_, TrivialItem>) -> String {
    let copy = HArray2::from_array_trivial(source);
    let view = copy.array2();
    format!(
        "{name} {} {} {} {} {} {} {}\n",
        metadata(&view),
        u8::from(view.is_deletable()),
        u8::from(source.is_deletable()),
        u8::from(view.data_ptr().is_null()),
        u8::from(view.data_ptr() == source.data_ptr()),
        copy.transient_base().ref_count(),
        flat_values(&view)
    )
}
fn owned_source() -> Array2<'static, TrivialItem> {
    let mut buffer = [
        TrivialItem(11),
        TrivialItem(22),
        TrivialItem(33),
        TrivialItem(44),
    ];
    let borrowed = Array2::from_buffer_bounds(&mut buffer, -1, 0, 3, 4).unwrap();
    let mut source = Array2::default();
    source.assign_trivial(&borrowed);
    source
}
fn owned_rows() -> String {
    let mut source = owned_source();
    let mut rows = snapshot("owned", &source);
    let copy = HArray2::from_array_trivial(&source);
    source.change_value(-1, 3).unwrap().0 = 99;
    rows += &format!(
        "independent {} {}\n",
        source.value(-1, 3).unwrap().0,
        copy.array2().value(-1, 3).unwrap().0
    );
    source.update_lower_row(8);
    source.update_lower_col(-7);
    rows + &snapshot("rebased", &source)
}
fn borrowed_rows() -> String {
    let mut buffer = [
        TrivialItem(11),
        TrivialItem(22),
        TrivialItem(33),
        TrivialItem(44),
    ];
    let source = Array2::from_buffer_bounds(&mut buffer, -2, -1, 5, 6).unwrap();
    let mut rows = snapshot("borrowed", &source);
    let empty = Array2::from_buffer_bounds(&mut buffer, 7, 6, -2, 1).unwrap();
    rows += &snapshot("borrowed_empty", &empty);
    rows
}
fn empty_rows() -> String {
    let empty = Array2::<TrivialItem>::default();
    let mut rows = snapshot("empty", &empty);
    let mut buffer = [];
    let borrowed = Array2::from_buffer_bounds(&mut buffer, 7, 6, -2, 1).unwrap();
    let mut empty_rows = Array2::default();
    empty_rows.assign_trivial(&borrowed);
    rows += &snapshot("empty_rows", &empty_rows);
    let borrowed = Array2::from_buffer_bounds(&mut buffer, -2, 1, 7, 6).unwrap();
    let mut empty_cols = Array2::default();
    empty_cols.assign_trivial(&borrowed);
    rows + &snapshot("empty_cols", &empty_cols)
}
fn inherited_flat_rows() -> String {
    let mut buffer = [
        TrivialItem(11),
        TrivialItem(22),
        TrivialItem(33),
        TrivialItem(44),
        TrivialItem(55),
        TrivialItem(66),
    ];
    let mut source = owned_source();
    let mut rows = String::new();
    for size in [6, 2, 0] {
        let mut donor = Array1::from_slice(&mut buffer[..size]);
        donor.update_lower_bound(77);
        source.change_array1().assign_trivial(&donor);
        rows += &snapshot(&format!("flat_{size}"), &source);
    }
    rows
}
fn handled_row() -> String {
    let mut buffer = [TrivialItem(11), TrivialItem(22)];
    let source = Array2::from_buffer_bounds(&mut buffer, -1, -1, 3, 4).unwrap();
    let first = Handle::new(HArray2::from_array_trivial(&source));
    let owner = first.get().unwrap();
    let one = owner.transient_base().ref_count();
    let second = first.clone();
    let two = owner.transient_base().ref_count();
    owner.change_array2().change_value(-1, 3).unwrap().0 = 55;
    let observed = second.get().unwrap().array2().value(-1, 3).unwrap().0;
    drop(second);
    format!(
        "handle {one} {two} {} {observed} {}\n",
        owner.transient_base().ref_count(),
        source.value(-1, 3).unwrap().0
    )
}
/// Builds every expected row from observed Rust state; `all_rows()`.
pub fn all_rows() -> String {
    owned_rows() + &borrowed_rows() + &empty_rows() + &inherited_flat_rows() + &handled_row()
}
