// Rust translation of ModelingAlgorithms/TKShHealing/ShapeFix/ShapeFix_FaceConnect.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `ShapeFix_FaceConnect`: the free edges of pairs of faces declared connected are sewn,
//! and the shell is rebuilt with the sewn edges (the wires reordered and fixed, the
//! vertices merged).
mod rebuild;
mod sew;
mod vertices;

use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Face, Shape, ShapeIterator, Shell};
use rustycad_tkbrep::toptools::ShapeMapHasher;
use rustycad_tkernel::ncollection::{DataMap, FlatDataMap, List};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;

/// `NCollection_DataMap<TopoDS_Shape, NCollection_List<TopoDS_Shape>, TopTools_ShapeMapHasher>`.
pub(super) type ShapeListMap = DataMap<Shape, List<Shape>, ShapeMapHasher>;

/// `ShapeFix_FaceConnect`: the faces declared connected (`myConnected`), the original
/// free edges of each face (`myOriFreeEdges`) and, for each original free edge, its
/// resulting free (`myResFreeEdges`) and shared (`myResSharEdges`) edges.
///
/// ```
/// use rustycad_tkbrep::topods::{Face, Shell};
/// use rustycad_tkshhealing::shape_fix::ShapeFixFaceConnect;
/// let mut connector = ShapeFixFaceConnect::new();
/// assert!(!connector.add(&Face::default(), &Face::default()));
/// assert!(connector.build(&Shell::default(), 1e-6, 1e-7).unwrap().is_null());
/// ```
#[derive(Default)]
pub struct ShapeFixFaceConnect {
    connected: ShapeListMap,
    ori_free_edges: ShapeListMap,
    res_free_edges: ShapeListMap,
    res_shar_edges: ShapeListMap,
}

/// Whether the list holds a shape the same as `shape`.
fn contains_same(list: &List<Shape>, shape: &Shape) -> bool {
    list.iter().any(|stored| stored.is_same(shape))
}

impl ShapeFixFaceConnect {
    /// `ShapeFix_FaceConnect()`: no connection; `ShapeFixFaceConnect::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `Add(theFirstFace, theSecondFace)`: the two faces declared connected (each in the
    /// list of the other, a face once with itself); false for a null face.
    /// Example: `connector.add(&face1, &face2)`.
    pub fn add(&mut self, first: &Face, second: &Face) -> bool {
        if first.is_null() || second.is_null() {
            return false;
        }
        let list = self
            .connected
            .try_bound(first.clone().into_shape(), List::default());
        if contains_same(list, second.as_ref()) {
            return true;
        }
        list.append(second.clone().into_shape());
        if !first.is_same(second) {
            let other = self
                .connected
                .try_bound(second.clone().into_shape(), List::default());
            other.append(first.clone().into_shape());
        }
        true
    }

    /// `Build(theShell, theSewingTolerance, theFixingTolerance)`: the free edges of the
    /// connected faces sewn pair by pair, then the shell rebuilt with the sewn edges;
    /// the shell itself when no connected face has free edges.
    /// Example: `let result = connector.build(&shell, 1e-6, 1e-7)?;`.
    pub fn build(
        &mut self,
        shell: &Shell,
        sewing_tolerance: f64,
        fixing_tolerance: f64,
    ) -> Result<Shell, Failure> {
        self.collect_free_edges(shell)?;
        self.sew_connected_faces(sewing_tolerance)?;
        self.rebuild_shell(shell, sewing_tolerance, fixing_tolerance)
    }

    /// `Clear()`: no connection and no edge map; `connector.clear()`.
    pub fn clear(&mut self) {
        self.connected.clear(false);
        self.ori_free_edges.clear(false);
        self.res_free_edges.clear(false);
        self.res_shar_edges.clear(false);
    }

    /// `collectFreeEdges(theShell)`: the non-degenerated edges met once in the faces of
    /// the shell, for the connected faces: each one is its own resulting free edge and
    /// has no shared edge yet.
    fn collect_free_edges(&mut self, shell: &Shell) -> Result<(), Failure> {
        self.ori_free_edges.clear(false);
        self.res_free_edges.clear(false);
        self.res_shar_edges.clear(false);
        let free = free_edges(shell)?;
        for (edge, face) in free.items() {
            if self.connected.seek(face).is_none() || BRepTool::degenerated(&topods::edge(edge)?)? {
                continue;
            }
            self.ori_free_edges
                .try_bound(face.clone(), List::default())
                .append(edge.clone());
            let mut own = List::default();
            own.append(edge.clone());
            if self.res_free_edges.try_bind(edge.clone(), own) {
                self.res_shar_edges.try_bind(edge.clone(), List::default());
            }
        }
        Ok(())
    }
}

/// The edges met an odd number of times in the faces of the shell, with their last face.
fn free_edges(shell: &Shell) -> Result<FlatDataMap<Shape, Shape, ShapeMapHasher>, Failure> {
    let mut free = FlatDataMap::default();
    for face in ShapeIterator::new(shell, true, true) {
        for edge in Explorer::new(&face, ShapeEnum::Edge, ShapeEnum::Shape)? {
            if !free.try_bind(edge.clone(), face.clone()) {
                free.unbind(&edge);
            }
        }
    }
    Ok(free)
}
