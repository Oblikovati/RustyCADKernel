// Rust translation of BRepClass3d_SolidExplorer.hxx and the shape, shell, face and
// tree members of BRepClass3d_SolidExplorer.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepClass3d_SolidExplorer`: the shells, faces, face intersectors and the UB-tree
//! of edges and vertices of a shape, for the solid classifier.
use super::bnd_box_tree::BRepClass3dBndBoxTreeSelectorPoint;
use crate::brep_bnd_lib::BRepBndLib;
use crate::int_curves_face::IntCurvesFaceIntersector;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorSurface;
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Face, Shape, Shell};
use rustycad_tkbrep::toptools::{IndexedMapOfShape, ShapeMapHasher};
use rustycad_tkernel::ncollection::{DataMap, UBTree, UBTreeFiller};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum, State};
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tkmath::gp::{Line3, Point2, Point3};

/// `BRepClass3d_MapOfInter` (`NCollection_DataMap<TopoDS_Shape, void*,
/// TopTools_ShapeMapHasher>` natively): the intersector of each face, owned.
pub type BRepClass3dMapOfInter = DataMap<Shape, IntCurvesFaceIntersector, ShapeMapHasher>;

/// `BRepClass3d_SolidExplorer` (docs/topalgo-design.md section 5): the shape, an
/// `IntCurvesFace_Intersector` per face, the map of the edges and vertices of its
/// faces with their boxes in a UB-tree, the shell and face explorers, and the state of
/// the segment search (`myFirstFace`, `myParamOnEdge`).
///
/// ```
/// use rustycad_tkbrep::topods::Shape;
/// use rustycad_tkmath::gp::Point3;
/// use rustycad_tktopalgo::brep_class3d::BRepClass3dSolidExplorer;
/// // A shape without faces is the whole space: every point is rejected.
/// let explorer = BRepClass3dSolidExplorer::with_shape(&Shape::default())?;
/// assert!(explorer.reject(&Point3::new(1.0, 2.0, 3.0)));
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
pub struct BRepClass3dSolidExplorer {
    pub(super) my_shape: Shape,
    pub(super) my_reject: bool,
    pub(super) my_first_face: i32,
    pub(super) my_param_on_edge: f64,
    my_shell_explorer: Explorer,
    my_face_explorer: Explorer,
    pub(super) my_map_of_inter: BRepClass3dMapOfInter,
    my_tree: UBTree<i32, BoundingBox3>,
    my_map_ev: IndexedMapOfShape,
}

impl Default for BRepClass3dSolidExplorer {
    /// `BRepClass3d_SolidExplorer()`: no shape, rejecting, `myParamOnEdge` 0.
    fn default() -> Self {
        Self {
            my_shape: Shape::default(),
            my_reject: true,
            my_first_face: 0,
            my_param_on_edge: 0.0,
            my_shell_explorer: Explorer::default(),
            my_face_explorer: Explorer::default(),
            my_map_of_inter: BRepClass3dMapOfInter::default(),
            my_tree: UBTree::default(),
            my_map_ev: IndexedMapOfShape::default(),
        }
    }
}

impl BRepClass3dSolidExplorer {
    /// `BRepClass3d_SolidExplorer()`; `BRepClass3dSolidExplorer::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `BRepClass3d_SolidExplorer(S)`: `InitShape(S)`;
    /// `BRepClass3dSolidExplorer::with_shape(&solid)?`.
    pub fn with_shape(shape: &Shape) -> Result<Self, Failure> {
        let mut explorer = Self::default();
        explorer.init_shape(shape)?;
        Ok(explorer)
    }

