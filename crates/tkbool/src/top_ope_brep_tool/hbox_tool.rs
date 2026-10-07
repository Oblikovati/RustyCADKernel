// Rust translation of ModelingAlgorithms/TKBool/TopOpeBRepTool/TopOpeBRepTool_HBoxTool.hxx/.cxx
// and TopOpeBRepTool_box.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TopOpeBRepTool_HBoxTool`: the bounding boxes of shapes, indexed in the order they
//! are added, and the `FBOX_*` functions over the process-wide box tool.
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Shape};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::ncollection::IndexedDataMap;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tktopalgo::brep_bnd_lib::BRepBndLib;
use std::sync::OnceLock;

/// The boxes of shapes (`TopOpeBRepTool_HBoxTool`), shared through handles; the shapes
/// are keyed with their orientation (the native default hasher, `IsEqual`);
/// `let hbt = TopOpeBRepToolHBoxTool::new(); hbt.add_box(&face)?;`.
pub struct TopOpeBRepToolHBoxTool {
    parent: StandardTransient,
    my_ims: SharedState<IndexedDataMap<Shape, BoundingBox3>>,
}
impl_transient!(
    TopOpeBRepToolHBoxTool,
    "TopOpeBRepTool_HBoxTool",
    StandardTransient,
    parent
);

impl Default for TopOpeBRepToolHBoxTool {
    /// `TopOpeBRepTool_HBoxTool()`: no box.
    fn default() -> Self {
        Self {
            parent: StandardTransient::default(),
            my_ims: SharedState::new(IndexedDataMap::default()),
        }
    }
}

/// Standard_ProgramError of the box tool.
fn program_error(message: &str) -> Failure {
    Failure::with_type("Standard_ProgramError", message, "TopOpeBRepTool_HBoxTool")
}

impl TopOpeBRepToolHBoxTool {
    /// `TopOpeBRepTool_HBoxTool()`; `TopOpeBRepToolHBoxTool::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `Clear()`; `hbt.clear()`.
    pub fn clear(&self) {
        self.my_ims.write().clear(true);
    }

    /// `AddBoxes(S, TS, TA)` (TA natively defaulted to SHAPE): the box of every
    /// sub-shape of type `ts` not in a `ta`; `hbt.add_boxes(&solid, ShapeEnum::Face, ShapeEnum::Shape)?`.
    pub fn add_boxes(&self, s: &Shape, ts: ShapeEnum, ta: ShapeEnum) -> Result<(), Failure> {
        for ss in Explorer::new(s, ts, ta)? {
            self.add_box(&ss)?;
        }
        Ok(())
    }

    /// `AddBox(S)`: the box of `s` unless it has one; `hbt.add_box(&face)?`.
    pub fn add_box(&self, s: &Shape) -> Result<(), Failure> {
        if self.has_box(s) {
            return Ok(());
        }
        let mut b = BoundingBox3::default();
        Self::compute_box(s, &mut b)?;
        self.my_ims.write().add(s.clone(), b);
        Ok(())
    }

    /// `ComputeBox(S, B)`: BRepBndLib::Add of a face, edge, solid, compound or vertex;
    /// Standard_ProgramError for another type; `TopOpeBRepToolHBoxTool::compute_box(&face, &mut b)?`.
    pub fn compute_box(s: &Shape, b: &mut BoundingBox3) -> Result<(), Failure> {
        match s.shape_type()? {
            ShapeEnum::Face
            | ShapeEnum::Edge
            | ShapeEnum::Solid
            | ShapeEnum::Compound
            | ShapeEnum::Vertex => BRepBndLib::add(s, b, true),
            t => Err(program_error(&format!(
                "HBT::ComputeBox : invalid type {t:?}; expected a face, edge, solid, compound or vertex"
            ))),
        }
    }

    /// `ComputeBoxOnVertices(S, B)`: the box of the vertex points enlarged by their
    /// greatest tolerance (at least Precision::Confusion()), or the box of half side
    /// 1e5 without vertex; `TopOpeBRepToolHBoxTool::compute_box_on_vertices(&face, &mut b)?`.
    pub fn compute_box_on_vertices(s: &Shape, b: &mut BoundingBox3) -> Result<(), Failure> {
        let vertices: Vec<Shape> = Explorer::new(s, ShapeEnum::Vertex, ShapeEnum::Shape)?.collect();
        if vertices.is_empty() {
            b.update_bounds([-1.0e5; 3], [1.0e5; 3]);
            return Ok(());
        }
        let mut tol: f64 = precision::confusion();
        for v in &vertices {
            let vertex = topods::vertex(v)?;
            let p = BRepTool::pnt(&vertex)?;
            b.update_point(p.x(), p.y(), p.z());
            tol = tol.max(BRepTool::vertex_tolerance(&vertex)?);
        }
        b.enlarge(tol);
        Ok(())
    }

