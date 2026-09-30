//! Sweep probe operations, the Rust twin of tools/oracle/tkprim_probe_sweep.cxx.
use super::common::{Reader, end, flag, int, show};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkprim::sweep::{SweepNumShape, SweepNumShapeIterator, SweepNumShapeTool};

/// `pr_num_shape`: `index type closed begInf endInf`.
fn num_shape(r: &mut Reader<'_>) -> SweepNumShape {
    let index = r.integer();
    let kind = ShapeEnum::ALL[r.integer() as usize];
    let (closed, beg) = (r.flag(), r.flag());
    SweepNumShape::with_flags(index, kind, closed, beg, r.flag())
}

/// `pr_print_num_shape`.
fn print_num_shape(s: &SweepNumShape) {
    int(s.index());
    int(s.kind() as i32);
    flag(s.closed());
    flag(s.beg_infinite());
    flag(s.end_infinite());
    int(s.orientation() as i32);
}

/// `pr_iterate`: up to 5 steps of More, Value and Orientation.
fn iterate(it: &mut SweepNumShapeIterator) {
    for _ in 0..5 {
        flag(it.more());
        if !it.more() {
            return;
        }
        print_num_shape(it.value());
        int(it.orientation() as i32);
        it.next();
    }
}

/// `sw_numshape shape`: the accessors, the tool and the iteration.
pub fn numshape_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let s = num_shape(&mut r);
    print_num_shape(&s);
    let tool = SweepNumShapeTool::new(&s);
    int(tool.nb_shapes());
    int(tool.index(&s));
    int(tool.kind(&s) as i32);
    int(tool.orientation(&s) as i32);
    for i in 0..=tool.nb_shapes() + 1 {
        let sub = tool.shape(i);
        print_num_shape(&sub);
        int(tool.index(&sub));
    }
    flag(tool.has_first_vertex());
    flag(tool.has_last_vertex());
    show(tool.first_vertex(), |v| print_num_shape(&v));
    show(tool.last_vertex(), |v| print_num_shape(&v));
    let mut it = SweepNumShapeIterator::new();
    it.init(&s);
    iterate(&mut it);
    end();
}

/// `sw_reinit first second steps`: Init on `first`, `steps` Next calls, Init on `second`.
pub fn reinit_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let first = num_shape(&mut r);
    let second = num_shape(&mut r);
    let steps = r.integer();
    let mut it = SweepNumShapeIterator::new();
    it.init(&first);
    for _ in 0..steps {
        it.next();
    }
    it.init(&second);
    flag(it.more());
    print_num_shape(it.value());
    iterate(&mut it);
    end();
}

/// `sw_init shape new`: Init of an existing shape and the default constructors.
pub fn init_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let mut s = num_shape(&mut r);
    let n = num_shape(&mut r);
    s.init(
        n.index(),
        n.kind(),
        n.closed(),
        n.beg_infinite(),
        n.end_infinite(),
    );
    print_num_shape(&s);
    print_num_shape(&SweepNumShape::default());
    let it = SweepNumShapeIterator::new();
    flag(it.more());
    print_num_shape(it.value());
    end();
}
