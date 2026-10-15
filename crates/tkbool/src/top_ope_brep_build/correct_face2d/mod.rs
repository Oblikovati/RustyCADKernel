// Rust translation of ModelingAlgorithms/TKBool/TopOpeBRepBuild/TopOpeBRepBuild_CorrectFace2d.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TopOpeBRepBuild_CorrectFace2d`: the p-curves of the wires of a face on a periodic
//! surface moved by periods so that each wire is connected in 2d and the wires lie in
//! one period of the outer wire.
mod moves;

use super::vertex_info::TopOpeBRepBuildTools2d;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Edge, Face, Shape, Wire};
use rustycad_tkbrep::toptools::ShapeMapHasher;
use rustycad_tkernel::ncollection::{IndexedDataMap, IndexedMap, List};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::gp::Point2;

/// The 2d corrections of the edges: the original edges and their corrected copies.
pub type MapOfTrans2dInfo = IndexedDataMap<Shape, Shape, ShapeMapHasher>;

/// The 2d corrector of a face (`TopOpeBRepBuild_CorrectFace2d`), with the map of the
/// corrected edges it fills (natively a pointer to a map of the caller);
/// `let mut cf = TopOpeBRepBuildCorrectFace2d::with_face(&face, &avoid, &mut map);`.
pub struct TopOpeBRepBuildCorrectFace2d<'a> {
    my_face: Face,
    my_corrected_face: Face,
    my_is_done: bool,
    my_error_status: i32,
    my_face_tolerance: f64,
    my_current_wire: Wire,
    my_ordered_wire_list: List<Shape>,
    my_avoid_map: IndexedMap<Shape>,
    my_map: Option<&'a mut MapOfTrans2dInfo>,
    my_copy_face: Face,
    my_copy_avoid_map: IndexedMap<Shape>,
    my_ed_map_inversed: IndexedDataMap<Shape, Shape, ShapeMapHasher>,
}

/// Standard_NullObject of the missing map (a null dereference natively).
fn no_map() -> Failure {
    Failure::with_type(
        "Standard_NullObject",
        "TopOpeBRepBuild_CorrectFace2d: no map of the 2d corrections",
        "TopOpeBRepBuild_CorrectFace2d",
    )
}

