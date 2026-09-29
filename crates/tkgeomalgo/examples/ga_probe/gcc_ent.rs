//! GccEnt probe operations, the Rust twin of tools/oracle/tkgeomalgo_probe_gccent.cxx.
//! The 2d readers and conic printers below serve the GccInt and GccAna probes too.
use super::common::{Reader, end, flag, int, pnt2d, real, xy};
use rustycad_tkgeomalgo::gcc_ent::{self, Position, QualifiedCirc, QualifiedLin};
use rustycad_tkmath::gp::{
    Axis2d, Axis22d, Circle2, Direction2, Ellipse2, Hyperbola2, Line2, Parabola2,
};

impl Reader<'_> {
    pub fn dir2d(&mut self) -> Direction2 {
        let x = self.real();
        Direction2::new(x, self.real()).expect("probe direction")
    }
    pub fn lin2d(&mut self) -> Line2 {
        let p = self.pnt2d();
        Line2::new(p, self.dir2d())
    }
    /// `x y dx dy radius`: the circle of X axis (x y, dx dy), direct sense.
    pub fn circ2d(&mut self) -> Circle2 {
        let (p, d) = (self.pnt2d(), self.dir2d());
        Circle2::from_x_axis(Axis2d::new(p, d), self.real(), true).expect("probe circle")
    }
}
pub fn dir2d(value: Direction2) {
    xy(value.xy());
}
pub fn ax22d(value: Axis22d) {
    pnt2d(value.location());
    dir2d(value.x_direction());
    dir2d(value.y_direction());
}
pub fn lin2d(value: Line2) {
    pnt2d(value.location());
    dir2d(value.direction());
}
pub fn circ2d(value: Circle2) {
    ax22d(value.axis());
    real(value.radius());
}
pub fn elips2d(value: Ellipse2) {
    ax22d(value.axis());
    real(value.major_radius());
    real(value.minor_radius());
}
pub fn hypr2d(value: Hyperbola2) {
    ax22d(value.axis());
    real(value.major_radius());
    real(value.minor_radius());
}
pub fn parab2d(value: Parabola2) {
    ax22d(value.axis());
    real(value.focal());
}
pub fn position(value: Position) {
    int(value as i32);
}

const NAMES: [&str; 8] = [
    "unqualified",
    "Enclosing",
    "ENCLOSED",
    "outSide",
    "NoQualifier",
    "",
    "bogus",
    "OUTSIDE ",
];

/// `position name_index`: PositionToString, Print, then both PositionFromString overloads.
pub fn position_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let p = Position::ALL[r.integer() as usize];
    let name = NAMES[r.integer() as usize];
    print!("{} ", gcc_ent::position_to_string(p));
    let mut printed: Vec<u8> = Vec::new();
    gcc_ent::print(p, &mut printed).expect("in-memory print");
    print!("{} ", String::from_utf8(printed).expect("ASCII name"));
    let found = gcc_ent::try_position_from_string(name);
    flag(found.is_some());
    position(found.unwrap_or(Position::NoQualifier));
    position(gcc_ent::position_from_string(name));
    end();
}

fn circ_flags(q: QualifiedCirc) {
    position(q.qualifier());
    flag(q.is_unqualified());
    flag(q.is_enclosing());
    flag(q.is_enclosed());
    flag(q.is_outside());
}
fn lin_flags(q: QualifiedLin) {
    position(q.qualifier());
    flag(q.is_unqualified());
    flag(q.is_enclosed());
    flag(q.is_outside());
}

/// `position circle(5) line(4)`: the qualified circle and line, then the seven GccEnt makers.
pub fn qualified_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let p = Position::ALL[r.integer() as usize];
    let (c, l) = (r.circ2d(), r.lin2d());
    let qc = QualifiedCirc::new(c, p);
    circ2d(qc.qualified());
    circ_flags(qc);
    let ql = QualifiedLin::new(l, p);
    lin2d(ql.qualified());
    lin_flags(ql);
    lin_flags(gcc_ent::unqualified_lin(l));
    circ_flags(gcc_ent::unqualified_circ(c));
    circ_flags(gcc_ent::enclosing(c));
    lin_flags(gcc_ent::enclosed_lin(l));
    circ_flags(gcc_ent::enclosed_circ(c));
    lin_flags(gcc_ent::outside_lin(l));
    circ_flags(gcc_ent::outside_circ(c));
    end();
}
