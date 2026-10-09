// Rust translation of ModelingAlgorithms/TKBool/TopOpeBRepBuild/TopOpeBRepBuild_ShapeSet.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TopOpeBRepBuild_ShapeSet`: the elements (edges of a face, vertices of an edge) of
//! the shapes being built, with the start elements and the neighbours of an element
//! through their sub-shapes. The virtual members overridden by the derived sets
//! (`TopOpeBRepBuild_WireEdgeSet`, ...) are trait methods.
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkbrep::toptools::ShapeMapHasher;
use rustycad_tkernel::ncollection::{IndexedDataMap, IndexedMap, List};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tktopalgo::brep_check::BRepCheckAnalyzer;
use std::io::Write;

/// An iteration over a copy of a list of shapes.
#[derive(Clone, Default)]
pub(crate) struct ShapeCursor {
    items: Vec<Shape>,
    position: usize,
}

impl ShapeCursor {
    /// The cursor at the first shape of the list.
    pub(crate) fn new(l: &List<Shape>) -> Self {
        Self {
            items: l.iter().cloned().collect(),
            position: 0,
        }
    }

    /// The cursor at the first shape of the vector.
    pub(crate) fn from_vec(items: Vec<Shape>) -> Self {
        Self { items, position: 0 }
    }

    /// `More()`.
    pub(crate) fn more(&self) -> bool {
        self.position < self.items.len()
    }

    /// `Next()`.
    pub(crate) fn next(&mut self) {
        self.position += 1;
    }

    /// `Value()`; Standard_NoSuchObject past the end.
    pub(crate) fn value(&self, member: &str) -> Result<&Shape, Failure> {
        self.items.get(self.position).ok_or_else(|| {
            Failure::with_type(
                "Standard_NoSuchObject",
                &format!("{member}: no current shape; expected More()"),
                "TopOpeBRepBuild_ShapeSet",
            )
        })
    }
}

/// The data of a shape set (the protected members of `TopOpeBRepBuild_ShapeSet`).
#[derive(Clone)]
pub struct TopOpeBRepBuildShapeSetBase {
    #[allow(dead_code)] // protected native member read by derived sets
    pub(crate) my_shape_type: ShapeEnum,
    pub(crate) my_sub_shape_type: ShapeEnum,
    pub(crate) my_sub_shape_explorer: ShapeCursor,
    pub(crate) my_start_shapes: List<Shape>,
    pub(crate) my_start_shapes_iter: ShapeCursor,
    pub(crate) my_sub_shape_map: IndexedDataMap<Shape, List<Shape>, ShapeMapHasher>,
    pub(crate) my_incident_shapes_iter: ShapeCursor,
    pub(crate) my_shapes: List<Shape>,
    pub(crate) my_shapes_iter: ShapeCursor,
    pub(crate) my_current_shape: Shape,
    pub(crate) my_current_shape_neighbours: List<Shape>,
    pub(crate) my_deb_number: i32,
    pub(crate) my_deb_name: String,
    pub(crate) my_omss: IndexedMap<Shape>,
    pub(crate) my_omes: IndexedMap<Shape>,
    pub(crate) my_omsh: IndexedMap<Shape>,
    pub(crate) my_check_shape: bool,
}

impl TopOpeBRepBuildShapeSetBase {
    /// `TopOpeBRepBuild_ShapeSet(SubShapeType, checkshape)`: faces of edges or edges of
    /// vertices; Standard_ProgramError for another type; the shape check is always off
    /// ("temporary NYI" natively).
    pub fn new(sub_shape_type: ShapeEnum, _checkshape: bool) -> Result<Self, Failure> {
        let my_shape_type = match sub_shape_type {
            ShapeEnum::Edge => ShapeEnum::Face,
            ShapeEnum::Vertex => ShapeEnum::Edge,
            t => {
                return Err(Failure::with_type(
                    "Standard_ProgramError",
                    &format!("ShapeSet : bad ShapeType {t:?}; expected EDGE or VERTEX"),
                    "TopOpeBRepBuild_ShapeSet",
                ));
            }
        };
        Ok(Self {
            my_shape_type,
            my_sub_shape_type: sub_shape_type,
            my_sub_shape_explorer: ShapeCursor::default(),
            my_start_shapes: List::default(),
            my_start_shapes_iter: ShapeCursor::default(),
            my_sub_shape_map: IndexedDataMap::default(),
            my_incident_shapes_iter: ShapeCursor::default(),
            my_shapes: List::default(),
            my_shapes_iter: ShapeCursor::default(),
            my_current_shape: Shape::default(),
            my_current_shape_neighbours: List::default(),
            my_deb_number: 0,
            my_deb_name: String::new(),
            my_omss: IndexedMap::default(),
            my_omes: IndexedMap::default(),
            my_omsh: IndexedMap::default(),
            my_check_shape: false,
        })
    }

