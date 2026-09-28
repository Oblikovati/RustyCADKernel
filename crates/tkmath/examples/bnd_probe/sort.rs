//! Bnd_BoundSortBox counterpart: initialization modes, Add failures and Compare order.
use super::{Reader, box_read, failure};
use rustycad_tkernel::ncollection::{Array1, List};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::bnd::{BoundSortBox, BoundingBox3};
use rustycad_tkmath::gp::{Direction3, Plane3, Point3};

fn sort_result(result: Result<&List<i32>, Failure>) {
    match result {
        Ok(list) => {
            print!("{}:", list.len());
            for index in list.iter() {
                print!("{index},");
            }
            print!(" ");
        }
        Err(error) => failure(&error),
    }
}
fn status(result: Result<(), Failure>) {
    match result {
        Ok(()) => print!("ok "),
        Err(error) => failure(&error),
    }
}
fn initialize(
    sorter: &mut BoundSortBox,
    mode: i32,
    boxes: &Array1<'static, BoundingBox3>,
    enclosing: &BoundingBox3,
) -> Result<(), Failure> {
    match mode {
        0 => sorter.initialize(boxes.clone()),
        1 => sorter.initialize_with_enclosing(enclosing, boxes.clone()),
        2 => {
            sorter.initialize_with_count(enclosing, boxes.len() as i32)?;
            for index in boxes.lower()..=boxes.upper() {
                sorter.add(&boxes[index], index)?;
            }
            Ok(())
        }
        _ => {
            sorter.initialize(boxes.clone())?;
            sorter.initialize(boxes.clone())
        }
    }
}

/// `sortbox mode n box... enclosing nq query... np plane...`
pub(crate) fn sortbox_case(values: &[f64]) {
    let mut reader = Reader::new(values);
    let (mode, count) = (reader.integer(), reader.integer());
    let mut boxes = Array1::with_bounds(1, count).unwrap();
    for index in 1..=count {
        boxes[index] = box_read(&mut reader);
    }
    let enclosing = box_read(&mut reader);
    let mut sorter = BoundSortBox::default();
    status(initialize(&mut sorter, mode, &boxes, &enclosing));
    if mode == 2 {
        status(sorter.add(&boxes[1], 1));
        status(sorter.add(&boxes[1], count + 1));
    }
    for _ in 0..reader.integer() {
        let query = box_read(&mut reader);
        sort_result(sorter.compare_box(&query));
    }
    for _ in 0..reader.integer() {
        let origin = Point3::from(reader.triple());
        let normal = Direction3::from_xyz(reader.triple()).unwrap();
        let plane = Plane3::new(origin, normal).unwrap();
        sort_result(sorter.compare_plane(plane));
    }
    println!();
}
