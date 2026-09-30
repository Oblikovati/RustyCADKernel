//! BRepSweep probe operations, the Rust twin of tools/oracle/tkprim_probe_brepsweep.cxx
//! (the records are documented there).
use super::common::{Reader, attempt, dir, end, flag, int, pnt, real, xyz};
use super::shape::shape;
use super::shapes::{TpShapes, tp_shapes};
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkbrep::toptools::IndexedMapOfShape;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{Axis1, Vector3};
use rustycad_tkprim::brep_sweep::{BRepSweepPrism, BRepSweepRevol};

/// `pr_generated`: -1 for null, else the index in `result` and the orientation, and the
/// digest when it is not there.
pub fn generated(result: &IndexedMapOfShape, s: &Shape) -> Result<(), Failure> {
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

/// `pr_generating`: the program and the swept shape.
pub fn generating(r: &mut Reader<'_>) -> Result<(TpShapes, Shape), Failure> {
    let t = tp_shapes(r)?;
    let index = r.integer();
    let s = t
        .shapes
        .get(index as usize)
        .cloned()
        .ok_or_else(|| Failure::out_of_range("pr_generating: shape index"))?;
    Ok((t, s))
}

/// The members of a prism or a revolution that the queries use.
pub trait SweepQueries {
    fn shape(&mut self) -> Result<Shape, Failure>;
    fn first_shape(&mut self) -> Result<Shape, Failure>;
    fn last_shape(&mut self) -> Result<Shape, Failure>;
    fn is_used(&self, s: &Shape) -> Result<bool, Failure>;
    fn gen_is_used(&self, s: &Shape) -> Option<Result<bool, Failure>>;
    fn shape_of(&mut self, s: &Shape) -> Result<Shape, Failure>;
    fn first_shape_of(&mut self, s: &Shape) -> Result<Shape, Failure>;
    fn last_shape_of(&mut self, s: &Shape) -> Result<Shape, Failure>;
}

macro_rules! sweep_queries {
    ($class:ty, $gen:expr) => {
        impl SweepQueries for $class {
            fn shape(&mut self) -> Result<Shape, Failure> {
                <$class>::shape(self)
            }
            fn first_shape(&mut self) -> Result<Shape, Failure> {
                <$class>::first_shape(self)
            }
            fn last_shape(&mut self) -> Result<Shape, Failure> {
                <$class>::last_shape(self)
            }
            fn is_used(&self, s: &Shape) -> Result<bool, Failure> {
                <$class>::is_used(self, s)
            }
            fn gen_is_used(&self, s: &Shape) -> Option<Result<bool, Failure>> {
                $gen(self, s)
            }
            fn shape_of(&mut self, s: &Shape) -> Result<Shape, Failure> {
                <$class>::shape_of(self, s)
            }
            fn first_shape_of(&mut self, s: &Shape) -> Result<Shape, Failure> {
                <$class>::first_shape_of(self, s)
            }
            fn last_shape_of(&mut self, s: &Shape) -> Result<Shape, Failure> {
                <$class>::last_shape_of(self, s)
            }
        }
    };
}
sweep_queries!(BRepSweepPrism, |p: &BRepSweepPrism, s: &Shape| Some(
    p.gen_is_used(s)
));
sweep_queries!(BRepSweepRevol, |_: &BRepSweepRevol, _: &Shape| None);

/// `pr_sweep_queries`.
fn sweep_queries<S: SweepQueries>(sweep: &mut S, t: &TpShapes) {
    let mut result = IndexedMapOfShape::default();
    let mut result_shape = Shape::default();
    attempt(|| {
        result_shape = sweep.shape()?;
        shape(&result_shape)
    });
    if !result_shape.is_null() {
        topexp::map_shapes(&result_shape, &mut result, true, true);
    }
    attempt(|| generated(&result, &sweep.first_shape()?));
    attempt(|| generated(&result, &sweep.last_shape()?));
    for s in &t.shapes {
        attempt(|| {
            flag(sweep.is_used(s)?);
            Ok(())
        });
        if let Some(used) = sweep.gen_is_used(s) {
            attempt(|| {
                flag(used?);
                Ok(())
            });
        }
        attempt(|| generated(&result, &sweep.shape_of(s)?));
        attempt(|| generated(&result, &sweep.first_shape_of(s)?));
        attempt(|| generated(&result, &sweep.last_shape_of(s)?));
    }
}

/// Prints the exception of a failed construction and ends the line.
fn failed(error: Failure) {
    print!("{} ", error.exception_type());
    end();
}

/// `bs_prism program index kind ...`.
pub fn prism_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (t, s) = match generating(&mut r) {
        Ok(found) => found,
        Err(error) => return failed(error),
    };
    let kind = r.integer();
    let built = if kind == 0 {
        let v = Vector3::from(r.xyz());
        let copy = r.flag();
        BRepSweepPrism::new(&s, v, copy, r.flag())
    } else {
        let d = r.dir();
        let (inf, copy) = (r.flag(), r.flag());
        BRepSweepPrism::infinite(&s, d, inf, copy, r.flag())
    };
    let mut prism = match built {
        Ok(prism) => prism,
        Err(error) => return failed(error),
    };
    xyz(*prism.vec().xyz());
    sweep_queries(&mut prism, &t);
    end();
}

/// `bs_revol program index kind P D ...`.
pub fn revol_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (t, s) = match generating(&mut r) {
        Ok(found) => found,
        Err(error) => return failed(error),
    };
    let kind = r.integer();
    let axis = Axis1::new(r.pnt(), r.dir());
    let built = if kind == 0 {
        let angle = r.real();
        BRepSweepRevol::new(&s, axis, angle, r.flag())
    } else {
        BRepSweepRevol::full(&s, axis, r.flag())
    };
    let mut revol = match built {
        Ok(revol) => revol,
        Err(error) => return failed(error),
    };
    pnt(revol.axe().location());
    dir(revol.axe().direction());
    real(revol.angle());
    sweep_queries(&mut revol, &t);
    end();
}
