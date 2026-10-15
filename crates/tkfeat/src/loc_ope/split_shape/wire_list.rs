// Rust translation of LocOpe_SplitShape::Add(Lwires, F) of
// ModelingAlgorithms/TKFeat/LocOpe/LocOpe_SplitShape.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Add(Lwires, F)`: a face cut by section wires running between its boundary wires.
//! The new wires walk the boundary and turn into a section at each of its ends, each
//! section being used once in each direction; the holes the new wires do not touch go
//! to the new face holding them.
use super::statics::{choose_direction, is_inside};
use super::{SplitShape, to_list, to_vec};
use crate::errors::no_such_object;
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::brep_tools::{self, WireExplorer};
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Face, Shape, Vertex, Wire};
use rustycad_tkbrep::toptools::ShapeMapHasher;
use rustycad_tkernel::ncollection::{DataMap, List};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tktopalgo::brep_lib::BRepLibMakeWire;

/// The walk over the boundary of the face and the sections.
struct Walk<'a> {
    face_ref: &'a Face,
    sections_times: DataMap<Shape, i32, ShapeMapHasher>,
    ver_wire_map: DataMap<Shape, Shape, ShapeMapHasher>,
    ver_sec_map: DataMap<Shape, List<Shape>, ShapeMapHasher>,
    break_vertices: Vec<Shape>,
    break_on_wires: Vec<Shape>,
    new_wires: Vec<Shape>,
    cur_wire: Wire,
    wexp: WireExplorer,
    mw: BRepLibMakeWire,
}

/// What the walk along the boundary met.
enum BoundaryEnd {
    /// The new wire closed.
    Closed,
    /// A section starts at the vertex; the section chosen to go on.
    Section(Vertex, Wire),
}

/// The vertex of a wire of `face_ref` that is the same as `v`, and that wire.
fn boundary_wire_of(face_ref: &Face, v: &Vertex) -> Result<Option<(Shape, Shape)>, Failure> {
    for a_wire in Explorer::new(face_ref, ShapeEnum::Wire, ShapeEnum::Shape)? {
        let mut a_ver = Shape::default();
        for x in Explorer::new(&a_wire, ShapeEnum::Vertex, ShapeEnum::Shape)? {
            a_ver = x;
            if a_ver.is_same(v) {
                break;
            }
        }
        if a_ver.is_same(v) {
            return Ok(Some((a_ver, a_wire)));
        }
    }
    Ok(None)
}

impl<'a> Walk<'a> {
    fn new(face_ref: &'a Face, inside: &[Shape]) -> Result<Self, Failure> {
        let mut walk = Self {
            face_ref,
            sections_times: DataMap::default(),
            ver_wire_map: DataMap::default(),
            ver_sec_map: DataMap::default(),
            break_vertices: Vec::new(),
            break_on_wires: Vec::new(),
            new_wires: Vec::new(),
            cur_wire: brep_tools::outer_wire(face_ref)?,
            wexp: WireExplorer::new(),
            mw: BRepLibMakeWire::new(),
        };
        for s in inside {
            walk.sections_times.bind(s.clone(), 2);
        }
        walk.map_section_ends(inside)?;
        let cur_wire = walk.cur_wire.clone();
        walk.wexp.init_on_face(&cur_wire, face_ref)?;
        Ok(walk)
    }

    /// The boundary wire of each section end, and the sections at each end.
    fn map_section_ends(&mut self, inside: &[Shape]) -> Result<(), Failure> {
        for s in inside {
            let (v0, v1) = topexp::wire_vertices(&topods::wire(s)?);
            for v in [&v0, &v1] {
                if self.ver_wire_map.is_bound(v) {
                    continue;
                }
                if let Some((a_ver, a_wire)) = boundary_wire_of(self.face_ref, v)? {
                    self.ver_wire_map.bind(a_ver, a_wire);
                }
            }
        }
        for s in inside {
            let (v1, v2) = topexp::wire_vertices(&topods::wire(s)?);
            for v in [v1, v2] {
                let key = v.into_shape();
                if !self.ver_sec_map.is_bound(&key) {
                    self.ver_sec_map.bind(key.clone(), List::default());
                }
                if let Some(l) = self.ver_sec_map.change_seek(&key) {
                    l.append(s.clone());
                }
            }
        }
        Ok(())
    }

    /// The next edge of the boundary wire, restarting at its first when past the end.
    fn advance(&mut self) -> Result<(), Failure> {
        self.wexp.next()?;
        if !self.wexp.more() {
            let cur_wire = self.cur_wire.clone();
            self.wexp.init_on_face(&cur_wire, self.face_ref)?;
        }
        Ok(())
    }

