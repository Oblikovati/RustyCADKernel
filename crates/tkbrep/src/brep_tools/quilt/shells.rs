// Rust translation of BRepTools_Quilt::Shells
// (ModelingData/TKBRep/BRepTools/BRepTools_Quilt.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepTools_Quilt::Shells`: the faces of the quilt connected into shells.
//!
//! The shells without free edge stay closed; a shell that meets an edge twice with the
//! same orientation becomes non-orientable. The other bound shapes (not faces, not an
//! edge of a face) follow the shells in the result compound in the order of a hashed
//! map, which follows the TShape addresses, natively as in Rust.
use super::Quilt;
use crate::brep::BRepBuilder;
use crate::topexp::Explorer;
use crate::topods::{self, Compound, Shape, ShapeIterator, Shell};
use crate::toptools::{MapOfShape, ShapeMapHasher};
use rustycad_tkernel::ncollection::DataMap;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{self, Orientation, ShapeEnum};

/// `NCollection_DataMap<TopoDS_Shape, TopoDS_Shape, TopTools_ShapeMapHasher>`.
type ShapeToShape = DataMap<Shape, Shape, ShapeMapHasher>;

/// The state of the algorithm: `M` (free edges to their shell, oriented as the edge
/// in it), `MF` (faces to their shell, oriented as the face in it) and the result.
struct ShellBuild {
    free_edges: ShapeToShape,
    face_shells: ShapeToShape,
    result: Compound,
    other_shapes: MapOfShape,
    edges_of_faces: MapOfShape,
}

fn find(map: &ShapeToShape, key: &Shape) -> Result<Shape, Failure> {
    map.find(key).cloned().map_err(|_| {
        Failure::with_type(
            "Standard_NoSuchObject",
            "BRepTools_Quilt::Shells: unbound shape",
            "",
        )
    })
}

/// `anOrien` reversed when the face is REVERSED in its shell.
fn in_shell(orientation: Orientation, face_in_shell: Orientation) -> Orientation {
    if face_in_shell == Orientation::Reversed {
        return top_abs::reverse(orientation);
    }
    orientation
}

impl Quilt {
    /// `Shells()`: a compound of the shells of the bound faces (in binding order), then
    /// of the other bound shapes that are no edge of a face;
    /// `quilt.shells()?`.
    pub fn shells(&self) -> Result<Shape, Failure> {
        let builder = BRepBuilder::default();
        let mut build = ShellBuild {
            free_edges: ShapeToShape::default(),
            face_shells: ShapeToShape::default(),
            result: Compound::default(),
            other_shapes: MapOfShape::default(),
            edges_of_faces: MapOfShape::default(),
        };
        builder.make_compound(&mut build.result);
        for index in 1..=self.bounds.extent() {
            let shape = self.bounds.find_from_index(index)?.clone();
            if shape.shape_type()? == ShapeEnum::Face {
                build.add_face(&shape)?;
            } else {
                build.other_shapes.add(shape);
            }
        }
        let mut cursor = build.free_edges.cursor();
        while cursor.more() {
            // Every shell with a free edge is not closed.
            cursor.value()?.set_closed(false)?;
            cursor.next();
        }
        for other in build.other_shapes.iter() {
            if !build.edges_of_faces.contains(other)
                && let Ok(bound) = self.bounds.find_from_key(other)
            {
                builder.add(&mut build.result, bound)?;
            }
        }
        Ok(build.result.into_shape())
    }
}

