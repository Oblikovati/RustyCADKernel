// Rust translation of ModelingAlgorithms/TKHLR/Contap/Contap_Line.hxx/.lxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Contap_Line: a contour line: a straight line, a circle, a walking line or an arc of
//! restriction, with its vertices ordered by their parameter on the line.
use super::point::ContapPoint;
use super::types::ContapIType;
use rustycad_tkernel::ncollection::HSequence;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkgeomalgo::int_start::TheArc;
use rustycad_tkgeomalgo::int_surf::{IntSurfLineOn2S, IntSurfPntOn2S, IntSurfTypeTrans};
use rustycad_tkmath::gp::{Axis2, Circle3, Direction3, Line3, Point3};

/// Contap_Line: the geometry of a contour line (`TypeContour`), its vertices and its
/// transition on the surface.
///
/// A copy of a line shares its vertex sequence and its walking points with the
/// original, as the native copy shares their handles: a vertex edited through one copy
/// is edited in all of them (docs/hlr-design.md section 6). The native default
/// constructor leaves the type and the radius uninitialized; the Rust default is a
/// walking line of radius 0.
///
/// ```
/// use rustycad_tkhlr::contap::{ContapIType, ContapLine, ContapPoint};
/// use rustycad_tkmath::gp::{Direction3, Line3, Point3};
/// let mut line = ContapLine::new();
/// line.set_line(Line3::new(Point3::new(0.0, 0.0, 0.0), Direction3::X));
/// let mut vertex = ContapPoint::new();
/// vertex.set_parameter(2.0);
/// line.add_vertex(&vertex);
/// vertex.set_parameter(1.0);
/// line.add_vertex(&vertex);
/// assert_eq!(line.type_contour(), ContapIType::Lin);
/// assert_eq!(line.vertex(1).unwrap().parameter_on_line(), 1.0);
/// ```
#[derive(Clone)]
pub struct ContapLine {
    trans: IntSurfTypeTrans,
    curv: Handle<IntSurfLineOn2S>,
    svtx: Handle<HSequence<ContapPoint>>,
    thearc: TheArc,
    typ_l: ContapIType,
    pt: Point3,
    dir1: Direction3,
    dir2: Direction3,
    rad: f64,
}

impl Default for ContapLine {
    fn default() -> Self {
        Self::new()
    }
}

/// The Standard_DomainError of an accessor used on the wrong kind of line.
fn wrong_type(member: &str, expected: ContapIType, found: ContapIType) -> Failure {
    Failure::with_message(
        "Standard_DomainError",
        &format!("Contap_Line::{member}: the line is {found:?}, expected {expected:?}"),
    )
}

impl ContapLine {
    /// Contap_Line(): no vertex, an undecided transition (see the type documentation);
    /// `ContapLine::new()`.
    pub fn new() -> Self {
        Self {
            trans: IntSurfTypeTrans::Undecided,
            curv: Handle::default(),
            svtx: Handle::new(HSequence::default()),
            thearc: TheArc::default(),
            typ_l: ContapIType::Walking,
            pt: Point3::default(),
            dir1: Direction3::default(),
            dir2: Direction3::default(),
            rad: 0.0,
        }
    }

    /// SetLineOn2S: a walking line on the points `l` (shared); `line.set_line_on_2s(points)`.
    pub fn set_line_on_2s(&mut self, l: Handle<IntSurfLineOn2S>) {
        self.curv = l;
        self.typ_l = ContapIType::Walking;
    }

    /// Clear: empties the walking points (shared with the copies), gives the line a new
    /// empty vertex sequence (the copies keep the old one) and makes it a walking line;
    /// `line.clear()`.
    pub fn clear(&mut self) {
        if let Some(curv) = self.curv.get() {
            curv.clear();
        }
        self.svtx = Handle::new(HSequence::default());
        self.typ_l = ContapIType::Walking;
    }

    /// LineOn2S: the walking points (a shared handle, null for other lines);
    /// `line.line_on_2s()`.
    pub fn line_on_2s(&self) -> Handle<IntSurfLineOn2S> {
        self.curv.clone()
    }

    /// ResetSeqOfVertex: a new empty vertex sequence (the copies keep the old one);
    /// `line.reset_seq_of_vertex()`.
    pub fn reset_seq_of_vertex(&mut self) {
        self.svtx = Handle::new(HSequence::default());
    }

    /// Add(IntSurf_PntOn2S): appends a walking point; a line without walking points
    /// dereferences a null handle natively (Standard_NullObject here);
    /// `line.add_point(&p)?`.
    pub fn add_point(&self, p: &IntSurfPntOn2S) -> Result<(), Failure> {
        let curv = self.curv.get().ok_or_else(|| {
            Failure::with_message(
                "Standard_NullObject",
                "Contap_Line::Add: no walking points, expected SetLineOn2S first",
            )
        })?;
        curv.add(p);
        Ok(())
    }

    /// SetValue(gp_Lin); `line.set_line(l)`.
    pub fn set_line(&mut self, l: Line3) {
        self.pt = l.location();
        self.dir1 = l.direction();
        self.typ_l = ContapIType::Lin;
    }

    /// SetValue(gp_Circ); `line.set_circle(c)`.
    pub fn set_circle(&mut self, c: Circle3) {
        self.pt = c.location();
        self.dir1 = c.position().direction();
        self.dir2 = c.position().x_direction();
        self.rad = c.radius();
        self.typ_l = ContapIType::Circle;
    }

