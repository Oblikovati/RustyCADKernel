// Rust translation of ChFi2d_Builder.hxx/.lxx and the initialization, query and history
// members of ChFi2d_Builder.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `ChFi2d_Builder`: the fillets and chamfers of the contour of a planar face, with the
//! history from the original edges to the modified ones.
mod chamfer;
mod chamfer_edge;
mod fillet;
mod fillet_edge;
mod wire;

use super::ConstructionError;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Edge, Face, Shape};
use rustycad_tkbrep::toptools::{IndexedMapOfShape, ShapeMapHasher};
use rustycad_tkernel::ncollection::DataMap;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dTrimmedCurve};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tktopalgo::brep_lib::build_curves3d_default;

/// `ChFi2d_Builder`: the status, the reference face and the modified one, the fillets
/// and chamfers added, and the history (original edge to modified edge).
///
/// ```
/// use rustycad_tkfillet::chfi2d::{ChFi2dBuilder, ConstructionError};
/// let builder = ChFi2dBuilder::new();
/// assert_eq!(builder.status(), ConstructionError::NotPlanar);
/// ```
#[derive(Clone)]
pub struct ChFi2dBuilder {
    status: ConstructionError,
    ref_face: Face,
    new_face: Face,
    fillets: Vec<Shape>,
    chamfers: Vec<Shape>,
    history: DataMap<Shape, Shape, ShapeMapHasher>,
}

impl Default for ChFi2dBuilder {
    fn default() -> Self {
        Self {
            status: ConstructionError::NotPlanar,
            ref_face: Face::default(),
            new_face: Face::default(),
            fillets: Vec::new(),
            chamfers: Vec::new(),
            history: DataMap::default(),
        }
    }
}

impl ChFi2dBuilder {
    /// `ChFi2d_Builder()`: NotPlanar until initialized.
    pub fn new() -> Self {
        Self::default()
    }

    /// `ChFi2d_Builder(F)`: a planar face, its edges given 3d curves; NoFace or
    /// NotPlanar otherwise.
    pub fn with_face(f: &Face) -> Result<Self, Failure> {
        let mut builder = Self::new();
        if f.is_null() {
            builder.status = ConstructionError::NoFace;
            return Ok(builder);
        }
        if !is_planar(f)? {
            return Ok(builder);
        }
        builder.set_faces(f, f);
        build_curves3d_default(&builder.new_face)?;
        builder.status = ConstructionError::Ready;
        Ok(builder)
    }

    /// `Init(F)`: a planar face (the fillets, chamfers and history cleared).
    pub fn init(&mut self, f: &Face) -> Result<(), Failure> {
        if f.is_null() {
            self.status = ConstructionError::NoFace;
            return Ok(());
        }
        self.clear();
        self.status = if is_planar(f)? {
            self.set_faces(f, f);
            ConstructionError::Ready
        } else {
            ConstructionError::NotPlanar
        };
        Ok(())
    }

    /// `Init(RefFace, ModFace)`: a face modified from the planar reference face; the
    /// edges of the modified face issued from reference edges go to the history, the
    /// others are fillets (circles) or chamfers (lines); InitialisationError otherwise.
    pub fn init_modified(&mut self, ref_face: &Face, mod_face: &Face) -> Result<(), Failure> {
        if ref_face.is_null() || mod_face.is_null() {
            self.status = ConstructionError::NoFace;
            return Ok(());
        }
        self.clear();
        if !is_planar(ref_face)? {
            self.status = ConstructionError::NotPlanar;
            return Ok(());
        }
        self.set_faces(ref_face, mod_face);
        self.status = ConstructionError::Ready;
        // Research in newFace all the edges which do not appear in RefFace.
        let mut ref_edges = IndexedMapOfShape::default();
        topexp::map_shapes_of_type(ref_face, ShapeEnum::Edge, &mut ref_edges)?;
        for current in self.new_edges(&ref_edges)? {
            if !self.classify_new_edge(&current, &ref_edges)? {
                self.status = ConstructionError::InitialisationError;
                return Ok(());
            }
        }
        Ok(())
    }

    /// The faces: the reference one and the modified one (FORWARD).
    fn set_faces(&mut self, ref_face: &Face, new_face: &Face) {
        self.ref_face = ref_face.clone();
        self.new_face = new_face.clone();
        self.new_face.set_orientation(Orientation::Forward);
    }

    /// Clears the fillets, the chamfers and the history.
    fn clear(&mut self) {
        self.fillets.clear();
        self.chamfers.clear();
        self.history.clear(true);
    }

    /// The edges of the modified face that are not in the reference face.
    fn new_edges(&self, ref_edges: &IndexedMapOfShape) -> Result<Vec<Edge>, Failure> {
        let mut out = Vec::new();
        let mut ex = Explorer::new(&self.new_face, ShapeEnum::Edge, ShapeEnum::Shape)?;
        while ex.more() {
            let current = topods::edge(ex.current())?;
            if !ref_edges.contains(&current) {
                out.push(current);
            }
            ex.next()?;
        }
        Ok(out)
    }