impl ShellBuild {
    /// Adds a face to the shell of its first free edge (or to a new closed shell), then
    /// fuses the shells of its other free edges and updates the free edges.
    fn add_face(&mut self, face: &Shape) -> Result<(), Failure> {
        for edge in Explorer::new(face, ShapeEnum::Edge, ShapeEnum::Shape)? {
            self.edges_of_faces.add(edge);
        }
        let mut shell = self.shell_of_first_free_edge(face)?;
        if shell.is_null() {
            let mut new_shell = Shell::default();
            BRepBuilder::default().make_shell(&mut new_shell);
            new_shell.set_closed(true)?;
            BRepBuilder::default().add(&mut self.result, &new_shell)?;
            self.face_shells
                .bind(face.clone(), new_shell.oriented(face.orientation()));
            shell = new_shell;
        }
        shell.set_free(true)?;
        let face_in_shell = find(&self.face_shells, face)?.orientation();
        let mut forward = shell.oriented(Orientation::Forward);
        BRepBuilder::default().add(&mut forward, &face.oriented(face_in_shell))?;
        let forward_face = face.oriented(Orientation::Forward);
        for edge in Explorer::new(&forward_face, ShapeEnum::Edge, ShapeEnum::Shape)? {
            self.connect_edge(face, &shell, &edge)?;
        }
        shell.set_free(false)
    }

    /// The shell of the first edge of the face that is free, with the face bound to it;
    /// a null shell when there is none.
    fn shell_of_first_free_edge(&mut self, face: &Shape) -> Result<Shell, Failure> {
        for edge in Explorer::new(face, ShapeEnum::Edge, ShapeEnum::Shape)? {
            if let Ok(bound) = self.free_edges.find(&edge) {
                let shell = topods::shell(bound)?;
                let orientation = if shell.orientation() == edge.orientation() {
                    top_abs::reverse(face.orientation())
                } else {
                    face.orientation()
                };
                self.face_shells
                    .bind(face.clone(), shell.oriented(orientation));
                return Ok(shell);
            }
        }
        Ok(Shell::default())
    }

    /// One edge of the face: a free edge of another shell fuses that shell into this one;
    /// a free edge met with the same orientation makes the shell non-orientable; the edge
    /// is then no longer free. Another edge becomes free in this shell.
    fn connect_edge(&mut self, face: &Shape, shell: &Shell, edge: &Shape) -> Result<(), Failure> {
        let face_in_shell = find(&self.face_shells, face)?.orientation();
        if !self.free_edges.is_bound(edge) {
            if !edge.is_null() {
                let orientation = in_shell(edge.orientation(), face_in_shell);
                self.free_edges
                    .bind(edge.clone(), shell.oriented(orientation));
            }
            return Ok(());
        }
        let old_shell = find(&self.free_edges, edge)?;
        if !old_shell.is_same(shell) {
            let reverse = in_shell(edge.orientation(), face_in_shell) == old_shell.orientation();
            self.fuse(shell, &old_shell, reverse)?;
        }
        let face_in_shell = find(&self.face_shells, face)?.orientation();
        let orientation = in_shell(edge.orientation(), face_in_shell);
        if find(&self.free_edges, edge)?.orientation() == orientation {
            shell.set_orientable(false)?;
        }
        self.free_edges.unbind(edge);
        Ok(())
    }

    /// Moves the faces of `old_shell` into `shell` (reversed with `reverse`), rebinds its
    /// free edges met in `shell`, and removes it from the result.
    fn fuse(&mut self, shell: &Shell, old_shell: &Shape, reverse: bool) -> Result<(), Failure> {
        let builder = BRepBuilder::default();
        for old_face in ShapeIterator::new(old_shell, true, true) {
            let face = topods::face(&old_face)?;
            let mut orientation = find(&self.face_shells, &face)?.orientation();
            if reverse {
                orientation = top_abs::reverse(orientation);
            }
            self.face_shells
                .bind(face.clone().into_shape(), shell.oriented(orientation));
            let mut forward = shell.oriented(Orientation::Forward);
            builder.add(&mut forward, &face.oriented(orientation))?;
        }
        // gka BUG 6491: the free edges of the old shell move to the new one.
        for edge in Explorer::new(shell, ShapeEnum::Edge, ShapeEnum::Shape)? {
            let Ok(bound) = self.free_edges.find(&edge).cloned() else {
                continue;
            };
            if bound.is_same(old_shell) {
                let orientation = if reverse {
                    top_abs::reverse(bound.orientation())
                } else {
                    bound.orientation()
                };
                self.free_edges.bind(edge, shell.oriented(orientation));
            }
        }
        builder.remove(&mut self.result, &old_shell.oriented(Orientation::Forward))
    }
}
