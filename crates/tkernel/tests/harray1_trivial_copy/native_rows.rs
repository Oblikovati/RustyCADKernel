use super::TrivialItem;
use rustycad_tkernel::{
    ncollection::{Array1, HArray1},
    standard::{Handle, Transient},
};

fn values(array: &Array1<'_, TrivialItem>) -> String {
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
fn snapshot(name: &str, source: &Array1<'_, TrivialItem>) -> String {
    let copy = HArray1::from_array_trivial(source);
    let view = copy.array1();
    format!(
        "{name} {} {} {} {} {} {} {} {} {}\n",
        view.lower(),
        view.upper(),
        view.len(),
        u8::from(view.is_deletable()),
        u8::from(source.is_deletable()),
        u8::from(view.data_ptr().is_null()),
        u8::from(view.data_ptr() == source.data_ptr()),
        copy.transient_base().ref_count(),
        values(&view)
    )
}
fn owned_source() -> Array1<'static, TrivialItem> {
    let mut buffer = [TrivialItem(11), TrivialItem(22), TrivialItem(33)];
    let mut borrowed = Array1::from_slice(&mut buffer);
    borrowed.update_lower_bound(-2);
    let mut source = Array1::default();
    source.assign_trivial(&borrowed);
    source
}
fn owned_rows() -> String {
    let mut source = owned_source();
    let mut rows = snapshot("owned", &source);
    let copy = HArray1::from_array_trivial(&source);
    source.change_value(-2).unwrap().0 = 99;
    rows += &format!(
        "independent {} {}\n",
        source.value(-2).unwrap().0,
        copy.array1().value(-2).unwrap().0
    );
    source.update_lower_bound(8);
    rows + &snapshot("rebased", &source)
}
fn borrowed_rows() -> String {
    let mut buffer = [TrivialItem(11), TrivialItem(22), TrivialItem(33)];
    let mut source = Array1::from_slice(&mut buffer);
    source.update_lower_bound(-3);
    let mut rows = snapshot("borrowed", &source);
    let mut empty = Array1::from_slice(&mut buffer[..0]);
    empty.update_lower_bound(7);
    rows += &snapshot("borrowed_empty", &empty);
    rows
}
fn empty_rows() -> String {
    let mut empty = Array1::<TrivialItem>::default();
    let mut rows = snapshot("empty", &empty);
    empty.update_lower_bound(7);
    rows += &snapshot("shifted_empty", &empty);
    rows
}
fn handled_row() -> String {
    let mut buffer = [TrivialItem(11), TrivialItem(22)];
    let mut source = Array1::from_slice(&mut buffer);
    source.update_lower_bound(-1);
    let first = Handle::new(HArray1::from_array_trivial(&source));
    let owner = first.get().unwrap();
    let one = owner.transient_base().ref_count();
    let second = first.clone();
    let two = owner.transient_base().ref_count();
    owner.change_array1().change_value(-1).unwrap().0 = 55;
    let observed = second.get().unwrap().array1().value(-1).unwrap().0;
    drop(second);
    format!(
        "handle {one} {two} {} {observed} {}\n",
        owner.transient_base().ref_count(),
        source.value(-1).unwrap().0
    )
}
/// Builds every expected line from actual Rust arrays; `all_rows()`.
pub fn all_rows() -> String {
    owned_rows() + &borrowed_rows() + &empty_rows() + &handled_row()
}