    /// The sub-shapes of the type of the set in a shape.
    pub(crate) fn sub_shapes(&self, s: &Shape) -> Result<Vec<Shape>, Failure> {
        Ok(Explorer::new(s, self.my_sub_shape_type, ShapeEnum::Shape)?.collect())
    }
}

/// A set of shapes and their elements (`TopOpeBRepBuild_ShapeSet`); the derived sets
/// override the virtual members; `ss.init_start_elements(); while ss.more_start_elements() { ss.next_start_element(); }`.
pub trait TopOpeBRepBuildShapeSet {
    /// The data of the set.
    fn shape_set_base(&self) -> &TopOpeBRepBuildShapeSetBase;

    /// The data of the set, changed.
    fn shape_set_base_mut(&mut self) -> &mut TopOpeBRepBuildShapeSetBase;

    /// `AddShape(S)` (virtual): a shape of the set (once), when it passes the check;
    /// `ss.add_shape(&wire)?`.
    fn add_shape(&mut self, s: &Shape) -> Result<(), Failure> {
        if !self.check_shape_of(s, false)? {
            return Ok(());
        }
        self.process_add_shape(s);
        Ok(())
    }

    /// `AddStartElement(S)` (virtual): a start element, also an element;
    /// `ss.add_start_element(&edge)?`.
    fn add_start_element(&mut self, s: &Shape) -> Result<(), Failure> {
        if !self.check_shape_of(s, false)? {
            return Ok(());
        }
        self.process_add_start_element(s)
    }

    /// `AddElement(S)` (virtual): an element, indexed by its sub-shapes;
    /// `ss.add_element(&edge)?`.
    fn add_element(&mut self, s: &Shape) -> Result<(), Failure> {
        if !self.check_shape_of(s, false)? {
            return Ok(());
        }
        self.process_add_element(s)
    }

    /// `ProcessAddShape(S)` (protected): the shape appended unless it is there (with its
    /// orientation).
    fn process_add_shape(&mut self, s: &Shape) {
        let base = self.shape_set_base_mut();
        if !base.my_omsh.contains(s) {
            base.my_omsh.add(s.clone());
            base.my_shapes.append(s.clone());
        }
    }

    /// `ProcessAddStartElement(S)` (protected).
    fn process_add_start_element(&mut self, s: &Shape) -> Result<(), Failure> {
        let base = self.shape_set_base_mut();
        if base.my_omss.contains(s) {
            return Ok(());
        }
        base.my_omss.add(s.clone());
        base.my_start_shapes.append(s.clone());
        self.process_add_element(s)
    }

    /// `ProcessAddElement(S)` (protected): the element appended to the list of each of
    /// its sub-shapes.
    fn process_add_element(&mut self, s: &Shape) -> Result<(), Failure> {
        let subs = self.shape_set_base().sub_shapes(s)?;
        let base = self.shape_set_base_mut();
        if base.my_omes.contains(s) {
            return Ok(());
        }
        base.my_omes.add(s.clone());
        for subshape in subs {
            if !base.my_sub_shape_map.contains(&subshape) {
                base.my_sub_shape_map.add(subshape.clone(), List::default());
            }
            base.my_sub_shape_map
                .change_from_key(&subshape)?
                .append(s.clone());
        }
        Ok(())
    }

    /// `StartElements()`; `ss.start_elements()`.
    fn start_elements(&self) -> &List<Shape> {
        &self.shape_set_base().my_start_shapes
    }

    /// `InitShapes()`; `ss.init_shapes()`.
    fn init_shapes(&mut self) {
        let base = self.shape_set_base_mut();
        base.my_shapes_iter = ShapeCursor::new(&base.my_shapes);
    }

    /// `MoreShapes()`; `ss.more_shapes()`.
    fn more_shapes(&self) -> bool {
        self.shape_set_base().my_shapes_iter.more()
    }

    /// `NextShape()`; `ss.next_shape()`.
    fn next_shape(&mut self) {
        self.shape_set_base_mut().my_shapes_iter.next();
    }

    /// `Shape()`; `ss.shape()?`.
    fn shape(&self) -> Result<&Shape, Failure> {
        self.shape_set_base()
            .my_shapes_iter
            .value("TopOpeBRepBuild_ShapeSet::Shape")
    }

