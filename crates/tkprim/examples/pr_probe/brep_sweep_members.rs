//! BRepSweep member probe operations, the Rust twin of
//! tools/oracle/tkprim_probe_brepsweep_members.cxx (the records are documented there).
use super::brep_sweep::generating;
use super::common::{Reader, attempt, end, flag, int, pnt, real, xyz};
use super::shape::shape;
use super::shapes::{TpShapes, tp_same_index, tp_shape_index};
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{Shape, ShapeIterator};
use rustycad_tkbrep::toptools::IndexedMapOfShape;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::gp::{Axis1, Transform3, Vector3};
use rustycad_tkmath::toploc::Location;
use rustycad_tkprim::brep_sweep::{
    BRepSweepBuilder, BRepSweepIterator, BRepSweepNumLinearRegularSweep, BRepSweepRotation,
    BRepSweepTool, BRepSweepTranslation,
};
use rustycad_tkprim::sweep::{SweepNumShape, SweepNumShapeTool};

/// `pr_dir_shapes`.
fn dir_shapes(n: &SweepNumShape) -> Vec<SweepNumShape> {
    let tool = SweepNumShapeTool::new(n);
    (1..=tool.nb_shapes()).map(|i| tool.shape(i)).collect()
}

/// `pr_empty_shapes`.
fn empty_shapes(
    sweep: &mut dyn BRepSweepNumLinearRegularSweep,
    s: &Shape,
    dirs: &[SweepNumShape],
) -> Result<(), Failure> {
    for d in dirs {
        let st = s.shape_type()?;
        let edge = d.kind() == ShapeEnum::Edge;
        if st == ShapeEnum::Vertex && !edge {
            attempt(|| shape(&sweep.make_empty_vertex(s, d)?));
        }
        if st == ShapeEnum::Vertex && edge {
            attempt(|| shape(&sweep.make_empty_directing_edge(s, d)?));
        }
        if st == ShapeEnum::Edge && !edge {
            attempt(|| shape(&sweep.make_empty_generating_edge(s, d)?));
        }
        if (st == ShapeEnum::Edge && edge) || (st == ShapeEnum::Face && !edge) {
            attempt(|| shape(&sweep.make_empty_face(s, d)?));
        }
        if st == ShapeEnum::Face && edge {
            attempt(|| {
                int(sweep.direct_solid(s, d)? as i32);
                Ok(())
            });
        }
    }
    Ok(())
}

/// `pr_add_tests`.
fn add_tests(
    sweep: &mut dyn BRepSweepNumLinearRegularSweep,
    s: &Shape,
    dirs: &[SweepNumShape],
) -> Result<(), Failure> {
    let e = dirs[0];
    if !sweep.has_shape(s, &e)? {
        return Ok(());
    }
    let side = sweep.shape_at(s, &e)?;
    let mut it = ShapeIterator::new(s, true, true);
    while let Ok(child) = it.value() {
        let child = child.clone();
        if sweep.has_shape(&child, &e)? {
            let sub = sweep.shape_at(&child, &e)?;
            flag(sweep.ggd_shape_is_to_add(&side, &sub, (s, &child, &e))?);
            flag(sweep.separated_wires(&side, &sub, (s, &child, &e))?);
        }
        it.next();
    }
    for d in &dirs[1..] {
        if sweep.has_shape(s, d)? {
            let sub = sweep.shape_at(s, d)?;
            flag(sweep.gdd_shape_is_to_add(&side, &sub, s, (&e, d))?);
        }
    }
    Ok(())
}

/// `pr_members`.
fn members(
    sweep: &mut dyn BRepSweepNumLinearRegularSweep,
    t: &TpShapes,
    gen_shape: &Shape,
    n: &SweepNumShape,
) -> Result<(), Failure> {
    flag(sweep.closed());
    let dirs = dir_shapes(n);
    let mut result = Shape::default();
    attempt(|| {
        result = sweep.shape()?;
        shape(&result)
    });
    for s in &t.shapes {
        attempt(|| {
            flag(sweep.is_invariant(s)?);
            Ok(())
        });
        for d in &dirs {
            attempt(|| {
                flag(sweep.has_shape(s, d)?);
                Ok(())
            });
        }
    }
    let mut subs = IndexedMapOfShape::default();
    topexp::map_shapes(gen_shape, &mut subs, true, true);
    for i in 1..=subs.extent() {
        let sub = subs.find_key(i)?.clone();
        empty_shapes(sweep, &sub, &dirs)?;
        attempt(|| add_tests(sweep, &sub, &dirs));
    }
    if !result.is_null() {
        attempt(|| shape(&sweep.split_shell(&result)?));
    }
    Ok(())
}

