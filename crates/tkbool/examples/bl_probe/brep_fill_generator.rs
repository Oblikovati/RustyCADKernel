//! BRepFill_Generator probe operations, the Rust twin of
//! tools/oracle/tkbool_probe_brepfill_generator.cxx (the records are documented there).
use super::common::{Reader, attempt, end, flag, int};
use super::shape::shape;
use super::shapes::{TpShapes, tp_shapes};
use rustycad_tkbool::brep_fill::BRepFillGenerator;
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{self, Shape};
use rustycad_tkbrep::toptools::IndexedMapOfShape;
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::Failure;

/// Prints the exception of a failed construction and ends the line.
pub fn failed(error: Failure) {
    print!("{} ", error.exception_type());
    end();
}

/// `bl_indexed`.
fn indexed(result: &IndexedMapOfShape, s: &Shape) -> Result<(), Failure> {
    if s.is_null() {
        int(-1);
        return Ok(());
    }
    let i = result.find_index(s) as i32;
    int(i);
    int(s.orientation() as i32);
    if i == 0 {
        shape(s)?;
    }
    Ok(())
}

/// `bl_list`.
pub fn list(result: &IndexedMapOfShape, l: &List<Shape>) -> Result<(), Failure> {
    int(l.len() as i32);
    for s in l.iter() {
        indexed(result, s)?;
    }
    Ok(())
}

/// The generator on the wires named by the request, with its input mode set.
fn generator_of(r: &mut Reader<'_>, t: &TpShapes) -> Result<BRepFillGenerator, Failure> {
    let mut generator = BRepFillGenerator::new();
    let n = r.integer();
    for _ in 0..n {
        let index = r.integer() as usize;
        let s = t
            .shapes
            .get(index)
            .cloned()
            .ok_or_else(|| Failure::out_of_range("bl_bf_generator: wire index"))?;
        generator.add_wire(&topods::wire(&s)?);
    }
    generator.set_mutable_input(r.flag());
    Ok(generator)
}

/// `bf_generator program n (wire)... mutable`.
pub fn generator_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let t = match tp_shapes(&mut r) {
        Ok(t) => t,
        Err(error) => return failed(error),
    };
    let mut generator = match generator_of(&mut r, &t) {
        Ok(generator) => generator,
        Err(error) => return failed(error),
    };
    flag(generator.is_mutable_input());
    if let Err(error) = generator.perform() {
        return failed(error);
    }
    int(generator.get_status() as i32);
    let shell: Shape = generator.shell().as_ref().clone();
    attempt(|| shape(&shell));
    let mut result = IndexedMapOfShape::default();
    if !shell.is_null() {
        topexp::map_shapes(&shell, &mut result, true, true);
    }
    int(generator.generated().extent() as i32);
    attempt(|| list(&result, generator.generated_shapes(&Shape::default())));
    for s in &t.shapes {
        attempt(|| list(&result, generator.generated_shapes(s)));
        attempt(|| indexed(&result, &generator.result_shape(s)?));
    }
    end();
}
