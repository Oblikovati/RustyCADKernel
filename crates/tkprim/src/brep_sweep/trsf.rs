// Rust translation of BRepSweep_Trsf.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepSweep_Trsf`: the sweeps that move the generating shape with a transformation,
//! sharing its sub-shapes when it is not copied (docs/prim-design.md section 3).
use super::BRepSweepIterator;
use super::num_linear_regular_sweep::BRepSweepNumLinearRegularSweep;
use crate::sweep::{SweepNumShape, SweepNumShapeIterator};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorCurve;
use rustycad_tkbrep::brep_lprop;
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{self, Edge, Shape, Vertex};
use rustycad_tkbrep::toptools::IndexedDataMapOfShapeListOfShape;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::geom_abs::Shape as Continuity;
use rustycad_tkmath::toploc::Location;

/// `ta`: the angular tolerance of the continuity tests, about 0.1 degree ("angular
/// being a little severe for sketched contours").
const CONTINUITY_ANGULAR: f64 = 0.00175;

/// The protected data `BRepSweep_Trsf` adds: the location of the last directing vertex
/// and the copy flag.
///
/// ```
/// use rustycad_tkmath::toploc::Location;
/// use rustycad_tkprim::brep_sweep::BRepSweepTrsfBase;
/// let base = BRepSweepTrsfBase::new(Location::default(), false);
/// assert!(!base.my_copy);
/// ```
#[derive(Clone, Debug, Default)]
pub struct BRepSweepTrsfBase {
    /// `myLocation`: the transformation of the sweep.
    pub my_location: Location,
    /// `myCopy`: whether the generating sub-shapes are copied instead of shared.
    pub my_copy: bool,
}

impl BRepSweepTrsfBase {
    /// The data of `BRepSweep_Trsf(aBuilder, aGenShape, aDirWire, aLocation, aCopy)`
    /// (protected); `BRepSweepTrsfBase::new(location, false)`.
    pub fn new(location: Location, copy: bool) -> Self {
        Self {
            my_location: location,
            my_copy: copy,
        }
    }
}

/// `BRepSweep_Trsf` (docs/prim-design.md section 3): a [`BRepSweepNumLinearRegularSweep`]
/// whose shapes at the directing vertices are the generating shape moved by a location.
/// Implementors provide the location and copy flag; `Init` and `Process` are provided,
/// and their `SetContinuity` is [`trsf_set_continuity`].
pub trait BRepSweepTrsf: BRepSweepNumLinearRegularSweep {
    /// The embedded `BRepSweep_Trsf` data.
    fn trsf_base(&self) -> &BRepSweepTrsfBase;

    /// `Init()`: without copy, shares or moves the generating shapes into the cells of
    /// the directing vertices; `sweep.init()?`.
    fn init(&mut self) -> Result<(), Failure> {
        if self.trsf_base().my_copy {
            return Ok(());
        }
        let (gen_shape, dir_wire) = (
            self.sweep_base().my_gen_shape.clone(),
            self.sweep_base().my_dir_wire,
        );
        let mut it = SweepNumShapeIterator::new();
        it.init(&dir_wire);
        while it.more() {
            self.process(&gen_shape, it.value())?;
            it.next();
        }
        Ok(())
    }

    /// `Process(aGenS, aDirV)`: the cell of `[gen_s, dir_v]` is `gen_s` itself (or moved
    /// at the last vertex of an open sweep) unless a sub-shape was invariant; returns
    /// whether one was; `sweep.process(&shape, &vertex)?`.
    fn process(&mut self, gen_s: &Shape, dir_v: &SweepNumShape) -> Result<bool, Failure> {
        let dotrsf = dir_v.index() == 2 && !self.sweep_base().my_dir_wire.closed();
        let i_d = self.sweep_base().my_dir_shape_tool.index(dir_v);
        let i_g = self.sweep_base().my_gen_shape_tool.index(gen_s);
        if self.is_invariant(gen_s)? {
            self.sweep_base_mut().set_cell(i_g, i_d, gen_s.clone())?;
            self.sweep_base_mut().set_built(i_g, i_d, true)?;
            return Ok(true);
        }
        let mut touch = false;
        let mut jt = BRepSweepIterator::new();
        jt.init(gen_s);
        while jt.more() {
            let sub = jt.value()?.clone();
            touch |= self.process(&sub, dir_v)?;
            jt.next();
        }
        if !touch || !dotrsf {
            let mut new_shape = gen_s.clone();
            if dotrsf {
                new_shape.move_by(&self.trsf_base().my_location, false)?;
            }
            self.sweep_base_mut().set_cell(i_g, i_d, new_shape)?;
            self.sweep_base_mut().set_built(i_g, i_d, true)?;
        }
        Ok(touch)
    }
}

/// `BRepSweep_Trsf::SetContinuity(aGenS, aDirS)`: the continuity of a closed
/// generating edge at its vertex (and of a closed sweep at its seam) on the new edge
/// and face, and of two edges of a generating wire at their common vertex on the new
/// edge between their faces; `trsf_set_continuity(&mut sweep, &edge, &dir_s)?`.
pub fn trsf_set_continuity<T: BRepSweepTrsf + ?Sized>(
    s: &mut T,
    gen_s: &Shape,
    dir_s: &SweepNumShape,
) -> Result<(), Failure> {
    match gen_s.shape_type()? {
        ShapeEnum::Edge => edge_continuity(s, gen_s, dir_s),
        ShapeEnum::Wire => wire_continuity(s, gen_s, dir_s),
        _ => Ok(()),
    }
}

