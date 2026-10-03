//! Native shell containment graph, including the three-on-vertices fallback.
use super::from_shell::{build_solid, infinite_state};
use rustycad_tkbrep::{
    brep::BRepTool,
    topexp::Explorer,
    topods::{self, Shape, Shell},
    toptools::{MapOfShape, ShapeMapHasher},
};
use rustycad_tkernel::{
    ncollection::{DataMap, IndexedDataMap},
    precision::confusion,
    standard::Failure,
};
use rustycad_tkg3d::top_abs::{ShapeEnum, State};
use rustycad_tktopalgo::brep_class3d::BRepClass3dSolidClassifier;
pub(super) type ShellStatus = DataMap<Shape, i32, ShapeMapHasher>;
pub(super) type ShellHoles = IndexedDataMap<Shape, Vec<Shape>, ShapeMapHasher>;
type HoleLists = DataMap<Shape, Vec<Shape>, ShapeMapHasher>;
pub(super) fn collect_shells(
    shells: &[Shell],
    status: &mut ShellStatus,
) -> Result<ShellHoles, Failure> {
    let mut holes = MapOfShape::default();
    let mut lists = HoleLists::default();
    for shell in shells {
        lists.bind(shell.clone().into_shape(), Vec::new());
    }
    for (index, shell) in shells.iter().enumerate() {
        if !is_closed_nonempty(shell)? {
            continue;
        }
        // OCCT catches failures for each root shell, retaining earlier mutations.
        let _ = classify_containment(index, shells, status, &mut lists, &mut holes);
    }
    remove_nested_holes(&lists, &mut holes)?;
    let mut result = ShellHoles::default();
    for shell in shells {
        if !holes.contains(shell) {
            result.add(shell.clone().into_shape(), lists.find(shell)?.clone());
        }
    }
    Ok(result)
}
pub(super) fn is_closed_nonempty(shell: &Shell) -> Result<bool, Failure> {
    Ok(BRepTool::is_closed(shell)?
        && Explorer::new(shell, ShapeEnum::Edge, ShapeEnum::Shape)?.more())
}
fn classify_containment(
    index: usize,
    shells: &[Shell],
    status: &mut ShellStatus,
    lists: &mut HoleLists,
    holes: &mut MapOfShape,
) -> Result<(), Failure> {
    let shell = &shells[index];
    let mut classifier = BRepClass3dSolidClassifier::with_shape(&build_solid(shell)?.into_shape())?;
    let infinity = cached_infinity(shell, &mut classifier, status)?;
    if infinity == State::Unknown {
        return Ok(());
    }
    for (other_index, other) in shells.iter().enumerate() {
        if other_index == index || !BRepTool::is_closed(other)? || holes.contains(other) {
            continue;
        }
        if lists.find(other)?.iter().any(|hole| hole.is_same(shell)) {
            continue;
        }
        if sample_shell(other, &mut classifier)? == infinity {
            continue;
        }
        lists.change_find(shell)?.push(other.clone().into_shape());
        if !holes.remove(other) {
            holes.add(other.clone().into_shape());
        }
    }
    Ok(())
}
fn cached_infinity(
    shell: &Shell,
    classifier: &mut BRepClass3dSolidClassifier,
    status: &mut ShellStatus,
) -> Result<State, Failure> {
    if let Ok(value) = status.find(shell) {
        return Ok(decode_infinity(*value));
    }
    classifier.perform_infinite_point(confusion())?;
    let value = match classifier.state() {
        State::In => 1,
        State::Out => 2,
        _ => 0,
    };
    status.bind(shell.clone().into_shape(), value);
    Ok(decode_infinity(value))
}
fn decode_infinity(value: i32) -> State {
    match value {
        1 => State::In,
        2 => State::Out,
        _ => State::Unknown,
    }
}
fn sample_shell(
    shell: &Shell,
    classifier: &mut BRepClass3dSolidClassifier,
) -> Result<State, Failure> {
    let mut vertices = Vec::new();
    let mut seen = MapOfShape::default();
    for vertex in Explorer::new(shell, ShapeEnum::Vertex, ShapeEnum::Shape)? {
        if seen.add(vertex.clone()) {
            vertices.push(vertex);
        }
        if vertices.len() == 10 {
            break;
        }
    }
    classify_vertices(&vertices, classifier)
}
fn classify_vertices(
    vertices: &[Shape],
    classifier: &mut BRepClass3dSolidClassifier,
) -> Result<State, Failure> {
    let mut state = State::Unknown;
    let mut on_count = 0;
    for vertex in vertices {
        if state != State::Unknown && (state != State::On || on_count >= 3) {
            break;
        }
        classifier.perform(&BRepTool::pnt(&topods::vertex(vertex)?)?, confusion())?;
        state = classifier.state();
        if state == State::On {
            on_count += 1;
        }
    }
    // The native middle-point alternative is disabled at the pinned revision.
    Ok(if on_count == 3 && state == State::On {
        State::Out
    } else {
        state
    })
}
fn remove_nested_holes(lists: &HoleLists, holes: &mut MapOfShape) -> Result<(), Failure> {
    for (shell, contained) in lists.items() {
        if holes.contains(shell) {
            continue;
        }
        for hole in contained {
            if !holes.contains(hole) {
                continue;
            }
            for nested in lists.find(hole)? {
                holes.remove(nested);
            }
        }
    }
    Ok(())
}
pub(super) fn shell_infinity(
    shell: &Shell,
    status: &ShellStatus,
    hole: bool,
) -> Result<State, Failure> {
    if let Ok(value) = status.find(shell) {
        // Native hole handling interprets cached zero as OUT; root handling does not.
        return Ok(if hole && *value != 1 {
            State::Out
        } else {
            decode_infinity(*value)
        });
    }
    infinite_state(&build_solid(shell)?)
}
