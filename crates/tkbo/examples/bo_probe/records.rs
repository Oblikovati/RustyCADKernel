use super::{flag, integer, real};
use rustycad_tkbo::bop_ds::{IndexRange, Pair, Pave, PaveBlock, Point};
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tkmath::gp::{Point2, Point3};
fn index_state(value: &IndexRange, query: i32) {
    integer(value.first());
    integer(value.last());
    let (first, last) = value.indices();
    integer(first);
    integer(last);
    flag(value.contains(query));
    let mut text = String::new();
    value.dump(&mut text).unwrap();
    print!("{text} ");
}
pub fn index_case(v: &[f64]) {
    index_state(&IndexRange::default(), v[4] as i32);
    let mut value = IndexRange::new(v[0] as i32, v[1] as i32);
    index_state(&value, v[4] as i32);
    value.set_first(v[2] as i32);
    value.set_last(v[3] as i32);
    index_state(&value, v[4] as i32);
    value.set_indices(v[1] as i32, v[0] as i32);
    index_state(&value, v[4] as i32);
}
fn pave_state(value: &Pave) {
    integer(value.index());
    real(value.parameter());
    let (index, parameter) = value.contents();
    integer(index);
    real(parameter);
    print!("{} ", value.hash_code());
    let mut text = String::new();
    value.dump(&mut text).unwrap();
    print!("{text} ");
}
pub fn pave_case(v: &[f64]) {
    pave_state(&Pave::default());
    let mut a = Pave::new(v[0] as i32, v[1]);
    let b = Pave::new(v[2] as i32, v[3]);
    pave_state(&a);
    pave_state(&b);
    flag(a.is_equal(&b));
    flag(a == b);
    flag(a.is_less(&b));
    flag(b.is_less(&a));
    a.set_index(v[2] as i32);
    a.set_parameter(v[3]);
    pave_state(&a);
    flag(a.is_equal(&b));
}
fn pair_state(value: &Pair) {
    let (a, b) = value.indices();
    integer(a);
    integer(b);
    print!("{} ", value.hash_code());
}
pub fn pair_case(v: &[f64]) {
    pair_state(&Pair::default());
    let mut a = Pair::new(v[0] as i32, v[1] as i32);
    let b = Pair::new(v[2] as i32, v[3] as i32);
    pair_state(&a);
    pair_state(&b);
    flag(a.is_equal(&b));
    flag(a == b);
    flag(a.is_less(&b));
    flag(b.is_less(&a));
    a.set_indices(v[3] as i32, v[2] as i32);
    pair_state(&a);
    flag(a == b);
}
fn point_state(value: &Point) {
    for coordinate in [
        value.pnt().x(),
        value.pnt().y(),
        value.pnt().z(),
        value.pnt2d1().x(),
        value.pnt2d1().y(),
        value.pnt2d2().x(),
        value.pnt2d2().y(),
    ] {
        real(coordinate);
    }
    integer(value.index());
}
pub fn point_case(v: &[f64]) {
    let mut value = Point::default();
    point_state(&value);
    value.set_pnt(Point3::new(v[0], v[1], v[2]));
    value.set_pnt2d1(Point2::new(v[3], v[4]));
    value.set_pnt2d2(Point2::new(v[5], v[6]));
    value.set_index(v[7] as i32);
    point_state(&value);
}

/// The `BOPDS_PaveBlock` endpoint, extra-pave, update and shrunk-box oracle request.
pub fn pave_block_case(v: &[f64]) {
    let mut block = PaveBlock::default();
    let mut edge = 99;
    integer(block.edge());
    integer(block.original_edge());
    flag(block.has_edge());
    flag(block.has_edge_into(&mut edge));
    integer(edge);
    block.set_edge(v[0] as i32);
    block.set_original_edge(v[1] as i32);
    block.set_pave1(Pave::new(v[2] as i32, v[3]));
    block.set_pave2(Pave::new(v[4] as i32, v[5]));
    flag(block.is_split_edge());
    let (first, last) = block.range();
    real(first);
    real(last);
    let (i1, i2) = block.indices();
    integer(i1);
    integer(i2);
    append_extra_paves(&mut block, v);
    integer(block.ext_paves().len() as i32);
    let mut found = -7;
    flag(block.contains_parameter(v[12], v[13], &mut found));
    integer(found);
    flag(block.is_to_update());
    let mut splits = Vec::new();
    block.update(&mut splits, v[14] != 0.0);
    write_splits(&splits);
    write_shrunk_data(&mut block, v);
}

/// Appends the three test paves; `append_extra_paves(&mut block, &request)`.
fn append_extra_paves(block: &mut PaveBlock, values: &[f64]) {
    for pair in [(6, 7), (8, 9), (10, 11)] {
        block.append_ext_pave(Pave::new(values[pair.0] as i32, values[pair.1]));
    }
}

/// Writes each generated interval in native field order.
fn write_splits(splits: &[PaveBlock]) {
    integer(splits.len() as i32);
    for split in splits {
        let (first, last) = split.range();
        let (i1, i2) = split.indices();
        real(first);
        real(last);
        integer(i1);
        integer(i2);
        integer(split.original_edge());
    }
}

/// Stores and prints shrunk bounds, matching `BOPDS_PaveBlock::ShrunkData`.
fn write_shrunk_data(block: &mut PaveBlock, values: &[f64]) {
    let bounds = BoundingBox3::new(
        rustycad_tkmath::gp::Point3::new(values[15], values[16], values[17]),
        rustycad_tkmath::gp::Point3::new(values[18], values[19], values[20]),
    );
    block.set_shrunk_data(values[21], values[22], &bounds, values[23] != 0.0);
    let (first, last, result, splittable) = block.shrunk_data();
    flag(block.has_shrunk_data());
    real(first);
    real(last);
    flag(splittable);
    for coordinate in result.get().unwrap() {
        real(coordinate);
    }
}
