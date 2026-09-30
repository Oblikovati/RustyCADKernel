// Rust translation of the output queries of BRepBuilderAPI_Sewing.cxx (NbFreeEdges to
// SectionToBoundary, NbDeletedFaces to ModifiedSubShape, Dump and WhichFace) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The results of the sewing: the free, contiguous, multiple and degenerated edges, the
//! deleted faces, the history of the shapes and the information dump.
use super::failures::{no_such_object, out_of_range};
use super::{BRepBuilderAPISewing, live_reshape};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Edge, Face, Shape, ShapeIterator};
use rustycad_tkbrep::toptools::{IndexedMapOfShape, MapOfShape};
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;
use std::io::Write;

/// The `_Raise_if(index < 0 || index > N)` check of the accessors by index; the map
/// then raises for index 0, as the native `operator()`.
fn checked_index(member: &str, index: i32, extent: i32) -> Result<usize, Failure> {
    if index < 1 || index > extent {
        return Err(out_of_range(member, index, extent));
    }
    Ok(index as usize)
}

/// The `index`-th key of a shape map, for the accessor `member`.
fn indexed_shape<'a>(
    map: &'a IndexedMapOfShape,
    member: &str,
    index: i32,
) -> Result<&'a Shape, Failure> {
    map.find_key(checked_index(member, index, map.extent() as i32)?)
}

impl BRepBuilderAPISewing {
    /// `NbFreeEdges()`: the edges of one face; `sewing.nb_free_edges()`.
    pub fn nb_free_edges(&self) -> i32 {
        self.my_free_edges.extent() as i32
    }
    /// `FreeEdge(index)`: Standard_OutOfRange outside `[1, NbFreeEdges]`;
    /// `sewing.free_edge(1)?`.
    pub fn free_edge(&self, index: i32) -> Result<Edge, Failure> {
        topods::edge(indexed_shape(
            &self.my_free_edges,
            "BRepBuilderAPI_Sewing::FreeEdge",
            index,
        )?)
    }
    /// `NbMultipleEdges()`: the edges of more than two faces; `sewing.nb_multiple_edges()`.
    pub fn nb_multiple_edges(&self) -> i32 {
        self.my_multiple_edges.extent() as i32
    }
    /// `MultipleEdge(index)`: Standard_OutOfRange outside `[1, NbMultipleEdges]`;
    /// `sewing.multiple_edge(1)?`.
    pub fn multiple_edge(&self, index: i32) -> Result<Edge, Failure> {
        let member = "BRepBuilderAPI_Sewing::MultipleEdge";
        topods::edge(indexed_shape(&self.my_multiple_edges, member, index)?)
    }
    /// `NbContigousEdges()`: the edges of two faces; `sewing.nb_contigous_edges()`.
    pub fn nb_contigous_edges(&self) -> i32 {
        self.my_contigous_edges.extent() as i32
    }
    /// `ContigousEdge(index)`: Standard_OutOfRange outside `[1, NbContigousEdges]`;
    /// `sewing.contigous_edge(1)?`.
    pub fn contigous_edge(&self, index: i32) -> Result<Edge, Failure> {
        let extent = self.nb_contigous_edges();
        let i = checked_index("BRepBuilderAPI_Sewing::ContigousEdge", index, extent)?;
        topods::edge(self.my_contigous_edges.find_key(i)?)
    }
    /// `ContigousEdgeCouple(index)`: the sections merged into the contiguous edge;
    /// `sewing.contigous_edge_couple(1)?.extent()`.
    pub fn contigous_edge_couple(&self, index: i32) -> Result<&List<Shape>, Failure> {
        let extent = self.nb_contigous_edges();
        let i = checked_index("BRepBuilderAPI_Sewing::ContigousEdgeCouple", index, extent)?;
        self.my_contigous_edges.find_from_index(i)
    }
    /// `IsSectionBound(section)`: whether a section of a contiguous edge comes from a
    /// cut boundary; `sewing.is_section_bound(&edge)`.
    pub fn is_section_bound(&self, section: &Edge) -> bool {
        self.my_contig_sec_bound.is_bound(section)
    }
    /// `SectionToBoundary(section)`: the free boundary cut into the section;
    /// Standard_NoSuchObject when the section is not bound;
    /// `sewing.section_to_boundary(&edge)?`.
    pub fn section_to_boundary(&self, section: &Edge) -> Result<Edge, Failure> {
        let Ok(bound) = self.my_contig_sec_bound.find(section) else {
            return Err(no_such_object("BRepBuilderAPI_Sewing::SectionToBoundary"));
        };
        topods::edge(bound)
    }
    /// `NbDeletedFaces()`: the faces removed as smaller than the tolerance;
    /// `sewing.nb_deleted_faces()`.
    pub fn nb_deleted_faces(&self) -> i32 {
        self.my_little_face.extent() as i32
    }
    /// `DeletedFace(index)`: Standard_OutOfRange outside `[1, NbDeletedFaces]`;
    /// `sewing.deleted_face(1)?`.
    pub fn deleted_face(&self, index: i32) -> Result<Face, Failure> {
        let member = "BRepBuilderAPI_Sewing::DeletedFace";
        topods::face(indexed_shape(&self.my_little_face, member, index)?)
    }
    /// `NbDegeneratedShapes()`; `sewing.nb_degenerated_shapes()`.
    pub fn nb_degenerated_shapes(&self) -> i32 {
        self.my_degenerated.extent() as i32
    }
    /// `DegeneratedShape(index)`: Standard_OutOfRange outside
    /// `[1, NbDegeneratedShapes]`; `sewing.degenerated_shape(1)?`.
    pub fn degenerated_shape(&self, index: i32) -> Result<&Shape, Failure> {
        let member = "BRepBuilderAPI_Sewing::DegereratedShape";
        indexed_shape(&self.my_degenerated, member, index)
    }

