// Rust translation of BRepBuilderAPI_NurbsConvert.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepBuilderAPI_NurbsConvert`: a copy of a shape whose curves and surfaces are all
//! converted to B-splines (`BRepTools_NurbsConvertModification`), with the vertex
//! tolerances raised to the tolerances of the converted edges.
use super::make_shape::{BRepBuilderAPIMakeShape, BRepBuilderAPIMakeShapeBase};
use super::modify_shape::BRepBuilderAPIModifyShape;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_tools::{
    BRepToolsModification, BRepToolsReShape, NurbsConvertModification, ReShape,
};
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Shape, ShapeIterator, Vertex};
use rustycad_tkbrep::toptools::{MapOfShape, ShapeMapHasher};
use rustycad_tkernel::ncollection::{DataMap, List};
use rustycad_tkernel::standard::real::epsilon;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::ShapeEnum;
use std::ops::{Deref, DerefMut};

/// `NCollection_DataMap<TopoDS_Shape, TopoDS_Shape, TopTools_ShapeMapHasher>`.
type VertexReplacements = DataMap<Shape, Shape, ShapeMapHasher>;

/// `BRepBuilderAPI_NurbsConvert` (docs/topalgo-design.md section 10): the modification
/// makes new edges and faces on B-spline geometry; a vertex of the initial shape whose
/// tolerance is below the tolerance of an updated edge is replaced by a new vertex (the
/// replacements are applied to the result through a `BRepTools_ReShape`), and the new
/// vertices get that tolerance. The `Copy` argument of the native members is unused.
///
/// ```
/// use rustycad_tkmath::gp::Point3;
/// use rustycad_tktopalgo::brep_builder_api::{
///     BRepBuilderAPIMakeEdge, BRepBuilderAPIMakeShape, BRepBuilderAPINurbsConvert,
/// };
/// let edge = BRepBuilderAPIMakeEdge::from_points(Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 0.0, 0.0))?.edge()?;
/// let mut convert = BRepBuilderAPINurbsConvert::with_shape(&edge, false)?;
/// assert!(convert.is_done());
/// assert!(!convert.shape()?.is_same(&edge));
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
pub struct BRepBuilderAPINurbsConvert {
    modify: BRepBuilderAPIModifyShape,
    my_vtx_to_replace: VertexReplacements,
    my_subs: BRepToolsReShape,
}

impl Default for BRepBuilderAPINurbsConvert {
    fn default() -> Self {
        Self::new()
    }
}

impl Deref for BRepBuilderAPINurbsConvert {
    type Target = BRepBuilderAPIModifyShape;
    fn deref(&self) -> &BRepBuilderAPIModifyShape {
        &self.modify
    }
}

impl DerefMut for BRepBuilderAPINurbsConvert {
    fn deref_mut(&mut self) -> &mut BRepBuilderAPIModifyShape {
        &mut self.modify
    }
}

impl BRepBuilderAPIMakeShape for BRepBuilderAPINurbsConvert {
    fn make_shape_base(&self) -> &BRepBuilderAPIMakeShapeBase {
        self.modify.make_shape_base()
    }

    fn make_shape_base_mut(&mut self) -> &mut BRepBuilderAPIMakeShapeBase {
        self.modify.make_shape_base_mut()
    }

    /// `Modified(S)`: the replacement of a replaced initial vertex, the image of any
    /// other vertex in the modifier, and for the other shapes their image through the
    /// re-shaper when vertices were replaced; the list is cleared first;
    /// `convert.modified(&face)?`.
    fn modified(&mut self, shape: &Shape) -> Result<&List<Shape>, Failure> {
        self.modify.make_shape_base_mut().my_generated.clear();
        let image = self.modified_image(shape)?;
        let generated = &mut self.modify.make_shape_base_mut().my_generated;
        generated.append(image);
        Ok(generated)
    }
}

impl BRepBuilderAPINurbsConvert {
    /// `BRepBuilderAPI_NurbsConvert()`: the modification, no shape yet;
    /// `BRepBuilderAPINurbsConvert::new()`.
    pub fn new() -> Self {
        let modification: Handle<dyn BRepToolsModification> =
            Handle::new(NurbsConvertModification::new()).cast();
        Self {
            modify: BRepBuilderAPIModifyShape::with_modification(&modification),
            my_vtx_to_replace: VertexReplacements::default(),
            my_subs: BRepToolsReShape::new(),
        }
    }

    /// `BRepBuilderAPI_NurbsConvert(S, Copy)`: then [`Self::perform`];
    /// `BRepBuilderAPINurbsConvert::with_shape(&s, false)?`.
    pub fn with_shape(shape: &Shape, copy: bool) -> Result<Self, Failure> {
        let mut convert = Self::new();
        convert.perform(shape, copy)?;
        Ok(convert)
    }