impl Default for TopOpeBRepBuildCorrectFace2d<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> TopOpeBRepBuildCorrectFace2d<'a> {
    /// `TopOpeBRepBuild_CorrectFace2d()`: not done, status 1, no face nor map;
    /// `TopOpeBRepBuildCorrectFace2d::new()`.
    pub fn new() -> Self {
        Self {
            my_face: Face::default(),
            my_corrected_face: Face::default(),
            my_is_done: false,
            my_error_status: 1,
            my_face_tolerance: 0.,
            my_current_wire: Wire::default(),
            my_ordered_wire_list: List::default(),
            my_avoid_map: IndexedMap::default(),
            my_map: None,
            my_copy_face: Face::default(),
            my_copy_avoid_map: IndexedMap::default(),
            my_ed_map_inversed: IndexedDataMap::default(),
        }
    }

    /// `TopOpeBRepBuild_CorrectFace2d(aFace, anAvoidMap, aMap)`: the face, the shapes
    /// not to move and the map of the corrections;
    /// `TopOpeBRepBuildCorrectFace2d::with_face(&face, &avoid, &mut map)`.
    pub fn with_face(
        a_face: &Face,
        an_avoid_map: &IndexedMap<Shape>,
        a_map: &'a mut MapOfTrans2dInfo,
    ) -> Self {
        let mut cf = Self::new();
        (cf.my_face, cf.my_avoid_map, cf.my_map) =
            (a_face.clone(), an_avoid_map.clone(), Some(a_map));
        cf
    }

    /// `SetMapOfTrans2dInfo(aMap)`; `cf.set_map_of_trans2d_info(&mut map)`.
    pub fn set_map_of_trans2d_info(&mut self, a_map: &'a mut MapOfTrans2dInfo) {
        self.my_map = Some(a_map);
    }

    /// `MapOfTrans2dInfo()`: Standard_NullObject without map; `cf.map_of_trans2d_info()?`.
    pub fn map_of_trans2d_info(&mut self) -> Result<&mut MapOfTrans2dInfo, Failure> {
        self.my_map.as_deref_mut().ok_or_else(no_map)
    }

    /// `Face()`; `cf.face()`.
    pub fn face(&self) -> &Face {
        &self.my_face
    }

    /// `IsDone()`; `cf.is_done()`.
    pub fn is_done(&self) -> bool {
        self.my_is_done
    }

    /// `ErrorStatus()`: 0 corrected, 1 not performed, 3 a wire not connectable, 4 no
    /// correction needed or possible, 5 no outer wire, 6 a wire not connected after a
    /// try; `cf.error_status()`.
    pub fn error_status(&self) -> i32 {
        self.my_error_status
    }

    /// `CorrectedFace()`; `cf.corrected_face()`.
    pub fn corrected_face(&self) -> &Face {
        &self.my_corrected_face
    }

    /// The copy of an edge (with its vertices) for `BuildCopyData`.
    fn copied_edge(bb: &BRepBuilder, e: &Shape) -> Result<Shape, Failure> {
        let mut an_edge = topods::edge(&e.empty_copied()?)?.into_shape();
        for v in Explorer::new(e, ShapeEnum::Vertex, ShapeEnum::Shape)? {
            bb.add(&mut an_edge, &v)?;
        }
        Ok(an_edge)
    }

    /// The copy of the edge `e` of a wire for `BuildCopyData`: from the copies made,
    /// else the corrected one of the map, else a new copy.
    fn copy_wire_edge(
        &mut self,
        e: &Shape,
        ed_map: &mut MapOfTrans2dInfo,
        use_pmap: bool,
    ) -> Result<Shape, Failure> {
        let bb = BRepBuilder::default();
        if let Ok(c) = ed_map.find_from_key(e) {
            return Ok(c.oriented(e.orientation()));
        }
        if use_pmap {
            let pmap = self.my_map.as_deref_mut().ok_or_else(no_map)?;
            if let Ok(c) = pmap.change_from_key(e) {
                c.set_orientation(e.orientation());
                let c = c.clone();
                ed_map.add(e.clone(), c.clone());
                return Ok(c);
            }
        }
        let mut an_edge = Self::copied_edge(&bb, e)?;
        ed_map.add(e.clone(), an_edge.clone());
        an_edge.set_orientation(e.orientation());
        Ok(an_edge)
    }

    /// `BuildCopyData(F, anAvoidMap, aCopyFace, aCopyAvoidMap, aNeedToUsePMap)`
    /// (private): a copy of the face with copies of its wires and edges (the corrected
    /// edges of the map when asked), the copies of the shapes to avoid, and (when
    /// asked) the originals of the copies.
    fn build_copy_data(
        &mut self,
        a_face: &Face,
        an_avoid_map: &IndexedMap<Shape>,
        use_pmap: bool,
    ) -> Result<(Face, IndexedMap<Shape>), Failure> {
        let mut ed_map = MapOfTrans2dInfo::default();
        let bb = BRepBuilder::default();
        let mut a_copy_face = topods::face(&a_face.empty_copied()?)?.into_shape();
        for w in Explorer::new(a_face, ShapeEnum::Wire, ShapeEnum::Shape)? {
            let mut a_wire = topods::wire(&w.empty_copied()?)?.into_shape();
            for e in Explorer::new(&w, ShapeEnum::Edge, ShapeEnum::Shape)? {
                let copy = self.copy_wire_edge(&e, &mut ed_map, use_pmap)?;
                bb.add(&mut a_wire, &copy)?;
            }
            a_wire.set_orientation(w.orientation());
            ed_map.add(w.clone(), a_wire.clone());
            bb.add(&mut a_copy_face, &a_wire)?;
        }
        let mut a_copy_avoid_map = IndexedMap::default();
        for a_sh in an_avoid_map.iter() {
            if let Ok(c) = ed_map.change_from_key(a_sh) {
                c.set_orientation(a_sh.orientation());
                a_copy_avoid_map.add(c.clone());
            }
        }
        if use_pmap {
            self.my_ed_map_inversed.clear(false);
            for i in 1..=ed_map.extent() {
                let (k, v) = (
                    ed_map.find_key(i)?.clone(),
                    ed_map.find_from_index(i)?.clone(),
                );
                self.my_ed_map_inversed.add(v, k);
            }
        }
        Ok((topods::face(&a_copy_face)?, a_copy_avoid_map))
    }

    /// The wires of the copy with no shape to avoid get their first edge avoided.
    fn avoid_void_wires(&mut self) -> Result<(), Failure> {
        let mut a_void_wires: IndexedMap<Shape> = IndexedMap::default();
        for a_wire in Explorer::new(&self.my_copy_face, ShapeEnum::Wire, ShapeEnum::Shape)? {
            if self.my_copy_avoid_map.contains(&a_wire) {
                continue;
            }
            let mut exists = false;
            for an_edge in Explorer::new(&a_wire, ShapeEnum::Edge, ShapeEnum::Shape)? {
                if self.my_copy_avoid_map.contains(&an_edge) {
                    exists = true;
                    break;
                }
            }
            if !exists {
                a_void_wires.add(a_wire);
            }
        }
        for w in a_void_wires.iter() {
            let mut a_em: IndexedMap<Shape, ShapeMapHasher> = IndexedMap::default();
            rustycad_tkbrep::topexp::map_shapes_of_type(w, ShapeEnum::Edge, &mut a_em)?;
            if a_em.extent() > 0 {
                self.my_copy_avoid_map.add(a_em.find_key(1)?.clone());
            }
        }
        Ok(())
    }

    /// `CheckFace()` (private): done with status 4 unless the surface is periodic in
    /// exactly one direction; status 3 for a wire without path, 6 for a wire not
    /// connectable on a trial copy.
    fn check_face(&mut self) -> Result<(), Failure> {
        let surf = BRepTool::surface(&self.my_copy_face)?;
        let s = surf
            .get()
            .ok_or_else(|| Failure::new("TopOpeBRepBuild_CorrectFace2d: null surface"))?;
        if s.is_u_periodic() == s.is_v_periodic() {
            (self.my_is_done, self.my_error_status) = (true, 4);
            return Ok(());
        }
        self.avoid_void_wires()?;
        for w in Explorer::new(&self.my_copy_face, ShapeEnum::Wire, ShapeEnum::Shape)? {
            self.my_current_wire = topods::wire(&w)?;
            if self.make_right_wire()? != 0 {
                (self.my_is_done, self.my_error_status) = (true, 3);
                return Ok(());
            }
        }
        let (copy_face, copy_avoid) = (self.my_copy_face.clone(), self.my_copy_avoid_map.clone());
        let (a_copy_face, a_copy_avoid_map) =
            self.build_copy_data(&copy_face, &copy_avoid, false)?;
        for w in Explorer::new(&a_copy_face, ShapeEnum::Wire, ShapeEnum::Shape)? {
            self.my_current_wire = topods::wire(&w)?;
            if !a_copy_avoid_map.contains(&w)
                && self.connect_wire(&a_copy_face, &a_copy_avoid_map, true)? != 0
            {
                (self.my_is_done, self.my_error_status) = (true, 6);
                return Ok(());
            }
        }
        Ok(())
    }

    /// `Perform()`: the face copied (with the corrected edges of the map), checked,
    /// each wire connected, then the wires moved into the period of the outer wire;
    /// `cf.perform()?`.
    pub fn perform(&mut self) -> Result<(), Failure> {
        self.my_copy_avoid_map.clear(false);
        let (face, avoid) = (self.my_face.clone(), self.my_avoid_map.clone());
        (self.my_copy_face, self.my_copy_avoid_map) = self.build_copy_data(&face, &avoid, true)?;
        self.my_corrected_face = self.my_copy_face.clone();
        self.my_face_tolerance = BRepTool::face_tolerance(&self.my_copy_face)?;
        self.check_face()?;
        if self.my_is_done {
            return Ok(());
        }
        let (copy_face, copy_avoid) = (self.my_copy_face.clone(), self.my_copy_avoid_map.clone());
        for w in Explorer::new(&copy_face, ShapeEnum::Wire, ShapeEnum::Shape)? {
            self.my_current_wire = topods::wire(&w)?;
            if !copy_avoid.contains(&w) && self.connect_wire(&copy_face, &copy_avoid, false)? != 0 {
                (self.my_is_done, self.my_error_status) = (false, 3);
                return Ok(());
            }
        }
        let Some(an_outer_wire) = self.outer_wire()? else {
            (self.my_is_done, self.my_error_status) = (false, 5);
            return Ok(());
        };
        self.move_wires2d(&an_outer_wire)?;
        (self.my_is_done, self.my_error_status) = (true, 0);
        Ok(())
    }

    /// `MakeRightWire()` (private): the edges of the current wire in path order (1 with
    /// status 4 when the path misses edges).
    fn make_right_wire(&mut self) -> Result<i32, Failure> {
        self.my_ordered_wire_list.clear();
        let n =
            Explorer::new(&self.my_current_wire, ShapeEnum::Edge, ShapeEnum::Shape)?.count() as i32;
        let a_l = TopOpeBRepBuildTools2d::path(&self.my_current_wire)?;
        if a_l.extent() != n {
            self.my_error_status = 4;
            return Ok(1);
        }
        let mut a_fl = List::default();
        for s in a_l.iter() {
            a_fl.prepend(s.clone());
        }
        self.my_ordered_wire_list = a_fl;
        Ok(0)
    }

    /// `GetP2dFL(aF, anEdge, P2dF, P2dL)`: the 2d points of the first and last vertices
    /// of the oriented edge on the face; `TopOpeBRepBuildCorrectFace2d::get_p2d_fl(&f, &e)?`.
    pub fn get_p2d_fl(a_f: &Face, an_edge: &Edge) -> Result<(Point2, Point2), Failure> {
        let pc = BRepTool::curve_on_face(an_edge, a_f)?;
        let c2d = pc.pcurve.get().ok_or_else(|| {
            Failure::with_type(
                "Standard_NullObject",
                "GetP2dFL: no p-curve",
                "TopOpeBRepBuild_CorrectFace2d",
            )
        })?;
        let (pf, pl) = (c2d.d0(pc.first)?, c2d.d0(pc.last)?);
        Ok(if an_edge.orientation() == Orientation::Reversed {
            (pl, pf)
        } else {
            (pf, pl)
        })
    }

    /// `MakeHeadList(aFirstEdge, aHeadList)` (private): the ordered edges rotated to
    /// start at `a_first_edge` (or at the first non-degenerated edge for a null one).
    fn make_head_list(&self, a_first_edge: &Shape) -> Result<List<Shape>, Failure> {
        let mut a_fe = a_first_edge.clone();
        let mut head = List::default();
        let mut flag = false;
        for an_edge in self.my_ordered_wire_list.iter() {
            if a_fe.is_null() && !BRepTool::degenerated(&topods::edge(an_edge)?)? {
                a_fe = an_edge.clone();
            }
            flag = flag || an_edge.is_equal(&a_fe);
            if flag {
                head.append(an_edge.clone());
            }
        }
        for an_edge in self.my_ordered_wire_list.iter() {
            if an_edge.is_equal(&a_fe) {
                break;
            }
            head.append(an_edge.clone());
        }
        Ok(head)
    }

    /// `CheckList(aFace, HeadList)`: the list reversed (with its first edge appended)
    /// when the first edge does not lead to the second one;
    /// `cf.check_list(&face, &mut list)?`.
    pub fn check_list(&self, a_face: &Face, head_list: &mut List<Shape>) -> Result<(), Failure> {
        let tol = precision::confusion();
        let mut seq: Vec<Shape> = head_list.iter().cloned().collect();
        let first = seq.first().cloned().ok_or_else(|| {
            Failure::with_type(
                "Standard_OutOfRange",
                "CheckList: empty list",
                "NCollection_Sequence",
            )
        })?;
        let r1 = first.orientation();
        let mut first_check = false;
        if r1 == Orientation::Reversed {
            let (_, mut pb) = Self::get_p2d_fl(a_face, &topods::edge(&first)?)?;
            for s in &seq[1..] {
                let an_edge = topods::edge(s)?;
                let (pf, pl) = Self::get_p2d_fl(a_face, &an_edge)?;
                if pf.distance(pb) < tol {
                    pb = pl;
                    if an_edge.orientation() == Orientation::Forward {
                        first_check = true;
                        break;
                    }
                }
            }
        }
        let first_edge = topods::edge(&first)?;
        let v1r = Explorer::new(&first_edge, ShapeEnum::Vertex, ShapeEnum::Shape)?
            .find(|v| v.orientation() == Orientation::Reversed)
            .unwrap_or_default();
        let second = seq.get(1).cloned().ok_or_else(|| {
            Failure::with_type(
                "Standard_OutOfRange",
                "CheckList: one edge",
                "NCollection_Sequence",
            )
        })?;
        let v2f = Explorer::new(&second, ShapeEnum::Vertex, ShapeEnum::Shape)?
            .find(|v| v.orientation() == Orientation::Forward)
            .unwrap_or_default();
        let dist = BRepTool::pnt(&topods::vertex(&v1r)?)?
            .distance(&BRepTool::pnt(&topods::vertex(&v2f)?)?);
        if ((dist >= tol || dist.is_nan()) && r1 == Orientation::Forward)
            || (!first_check && r1 == Orientation::Reversed)
        {
            seq.push(first_edge.into_shape());
            head_list.clear();
            for s in seq[1..].iter().rev() {
                head_list.append(s.clone());
            }
        }
        Ok(())
    }
}