/// `pr_dir_record`.
fn dir_record(r: &mut Reader<'_>) -> SweepNumShape {
    let index = r.integer();
    let kind = ShapeEnum::ALL[r.integer() as usize];
    let (closed, beg) = (r.flag(), r.flag());
    SweepNumShape::with_flags(index, kind, closed, beg, r.flag())
}

/// The body of `bs_members` after the program.
fn members_body(
    r: &mut Reader<'_>,
    kind: i32,
    t: &TpShapes,
    s: &Shape,
    n: SweepNumShape,
) -> Result<(), Failure> {
    if kind == 0 {
        let v = Vector3::from(r.xyz());
        let (copy, canonize) = (r.flag(), r.flag());
        let mut transform = Transform3::default();
        transform.set_translation(v);
        let mut sweep =
            BRepSweepTranslation::new(s, n, Location::new(transform), v, copy, canonize)?;
        xyz(*sweep.vec().xyz());
        return members(&mut sweep, t, s, &n);
    }
    let axis = Axis1::new(r.pnt(), r.dir());
    let (angle, copy) = (r.real(), r.flag());
    let mut transform = Transform3::default();
    transform.set_rotation(axis, angle)?;
    let mut sweep = BRepSweepRotation::new(s, n, Location::new(transform), axis, angle, copy)?;
    pnt(sweep.axe().location());
    real(sweep.angle());
    members(&mut sweep, t, s, &n)
}

/// `bs_members kind program index nsh params copy`.
pub fn members_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let kind = r.integer();
    let (t, s) = match generating(&mut r) {
        Ok(found) => found,
        Err(error) => {
            print!("{} ", error.exception_type());
            end();
            return;
        }
    };
    let n = dir_record(&mut r);
    attempt(|| members_body(&mut r, kind, &t, &s, n));
    end();
}

/// The tool part of `bs_tools`.
fn tool_queries(t: &TpShapes, s: &Shape) {
    let tool = BRepSweepTool::new(s);
    int(tool.nb_shapes());
    for x in &t.shapes {
        int(tool.index(x));
    }
    for i in 0..=tool.nb_shapes() + 1 {
        attempt(|| {
            let x = tool.shape(i)?;
            int(tp_shape_index(t, &x));
            int(tool.kind(&x)? as i32);
            int(tool.orientation(&x) as i32);
            Ok(())
        });
    }
    let mut o = s.clone();
    tool.set_orientation(&mut o, Orientation::Reversed);
    int(o.orientation() as i32);
}

/// The builder part of `bs_tools`.
fn builder_queries(s: &Shape) -> Result<(), Failure> {
    let b = BRepSweepBuilder::new(BRepBuilder::default());
    let mut c: [Shape; 5] = Default::default();
    b.make_compound(&mut c[0])?;
    b.make_comp_solid(&mut c[1])?;
    b.make_solid(&mut c[2])?;
    b.make_shell(&mut c[3])?;
    b.make_wire(&mut c[4])?;
    for x in &c {
        int(x.shape_type()? as i32);
    }
    b.add_oriented(&mut c[0], s, Orientation::Internal)?;
    b.add(&mut c[0], s)?;
    shape(&c[0])
}

/// `bs_tools program index`.
pub fn tools_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (t, s) = match generating(&mut r) {
        Ok(found) => found,
        Err(error) => {
            print!("{} ", error.exception_type());
            end();
            return;
        }
    };
    tool_queries(&t, &s);
    let mut it = BRepSweepIterator::new();
    flag(it.more());
    it.init(&s);
    while it.more() {
        if let (Ok(value), Ok(orientation)) = (it.value(), it.orientation()) {
            int(tp_same_index(&t, value));
            int(orientation as i32);
        }
        it.next();
    }
    attempt(|| builder_queries(&s));
    attempt(|| {
        let b = BRepSweepBuilder::new(BRepBuilder::default());
        let mut compound = Shape::default();
        b.make_compound(&mut compound)?;
        b.make_wire(&mut compound)
    });
    end();
}