    /// `InitShape(S)`: an intersector per face (tolerance Precision::Confusion(),
    /// restricted, no boundary tolerances), rejecting only a shape without faces; the
    /// edges (not degenerated) and vertices of the faces that are neither INTERNAL nor
    /// EXTERNAL, skipping such edges too, go into the map and, with their
    /// BRepBndLib::Add boxes, into the UB-tree; `explorer.init_shape(&solid)?`.
    pub fn init_shape(&mut self, shape: &Shape) -> Result<(), Failure> {
        self.my_map_ev.clear(true);
        self.my_tree.clear();
        self.my_shape = shape.clone();
        self.my_first_face = 0;
        self.my_param_on_edge = 0.512345;
        self.my_map_of_inter.clear(true);
        self.my_reject = true; //-- case of infinite solid (without any face)
        for face in Explorer::new(shape, ShapeEnum::Face, ShapeEnum::Shape)? {
            let face = topods::face(&face)?;
            let intersector =
                IntCurvesFaceIntersector::new(&face, precision::confusion(), true, false)?;
            self.my_map_of_inter.bind(face.into_shape(), intersector);
            self.my_reject = false; //-- at least one face in the solid
        }
        self.map_boundary_shapes()?;
        self.fill_tree()
    }

    /// The edges and vertices of the faces, except the INTERNAL and EXTERNAL faces and
    /// edges and the degenerated edges (the tree filler avoids them).
    fn map_boundary_shapes(&mut self) -> Result<(), Failure> {
        let boundary = |o: Orientation| o != Orientation::Internal && o != Orientation::External;
        for face in Explorer::new(&self.my_shape, ShapeEnum::Face, ShapeEnum::Shape)? {
            if !boundary(face.orientation()) {
                continue;
            }
            for edge in Explorer::new(&face, ShapeEnum::Edge, ShapeEnum::Shape)? {
                if boundary(edge.orientation()) && !BRepTool::degenerated(&topods::edge(&edge)?)? {
                    topexp::map_shapes(&edge, &mut self.my_map_ev, true, true);
                }
            }
        }
        Ok(())
    }

    /// Fills the UB-tree with the index and BRepBndLib::Add box of each mapped shape.
    fn fill_tree(&mut self) -> Result<(), Failure> {
        let mut filler = UBTreeFiller::new(&mut self.my_tree, None, true);
        for i in 1..=self.my_map_ev.extent() {
            let mut bounds = BoundingBox3::default();
            BRepBndLib::add(self.my_map_ev.find_key(i)?, &mut bounds, true)?;
            filler.add(i as i32, bounds);
        }
        filler.fill();
        Ok(())
    }

    /// `Destroy()`: deletes the face intersectors; `explorer.destroy()`.
    pub fn destroy(&mut self) {
        self.my_map_of_inter.clear(true);
    }

    /// `Reject(P)`: true for a shape without faces (the point is not used);
    /// `explorer.reject(&point)`.
    pub fn reject(&self, _point: &Point3) -> bool {
        self.my_reject
    }

    /// `InitShell()`: starts the exploration of the shells; `explorer.init_shell()?`.
    pub fn init_shell(&mut self) -> Result<(), Failure> {
        self.my_shell_explorer
            .init(&self.my_shape, ShapeEnum::Shell, ShapeEnum::Shape)
    }

    /// `MoreShell()`; `explorer.more_shell()`.
    pub fn more_shell(&self) -> bool {
        self.my_shell_explorer.more()
    }

    /// `NextShell()`; `explorer.next_shell()?`.
    pub fn next_shell(&mut self) -> Result<(), Failure> {
        self.my_shell_explorer.next()
    }

    /// `CurrentShell()`: Standard_TypeMismatch past the last shell (the explorer then
    /// shows the explored shape); `explorer.current_shell()?`.
    pub fn current_shell(&self) -> Result<Shell, Failure> {
        topods::shell(self.my_shell_explorer.current())
    }

    /// `RejectShell(L)`: never; `explorer.reject_shell(&line)`.
    pub fn reject_shell(&self, _line: &Line3) -> bool {
        false
    }

    /// `InitFace()`: starts the exploration of the faces of the current shell;
    /// Standard_TypeMismatch past the last shell; `explorer.init_face()?`.
    pub fn init_face(&mut self) -> Result<(), Failure> {
        let shell = topods::shell(self.my_shell_explorer.current())?;
        self.my_face_explorer
            .init(&shell, ShapeEnum::Face, ShapeEnum::Shape)
    }

    /// `MoreFace()`; `explorer.more_face()`.
    pub fn more_face(&self) -> bool {
        self.my_face_explorer.more()
    }