    /// `InitStartElements()`; `ss.init_start_elements()`.
    fn init_start_elements(&mut self) {
        let base = self.shape_set_base_mut();
        base.my_start_shapes_iter = ShapeCursor::new(&base.my_start_shapes);
    }

    /// `MoreStartElements()`; `ss.more_start_elements()`.
    fn more_start_elements(&self) -> bool {
        self.shape_set_base().my_start_shapes_iter.more()
    }

    /// `NextStartElement()`; `ss.next_start_element()`.
    fn next_start_element(&mut self) {
        self.shape_set_base_mut().my_start_shapes_iter.next();
    }

    /// `StartElement()`; `ss.start_element()?`.
    fn start_element(&self) -> Result<&Shape, Failure> {
        self.shape_set_base()
            .my_start_shapes_iter
            .value("TopOpeBRepBuild_ShapeSet::StartElement")
    }

    /// `InitNeighbours(S)` (virtual): the neighbours of `s` through its first sub-shape
    /// having some; `ss.init_neighbours(&edge)?`.
    fn init_neighbours(&mut self, s: &Shape) -> Result<(), Failure> {
        let subs = self.shape_set_base().sub_shapes(s)?;
        let base = self.shape_set_base_mut();
        base.my_sub_shape_explorer = ShapeCursor::from_vec(subs);
        base.my_current_shape = s.clone();
        self.find_neighbours()
    }

    /// `MoreNeighbours()`; `ss.more_neighbours()`.
    fn more_neighbours(&self) -> bool {
        self.shape_set_base().my_incident_shapes_iter.more()
    }

    /// `NextNeighbour()`: the next neighbour, through the next sub-shapes when those of
    /// this one are exhausted; `ss.next_neighbour()?`.
    fn next_neighbour(&mut self) -> Result<(), Failure> {
        let base = self.shape_set_base_mut();
        base.my_incident_shapes_iter.next();
        if !base.my_incident_shapes_iter.more() && base.my_sub_shape_explorer.more() {
            base.my_sub_shape_explorer.next();
            return self.find_neighbours();
        }
        Ok(())
    }

    /// `Neighbour()`; `ss.neighbour()?`.
    fn neighbour(&self) -> Result<&Shape, Failure> {
        self.shape_set_base()
            .my_incident_shapes_iter
            .value("TopOpeBRepBuild_ShapeSet::Neighbour")
    }

    /// `ChangeStartShapes()`; `ss.change_start_shapes().clear()`.
    fn change_start_shapes(&mut self) -> &mut List<Shape> {
        &mut self.shape_set_base_mut().my_start_shapes
    }

    /// `FindNeighbours()` (virtual): from the current sub-shape, the first with
    /// neighbours (MakeNeighboursList); `ss.find_neighbours()?`.
    fn find_neighbours(&mut self) -> Result<(), Failure> {
        while self.shape_set_base().my_sub_shape_explorer.more() {
            let v = self
                .shape_set_base()
                .my_sub_shape_explorer
                .value("TopOpeBRepBuild_ShapeSet::FindNeighbours")?
                .clone();
            let current = self.shape_set_base().my_current_shape.clone();
            let l = self.make_neighbours_list(&current, &v)?;
            let base = self.shape_set_base_mut();
            base.my_incident_shapes_iter = ShapeCursor::new(&l);
            if base.my_incident_shapes_iter.more() {
                break;
            }
            base.my_sub_shape_explorer.next();
        }
        Ok(())
    }

    /// `MakeNeighboursList(E, V)` (virtual): the elements of the sub-shape `v`
    /// (Standard_NoSuchObject for an unknown one); `ss.make_neighbours_list(&e, &v)?`.
    fn make_neighbours_list(&mut self, _e: &Shape, v: &Shape) -> Result<List<Shape>, Failure> {
        Ok(self
            .shape_set_base()
            .my_sub_shape_map
            .find_from_key(v)?
            .clone())
    }

    /// `MaxNumberSubShape(Shape)`: the greatest element count of the known sub-shapes of
    /// the shape; `ss.max_number_sub_shape(&edge)?`.
    fn max_number_sub_shape(&self, shape: &Shape) -> Result<i32, Failure> {
        let base = self.shape_set_base();
        let mut m: i32 = 0;
        for sub in base.sub_shapes(shape)? {
            if let Some(l) = base.my_sub_shape_map.seek(&sub) {
                m = m.max(l.len() as i32);
            }
        }
        Ok(m)
    }

    /// `CheckShape(checkshape)`; `ss.set_check_shape(true)`.
    fn set_check_shape(&mut self, checkshape: bool) {
        self.shape_set_base_mut().my_check_shape = checkshape;
    }