    /// `IsDegenerated(shape)`: a face removed by the sewing, a degenerated edge, or a
    /// wire of degenerated edges (after the context); Standard_NullObject for a null
    /// shape (a null dereference natively); `sewing.is_degenerated(&face)?`.
    pub fn is_degenerated(&self, shape: &Shape) -> Result<bool, Failure> {
        let new_shape = live_reshape(&self.my_re_shape)?.apply(shape, ShapeEnum::Shape)?;
        if shape.shape_type()? == ShapeEnum::Face {
            return Ok(new_shape.is_null());
        }
        if new_shape.is_null() {
            return Ok(false);
        }
        match new_shape.shape_type()? {
            ShapeEnum::Edge => BRepTool::degenerated(&topods::edge(&new_shape)?),
            ShapeEnum::Wire => degenerated_wire(&new_shape),
            _ => Ok(false),
        }
    }

    /// `IsModified(shape)`: whether an input shape was replaced;
    /// `sewing.is_modified(&face)`.
    pub fn is_modified(&self, shape: &Shape) -> bool {
        !self.modified(shape).is_same(shape)
    }
    /// `Modified(shape)`: the image of an input shape, or the shape itself;
    /// `sewing.modified(&face)`.
    pub fn modified(&self, shape: &Shape) -> Shape {
        match self.my_old_shapes.seek(shape) {
            Some(new_shape) => new_shape.clone(),
            None => shape.clone(),
        }
    }
    /// `IsModifiedSubShape(shape)`: whether the context replaced a sub-shape;
    /// `sewing.is_modified_sub_shape(&edge)?`.
    pub fn is_modified_sub_shape(&self, shape: &Shape) -> Result<bool, Failure> {
        Ok(!self.modified_sub_shape(shape)?.is_same(shape))
    }
    /// `ModifiedSubShape(shape)`: the image of a sub-shape by the context;
    /// `sewing.modified_sub_shape(&edge)?`.
    pub fn modified_sub_shape(&self, shape: &Shape) -> Result<Shape, Failure> {
        live_reshape(&self.my_re_shape)?.apply(shape, ShapeEnum::Shape)
    }

    /// `WhichFace(theEdg, index)`: the `index`-th face of the boundary of an edge (a
    /// section is first mapped to its boundary), or a null face;
    /// `sewing.which_face(&edge, 1)?`.
    pub fn which_face(&self, edge: &Edge, index: i32) -> Result<Face, Failure> {
        let bound = self.bound_of(edge);
        let Some(faces) = self.my_bound_faces.seek(&bound) else {
            return Ok(Face::default());
        };
        // The native loop meets `i == index` only for an index in [1, Extent].
        let Some(face) = usize::try_from(index - 1)
            .ok()
            .and_then(|i| faces.iter().nth(i))
        else {
            return Ok(Face::default());
        };
        topods::face(face)
    }

