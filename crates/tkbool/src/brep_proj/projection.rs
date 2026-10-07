// Rust translation of ModelingAlgorithms/TKBool/BRepProj/BRepProj_Projection.hxx/.cxx/.lxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The projection of a wire on a shape: the wire swept along the direction (a prism)
//! or towards the point (a ruled cone), cut by the shape, the section edges connected
//! into wires.
use crate::brep_fill::BRepFillGenerator;
use rustycad_tkbo::brep_algo_api::BRepAlgoAPISection;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_tools::{BRepToolsModification, Modifier, TrsfModification};
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Compound, Edge, Shape, Wire};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tkmath::gp::{Direction3, Point3, Transform3, Vector3, Xyz};
use rustycad_tkprim::brep_sweep::BRepSweepPrism;
use rustycad_tkshhealing::shape_analysis::ShapeAnalysisFreeBounds;
use rustycad_tktopalgo::brep_bnd_lib::BRepBndLib;
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShape;
use rustycad_tktopalgo::brep_lib::BRepLibMakeVertex;

/// The projection of a wire on a shape (`BRepProj_Projection`);
/// `let p = BRepProjProjection::along(&wire, &shape, Direction3::Z)?;`.
pub struct BRepProjProjection {
    // The native myLsh is never used and is not ported.
    my_is_done: bool,
    my_shape: Compound,
    my_section: Option<Vec<Shape>>,
    my_itr: i32,
}

/// The vector of coordinates.
fn vector_of(x: Xyz) -> Vector3 {
    Vector3::new(x.x(), x.y(), x.z())
}

/// The bounding box of a shape.
fn bounds_of(s: &Shape) -> Result<BoundingBox3, Failure> {
    let mut b = BoundingBox3::default();
    BRepBndLib::add(s, &mut b, true)?;
    Ok(b)
}

/// `DistanceOut(S1, S2)`: the distance between the bounding boxes.
fn distance_out(s1: &Shape, s2: &Shape) -> Result<f64, Failure> {
    Ok(bounds_of(s1)?.distance(&bounds_of(s2)?))
}

/// The diagonal length of a bounding box (Standard_ConstructionError when void).
fn diagonal(b: &BoundingBox3) -> Result<f64, Failure> {
    if b.is_void() {
        return Err(Failure::with_type(
            "Standard_ConstructionError",
            "Bnd_Box is void",
            "Bnd_Box",
        ));
    }
    let min = Xyz::new(b.x_min(), b.y_min(), b.z_min());
    let max = Xyz::new(b.x_max(), b.y_max(), b.z_max());
    Ok((max - min).modulus())
}

/// `DistanceIn(S1, S2)`: the greatest distance between the shapes, the diagonals of
/// their bounding boxes plus the distance between the boxes.
fn distance_in(s1: &Shape, s2: &Shape) -> Result<f64, Failure> {
    let (sbbox, lbbox) = (bounds_of(s1)?, bounds_of(s2)?);
    Ok(diagonal(&lbbox)? + diagonal(&sbbox)? + distance_out(s1, s2)?)
}

/// The checks of the inputs: not null, an edge or a wire to project.
fn check_input(wire: &Shape, shape: &Shape) -> Result<(), Failure> {
    if wire.is_null() || shape.is_null() {
        return Err(Failure::with_type(
            "Standard_NullObject",
            "BRepProj_Projection.cxx: null input shape",
            "BRepProj_Projection",
        ));
    }
    let kind = wire.shape_type()?;
    if kind != ShapeEnum::Edge && kind != ShapeEnum::Wire {
        return Err(Failure::with_type(
            "Standard_ConstructionError",
            "BRepProj_Projection.cxx: projected shape is neither wire nor edge",
            "BRepProj_Projection",
        ));
    }
    Ok(())
}

/// The shape transformed by a copy (to avoid adding new p-curves to the input).
fn transformed_copy(shape: &Shape, t: Transform3) -> Result<Shape, Failure> {
    let modification: Handle<dyn BRepToolsModification> =
        Handle::new(TrsfModification::new(t)).cast();
    Modifier::with_modification(shape, &modification)?.modified_shape(shape)
}

