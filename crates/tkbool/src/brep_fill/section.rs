// Rust translation of ModelingAlgorithms/TKBool/BRepFill/BRepFill_Section.hxx/.cxx/.lxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepFill_Section`: a profile of a pipe shell, its wire without locations and its
//! placement options.
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Edge, Shape, ShapeIterator, Vertex, Wire};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkshhealing::shape_upgrade::ShapeUpgradeRemoveLocations;

/// A section of a pipe shell (`BRepFill_Section`): the original profile, the wire
/// that replaces it (a degenerated edge for a vertex), its location vertex and flags;
/// `let section = BRepFillSection::with_profile(&wire, &Vertex::default(), false, false)?;`.
#[derive(Clone, Debug, Default)]
pub struct BRepFillSection {
    my_original_shape: Shape,
    wire: Wire,
    vertex: Vertex,
    islaw: bool,
    ispunctual: bool,
    contact: bool,
    correction: bool,
}

impl BRepFillSection {
    /// `BRepFill_Section()`: an empty section; `BRepFillSection::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `BRepFill_Section(Profile, V, WithContact, WithCorrection)`: the profile with
    /// its locations removed; a vertex becomes a closed wire of one degenerated edge;
    /// Standard_Failure for another shape type.
    pub fn with_profile(
        profile: &Shape,
        v: &Vertex,
        with_contact: bool,
        with_correction: bool,
    ) -> Result<Self, Failure> {
        let mut section = Self {
            vertex: v.clone(),
            contact: with_contact,
            correction: with_correction,
            ..Self::default()
        };
        section.my_original_shape = profile.clone();
        let remover = ShapeUpgradeRemoveLocations::new();
        remover.set_remove_level(ShapeEnum::Compound);
        remover.remove(profile)?;
        let a_profile: Shape = remover.get_result();
        match a_profile.shape_type()? {
            ShapeEnum::Wire => section.wire = topods::wire(&a_profile)?,
            ShapeEnum::Vertex => {
                section.ispunctual = true;
                section.wire = punctual_wire(&topods::vertex(&a_profile)?)?;
            }
            _ => {
                return Err(Failure::with_type(
                    "Standard_Failure",
                    "BRepFill_Section: bad shape type of section",
                    "BRepFill_Section",
                ));
            }
        }
        Ok(section)
    }

    /// `Set(IsLaw)`; `section.set(true)`.
    pub fn set(&mut self, is_law: bool) {
        self.islaw = is_law;
    }

    /// `OriginalShape()`; `section.original_shape()`.
    pub fn original_shape(&self) -> &Shape {
        &self.my_original_shape
    }

    /// `Wire()`; `section.wire()`.
    pub fn wire(&self) -> &Wire {
        &self.wire
    }

    /// `Vertex()`; `section.vertex()`.
    pub fn vertex(&self) -> &Vertex {
        &self.vertex
    }

    /// `IsLaw()`; `section.is_law()`.
    pub fn is_law(&self) -> bool {
        self.islaw
    }

    /// `IsPunctual()`; `section.is_punctual()`.
    pub fn is_punctual(&self) -> bool {
        self.ispunctual
    }

    /// `WithContact()`; `section.with_contact()`.
    pub fn with_contact(&self) -> bool {
        self.contact
    }

    /// `WithCorrection()`; `section.with_correction()`.
    pub fn with_correction(&self) -> bool {
        self.correction
    }

    /// `ModifiedShape(theShape)`: the shape of the section wire that replaces a
    /// sub-shape of the original profile (the wire for the profile, the edge of the
    /// same rank, the vertex of the same explorer rank); a null shape otherwise.
    pub fn modified_shape(&self, the_shape: &Shape) -> Result<Shape, Failure> {
        match the_shape.shape_type()? {
            ShapeEnum::Wire if the_shape.is_same(&self.my_original_shape) => {
                Ok(self.wire.as_ref().clone())
            }
            ShapeEnum::Edge => self.modified_edge(the_shape),
            ShapeEnum::Vertex => self.modified_vertex(the_shape),
            _ => Ok(Shape::default()),
        }
    }

    /// The edge of the wire at the rank of `edge` among the children of the profile.
    fn modified_edge(&self, edge: &Shape) -> Result<Shape, Failure> {
        let mut itor = ShapeIterator::new(&self.my_original_shape, true, true);
        let mut itw = ShapeIterator::new(&self.wire, true, true);
        while itor.more() {
            if itor.value()?.is_same(edge) {
                return Ok(itw.value()?.clone());
            }
            itor.next();
            itw.next();
        }
        Ok(Shape::default())
    }

    /// The vertex of the wire for the profile vertex, or at the explorer rank of a
    /// vertex of the profile.
    fn modified_vertex(&self, vertex: &Shape) -> Result<Shape, Failure> {
        let mut explo = Explorer::new(&self.wire, ShapeEnum::Vertex, ShapeEnum::Shape)?;
        if vertex.is_same(&self.my_original_shape) {
            return Ok(explo.current().clone());
        }
        let mut exp_orig =
            Explorer::new(&self.my_original_shape, ShapeEnum::Vertex, ShapeEnum::Shape)?;
        while exp_orig.more() {
            if exp_orig.current().is_same(vertex) {
                return Ok(explo.current().clone());
            }
            exp_orig.next()?;
            explo.next()?;
        }
        Ok(Shape::default())
    }
}

/// The closed wire of one degenerated edge on a vertex.
fn punctual_wire(vertex: &Vertex) -> Result<Wire, Failure> {
    let builder = BRepBuilder::default();
    let mut deg_edge = Edge::default();
    builder.make_edge(&mut deg_edge)?;
    builder.add(&mut deg_edge, &vertex.oriented(Orientation::Forward))?;
    builder.add(&mut deg_edge, &vertex.oriented(Orientation::Reversed))?;
    builder.degenerated(&deg_edge, true)?;
    let mut wire = Wire::default();
    builder.make_wire(&mut wire);
    builder.add(&mut wire, &deg_edge)?;
    wire.set_closed(true)?;
    Ok(wire)
}