    /// A new edge goes to the history when issued from a reference edge, to the fillets
    /// when it is a circle, to the chamfers when it is a line; false otherwise.
    fn classify_new_edge(
        &mut self,
        current: &Edge,
        ref_edges: &IndexedMapOfShape,
    ) -> Result<bool, Failure> {
        if let Some(basis) = is_issued_from(current, ref_edges)? {
            self.history
                .bind(basis.into_shape(), current.clone().into_shape());
            return Ok(true);
        }
        // this edge is a chamfer or a fillet
        let curve = BRepTool::curve_with_location(current)?.curve;
        if curve.is_kind_name("Geom_Circle") {
            self.fillets.push(current.clone().into_shape());
        } else if curve.is_kind_name("Geom_Line") {
            self.chamfers.push(current.clone().into_shape());
        } else {
            return Ok(false);
        }
        Ok(true)
    }

    /// `Result()`: the modified face with the orientation of the reference face.
    pub fn result(&self) -> Face {
        let mut face = self.new_face.clone();
        face.set_orientation(self.ref_face.orientation());
        face
    }

    /// `IsModified(E)`: whether the edge has a descendant.
    pub fn is_modified(&self, e: &Edge) -> bool {
        self.history.is_bound(e)
    }

    /// `FilletEdges()`.
    pub fn fillet_edges(&self) -> &[Shape] {
        &self.fillets
    }

    /// `NbFillet()`.
    pub fn nb_fillet(&self) -> i32 {
        self.fillets.len() as i32
    }

    /// `ChamferEdges()`.
    pub fn chamfer_edges(&self) -> &[Shape] {
        &self.chamfers
    }

    /// `NbChamfer()`.
    pub fn nb_chamfer(&self) -> i32 {
        self.chamfers.len() as i32
    }

    /// `HasDescendant(E)`.
    pub fn has_descendant(&self, e: &Edge) -> bool {
        self.history.is_bound(e)
    }

    /// `DescendantEdge(E)`: Standard_NoSuchObject without a descendant.
    pub fn descendant_edge(&self, e: &Edge) -> Result<Edge, Failure> {
        topods::edge(self.history.find(e)?)
    }

    /// `BasisEdge(E)`: the original edge of a modified edge (the edge itself when it
    /// was not modified).
    pub fn basis_edge(&self, e: &Edge) -> Result<Edge, Failure> {
        for (key, value) in self.history.items() {
            if topods::edge(value)?.is_same(e) {
                return topods::edge(key);
            }
        }
        Ok(e.clone())
    }

    /// `Status()`.
    pub fn status(&self) -> ConstructionError {
        self.status
    }

    /// `IsAFillet(E)`.
    fn is_a_fillet(&self, e: &Edge) -> bool {
        self.fillets.iter().any(|f| f.is_same(e))
    }

    /// `IsAChamfer(E)`.
    fn is_a_chamfer(&self, e: &Edge) -> bool {
        self.chamfers.iter().any(|c| c.is_same(e))
    }

    /// Whether either edge is a fillet or a chamfer already.
    fn any_fillet_or_chamfer(&self, e1: &Edge, e2: &Edge) -> bool {
        self.is_a_fillet(e1)
            || self.is_a_chamfer(e1)
            || self.is_a_fillet(e2)
            || self.is_a_chamfer(e2)
    }
}

/// Whether the surface of the face is a Geom_Plane.
fn is_planar(f: &Face) -> Result<bool, Failure> {
    let (surface, _) = BRepTool::surface_with_location(f)?;
    Ok(surface.is_kind_name("Geom_Plane"))
}

/// `IsIssuedFrom(E, Map, BasisEdge)`: the edge of the map with the same 3d curve whose
/// range overlaps an end of the range of `e` (with the orientation of `e`), or `None`.
fn is_issued_from(e: &Edge, map: &IndexedMapOfShape) -> Result<Option<Edge>, Failure> {
    let c1 = BRepTool::curve_with_location(e)?;
    let (f1, l1) = (c1.first, c1.last);
    for i in 1..=map.extent() {
        let current = topods::edge(map.find_key(i)?)?;
        let c2 = BRepTool::curve_with_location(&current)?;
        let (f2, l2) = (c2.first, c2.last);
        let overlaps = (f1 > f2 && f1 < l2)
            || (l1 > f2 && l1 < l2)
            || (f1 > l2 && f1 < f2)
            || (l1 > l2 && l1 < f2);
        if c1.curve == c2.curve && overlaps {
            let mut basis = current;
            basis.set_orientation(e.orientation());
            return Ok(Some(basis));
        }
    }
    Ok(None)
}

/// `IsLineOrCircle(E, F)`: whether the curve of the edge on the face (or the basis of a
/// trimmed one) is a 2d line or circle.
fn is_line_or_circle(e: &Edge, f: &Face) -> Result<bool, Failure> {
    let basis = basis_pcurve(e, f)?;
    Ok(basis.is_instance_name("Geom2d_Circle") || basis.is_instance_name("Geom2d_Line"))
}

/// The curve of the edge on the face, or the basis curve of a trimmed one.
fn basis_pcurve(
    e: &Edge,
    f: &Face,
) -> Result<rustycad_tkernel::standard::Handle<dyn Geom2dCurve>, Failure> {
    let c = BRepTool::curve_on_face(e, f)?.pcurve;
    let trimmed = rustycad_tkernel::standard::Handle::<Geom2dTrimmedCurve>::down_cast(&c);
    match trimmed.get() {
        Some(t) => Ok(t.basis_curve()),
        None => Ok(c),
    }
}
