//! Scripted shape graphs and address-free descriptors, the Rust twin of
//! tools/oracle/tkbrep_probe_graph.cxx.
use super::common::{Reader, failure, int, word};
use rustycad_tkbrep::topods::{
    Builder, CompSolid, Compound, Shape, Shell, Solid, TEdge, TFace, TShape, TShapeBase, TVertex,
    Wire,
};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::gp::{Axis1, Direction3, Point3, Transform3, Vector3};
use rustycad_tkmath::toploc::{Datum3, Location};

/// `ProbeTVertex`: a vertex TShape without geometry.
pub struct ProbeTVertex {
    parent: TVertex,
}
impl_transient!(ProbeTVertex, "ProbeTVertex", TVertex, parent, interfaces: [dyn TShape]);
impl TShape for ProbeTVertex {
    fn tshape_base(&self) -> &TShapeBase {
        self.parent.tshape_base()
    }
    fn empty_copy(&self) -> Result<Handle<dyn TShape>, Failure> {
        Ok(Handle::new(ProbeTVertex {
            parent: TVertex::new(),
        })
        .into_cast())
    }
}

/// `ProbeTEdge`: an edge TShape that inherits `TopoDS_TEdge::EmptyCopy`.
pub struct ProbeTEdge {
    parent: TEdge,
}
impl_transient!(ProbeTEdge, "ProbeTEdge", TEdge, parent, interfaces: [dyn TShape]);
impl TShape for ProbeTEdge {
    fn tshape_base(&self) -> &TShapeBase {
        TShape::tshape_base(&self.parent)
    }
    fn empty_copy(&self) -> Result<Handle<dyn TShape>, Failure> {
        TShape::empty_copy(&self.parent)
    }
}

/// `br_make`: a new shape of the given `TopAbs_ShapeEnum` value, FORWARD at the identity.
pub fn make(kind: i32) -> Shape {
    let builder = Builder;
    let tshape: Handle<dyn TShape> = match kind {
        0 => return made(|s: &mut Compound| builder.make_compound(s)),
        1 => return made(|s: &mut CompSolid| builder.make_comp_solid(s)),
        2 => return made(|s: &mut Solid| builder.make_solid(s)),
        3 => return made(|s: &mut Shell| builder.make_shell(s)),
        4 => Handle::new(TFace::new()).into_cast(),
        5 => return made(|s: &mut Wire| builder.make_wire(s)),
        6 => Handle::new(ProbeTEdge {
            parent: TEdge::new(),
        })
        .into_cast(),
        _ => Handle::new(ProbeTVertex {
            parent: TVertex::new(),
        })
        .into_cast(),
    };
    Shape::from_parts(tshape, Location::default(), Orientation::Forward)
}

fn made<T: Default + Into<Shape>>(make: impl FnOnce(&mut T)) -> Shape {
    let mut typed = T::default();
    make(&mut typed);
    typed.into()
}

pub struct Graph {
    pub locs: Vec<Location>,
    pub datums: Vec<Handle<Datum3>>,
    pub shapes: Vec<Shape>,
}
impl Graph {
    pub fn shape(&self, i: i32) -> Shape {
        usize::try_from(i)
            .ok()
            .and_then(|i| self.shapes.get(i).cloned())
            .unwrap_or_default()
    }
    pub fn loc(&self, i: i32) -> Location {
        usize::try_from(i)
            .ok()
            .and_then(|i| self.locs.get(i).cloned())
            .unwrap_or_default()
    }
    pub fn datum_id(&self, datum: &Handle<Datum3>) -> i32 {
        self.datums
            .iter()
            .position(|d| d == datum)
            .map_or(-1, |i| i as i32)
    }
    pub fn tshape_id(&self, shape: &Shape) -> i32 {
        self.shapes
            .iter()
            .position(|s| !s.is_null() && s.is_partner(shape))
            .map_or(-1, |i| i as i32)
    }
    fn at(&mut self, i: i32) -> &mut Shape {
        &mut self.shapes[i as usize]
    }
}

fn new_datum(g: &mut Graph, t: Transform3) {
    let d = Handle::new(Datum3::new(t));
    g.datums.push(d.clone());
    g.locs.push(Location::from_datum(d));
}