    /// `CheckShape()`; `ss.check_shape()`.
    fn check_shape(&self) -> bool {
        self.shape_set_base().my_check_shape
    }

    /// `CheckShape(S, checkgeom)` (checkgeom natively defaulted to false): the validity
    /// of the shape when the check is on, else true; `ss.check_shape_of(&s, false)?`.
    fn check_shape_of(&self, s: &Shape, checkgeom: bool) -> Result<bool, Failure> {
        if !self.shape_set_base().my_check_shape {
            return Ok(true);
        }
        BRepCheckAnalyzer::new(s, checkgeom, false, false)?.is_valid()
    }

    /// `DumpName(OS, str)`: `str(name,number)`; `ss.dump_name(&mut out, "set")?`.
    fn dump_name(&self, os: &mut dyn Write, s: &str) -> std::io::Result<()> {
        let base = self.shape_set_base();
        write!(os, "{s}({},{})", base.my_deb_name, base.my_deb_number)
    }

    /// `DumpCheck(OS, str, S, chk)`: nothing in a release build; `ss.dump_check(&mut out, "", &s, true)`.
    fn dump_check(&self, _os: &mut dyn Write, _s: &str, _shape: &Shape, _chk: bool) {}

    /// `DumpSS()` (virtual): nothing in a release build; `ss.dump_ss()`.
    fn dump_ss(&mut self) {}

    /// `DumpBB()` (virtual): nothing in a release build; `ss.dump_bb()`.
    fn dump_bb(&mut self) {}

    /// `DEBName(N)`; `ss.set_deb_name("set")`.
    fn set_deb_name(&mut self, n: &str) {
        self.shape_set_base_mut().my_deb_name = n.to_owned();
    }

    /// `DEBName()`; `ss.deb_name()`.
    fn deb_name(&self) -> &str {
        &self.shape_set_base().my_deb_name
    }

    /// `DEBNumber(I)`; `ss.set_deb_number(2)`.
    fn set_deb_number(&mut self, i: i32) {
        self.shape_set_base_mut().my_deb_number = i;
    }

    /// `DEBNumber()`; `ss.deb_number()`.
    fn deb_number(&self) -> i32 {
        self.shape_set_base().my_deb_number
    }

    /// `SName(S, sb, sa)` (virtual): empty in a release build; `ss.sname(&s, "", "")`.
    fn sname(&self, _s: &Shape, _sb: &str, _sa: &str) -> String {
        String::new()
    }

    /// `SNameori(S, sb, sa)` (virtual): empty in a release build; `ss.snameori(&s, "", "")`.
    fn snameori(&self, _s: &Shape, _sb: &str, _sa: &str) -> String {
        String::new()
    }

    /// `SName(L, sb, sa)` (virtual): empty in a release build; `ss.sname_list(&l, "", "")`.
    fn sname_list(&self, _l: &List<Shape>, _sb: &str, _sa: &str) -> String {
        String::new()
    }

    /// `SNameori(L, sb, sa)` (virtual): empty in a release build; `ss.snameori_list(&l, "", "")`.
    fn snameori_list(&self, _l: &List<Shape>, _sb: &str, _sa: &str) -> String {
        String::new()
    }
}

/// The plain shape set (the concrete `TopOpeBRepBuild_ShapeSet`);
/// `let ss = TopOpeBRepBuildPlainShapeSet::new(ShapeEnum::Edge, true)?;`.
#[derive(Clone)]
pub struct TopOpeBRepBuildPlainShapeSet {
    base: TopOpeBRepBuildShapeSetBase,
}

impl TopOpeBRepBuildPlainShapeSet {
    /// `TopOpeBRepBuild_ShapeSet(SubShapeType, checkshape)` (checkshape natively
    /// defaulted to true); `TopOpeBRepBuildPlainShapeSet::new(ShapeEnum::Edge, true)?`.
    pub fn new(sub_shape_type: ShapeEnum, checkshape: bool) -> Result<Self, Failure> {
        Ok(Self {
            base: TopOpeBRepBuildShapeSetBase::new(sub_shape_type, checkshape)?,
        })
    }
}

impl TopOpeBRepBuildShapeSet for TopOpeBRepBuildPlainShapeSet {
    fn shape_set_base(&self) -> &TopOpeBRepBuildShapeSetBase {
        &self.base
    }

    fn shape_set_base_mut(&mut self) -> &mut TopOpeBRepBuildShapeSetBase {
        &mut self.base
    }
}