    /// `Perform(S, Copy)`: converts `S` (the flag is unused natively), then raises the
    /// vertex tolerances; `convert.perform(&s, false)?`.
    pub fn perform(&mut self, shape: &Shape, _copy: bool) -> Result<(), Failure> {
        let modification = self.modify.my_modification.clone();
        self.modify
            .do_modif_shape_modification(shape, &modification)?;
        self.correct_vertex_tol()
    }

    /// The private `CorrectVertexTol()`: the vertices of each updated edge get its
    /// tolerance plus `Epsilon`; a vertex of the initial shape is replaced by a new one
    /// instead, and the replacements are applied to the result.
    fn correct_vertex_tol(&mut self) -> Result<(), Failure> {
        let init_vertices = self.initial_vertices()?;
        self.my_vtx_to_replace.clear(true);
        for edge in self.updated_edges() {
            let tolerance = BRepTool::edge_tolerance(&topods::edge(&edge)?)?;
            for child in ShapeIterator::new(&edge, true, true) {
                let vertex = topods::vertex(&child)?;
                self.correct_vertex(&vertex, tolerance, &init_vertices)?;
            }
        }
        if self.my_vtx_to_replace.is_empty() {
            return Ok(());
        }
        self.apply_replacements()
    }

    /// The vertices of the initial shape.
    fn initial_vertices(&self) -> Result<MapOfShape, Failure> {
        let mut vertices = MapOfShape::default();
        let mut explorer = Explorer::new(
            &self.modify.my_initial_shape,
            ShapeEnum::Vertex,
            ShapeEnum::Shape,
        )?;
        while explorer.more() {
            vertices.add(explorer.current().clone());
            explorer.next()?;
        }
        Ok(vertices)
    }

    /// `aModif->GetUpdatedEdges()`.
    fn updated_edges(&self) -> Vec<Shape> {
        let modification: Handle<NurbsConvertModification> = self.modify.my_modification.cast();
        modification
            .get()
            .map_or_else(Vec::new, NurbsConvertModification::updated_edges)
    }

    /// One vertex of an updated edge of tolerance `edge_tol`.
    fn correct_vertex(
        &mut self,
        vertex: &Vertex,
        edge_tol: f64,
        init_vertices: &MapOfShape,
    ) -> Result<(), Failure> {
        let builder = BRepBuilder::default();
        let raised: f64 = edge_tol + epsilon(edge_tol);
        if !init_vertices.contains(vertex) {
            return builder.update_vertex_tolerance(vertex, raised);
        }
        if let Some(replacement) = self.my_vtx_to_replace.seek(vertex) {
            return builder.update_vertex_tolerance(&topods::vertex(replacement)?, raised);
        }
        if BRepTool::vertex_tolerance(vertex)? < edge_tol {
            let mut new_vertex = Vertex::default();
            builder.make_vertex_at(&mut new_vertex, BRepTool::pnt(vertex)?, raised)?;
            new_vertex.set_orientation(vertex.orientation());
            self.my_vtx_to_replace
                .bind(vertex.clone().into_shape(), new_vertex.into_shape());
        }
        Ok(())
    }

    /// `mySubs` records every replacement and the result becomes its image.
    fn apply_replacements(&mut self) -> Result<(), Failure> {
        self.my_subs.clear();
        for (old, new) in self.my_vtx_to_replace.items() {
            self.my_subs.replace(old, new)?;
        }
        let result = self.modify.make_shape_base().my_shape.clone();
        self.my_subs.apply(&result, ShapeEnum::Shape)?;
        self.modify.make_shape_base_mut().my_shape = self.my_subs.value(&result)?;
        Ok(())
    }

    /// `ModifiedShape(S)`: the replacement of a replaced initial vertex, else the image
    /// of `S` in the modifier (through the re-shaper when vertices were replaced);
    /// Standard_NoSuchObject for a foreign shape; `convert.modified_shape(&face)?`.
    pub fn modified_shape(&self, shape: &Shape) -> Result<Shape, Failure> {
        if shape.shape_type()? == ShapeEnum::Vertex
            && let Some(replacement) = self.my_vtx_to_replace.seek(shape)
        {
            return Ok(replacement.clone());
        }
        let image = self.modify.my_modifier.modified_shape(shape)?;
        if self.my_vtx_to_replace.is_empty() {
            return Ok(image);
        }
        self.my_subs.value(&image)
    }

    /// The image `Modified(S)` appends: unlike [`Self::modified_shape`], a vertex that
    /// was not replaced gets its modifier image without the re-shaper (natively too).
    fn modified_image(&self, shape: &Shape) -> Result<Shape, Failure> {
        if shape.shape_type()? == ShapeEnum::Vertex {
            if let Some(replacement) = self.my_vtx_to_replace.seek(shape) {
                return Ok(replacement.clone());
            }
            return self.modify.my_modifier.modified_shape(shape);
        }
        let image = self.modify.my_modifier.modified_shape(shape)?;
        if self.my_vtx_to_replace.is_empty() {
            return Ok(image);
        }
        self.my_subs.value(&image)
    }
}
