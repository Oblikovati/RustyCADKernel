//! BRepPrimAPI sweep and half-space probe operations, the Rust twin of
//! tools/oracle/tkprim_probe_brepprimapi_sweeps.cxx (the records are documented there).
use super::brep_sweep::{generated, generating};
use super::common::{Reader, attempt, dir, end, flag, int, pnt, real, xyz};
use super::shape::{digest, shape};
use super::shapes::TpShapes;
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{self, Shape};
use rustycad_tkbrep::toptools::IndexedMapOfShape;
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{Axis1, Vector3};
use rustycad_tkprim::brep_prim_api::{
    BRepPrimAPIMakeHalfSpace, BRepPrimAPIMakePrism, BRepPrimAPIMakeRevol, BRepPrimAPIMakeSweep,
};
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShape;

/// Prints the exception of a failed construction and ends the line.
fn failed(error: Failure) {
    print!("{} ", error.exception_type());
    end();
}

/// `pr_pa_list`.
fn list(result: &IndexedMapOfShape, l: &List<Shape>) -> Result<(), Failure> {
    int(l.len() as i32);
    for s in l.iter() {
        generated(result, s)?;
    }
    Ok(())
}

/// The `FirstShape(S)` and `LastShape(S)` overloads of the sweep makers.
trait SweepOverloads: BRepPrimAPIMakeSweep {
    fn first_of(&mut self, s: &Shape) -> Result<Shape, Failure>;
    fn last_of(&mut self, s: &Shape) -> Result<Shape, Failure>;
}

impl SweepOverloads for BRepPrimAPIMakePrism {
    fn first_of(&mut self, s: &Shape) -> Result<Shape, Failure> {
        self.first_shape_of(s)
    }
    fn last_of(&mut self, s: &Shape) -> Result<Shape, Failure> {
        self.last_shape_of(s)
    }
}

impl SweepOverloads for BRepPrimAPIMakeRevol {
    fn first_of(&mut self, s: &Shape) -> Result<Shape, Failure> {
        self.first_shape_of(s)
    }
    fn last_of(&mut self, s: &Shape) -> Result<Shape, Failure> {
        self.last_shape_of(s)
    }
}

/// `pr_pa_sweep_history`.
fn sweep_history<M: SweepOverloads>(m: &mut M, t: &TpShapes) {
    flag(m.is_done());
    let mut result = IndexedMapOfShape::default();
    let mut result_shape = Shape::default();
    attempt(|| {
        result_shape = m.shape()?.clone();
        shape(&result_shape)
    });
    if !result_shape.is_null() {
        topexp::map_shapes(&result_shape, &mut result, true, true);
    }
    attempt(|| generated(&result, &m.first_shape()?));
    attempt(|| generated(&result, &m.last_shape()?));
    for s in &t.shapes {
        attempt(|| list(&result, m.generated(s)?));
        attempt(|| {
            int(m.modified(s)?.len() as i32);
            Ok(())
        });
        attempt(|| {
            flag(m.is_deleted(s)?);
            Ok(())
        });
        attempt(|| generated(&result, &m.first_of(s)?));
        attempt(|| generated(&result, &m.last_of(s)?));
    }
}

/// The prism of `kind ...`.
fn prism(r: &mut Reader<'_>, s: &Shape) -> Result<BRepPrimAPIMakePrism, Failure> {
    if r.integer() == 0 {
        let v = Vector3::from(r.xyz());
        let copy = r.flag();
        return BRepPrimAPIMakePrism::new(s, v, copy, r.flag());
    }
    let d = r.dir();
    let (inf, copy) = (r.flag(), r.flag());
    BRepPrimAPIMakePrism::infinite(s, d, inf, copy, r.flag())
}

/// `pa_prism program index kind ...`.
pub fn prism_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (t, s) = match generating(&mut r) {
        Ok(found) => found,
        Err(error) => return failed(error),
    };
    let mut m = match prism(&mut r, &s) {
        Ok(m) => m,
        Err(error) => return failed(error),
    };
    xyz(*m.prism().vec().xyz());
    sweep_history(&mut m, &t);
    end();
}

/// The revolution of `kind P D ...`.
fn revol(r: &mut Reader<'_>, s: &Shape) -> Result<BRepPrimAPIMakeRevol, Failure> {
    let kind = r.integer();
    let axis = Axis1::new(r.pnt(), r.dir());
    if kind == 0 {
        let angle = r.real();
        return BRepPrimAPIMakeRevol::new(s, axis, angle, r.flag());
    }
    BRepPrimAPIMakeRevol::full(s, axis, r.flag())
}

/// `pa_revol program index kind P D ...`.
pub fn revol_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (t, s) = match generating(&mut r) {
        Ok(found) => found,
        Err(error) => return failed(error),
    };
    let mut m = match revol(&mut r, &s) {
        Ok(m) => m,
        Err(error) => return failed(error),
    };
    let axe = m.revol().axe();
    pnt(axe.location());
    dir(axe.direction());
    real(m.revol().angle());
    flag(m.has_degenerated());
    let mut result = IndexedMapOfShape::default();
    if m.is_done() {
        let built = m.shape().cloned().unwrap_or_default();
        topexp::map_shapes(&built, &mut result, true, true);
    }
    attempt(|| list(&result, m.degenerated()));
    sweep_history(&mut m, &t);
    end();
}

/// The half-space of `kind P`.
fn half_space(r: &mut Reader<'_>, s: &Shape) -> Result<BRepPrimAPIMakeHalfSpace, Failure> {
    let kind = r.integer();
    let p = r.pnt();
    if kind == 0 {
        return BRepPrimAPIMakeHalfSpace::with_face(&topods::face(s)?, p);
    }
    BRepPrimAPIMakeHalfSpace::with_shell(&topods::shell(s)?, p)
}

/// `pa_halfspace program index kind P`.
pub fn halfspace_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (_t, s) = match generating(&mut r) {
        Ok(found) => found,
        Err(error) => return failed(error),
    };
    let mut h = match half_space(&mut r, &s) {
        Ok(h) => h,
        Err(error) => return failed(error),
    };
    flag(h.is_done());
    attempt(|| digest(h.solid()?.clone()));
    attempt(|| shape(h.shape()?));
    end();
}
