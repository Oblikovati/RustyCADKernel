// SPDX-License-Identifier: GPL-2.0-only
//! Exact combinatorial topology: edge uses per shell and the Euler-Poincare census.
//! Shells of solids must be closed two-manifolds with opposite edge uses; free shells
//! (sheets) may have boundary edges, so only manifoldness and orientation apply to them.
use super::naming::TopologyNames;
use super::report::{EulerCensus, Finding, FindingKind, Severity, TopologyRef};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topexp::{Explorer, map_shapes_of_type};
use rustycad_tkbrep::topods::{self, Shape};
use rustycad_tkbrep::toptools::IndexedMapOfShape;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};

/// The census findings of `root` and one Euler census per closed solid.
/// Example: `let (findings, census) = topology_census(&solid, &names)?;`.
pub(super) fn topology_census(
    root: &Shape,
    names: &TopologyNames,
) -> Result<(Vec<Finding>, Vec<EulerCensus>), Failure> {
    let actual = root.shape_type()?;
    if !matches!(
        actual,
        ShapeEnum::Compound | ShapeEnum::CompSolid | ShapeEnum::Solid | ShapeEnum::Shell
    ) {
        let kind = FindingKind::UnsupportedShape { actual };
        return Ok((vec![defect(kind, names.name(root), None)], Vec::new()));
    }
    let (mut findings, mut census) = (Vec::new(), Vec::new());
    for solid in children(root, ShapeEnum::Solid, ShapeEnum::Shape)? {
        solid_census(&solid, names, &mut findings, &mut census)?;
    }
    for shell in children(root, ShapeEnum::Shell, ShapeEnum::Solid)? {
        findings.extend(edge_use_findings(&shell, names, false)?);
    }
    Ok((findings, census))
}

fn solid_census(
    solid: &Shape,
    names: &TopologyNames,
    findings: &mut Vec<Finding>,
    census: &mut Vec<EulerCensus>,
) -> Result<(), Failure> {
    let before = findings.len();
    for shell in children(solid, ShapeEnum::Shell, ShapeEnum::Shape)? {
        findings.extend(edge_use_findings(&shell, names, true)?);
    }
    if findings.len() > before {
        return Ok(());
    }
    let counts = euler_census(solid, names)?;
    census.push(counts);
    if violates_euler_poincare(counts.genus) {
        let kind = FindingKind::EulerPoincare { census: counts };
        findings.push(defect(kind, names.name(solid), None));
    }
    Ok(())
}

/// One oriented use of an edge by a face.
struct EdgeUse {
    orientation: Orientation,
    face: Shape,
}

fn edge_use_findings(
    shell: &Shape,
    names: &TopologyNames,
    closed: bool,
) -> Result<Vec<Finding>, Failure> {
    let (edges, uses) = edge_uses(shell)?;
    let mut findings = Vec::new();
    for (index, edge_uses) in uses.iter().enumerate() {
        let edge = names.name(edges.find_key(index + 1)?);
        findings.extend(edge_use_finding(edge_uses, edge, names, closed));
    }
    Ok(findings)
}

/// Free (closed shells only), non-manifold, or same-direction uses of one edge.
fn edge_use_finding(
    uses: &[EdgeUse],
    edge: TopologyRef,
    names: &TopologyNames,
    closed: bool,
) -> Option<Finding> {
    let (kind, witness) = match uses.len() {
        1 if closed => (FindingKind::FreeEdge, 0),
        2 if uses[0].orientation == uses[1].orientation => {
            (FindingKind::InconsistentOrientation, 1)
        }
        n if n > 2 => (FindingKind::NonManifoldEdge { uses: n }, 0),
        _ => return None,
    };
    Some(defect(kind, edge, Some(names.name(&uses[witness].face))))
}

