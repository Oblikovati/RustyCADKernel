//! `RefineShell` (BOPAlgo_ShellSplitter.cxx): splits a shell on its branch edges.
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{Edge, Shape, ShapeIterator, Shell};
use rustycad_tkbrep::toptools::{IndexedDataMapOfShapeListOfShape, ShapeMapHasher};
use rustycad_tkernel::ncollection::{IndexedMap, List, Map};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};

/// Splits `shell` on the edges shared by more than two faces, by two faces using them with
/// the same orientation, or internal in a face; appends the parts (or the shell itself when
/// there is no such edge) to `splits`.
pub(super) fn refine_shell(
    shell: &Shape,
    edge_faces: &IndexedDataMapOfShapeListOfShape,
    splits: &mut List<Shape>,
) -> Result<(), Failure> {
    let faces: Vec<Shape> = ShapeIterator::new(shell, true, true).collect();
    if faces.is_empty() {
        return Ok(());
    }
    let stops = branch_edges(edge_faces)?;
    if stops.is_empty() {
        splits.append(shell.clone());
        return Ok(());
    }
    let mut processed: Map<Shape> = Map::new(1);
    for face in &faces {
        if !processed.add(face.clone()) {
            continue;
        }
        let part = reachable_faces(face, edge_faces, &stops, &mut processed)?;
        splits.append(shell_of(&part)?);
    }
    Ok(())
}

/// The edges the shell is split on (`aMEStop`).
fn branch_edges(
    edge_faces: &IndexedDataMapOfShapeListOfShape,
) -> Result<Map<Shape, ShapeMapHasher>, Failure> {
    let mut stops: Map<Shape, ShapeMapHasher> = Map::with_hasher(ShapeMapHasher, 1);
    for index in 1..=edge_faces.extent() {
        let edge = edge_faces.find_key(index)?;
        let faces: Vec<&Shape> = edge_faces.find_from_index(index)?.iter().collect();
        if faces.len() > 2 || (faces.len() == 2 && same_use(edge, faces[0], faces[1])?) {
            stops.add(edge.clone());
            continue;
        }
        if counted_uses(edge, &faces)? > 2 {
            stops.add(edge.clone());
        }
    }
    Ok(stops)
}

/// Whether two faces use the edge with the same orientation.
fn same_use(edge: &Shape, first: &Shape, second: &Shape) -> Result<bool, Failure> {
    Ok(find_shape(edge, first)?.orientation() == find_shape(edge, second)?.orientation())
}

/// The occurrence of `target` in `shape` (`FindShape`); a null shape when absent.
fn find_shape(target: &Shape, shape: &Shape) -> Result<Shape, Failure> {
    for found in Explorer::new(shape, target.shape_type()?, ShapeEnum::Shape)? {
        if found.is_same(target) {
            return Ok(found);
        }
    }
    Ok(Shape::default())
}

/// The number of faces using the edge, a face where it is internal counted twice.
fn counted_uses(edge: &Shape, faces: &[&Shape]) -> Result<usize, Failure> {
    let mut count = 0;
    for face in faces {
        if count > 2 {
            break;
        }
        count += 1;
        let occurrence = find_shape(edge, face)?;
        if !occurrence.is_null() && occurrence.orientation() == Orientation::Internal {
            count += 1;
        }
    }
    Ok(count)
}

/// The faces reached from `seed` without crossing a branch edge (`aMFB`).
fn reachable_faces(
    seed: &Shape,
    edge_faces: &IndexedDataMapOfShapeListOfShape,
    stops: &Map<Shape, ShapeMapHasher>,
    processed: &mut Map<Shape>,
) -> Result<IndexedMap<Shape, ShapeMapHasher>, Failure> {
    let mut part: IndexedMap<Shape, ShapeMapHasher> = IndexedMap::with_hasher(ShapeMapHasher, 1);
    part.add(seed.clone());
    let mut front = vec![seed.clone()];
    loop {
        let mut next = Vec::new();
        for face in &front {
            for edge in Explorer::new(face, ShapeEnum::Edge, ShapeEnum::Shape)? {
                if stops.contains(&edge)
                    || edge.orientation() == Orientation::Internal
                    || BRepTool::degenerated(&Edge::unchecked(edge.clone()))?
                {
                    continue;
                }
                for other in edge_faces.find_from_key(&edge)?.iter() {
                    if other.is_same(face) || part.contains(other) {
                        continue;
                    }
                    if processed.add(other.clone()) {
                        part.add(other.clone());
                        next.push(other.clone());
                    }
                }
            }
        }
        if next.is_empty() {
            return Ok(part);
        }
        front = next;
    }
}

fn shell_of(faces: &IndexedMap<Shape, ShapeMapHasher>) -> Result<Shape, Failure> {
    let builder = rustycad_tkbrep::brep::BRepBuilder::default();
    let mut shell = Shell::default();
    builder.make_shell(&mut shell);
    let mut shell = shell.into_shape();
    for face in faces.iter() {
        builder.add(&mut shell, face)?;
    }
    Ok(shell)
}