/// `tol3d`: twice the vertex tolerance, at least Precision::Confusion() (IFV 24.05.00
/// buc60684).
fn vertex_tolerance_3d(vertex: &Vertex) -> Result<f64, Failure> {
    Ok(precision::confusion().max(2.0 * BRepTool::vertex_tolerance(vertex)?))
}

/// `BRep_Builder::Continuity(e_wnt, f1_wnt, f2_wnt, cont)` on the new edge and faces
/// of a sweep, when `cont` is at least G1.
fn set_regularity<T: BRepSweepTrsf + ?Sized>(
    s: &mut T,
    edge_of: (&Shape, &SweepNumShape),
    faces_of: [&Shape; 2],
    dir_s: &SweepNumShape,
    cont: Continuity,
) -> Result<(), Failure> {
    if (cont as i32) < (Continuity::G1 as i32) {
        return Ok(());
    }
    let e_wnt = topods::edge(&s.shape_at(edge_of.0, edge_of.1)?)?;
    let f1 = topods::face(&s.shape_at(faces_of[0], dir_s)?)?;
    let f2 = topods::face(&s.shape_at(faces_of[1], dir_s)?)?;
    let builder = *s.sweep_base().my_builder.builder();
    builder.continuity_on_faces(&e_wnt, &f1, &f2, cont)
}

/// The continuity of a generating edge: at its vertex when closed, and at the seam of
/// a closed sweep.
fn edge_continuity<T: BRepSweepTrsf + ?Sized>(
    s: &mut T,
    gen_s: &Shape,
    dir_s: &SweepNumShape,
) -> Result<(), Failure> {
    if !s.has_shape(gen_s, dir_s)? {
        return Ok(());
    }
    let edge = topods::edge(gen_s)?;
    let (d, f) = topexp::vertices(&edge, false);
    if d.is_same(&f) {
        let tol3d = vertex_tolerance_3d(&d)?;
        let e = BRepAdaptorCurve::with_edge(&edge)?;
        let ud = BRepTool::parameter(&d, &edge)?;
        let uf = BRepTool::parameter(&f, &edge)?;
        let cont =
            brep_lprop::continuity_with_tolerances(&e, &e, ud, uf, tol3d, CONTINUITY_ANGULAR)?;
        set_regularity(s, (&d, dir_s), [gen_s, gen_s], dir_s, cont)?;
    }
    if dir_s.closed() {
        seam_continuity(s, gen_s, &edge, dir_s)?;
    }
    Ok(())
}

/// The continuity of a closed sweep at the new edge of its first vertex: the
/// continuity of the 3D curve of the generating edge.
fn seam_continuity<T: BRepSweepTrsf + ?Sized>(
    s: &mut T,
    gen_s: &Shape,
    edge: &Edge,
    dir_s: &SweepNumShape,
) -> Result<(), Failure> {
    let dirv = s.sweep_base().my_dir_shape_tool.shape(2);
    let new_face = s.shape_at(gen_s, dir_s)?;
    let new_edge = s.shape_at(gen_s, &dirv)?;
    if !s.gdd_shape_is_to_add(&new_face, &new_edge, gen_s, (dir_s, &dirv))? {
        return Ok(());
    }
    let found = BRepTool::curve_with_location(edge)?;
    let curve = found.curve.get().ok_or_else(|| {
        Failure::with_message(
            "Standard_NullObject",
            "BRepSweep_Trsf::SetContinuity: the edge has no 3D curve; expected a geometric edge",
        )
    })?;
    let cont = curve.continuity();
    set_regularity(s, (gen_s, &dirv), [gen_s, gen_s], dir_s, cont)
}

/// The continuity of a generating wire: for each vertex, every pair of distinct edges
/// that both sweep a face.
fn wire_continuity<T: BRepSweepTrsf + ?Sized>(
    s: &mut T,
    gen_s: &Shape,
    dir_s: &SweepNumShape,
) -> Result<(), Failure> {
    let mut map = IndexedDataMapOfShapeListOfShape::default();
    topexp::map_shapes_and_ancestors(gen_s, ShapeEnum::Vertex, ShapeEnum::Edge, &mut map)?;
    for i in 1..=map.extent() {
        let vertex = topods::vertex(map.find_key(i)?)?;
        let edges: Vec<Shape> = map.find_from_index(i)?.iter().cloned().collect();
        for (a, first) in edges.iter().enumerate() {
            for second in &edges[a + 1..] {
                pair_continuity(s, &vertex, [first, second], dir_s)?;
            }
        }
    }
    Ok(())
}

/// The continuity of two edges of a generating wire at their common vertex.
fn pair_continuity<T: BRepSweepTrsf + ?Sized>(
    s: &mut T,
    vertex: &Vertex,
    edges: [&Shape; 2],
    dir_s: &SweepNumShape,
) -> Result<(), Failure> {
    let (e1, e2) = (topods::edge(edges[0])?, topods::edge(edges[1])?);
    if e1.is_same(&e2) || !s.has_shape(&e1, dir_s)? || !s.has_shape(&e2, dir_s)? {
        return Ok(());
    }
    let u1 = BRepTool::parameter(vertex, &e1)?;
    let u2 = BRepTool::parameter(vertex, &e2)?;
    let tol3d = vertex_tolerance_3d(vertex)?;
    let c1 = BRepAdaptorCurve::with_edge(&e1)?;
    let c2 = BRepAdaptorCurve::with_edge(&e2)?;
    let cont = brep_lprop::continuity_with_tolerances(&c1, &c2, u1, u2, tol3d, CONTINUITY_ANGULAR)?;
    set_regularity(s, (vertex, dir_s), [&e1, &e2], dir_s, cont)
}
