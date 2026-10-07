// Rust translation of BRepFilletAPI_MakeFillet2d.hxx/.cxx/.lxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepFilletAPI_MakeFillet2d`: the fillets and chamfers of the vertices of a planar
//! face, as a `BRepBuilderAPI_MakeShape` over `ChFi2d_Builder`.
use crate::chfi2d::{ChFi2dBuilder, ConstructionError};
use rustycad_tkbrep::topods::{self, Edge, Face, Shape, Vertex};
use rustycad_tkernel::message::ProgressRange;
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::Failure;
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakeShape, BRepBuilderAPIMakeShapeBase};

/// `BRepFilletAPI_MakeFillet2d`: edits the contour of a planar face with fillets and
/// chamfers; `build` takes the modified face when the last edit is done.
///
/// ```
/// use rustycad_tkfillet::brep_fillet_api::BRepFilletAPIMakeFillet2d;
/// use rustycad_tkfillet::chfi2d::ConstructionError;
/// let api = BRepFilletAPIMakeFillet2d::new();
/// assert_eq!(api.status(), ConstructionError::NotPlanar);
/// ```
#[derive(Default)]
pub struct BRepFilletAPIMakeFillet2d {
    base: BRepBuilderAPIMakeShapeBase,
    make_chfi2d: ChFi2dBuilder,
}

impl BRepBuilderAPIMakeShape for BRepFilletAPIMakeFillet2d {
    fn make_shape_base(&self) -> &BRepBuilderAPIMakeShapeBase {
        &self.base
    }

    fn make_shape_base_mut(&mut self) -> &mut BRepBuilderAPIMakeShapeBase {
        &mut self.base
    }

    /// `Build()`: done with the modified face when the status is IsDone, else not done.
    fn build(&mut self, _range: &ProgressRange<'_>) -> Result<(), Failure> {
        // test if the operation is done
        if self.status() == ConstructionError::IsDone {
            self.base.set_done();
            self.base.my_shape = self.make_chfi2d.result().into_shape();
        } else {
            self.base.set_not_done();
        }
        Ok(())
    }

    /// `Modified(E)`: the descendant of the edge (Standard_TypeMismatch for a shape that
    /// is not an edge, Standard_NoSuchObject without a descendant).
    fn modified(&mut self, shape: &Shape) -> Result<&List<Shape>, Failure> {
        self.base.my_generated.clear();
        let descendant = self.descendant_edge(&topods::edge(shape)?)?;
        self.base.my_generated.append(descendant.into_shape());
        Ok(&self.base.my_generated)
    }
}

impl BRepFilletAPIMakeFillet2d {
    /// `BRepFilletAPI_MakeFillet2d()`: an empty algorithm;
    /// `BRepFilletAPIMakeFillet2d::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `BRepFilletAPI_MakeFillet2d(F)`: the algorithm on the planar face;
    /// `BRepFilletAPIMakeFillet2d::with_face(&face)?`.
    pub fn with_face(f: &Face) -> Result<Self, Failure> {
        let mut api = Self::new();
        api.init(f)?;
        Ok(api)
    }

    /// `Init(F)`: restart on the planar face; `api.init(&face)?`.
    pub fn init(&mut self, f: &Face) -> Result<(), Failure> {
        self.make_chfi2d.init(f)
    }

    /// `Init(RefFace, ModFace)`: restart on a face already modified from the reference
    /// face; `api.init_modified(&reference, &modified)?`.
    pub fn init_modified(&mut self, ref_face: &Face, mod_face: &Face) -> Result<(), Failure> {
        self.make_chfi2d.init_modified(ref_face, mod_face)
    }

    /// `AddFillet(V, Radius)`: the fillet of the vertex; `api.add_fillet(&v, 1.0)?`.
    pub fn add_fillet(&mut self, v: &Vertex, radius: f64) -> Result<Edge, Failure> {
        self.make_chfi2d.add_fillet(v, radius)
    }

    /// `ModifyFillet(Fillet, Radius)`; `api.modify_fillet(&fillet, 2.0)?`.
    pub fn modify_fillet(&mut self, fillet: &Edge, radius: f64) -> Result<Edge, Failure> {
        self.make_chfi2d.modify_fillet(fillet, radius)
    }

    /// `RemoveFillet(Fillet)`: the restored vertex; `api.remove_fillet(&fillet)?`.
    pub fn remove_fillet(&mut self, fillet: &Edge) -> Result<Vertex, Failure> {
        self.make_chfi2d.remove_fillet(fillet)
    }

    /// `AddChamfer(E1, E2, D1, D2)`: the chamfer of two adjacent edges by two distances;
    /// `api.add_chamfer(&e1, &e2, 1.0, 2.0)?`.
    pub fn add_chamfer(&mut self, e1: &Edge, e2: &Edge, d1: f64, d2: f64) -> Result<Edge, Failure> {
        self.make_chfi2d.add_chamfer(e1, e2, d1, d2)
    }