    /// `NextFace()`; `explorer.next_face()?`.
    pub fn next_face(&mut self) -> Result<(), Failure> {
        self.my_face_explorer.next()
    }

    /// `CurrentFace()`: Standard_TypeMismatch past the last face (the explorer then
    /// shows the shell); `explorer.current_face()?`.
    pub fn current_face(&self) -> Result<Face, Failure> {
        topods::face(self.my_face_explorer.current())
    }

    /// `RejectFace(L)`: never; `explorer.reject_face(&line)`.
    pub fn reject_face(&self, _line: &Line3) -> bool {
        false
    }

    /// `GetFaceSegmentIndex()`: the rank of the face of the last segment;
    /// `explorer.get_face_segment_index()`.
    pub fn get_face_segment_index(&self) -> i32 {
        self.my_first_face
    }

    /// `DumpSegment(P, L, Par, S)`: does nothing (its body is debug-only natively);
    /// `explorer.dump_segment(&p, &line, 1.0, State::In)`.
    pub fn dump_segment(&self, _point: &Point3, _line: &Line3, _par: f64, _state: State) {}

    /// `GetShape()`; `explorer.get_shape()`.
    pub fn get_shape(&self) -> &Shape {
        &self.my_shape
    }

    /// `Intersector(F)`: the intersector of the face (IsSame), to perform with (a
    /// non-const reference from a const member natively); Standard_NoSuchObject for a
    /// face of another shape; `explorer.change_intersector(&face)?`.
    pub fn change_intersector(
        &mut self,
        face: &Face,
    ) -> Result<&mut IntCurvesFaceIntersector, Failure> {
        self.my_map_of_inter.change_find(face)
    }

    /// `Intersector(F)` read only (to pass to `ClassifyUVPoint`, say);
    /// Standard_NoSuchObject for a face of another shape; `explorer.intersector(&face)?`.
    pub fn intersector(&self, face: &Face) -> Result<&IntCurvesFaceIntersector, Failure> {
        self.my_map_of_inter.find(face)
    }

    /// `GetTree()`: the UB-tree of the edge and vertex boxes; `explorer.get_tree()`.
    pub fn get_tree(&self) -> &UBTree<i32, BoundingBox3> {
        &self.my_tree
    }

    /// `GetMapEV()`: the edges and vertices indexed by the tree; `explorer.get_map_ev()`.
    pub fn get_map_ev(&self) -> &IndexedMapOfShape {
        &self.my_map_ev
    }

    /// Protected `ClassifyUVPoint(theIntersector, theSurf, theP2d)`: ON when the
    /// surface point lies in the tolerance zone of an edge or vertex of the tree, else
    /// the intersector's classification; `explorer.classify_uv_point(&inter, &surf, p)?`.
    pub fn classify_uv_point(
        &self,
        intersector: &IntCurvesFaceIntersector,
        surface: &BRepAdaptorSurface,
        point: Point2,
    ) -> Result<State, Failure> {
        // first find if the point is near an edge/vertex
        let p3d = surface.value(point.x(), point.y())?;
        if self.selects_point(p3d)? {
            return Ok(State::On);
        }
        intersector.classify_uv_point(point)
    }

    /// Whether the point lies in the tolerance zone of an edge or vertex of the tree.
    pub(super) fn selects_point(&self, point: Point3) -> Result<bool, Failure> {
        let mut selector = BRepClass3dBndBoxTreeSelectorPoint::new(&self.my_map_ev);
        selector.set_current_point(point);
        let selected: i32 = self.my_tree.select(&mut selector);
        if let Some(failure) = selector.take_failure() {
            return Err(failure);
        }
        Ok(selected > 0)
    }

    /// `Segment(P, L, Par)`: the first segment, from the first face; see
    /// [`BRepClass3dSolidExplorer::other_segment`]; `explorer.segment(&p, &mut line, &mut par)?`.
    pub fn segment(
        &mut self,
        point: &Point3,
        line: &mut Line3,
        par: &mut f64,
    ) -> Result<i32, Failure> {
        self.my_first_face = 0;
        self.other_segment(point, line, par)
    }
}