    /// SetValue(Adaptor2d_Curve2d): an arc of restriction; `line.set_arc(&arc)`.
    pub fn set_arc(&mut self, a: &TheArc) {
        self.thearc = a.clone();
        self.typ_l = ContapIType::Restriction;
    }

    /// Add(Contap_Point): inserts a copy of `p` after the last vertex whose parameter
    /// is lower (appended when it is the greatest, prepended when none is lower);
    /// `line.add_vertex(&p)`.
    pub fn add_vertex(&self, p: &ContapPoint) {
        self.svtx.change_sequence(|seq| insert_vertex(seq, p));
    }

    /// NbVertex; `line.nb_vertex()`.
    pub fn nb_vertex(&self) -> i32 {
        self.svtx
            .get()
            .map_or(0, |svtx| svtx.sequence(|seq| seq.length()))
    }

    /// Vertex(Index): a copy of the vertex of rank `index`; Standard_OutOfRange outside
    /// `1..=NbVertex()`. The native reference is written through `change_vertex`;
    /// `line.vertex(1)?`.
    pub fn vertex(&self, index: i32) -> Result<ContapPoint, Failure> {
        self.change_vertex(index, |point| point.clone())
    }

    /// The native `Vertex(Index)` used to edit the vertex in place (in every copy of
    /// the line); `line.change_vertex(1, |p| p.set_multiple())?`.
    pub fn change_vertex<R>(
        &self,
        index: i32,
        edit: impl FnOnce(&mut ContapPoint) -> R,
    ) -> Result<R, Failure> {
        self.svtx.change_sequence(|seq| {
            let slot = usize::try_from(index)
                .ok()
                .and_then(|i| seq.change_value(i).ok());
            slot.map(edit).ok_or_else(|| {
                Failure::out_of_range(&format!(
                    "Contap_Line::Vertex: index {index} outside 1..={}",
                    seq.len()
                ))
            })
        })
    }

    /// TypeContour; `line.type_contour()`.
    pub fn type_contour(&self) -> ContapIType {
        self.typ_l
    }

    /// NbPnts: the walking points; Standard_DomainError on another line;
    /// `line.nb_pnts()?`.
    pub fn nb_pnts(&self) -> Result<i32, Failure> {
        self.expect(ContapIType::Walking, "NbPnts")?;
        Ok(self.walking_points("NbPnts")?.nb_points())
    }

    /// Point(Index): a walking point; Standard_DomainError on another line,
    /// Standard_OutOfRange outside the points; `line.point(1)?`.
    pub fn point(&self, index: i32) -> Result<IntSurfPntOn2S, Failure> {
        self.expect(ContapIType::Walking, "Point")?;
        self.walking_points("Point")?.value(index)
    }

    fn walking_points(&self, member: &str) -> Result<&IntSurfLineOn2S, Failure> {
        self.curv.get().ok_or_else(|| {
            Failure::with_message(
                "Standard_NullObject",
                &format!("Contap_Line::{member}: no walking points, expected SetLineOn2S first"),
            )
        })
    }

    /// Line: Standard_DomainError unless the line is straight; `line.line()?`.
    pub fn line(&self) -> Result<Line3, Failure> {
        self.expect(ContapIType::Lin, "Line")?;
        Ok(Line3::new(self.pt, self.dir1))
    }

    /// Circle: Standard_DomainError unless the line is a circle; the frame of the
    /// circle is rebuilt from its axis and X direction; `line.circle()?`.
    pub fn circle(&self) -> Result<Circle3, Failure> {
        self.expect(ContapIType::Circle, "Circle")?;
        let frame = Axis2::with_x_direction(self.pt, self.dir1, self.dir2)?;
        Circle3::new(frame, self.rad)
    }

    /// Arc: Standard_DomainError unless the line is an arc of restriction;
    /// `line.arc()?`.
    pub fn arc(&self) -> Result<TheArc, Failure> {
        self.expect(ContapIType::Restriction, "Arc")?;
        Ok(self.thearc.clone())
    }

    fn expect(&self, expected: ContapIType, member: &str) -> Result<(), Failure> {
        if self.typ_l == expected {
            return Ok(());
        }
        Err(wrong_type(member, expected, self.typ_l))
    }

    /// SetTransitionOnS; `line.set_transition_on_s(IntSurfTypeTrans::In)`.
    pub fn set_transition_on_s(&mut self, t: IntSurfTypeTrans) {
        self.trans = t;
    }

    /// TransitionOnS; `line.transition_on_s()`.
    pub fn transition_on_s(&self) -> IntSurfTypeTrans {
        self.trans
    }
}

// Strict comparison preserves the native reverse ordering of equal parameters.
fn insert_vertex(seq: &mut rustycad_tkernel::ncollection::Sequence<ContapPoint>, p: &ContapPoint) {
    let n = seq.len();
    let parameter = p.parameter_on_line();
    if n == 0 || parameter > seq.value(n).expect("valid last vertex").parameter_on_line() {
        seq.append(p.clone());
        return;
    }
    for i in (1..n).rev() {
        if parameter
            > seq
                .value(i)
                .expect("valid vertex index")
                .parameter_on_line()
        {
            seq.insert_before(i + 1, p.clone())
                .expect("valid insertion index");
            return;
        }
    }
    seq.prepend(p.clone());
}