    /// The explorer on `wire` at the vertex `v`.
    fn restart_at(&mut self, wire: &Shape, v: &Shape) -> Result<(), Failure> {
        self.cur_wire = topods::wire(wire)?;
        let cur_wire = self.cur_wire.clone();
        self.wexp.init_on_face(&cur_wire, self.face_ref)?;
        while !self.wexp.current_vertex().is_same(v) {
            self.wexp.next()?;
        }
        Ok(())
    }

    /// The next new wire starts at the first break vertex left.
    fn resume_at_break(&mut self) -> Result<(), Failure> {
        if self.break_vertices.is_empty() {
            return Err(no_such_object(
                "LocOpe_SplitShape::Add",
                "no break vertex left to resume the walk",
            ));
        }
        let start = self.break_vertices.remove(0);
        let wire = self.break_on_wires.remove(0);
        self.restart_at(&wire, &start)?;
        self.mw = BRepLibMakeWire::new();
        Ok(())
    }

    fn closed(&self) -> Result<bool, Failure> {
        self.mw.wire()?.closed()
    }

    /// Walks the boundary from the current edge until the new wire closes or a section
    /// starts.
    fn walk_boundary(&mut self) -> Result<BoundaryEnd, Failure> {
        let mut last_edge = self.wexp.current().clone();
        self.mw.add_edge(&last_edge)?;
        self.advance()?;
        loop {
            if self.closed()? {
                return Ok(BoundaryEnd::Closed);
            }
            let cur_vertex = self.wexp.current_vertex().clone();
            if let Some(sections) = self.ver_sec_map.seek(&cur_vertex) {
                let chosen = choose_direction(&last_edge, &cur_vertex, self.face_ref, sections)?;
                return Ok(BoundaryEnd::Section(cur_vertex, topods::wire(&chosen)?));
            }
            last_edge = self.wexp.current().clone();
            self.mw.add_edge(&last_edge)?;
            self.advance()?;
        }
    }

    /// Uses the section once more.
    fn consume(&mut self, section: &Shape) -> Result<(), Failure> {
        let times = self.sections_times.change_seek(section).ok_or_else(|| {
            no_such_object(
                "LocOpe_SplitShape::Add",
                "section wire used more than twice",
            )
        })?;
        *times -= 1;
        if *times == 0 {
            self.sections_times.unbind(section);
        }
        Ok(())
    }

    /// Whether the walk goes on along the boundary at `start` rather than into the next
    /// section: the boundary wire holds several section ends and its edge turns less.
    fn boundary_first(
        &self,
        section: &Wire,
        start: &Vertex,
        next_section: &Shape,
    ) -> Result<bool, Failure> {
        let times = self
            .ver_wire_map
            .values()
            .filter(|w| w.is_same(&self.cur_wire))
            .count();
        if times == 1 {
            // it is inner touching wire
            return Ok(false);
        }
        // we have to choose the direction
        let a_start_edge = self.wexp.current().as_ref().clone();
        let mut ldirs = List::default();
        ldirs.append(a_start_edge.clone());
        ldirs.append(next_section.clone());
        let the_direction = choose_direction(section, start, self.face_ref, &ldirs)?;
        Ok(the_direction.is_same(&a_start_edge))
    }

    /// Follows the sections from `section` at `break_vertex`; true when every section
    /// was used twice.
    fn follow_sections(
        &mut self,
        mut section: Wire,
        mut break_vertex: Vertex,
    ) -> Result<bool, Failure> {
        loop {
            self.mw.add_wire(&section)?;
            self.consume(&section)?;
            if self.closed()? {
                self.new_wires.push(self.mw.wire()?.into_shape());
                if self.sections_times.is_empty() {
                    return Ok(true);
                }
                self.resume_at_break()?;
                return Ok(false);
            }
            let (v1, v2) = topexp::wire_vertices(&section);
            let start = if v1.is_same(&break_vertex) { v2 } else { v1 };
            let boundary = self.ver_wire_map.seek(&start).cloned().ok_or_else(|| {
                no_such_object(
                    "LocOpe_SplitShape::Add",
                    "section end not on a boundary wire",
                )
            })?;
            self.restart_at(&boundary, &start)?;
            let lsections = to_vec(self.ver_sec_map.seek(&start).ok_or_else(|| {
                no_such_object("LocOpe_SplitShape::Add", "vertex without section")
            })?);
            if lsections.len() == 1 {
                return Ok(false);
            }
            // else: choose the way
            let next_section = if section.is_same(&lsections[0]) {
                lsections[lsections.len() - 1].clone()
            } else {
                lsections[0].clone()
            };
            if self.boundary_first(&section, &start, &next_section)? {
                return Ok(false);
            }
            section = topods::wire(&next_section)?;
            break_vertex = start;
        }
    }