fn read_location(g: &mut Graph, r: &mut Reader<'_>) {
    let kind = r.integer();
    let mut t = Transform3::default();
    match kind {
        0 => {
            let (x, y, z) = (r.real(), r.real(), r.real());
            t.set_translation(Vector3::new(x, y, z));
            return new_datum(g, t);
        }
        1 => {
            let (x, y, z, a) = (r.real(), r.real(), r.real(), r.real());
            let axis = Axis1::new(Point3::default(), Direction3::new(x, y, z).unwrap());
            t.set_rotation(axis, a).unwrap();
            return new_datum(g, t);
        }
        4 => {
            t.set_scale(Point3::default(), r.real()).unwrap();
            return new_datum(g, t);
        }
        _ => {}
    }
    let i = r.integer();
    let result = match kind {
        2 => {
            let j = r.integer();
            g.loc(i).multiplied(&g.loc(j))
        }
        3 => g.loc(i).powered(r.integer()),
        5 => g.loc(i).inverted(),
        _ => Ok(Location::default()),
    };
    let result = result.unwrap_or_else(|e| {
        failure(&e);
        Location::default()
    });
    g.locs.push(result);
}

/// `br_chain`: `count (datum power)...`, head first.
pub fn chain(g: &Graph, l: &Location) {
    let mut items = Vec::new();
    let mut c = l.clone();
    while !c.is_identity() {
        items.push((
            g.datum_id(c.first_datum().unwrap()),
            c.first_power().unwrap(),
        ));
        c = c.next_location().clone();
    }
    int(items.len() as i32);
    for (d, p) in items {
        int(d);
        int(p);
    }
}

/// `br_desc`: `N` or the TShape id, the orientation and the location chain.
pub fn desc(g: &Graph, s: &Shape) {
    if s.is_null() {
        return word("N");
    }
    int(g.tshape_id(s));
    int(s.orientation() as i32);
    chain(g, s.location());
}

fn orientation(o: i32) -> Orientation {
    Orientation::ALL[o as usize]
}

fn child(g: &Graph, r: &mut Reader<'_>) -> Result<Shape, Failure> {
    let (c, o, l) = (r.integer(), r.integer(), r.integer());
    g.shape(c)
        .oriented(orientation(o))
        .located(&g.loc(l), false)
}

fn set_flag(s: &Shape, bit: i32, v: bool) -> Result<(), Failure> {
    match bit {
        0 => s.set_free(v),
        1 => s.set_modified(v),
        2 => s.set_checked(v),
        3 => s.set_orientable(v),
        4 => s.set_closed(v),
        5 => s.set_infinite(v),
        6 => s.set_convex(v),
        _ => s.set_locked(v),
    }
}

fn command(g: &mut Graph, r: &mut Reader<'_>) -> Result<(), Failure> {
    match r.integer() {
        1 => g.shapes.push(make(r.integer())),
        2 => {
            let p = r.integer();
            let c = child(g, r)?;
            Builder.add(g.at(p), &c)?;
        }
        3 => {
            let p = r.integer();
            let c = child(g, r)?;
            Builder.remove(g.at(p), &c)?;
        }
        4 => {
            let (s, o, l) = (r.integer(), r.integer(), r.integer());
            let loc = g.loc(l);
            g.at(s).set_orientation(orientation(o));
            g.at(s).set_location(&loc, false)?;
        }
        5 => {
            let (s, o, l) = (r.integer(), r.integer(), r.integer());
            let moved = g
                .shape(s)
                .oriented(orientation(o))
                .moved(&g.loc(l), false)?;
            g.shapes.push(moved);
        }
        6 => {
            let (s, bit) = (r.integer(), r.integer());
            set_flag(&g.shape(s), bit, r.flag())?;
        }
        7 => {
            let copy = g.shape(r.integer()).empty_copied()?;
            g.shapes.push(copy);
        }
        8 => {
            let (s, l) = (r.integer(), r.integer());
            let loc = g.loc(l);
            g.at(s).set_location(&loc, r.flag())?;
        }
        9 => {
            let (s, l) = (r.integer(), r.integer());
            let loc = g.loc(l);
            g.at(s).move_by(&loc, r.flag())?;
        }
        10 => g.shapes.push(g.shape(r.integer()).reversed()),
        11 => g.shapes.push(g.shape(r.integer()).complemented()),
        12 => {
            let s = r.integer();
            g.shapes.push(g.shape(s).composed(orientation(r.integer())));
        }
        13 => {
            let s = r.integer();
            g.at(s).nullify();
        }
        _ => g.shapes.push(Shape::default()),
    }
    Ok(())
}

/// `br_graph`: the locations, then the commands, each printing `-` or its exception.
pub fn graph(r: &mut Reader<'_>) -> Graph {
    let mut g = Graph {
        locs: vec![Location::default()],
        datums: Vec::new(),
        shapes: Vec::new(),
    };
    for _ in 0..r.integer() {
        read_location(&mut g, r);
    }
    for _ in 0..r.integer() {
        match command(&mut g, r) {
            Ok(()) => word("-"),
            Err(e) => failure(&e),
        }
    }
    g
}
