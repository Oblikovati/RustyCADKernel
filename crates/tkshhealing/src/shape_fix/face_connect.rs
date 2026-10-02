// Rust translation of ShapeFix/ShapeFix_FaceConnect.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871. Sewing is delegated to the pinned
// BRepBuilderAPI_Sewing port; only registered face groups are submitted to it.

//! Sew selected neighbouring faces inside a shell.
use rustycad_tkbrep::{topods, topods::Builder, topods::ShapeIterator, topods::Shell};
use rustycad_tkernel::message::ProgressRange;
use rustycad_tkernel::standard::Failure;
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPISewing;

/// Retains requested face connections across `build()` calls; `add(face1, face2)`.
#[derive(Default)]
pub struct ShapeFixFaceConnect {
    connections: Vec<(topods::Face, topods::Face)>,
}

impl ShapeFixFaceConnect {
    /// `ShapeFix_FaceConnect()`: start with no selected face pairs;
    /// `ShapeFixFaceConnect::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register two non-null faces for sewing. Repeated and self connections succeed;
    /// `assert!(connector.add(&face, &face))`.
    pub fn add(&mut self, first: &topods::Face, second: &topods::Face) -> bool {
        if first.is_null() || second.is_null() {
            return false;
        }
        if self
            .connections
            .iter()
            .any(|(a, b)| same_pair(a, b, first, second))
        {
            return true;
        }
        self.connections.push((first.clone(), second.clone()));
        true
    }

    /// Discard registered face pairs; `connector.clear()`.
    pub fn clear(&mut self) {
        self.connections.clear();
    }

    /// Sew registered face groups and retain every other shell face unchanged;
    /// `let result = connector.build(&shell, 1e-6, 1e-7)?`.
    pub fn build(
        &self,
        shell: &Shell,
        sewing_tolerance: f64,
        _fixing_tolerance: f64,
    ) -> Result<Shell, Failure> {
        let faces = shell_faces(shell)?;
        let groups = registered_groups(&faces, &self.connections);
        if groups.is_empty() {
            return Ok(shell.clone());
        }
        let replacements = sew_groups(&groups, sewing_tolerance)?;
        if replacements
            .iter()
            .all(|(original, image)| original.is_same(image))
        {
            return Ok(shell.clone());
        }
        rebuild_shell(&faces, &replacements)
    }
}

fn same_pair(
    a: &topods::Face,
    b: &topods::Face,
    first: &topods::Face,
    second: &topods::Face,
) -> bool {
    (a.is_same(first) && b.is_same(second)) || (a.is_same(second) && b.is_same(first))
}

fn shell_faces(shell: &Shell) -> Result<Vec<topods::Face>, Failure> {
    let shape = shell.clone().into_shape();
    let mut iterator = ShapeIterator::new(&shape, false, true);
    let mut faces = Vec::new();
    while let Ok(child) = iterator.value() {
        faces.push(topods::face(child)?);
        iterator.next();
    }
    Ok(faces)
}

fn registered_groups(
    faces: &[topods::Face],
    connections: &[(topods::Face, topods::Face)],
) -> Vec<Vec<topods::Face>> {
    let mut groups = Vec::new();
    let mut visited = Vec::new();
    for face in faces {
        if contains_face(&visited, face) || !has_connection(face, connections) {
            continue;
        }
        let group = collect_group(face, faces, connections, &mut visited);
        if group.len() > 1 {
            groups.push(group);
        }
    }
    groups
}

fn has_connection(face: &topods::Face, connections: &[(topods::Face, topods::Face)]) -> bool {
    connections
        .iter()
        .any(|(a, b)| a.is_same(face) || b.is_same(face))
}

fn collect_group(
    first: &topods::Face,
    shell_faces: &[topods::Face],
    connections: &[(topods::Face, topods::Face)],
    visited: &mut Vec<topods::Face>,
) -> Vec<topods::Face> {
    let mut pending = vec![first.clone()];
    let mut group = Vec::new();
    while let Some(face) = pending.pop() {
        if contains_face(visited, &face) || !contains_face(shell_faces, &face) {
            continue;
        }
        visited.push(face.clone());
        group.push(face.clone());
        append_neighbors(&face, connections, &mut pending);
    }
    group
}

fn append_neighbors(
    face: &topods::Face,
    connections: &[(topods::Face, topods::Face)],
    pending: &mut Vec<topods::Face>,
) {
    for (first, second) in connections {
        let neighbor = if first.is_same(face) {
            Some(second)
        } else if second.is_same(face) {
            Some(first)
        } else {
            None
        };
        if let Some(neighbor) = neighbor.filter(|candidate| !contains_face(pending, candidate)) {
            pending.push(neighbor.clone());
        }
    }
}

fn contains_face(faces: &[topods::Face], candidate: &topods::Face) -> bool {
    faces.iter().any(|face| face.is_same(candidate))
}

fn sew_groups(
    groups: &[Vec<topods::Face>],
    sewing_tolerance: f64,
) -> Result<Vec<(topods::Face, topods::Face)>, Failure> {
    let mut replacements = Vec::new();
    for group in groups {
        sew_group(group, sewing_tolerance, &mut replacements)?;
    }
    Ok(replacements)
}

fn sew_group(
    group: &[topods::Face],
    sewing_tolerance: f64,
    replacements: &mut Vec<(topods::Face, topods::Face)>,
) -> Result<(), Failure> {
    let mut sewing = BRepBuilderAPISewing::new(sewing_tolerance, true, true, true, false);
    for face in group {
        sewing.add(&face.clone().into_shape())?;
    }
    sewing.perform(&ProgressRange::default())?;
    for face in group {
        let image = topods::face(&sewing.modified(&face.clone().into_shape()))?;
        replacements.push((face.clone(), image));
    }
    Ok(())
}

fn rebuild_shell(
    faces: &[topods::Face],
    replacements: &[(topods::Face, topods::Face)],
) -> Result<Shell, Failure> {
    let builder = Builder;
    let mut result = Shell::default();
    builder.make_shell(&mut result);
    let mut result_shape = result.into_shape();
    for face in faces {
        let image = replacement_for(face, replacements);
        builder.add(&mut result_shape, &image.clone().into_shape())?;
    }
    topods::shell(&result_shape)
}

fn replacement_for<'a>(
    face: &'a topods::Face,
    replacements: &'a [(topods::Face, topods::Face)],
) -> &'a topods::Face {
    replacements
        .iter()
        .find(|(original, _)| original.is_same(face))
        .map_or(face, |(_, image)| image)
}