/// The faces of a compound as a compound (the section may refuse e.g. vertices);
/// faces, shells and solids as they are.
fn section_target(the_shape: &Shape) -> Result<Shape, Failure> {
    let kind = the_shape.shape_type()?;
    if matches!(
        kind,
        ShapeEnum::Face | ShapeEnum::Shell | ShapeEnum::Solid | ShapeEnum::CompSolid
    ) {
        return Ok(the_shape.clone());
    }
    let mut c = Compound::default();
    if kind == ShapeEnum::Compound {
        let b = BRepBuilder::default();
        let mut exp = Explorer::new(the_shape, ShapeEnum::Face, ShapeEnum::Shape)?;
        while exp.more() {
            if c.is_null() {
                b.make_compound(&mut c);
            }
            b.add(&mut c, exp.current())?;
            exp.next()?;
        }
    }
    if c.is_null() {
        return Err(Failure::with_type(
            "Standard_ConstructionError",
            "BRepProj_Projection.cxx: target shape has no faces",
            "BRepProj_Projection",
        ));
    }
    Ok(c.into_shape())
}

impl BRepProjProjection {
    /// The state before the section.
    fn empty() -> Self {
        Self {
            my_is_done: false,
            my_shape: Compound::default(),
            my_section: None,
            my_itr: 0,
        }
    }

    /// `BRepProj_Projection(Wire, Shape, D)`: the cylindrical projection along the
    /// direction (the wire moved back and swept over the extent of both shapes).
    pub fn along(wire: &Shape, shape: &Shape, d: Direction3) -> Result<Self, Failure> {
        check_input(wire, shape)?;
        // The "length" of the cylindrical surface to build.
        let mdis: f64 = distance_in(wire, shape)?;
        let vsup = vector_of(Xyz::from(d) * 2. * mdis);
        let vinf = vector_of(Xyz::from(d) * -mdis);
        // The base of the cylindrical surface translated by -mdis.
        let mut t = Transform3::default();
        t.set_translation(vinf);
        let wire_base = transformed_copy(wire, t)?;
        let mut cyl_surf = BRepSweepPrism::new(&wire_base, vsup, false, true)?;
        let tool = cyl_surf.shape()?;
        let mut projection = Self::empty();
        projection.build_section(shape, &tool)?;
        Ok(projection)
    }

    /// `BRepProj_Projection(Wire, Shape, P)`: the conical projection from the point
    /// (the wire scaled from the point beyond the shapes, ruled to the point).
    pub fn from_point(wire: &Shape, shape: &Shape, p: Point3) -> Result<Self, Failure> {
        check_input(wire, shape)?;
        let b = BRepBuilder::default();
        let a_wire: Wire = if wire.shape_type()? == ShapeEnum::Edge {
            let mut w = Wire::default();
            b.make_wire(&mut w);
            b.add(&mut w, wire)?;
            w
        } else {
            topods::wire(wire)?
        };
        // The "length" of the conical surface to build.
        let mdis: f64 = distance_in(wire, shape)?;
        let first = Explorer::new(a_wire.as_ref(), ShapeEnum::Vertex, ShapeEnum::Shape)?;
        let pc: Point3 = BRepTool::pnt(&topods::vertex(first.current())?)?;
        let mut scale: f64 = pc.distance(&p);
        if scale.abs() < precision::confusion() {
            return Err(Failure::with_type(
                "Standard_ConstructionError",
                "Projection",
                "BRepProj_Projection",
            ));
        }
        scale = 1. + mdis / scale;
        let mut t = Transform3::default();
        t.set_scale(p, scale)?;
        let shape_gen1 = transformed_copy(a_wire.as_ref(), t)?;
        let surf_shell = ruled_cone(&shape_gen1, p)?;
        let mut projection = Self::empty();
        projection.build_section(shape, &surf_shell)?;
        Ok(projection)
    }