    /// `AddChamfer(E, V, D, Ang)`: the chamfer at the vertex by a distance on the edge and
    /// an angle; `api.add_chamfer_angle(&e, &v, 1.0, 0.5)?`.
    pub fn add_chamfer_angle(
        &mut self,
        e: &Edge,
        v: &Vertex,
        d: f64,
        ang: f64,
    ) -> Result<Edge, Failure> {
        self.make_chfi2d.add_chamfer_angle(e, v, d, ang)
    }

    /// `ModifyChamfer(Chamfer, E1, E2, D1, D2)`;
    /// `api.modify_chamfer(&chamfer, &e1, &e2, 1.0, 1.0)?`.
    pub fn modify_chamfer(
        &mut self,
        chamfer: &Edge,
        e1: &Edge,
        e2: &Edge,
        d1: f64,
        d2: f64,
    ) -> Result<Edge, Failure> {
        self.make_chfi2d.modify_chamfer(chamfer, e1, e2, d1, d2)
    }

    /// `ModifyChamfer(Chamfer, E, D, Ang)`;
    /// `api.modify_chamfer_angle(&chamfer, &e, 1.0, 0.5)?`.
    pub fn modify_chamfer_angle(
        &mut self,
        chamfer: &Edge,
        e: &Edge,
        d: f64,
        ang: f64,
    ) -> Result<Edge, Failure> {
        self.make_chfi2d.modify_chamfer_angle(chamfer, e, d, ang)
    }

    /// `RemoveChamfer(Chamfer)`: the restored vertex; `api.remove_chamfer(&chamfer)?`.
    pub fn remove_chamfer(&mut self, chamfer: &Edge) -> Result<Vertex, Failure> {
        self.make_chfi2d.remove_chamfer(chamfer)
    }

    /// `IsModified(E)`; `api.is_modified(&e)`.
    pub fn is_modified(&self, e: &Edge) -> bool {
        self.make_chfi2d.is_modified(e)
    }

    /// `FilletEdges()`; `api.fillet_edges().len()`.
    pub fn fillet_edges(&self) -> &[Shape] {
        self.make_chfi2d.fillet_edges()
    }

    /// `NbFillet()`; `api.nb_fillet()`.
    pub fn nb_fillet(&self) -> i32 {
        self.make_chfi2d.nb_fillet()
    }

    /// `ChamferEdges()`; `api.chamfer_edges().len()`.
    pub fn chamfer_edges(&self) -> &[Shape] {
        self.make_chfi2d.chamfer_edges()
    }

    /// `NbChamfer()`; `api.nb_chamfer()`.
    pub fn nb_chamfer(&self) -> i32 {
        self.make_chfi2d.nb_chamfer()
    }

    /// `NbCurves()`: the fillets and the chamfers; `api.nb_curves()`.
    pub fn nb_curves(&self) -> i32 {
        self.nb_fillet() + self.nb_chamfer()
    }

    /// `NewEdges(I)`: the fillet `I` (1-based), then the chamfer `I - NbFillet()`;
    /// Standard_OutOfRange beyond them; `api.new_edges(1)?`.
    pub fn new_edges(&mut self, i: i32) -> Result<&List<Shape>, Failure> {
        self.base.my_generated.clear();
        let (list, index) = if i <= self.nb_fillet() {
            (self.make_chfi2d.fillet_edges(), i)
        } else {
            (self.make_chfi2d.chamfer_edges(), i - self.nb_fillet())
        };
        let edge = usize::try_from(index - 1)
            .ok()
            .and_then(|k| list.get(k))
            .ok_or_else(|| {
                Failure::out_of_range(&format!(
                    "NCollection_Sequence::Value: index {index} not in 1..={}",
                    list.len()
                ))
            })?;
        self.base.my_generated.append(edge.clone());
        Ok(&self.base.my_generated)
    }

    /// `HasDescendant(E)`; `api.has_descendant(&e)`.
    pub fn has_descendant(&self, e: &Edge) -> bool {
        self.make_chfi2d.has_descendant(e)
    }

    /// `DescendantEdge(E)`: Standard_NoSuchObject without a descendant;
    /// `api.descendant_edge(&e)?`.
    pub fn descendant_edge(&self, e: &Edge) -> Result<Edge, Failure> {
        self.make_chfi2d.descendant_edge(e)
    }

    /// `BasisEdge(E)`: the original edge of a modified edge; `api.basis_edge(&e)?`.
    pub fn basis_edge(&self, e: &Edge) -> Result<Edge, Failure> {
        self.make_chfi2d.basis_edge(e)
    }

    /// `Status()`; `api.status()`.
    pub fn status(&self) -> ConstructionError {
        self.make_chfi2d.status()
    }
}