    /// The boundary a section was cut from, or the edge itself (`mySectionBound`).
    pub(crate) fn bound_of(&self, edge: &Shape) -> Shape {
        match self.my_section_bound.find(edge) {
            Ok(bound) => bound.clone(),
            Err(_) => edge.clone(),
        }
    }

    /// `Dump()`: the counts of the sewing, written to `out` (std::cout natively);
    /// `sewing.dump(&mut std::io::stdout())?`.
    pub fn dump(&self, out: &mut dyn Write) -> Result<(), Failure> {
        let nb_bounds = self.my_bound_faces.extent() as i32;
        let mut nb_sections: i32 = 0;
        let (mut vertices, mut edges) = (MapOfShape::default(), MapOfShape::default());
        let reshape = live_reshape(&self.my_re_shape)?;
        for i in 1..=self.my_bound_faces.extent() {
            let bound = self.my_bound_faces.find_key(i)?.clone();
            nb_sections += match self.my_bound_sections.find(&bound) {
                Ok(sections) => sections.extent(),
                Err(_) => 1,
            };
            let image = reshape.apply(&bound, ShapeEnum::Shape)?;
            for e in Explorer::new(&image, ShapeEnum::Edge, ShapeEnum::Shape)? {
                let (v1, v2) = topexp::vertices(&topods::edge(&e)?, false);
                edges.add(e);
                vertices.add(v1.into_shape());
                vertices.add(v2.into_shape());
            }
        }
        let counts = DumpCounts {
            nb_bounds,
            nb_sections,
            nb_edges: edges.extent() as i32,
            nb_nodes: vertices.extent() as i32,
        };
        self.write_dump(out, &counts)
            .map_err(|e| Failure::new(&e.to_string()))
    }

    /// The text of `Dump` from its counts.
    fn write_dump(&self, out: &mut dyn Write, counts: &DumpCounts) -> std::io::Result<()> {
        let rule = " ===========================================================";
        write!(
            out,
            " \n                        Information                         \n{rule}\n \n"
        )?;
        writeln!(
            out,
            " Number of input shapes      : {}",
            self.my_old_shapes.extent()
        )?;
        writeln!(out, " Number of actual shapes     : {}", self.my_nb_shapes)?;
        writeln!(out, " Number of Bounds            : {}", counts.nb_bounds)?;
        writeln!(out, " Number of Sections          : {}", counts.nb_sections)?;
        writeln!(out, " Number of Edges             : {}", counts.nb_edges)?;
        writeln!(
            out,
            " Number of Vertices          : {}",
            self.my_nb_vertices
        )?;
        writeln!(out, " Number of Nodes             : {}", counts.nb_nodes)?;
        writeln!(
            out,
            " Number of Free Edges        : {}",
            self.my_free_edges.extent()
        )?;
        writeln!(
            out,
            " Number of Contiguous Edges  : {}",
            self.my_contigous_edges.extent()
        )?;
        writeln!(
            out,
            " Number of Multiple Edges    : {}",
            self.my_multiple_edges.extent()
        )?;
        writeln!(
            out,
            " Number of Degenerated Edges : {}",
            self.my_degenerated.extent()
        )?;
        write!(out, "{rule}\n \n")
    }
}

/// The counts `Dump` computes from the boundaries.
struct DumpCounts {
    nb_bounds: i32,
    nb_sections: i32,
    nb_edges: i32,
    nb_nodes: i32,
}

/// The wire test of `IsDegenerated`: every edge (until a non-degenerated one) is
/// degenerated; an empty wire is.
fn degenerated_wire(wire: &Shape) -> Result<bool, Failure> {
    let mut is_degenerated = true;
    let mut it = ShapeIterator::new(wire, true, true);
    while is_degenerated && let Ok(child) = it.value() {
        is_degenerated = BRepTool::degenerated(&topods::edge(child)?)?;
        it.next();
    }
    Ok(is_degenerated)
}