/// Every boundary use of every non-degenerate edge of `shell`, grouped by edge.
fn edge_uses(shell: &Shape) -> Result<(IndexedMapOfShape, Vec<Vec<EdgeUse>>), Failure> {
    let (mut edges, mut uses) = (IndexedMapOfShape::default(), Vec::<Vec<EdgeUse>>::new());
    for face in children(shell, ShapeEnum::Face, ShapeEnum::Shape)? {
        for edge in children(&face, ShapeEnum::Edge, ShapeEnum::Shape)? {
            let orientation = edge.orientation();
            let bounding = matches!(orientation, Orientation::Forward | Orientation::Reversed);
            if !bounding || BRepTool::degenerated(&topods::edge(&edge)?)? {
                continue;
            }
            let index = edges.add(edge);
            uses.resize_with(edges.extent(), Vec::new);
            uses[index - 1].push(EdgeUse {
                orientation,
                face: face.clone(),
            });
        }
    }
    Ok((edges, uses))
}

/// `V - E + F - H = 2 (S - G)` over the solid, `H = L - F`, degenerate edges excluded.
fn euler_census(solid: &Shape, names: &TopologyNames) -> Result<EulerCensus, Failure> {
    let (vertices, faces) = (
        count(solid, ShapeEnum::Vertex)?,
        count(solid, ShapeEnum::Face)?,
    );
    let (loops, shells) = (
        count(solid, ShapeEnum::Wire)?,
        count(solid, ShapeEnum::Shell)?,
    );
    let edges = non_degenerate_edges(solid)?;
    let characteristic = vertices as i64 - edges as i64 + 2 * faces as i64 - loops as i64;
    let genus = genus(characteristic, shells);
    let solid = names.name(solid);
    Ok(EulerCensus {
        solid,
        vertices,
        edges,
        faces,
        loops,
        shells,
        genus,
    })
}

/// `G = S - chi / 2`; None for an odd characteristic, which no closed orientable
/// two-manifold has; `genus(2, 1) == Some(0)`.
fn genus(characteristic: i64, shells: usize) -> Option<i64> {
    (characteristic % 2 == 0).then(|| shells as i64 - characteristic / 2)
}

/// An odd characteristic or a negative genus; `violates_euler_poincare(Some(-1))`.
fn violates_euler_poincare(genus: Option<i64>) -> bool {
    genus.is_none_or(|genus| genus < 0)
}

/// Distinct sub-shapes of `kind` (orientation ignored); `count(&solid, ShapeEnum::Face)?`.
fn count(shape: &Shape, kind: ShapeEnum) -> Result<usize, Failure> {
    let mut map = IndexedMapOfShape::default();
    map_shapes_of_type(shape, kind, &mut map)?;
    Ok(map.extent())
}

fn non_degenerate_edges(solid: &Shape) -> Result<usize, Failure> {
    let mut map = IndexedMapOfShape::default();
    map_shapes_of_type(solid, ShapeEnum::Edge, &mut map)?;
    let mut count = 0;
    for index in 1..=map.extent() {
        if !BRepTool::degenerated(&topods::edge(map.find_key(index)?)?)? {
            count += 1;
        }
    }
    Ok(count)
}

/// Sub-shapes of `kind` not under an `avoid` ancestor, orientation composed.
pub(super) fn children(
    shape: &Shape,
    kind: ShapeEnum,
    avoid: ShapeEnum,
) -> Result<Vec<Shape>, Failure> {
    let mut found = Vec::new();
    let mut explorer = Explorer::new(shape, kind, avoid)?;
    while explorer.more() {
        found.push(explorer.current().clone());
        explorer.next()?;
    }
    Ok(found)
}

fn defect(kind: FindingKind, subject: TopologyRef, context: Option<TopologyRef>) -> Finding {
    Finding {
        severity: Severity::Defect,
        kind,
        subject,
        context,
    }
}

#[cfg(test)]
mod tests {
    use super::{genus, violates_euler_poincare};

    #[test]
    fn genus_follows_euler_poincare() {
        // Cube 8 - 12 + 6 - 0, torus face 1 - 2 + 1 - 0, two cubes in one solid (a void).
        assert_eq!(genus(2, 1), Some(0));
        assert_eq!(genus(0, 1), Some(1));
        assert_eq!(genus(4, 2), Some(0));
        assert_eq!(genus(3, 1), None);
        assert!(violates_euler_poincare(None));
        assert!(violates_euler_poincare(genus(4, 1)));
        assert!(!violates_euler_poincare(genus(-2, 1)));
    }
}