    /// The global walk, until every section was used in both directions.
    fn run(&mut self) -> Result<(), Failure> {
        loop {
            match self.walk_boundary()? {
                BoundaryEnd::Closed => {
                    self.new_wires.push(self.mw.wire()?.into_shape());
                    self.resume_at_break()?;
                    continue;
                }
                BoundaryEnd::Section(cur_vertex, section) => {
                    self.break_vertices.push(cur_vertex.as_ref().clone());
                    self.break_on_wires.push(self.cur_wire.as_ref().clone());
                    if self.follow_sections(section, cur_vertex)? {
                        return Ok(());
                    }
                }
            }
            if self.sections_times.is_empty() {
                return Ok(());
            }
        }
    }
}

/// The wires of `face_ref` none of whose first edges is in a new wire.
fn holes(face_ref: &Face, new_wires: &[Shape]) -> Result<Vec<Shape>, Failure> {
    let mut holes = Vec::new();
    for a_wire in Explorer::new(face_ref, ShapeEnum::Wire, ShapeEnum::Shape)? {
        let an_edge = Explorer::new(&a_wire, ShapeEnum::Edge, ShapeEnum::Shape)?
            .current()
            .clone();
        let mut found = false;
        for nw in new_wires {
            let mut edges = Explorer::new(nw, ShapeEnum::Edge, ShapeEnum::Shape)?;
            if edges.any(|e| an_edge.is_same(&e)) {
                found = true;
                break;
            }
        }
        if !found {
            holes.push(a_wire);
        }
    }
    Ok(holes)
}

/// The faces on the new wires, each hole given to the first face holding it.
fn new_faces(face_ref: &Face, new_wires: &[Shape]) -> Result<Vec<Shape>, Failure> {
    let bb = BRepBuilder::default();
    let mut faces = Vec::new();
    for w in new_wires {
        let mut new_face = topods::face(&face_ref.empty_copied()?)?;
        new_face.set_orientation(Orientation::Forward);
        bb.add(&mut new_face, w)?;
        faces.push(new_face);
    }
    // Inserting holes
    for hole in holes(face_ref, new_wires)? {
        let a_hole = topods::wire(&hole)?;
        for face in faces.iter_mut() {
            if is_inside(face, &a_hole)? {
                bb.add(face, &a_hole)?;
                break;
            }
        }
    }
    Ok(faces.into_iter().map(|f| f.into_shape()).collect())
}

impl SplitShape {
    /// The first descendant of `f` holding one of the wires and those it holds,
    /// removed from the descendants.
    fn take_face_with_sections(
        &mut self,
        lwires: &List<Shape>,
        f: &Face,
    ) -> Result<Option<(Face, Vec<Shape>)>, Failure> {
        let mut lf = to_vec(self.list_of(f)?);
        for k in 0..lf.len() {
            let fac = topods::face(&lf[k])?;
            let mut inside = Vec::new();
            for w in lwires.iter() {
                if is_inside(&fac, &topods::wire(w)?)? {
                    inside.push(w.clone());
                }
            }
            if !inside.is_empty() {
                let mut face_ref = topods::face(&lf.remove(k))?;
                face_ref.set_orientation(Orientation::Forward);
                *self.list_mut(f)? = to_list(lf);
                return Ok(Some((face_ref, inside)));
            }
        }
        Ok(None)
    }

    /// `Add(Lwires, F)`: cuts the descendant of `f` holding the section wires of
    /// `lwires`; false when none holds one; `split.add_wires(&wires, &f)?`.
    pub fn add_wires(&mut self, lwires: &List<Shape>, f: &Face) -> Result<bool, Failure> {
        if self.done {
            return Ok(false);
        }
        if self.list_of(f)?.is_empty() {
            self.rebuild(f)?;
        }
        // On cherche la face descendante de F qui continent le wire
        brep_tools::update_face(f)?;
        let Some((face_ref, inside)) = self.take_face_with_sections(lwires, f)? else {
            return Ok(false);
        };
        let mut walk = Walk::new(&face_ref, &inside)?;
        walk.run()?;
        let faces = new_faces(&face_ref, &walk.new_wires)?;
        // Update "myMap"
        let lf = self.list_mut(f)?;
        for face in faces {
            lf.append(face);
        }
        // Update of descendants of wires
        for x in Explorer::new(f, ShapeEnum::Wire, ShapeEnum::Shape)? {
            self.list_mut(&x)?.clear();
        }
        // JAG 10.11.95 Codage des regularites
        for w in &inside {
            Self::code_regularities(w, f)?;
        }
        Ok(true)
    }
}