    /// `BuildSection(Shape, Tool)` (private): the section of the faces of the shape by
    /// the tool (approximated, with p-curves on both, oriented boxes), its edges
    /// connected into wires within Precision::Confusion().
    fn build_section(&mut self, the_shape: &Shape, the_tool: &Shape) -> Result<(), Failure> {
        (self.my_is_done, self.my_section, self.my_shape, self.my_itr) =
            (false, None, Compound::default(), 0);
        let a_shape = section_target(the_shape)?;
        // The section computes the p-curves on both shapes for a higher precision.
        let mut a_section_tool = BRepAlgoAPISection::from_shapes(&a_shape, the_tool, None, false)?;
        a_section_tool.approximation(true);
        a_section_tool.compute_pcurve_on1(true);
        a_section_tool.compute_pcurve_on2(true);
        // Oriented bounding boxes speed up the section.
        a_section_tool.set_use_obb(true);
        a_section_tool.build()?;
        if !a_section_tool.is_done() {
            return Ok(());
        }
        let mut an_edges: Vec<Shape> = Vec::new();
        let mut exp = Explorer::new(a_section_tool.shape(), ShapeEnum::Edge, ShapeEnum::Shape)?;
        while exp.more() {
            an_edges.push(exp.current().clone());
            exp.next()?;
        }
        // No edge: the section yields no result.
        if an_edges.is_empty() {
            return Ok(());
        }
        let section = ShapeAnalysisFreeBounds::connect_edges_to_wires(
            &mut an_edges,
            precision::confusion(),
            true,
        )?;
        self.my_is_done = !section.is_empty();
        if self.my_is_done {
            let b = BRepBuilder::default();
            b.make_compound(&mut self.my_shape);
            for w in &section {
                b.add(&mut self.my_shape, w)?;
            }
            // The iteration initialized, for compatibility with previous versions.
            self.my_itr = 1;
        }
        self.my_section = Some(section);
        Ok(())
    }

    /// `IsDone()`; `p.is_done()`.
    pub fn is_done(&self) -> bool {
        self.my_is_done
    }

    /// `Init()`: the iteration on the wires restarted; `p.init()`.
    pub fn init(&mut self) {
        self.my_itr = 1;
    }

    /// `More()`: whether a wire remains; `p.more()`.
    pub fn more(&self) -> bool {
        self.my_itr > 0
            && self
                .my_section
                .as_ref()
                .is_some_and(|s| self.my_itr as usize <= s.len())
    }

    /// `Next()`; `p.next()`.
    pub fn next(&mut self) {
        self.my_itr += 1;
    }

    /// `Current()`: the current wire; Standard_OutOfRange past the end (natively a
    /// null handle or a sequence error); `p.current()?`.
    pub fn current(&self) -> Result<Wire, Failure> {
        let section = self.my_section.as_ref().ok_or_else(no_section)?;
        let w = section
            .get((self.my_itr - 1).max(-1) as usize)
            .ok_or_else(no_section)?;
        topods::wire(w)
    }

    /// `Shape()`: the compound of the wires; `p.shape()`.
    pub fn shape(&self) -> Compound {
        self.my_shape.clone()
    }
}

/// The ruled shell from the degenerated wire at the point to the scaled wire.
fn ruled_cone(shape_gen1: &Shape, p: Point3) -> Result<Shape, Failure> {
    let bb = BRepBuilder::default();
    let a_vertex = BRepLibMakeVertex::new(p)?.vertex()?;
    let mut deg_edge = Edge::default();
    bb.make_edge(&mut deg_edge)?;
    bb.add(&mut deg_edge, &a_vertex.oriented(Orientation::Forward))?;
    bb.add(&mut deg_edge, &a_vertex.oriented(Orientation::Reversed))?;
    bb.degenerated(&deg_edge, true)?;
    let mut deg_wire = Wire::default();
    bb.make_wire(&mut deg_wire);
    bb.add(&mut deg_wire, &deg_edge)?;
    deg_wire.set_closed(true)?;
    let mut ruled_surf = BRepFillGenerator::new();
    ruled_surf.add_wire(&deg_wire);
    ruled_surf.add_wire(&topods::wire(shape_gen1)?);
    ruled_surf.perform()?;
    Ok(ruled_surf.shell().as_ref().clone())
}

/// Standard_OutOfRange of the current wire past the section.
fn no_section() -> Failure {
    Failure::with_type(
        "Standard_OutOfRange",
        "BRepProj_Projection::Current: no current wire; expected More()",
        "NCollection_Sequence",
    )
}