    /// `DumpB(B)`: nothing in a release build, as natively; `TopOpeBRepToolHBoxTool::dump_b(&b)`.
    pub fn dump_b(_b: &BoundingBox3) {}

    /// `Box(S)`: the box of `s`; Standard_ProgramError when it has none;
    /// `hbt.box_of_shape(&face)?`.
    pub fn box_of_shape(&self, s: &Shape) -> Result<BoundingBox3, Failure> {
        let ims = self.my_ims.read();
        ims.find_from_key(s).cloned().map_err(|_| {
            program_error("HBT::Box1: the shape has no box; expected a shape added by AddBox")
        })
    }

    /// `Box(I)`: the box of index `i`; Standard_ProgramError out of 1..=Extent;
    /// `hbt.box_of_index(1)?`.
    pub fn box_of_index(&self, i: i32) -> Result<BoundingBox3, Failure> {
        let ims = self.my_ims.read();
        let n = ims.len() as i32;
        if !(1..=n).contains(&i) {
            return Err(program_error(&format!(
                "HBT::Box2: index {i}; expected 1..={n}"
            )));
        }
        ims.find_from_index(i as usize).cloned()
    }

    /// `HasBox(S)`; `hbt.has_box(&face)`.
    pub fn has_box(&self, s: &Shape) -> bool {
        self.my_ims.read().contains(s)
    }

    /// `Shape(I)`: the shape of index `i`; Standard_ProgramError out of 1..=Extent;
    /// `hbt.shape(1)?`.
    pub fn shape(&self, i: i32) -> Result<Shape, Failure> {
        let ims = self.my_ims.read();
        let n = ims.len() as i32;
        if !(1..=n).contains(&i) {
            return Err(program_error(&format!(
                "HBT::Box4: index {i}; expected 1..={n}"
            )));
        }
        ims.find_key(i as usize).cloned()
    }

    /// `Index(S)`: the index of `s`, 0 when it has no box; `hbt.index(&face)`.
    pub fn index(&self, s: &Shape) -> i32 {
        self.my_ims.read().find_index(s) as i32
    }

    /// `Extent()`; `hbt.extent()`.
    pub fn extent(&self) -> i32 {
        self.my_ims.read().len() as i32
    }

    /// `ChangeIMS()`: the map modified by `f`; `hbt.change_ims(|m| m.clear())`.
    pub fn change_ims<R>(
        &self,
        f: impl FnOnce(&mut IndexedDataMap<Shape, BoundingBox3>) -> R,
    ) -> R {
        f(&mut self.my_ims.write())
    }

    /// `IMS()`: a copy of the map of the shapes and their boxes; `hbt.ims()`.
    pub fn ims(&self) -> IndexedDataMap<Shape, BoundingBox3> {
        self.my_ims.read().clone()
    }
}

/// `FBOX_GetHBoxTool()`: the process-wide box tool (a function static natively);
/// `let hbt = fbox_get_hbox_tool();`.
pub fn fbox_get_hbox_tool() -> Handle<TopOpeBRepToolHBoxTool> {
    static GLOBAL_TOOLBOX_HBOXTOOL: OnceLock<Handle<TopOpeBRepToolHBoxTool>> = OnceLock::new();
    GLOBAL_TOOLBOX_HBOXTOOL
        .get_or_init(|| Handle::new(TopOpeBRepToolHBoxTool::new()))
        .clone()
}

/// `FBOX_Prepare()`: clears the process-wide box tool; `fbox_prepare()`.
pub fn fbox_prepare() {
    if let Some(hbt) = fbox_get_hbox_tool().get() {
        hbt.clear();
    }
}

/// `FBOX_Box(S)`: the box of `s` in the process-wide box tool; Standard_ProgramError
/// when it has none; `fbox_box(&face)?`.
pub fn fbox_box(s: &Shape) -> Result<BoundingBox3, Failure> {
    let hbt = fbox_get_hbox_tool();
    hbt.get()
        .ok_or_else(|| program_error("FBOX_Box: no box tool; expected the global box tool"))?
        .box_of_shape(s)
}
