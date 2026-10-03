//! Source GetShells traversal; retain its edge-state consumption and face retry order.
use super::groups::{FaceConnectivity, ShapeMap};
use rustycad_tkbrep::{
    brep::{BRepBuilder, BRepTool},
    topods::{Edge, Face, Shell},
    toptools::MapOfShape,
};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::Orientation;
#[derive(Default)]
pub(super) struct ShellGroups {
    pub shells: Vec<Shell>,
    pub errors: Vec<rustycad_tkbrep::topods::Shape>,
    pub face_shells: ShapeMap<Shell>,
    pub changed: bool,
}
struct GroupBuild<'a> {
    graph: &'a FaceConnectivity,
    multiple: &'a MapOfShape,
    remaining: Vec<Face>,
    unconnected: Vec<Face>,
    processed: ShapeMap<(bool, bool)>,
    shell: Shell,
    face_count: usize,
    output: &'a mut ShellGroups,
}
struct OrientationScan {
    bad: usize,
    good: usize,
    pending: ShapeMap<(bool, bool)>,
}
pub(super) fn build_shell_group(
    faces: &mut Vec<Face>,
    multiple: &MapOfShape,
    output: &mut ShellGroups,
) -> Result<(), Failure> {
    if faces.is_empty() {
        return Ok(());
    }
    let graph = FaceConnectivity::new(faces)?;
    let mut groups = graph.connected_groups()?;
    if groups.is_empty() {
        return Ok(());
    }
    let first = groups.remove(0);
    let mut work = GroupBuild {
        graph: &graph,
        multiple,
        remaining: first,
        unconnected: Vec::new(),
        processed: ShapeMap::default(),
        shell: empty_shell(),
        face_count: 1,
        output,
    };
    work.process()?;
    work.finish()?;
    for group in groups {
        work.unconnected.extend(group);
    }
    work.remaining.extend(work.unconnected);
    *faces = work.remaining;
    Ok(())
}
impl GroupBuild<'_> {
    fn process(&mut self) -> Result<(), Failure> {
        let mut index = 0;
        while index < self.remaining.len() {
            let face = self.remaining[index].clone();
            let scan = scan_edges(
                self.graph.face_edges(&face),
                self.multiple,
                &mut self.processed,
            );
            if scan.bad == 0 && scan.good == 0 && scan.pending.is_empty() {
                index += 1;
                continue;
            }
            if scan.bad != 0 && scan.good != 0 {
                self.output.errors.push(face.into_shape());
                self.remaining.remove(index);
                self.face_count += 1;
                index += 1;
                continue;
            }
            if scan.bad != 0 || scan.good != 0 || self.face_count == 1 {
                self.add_face(face, scan)?;
                self.remaining.remove(index);
                index = 0;
                continue;
            }
            if index + 1 == self.remaining.len() && self.face_count <= 2 {
                self.defer_single()?;
                index = 0;
                continue;
            }
            index += 1;
        }
        Ok(())
    }
    fn add_face(&mut self, mut face: Face, scan: OrientationScan) -> Result<(), Failure> {
        if scan.bad != 0 {
            face.reverse();
            self.output.changed = true;
        }
        for (edge, pair) in scan.pending.items() {
            let pair = if scan.bad != 0 {
                (!pair.0, !pair.1)
            } else {
                *pair
            };
            if let Some(previous) = self.processed.change_seek(edge) {
                *previous = pair;
            } else {
                self.processed.bind(edge.clone(), pair);
            }
        }
        self.face_count += 1;
        BRepBuilder::default().add(&mut self.shell, &face)?;
        self.output
            .face_shells
            .bind(face.into_shape(), self.shell.clone());
        if !self.multiple.is_empty() && BRepTool::is_closed(&self.shell)? {
            self.shell.set_closed(true)?;
            self.output.shells.push(self.shell.clone());
            self.reset_shell();
        }
        Ok(())
    }
    fn reset_shell(&mut self) {
        self.shell = empty_shell();
        self.face_count = 1;
    }
    fn defer_single(&mut self) -> Result<(), Failure> {
        let mut iterator = rustycad_tkbrep::topods::ShapeIterator::new(&self.shell, false, true);
        if iterator.more() {
            let shape = iterator.value()?;
            self.unconnected.push(rustycad_tkbrep::topods::face(shape)?);
            self.output.face_shells.unbind(shape);
        }
        iterator.next();
        self.reset_shell();
        Ok(())
    }
    fn finish(&mut self) -> Result<(), Failure> {
        if self
            .output
            .shells
            .iter()
            .any(|shell| shell.is_same(&self.shell))
        {
            return Ok(());
        }
        let faces = rustycad_tkbrep::topods::ShapeIterator::new(&self.shell, false, true)
            .collect::<Vec<_>>();
        if faces.len() > 1 {
            if self.multiple.is_empty() {
                self.shell.set_closed(BRepTool::is_closed(&self.shell)?)?;
            }
            self.output.shells.push(self.shell.clone());
        } else if let Some(face) = faces.first() {
            self.output.face_shells.unbind(face);
            self.remaining.push(rustycad_tkbrep::topods::face(face)?);
        }
        Ok(())
    }
}
fn scan_edges(
    edges: &[Edge],
    multiple: &MapOfShape,
    processed: &mut ShapeMap<(bool, bool)>,
) -> OrientationScan {
    let mut scan = OrientationScan {
        bad: 0,
        good: 0,
        pending: ShapeMap::default(),
    };
    for edge in edges {
        if multiple.contains(edge) {
            continue;
        }
        let orientation = edge.orientation();
        if let Some(pair) = processed.change_seek(edge) {
            consume_edge(orientation, pair, &mut scan);
            if !pair.0 && !pair.1 {
                processed.unbind(edge);
            }
        } else {
            let pair = scan
                .pending
                .try_bound(edge.clone().into_shape(), (false, false));
            pair.0 |= orientation == Orientation::Forward;
            pair.1 |= orientation == Orientation::Reversed;
        }
    }
    scan
}
fn consume_edge(orientation: Orientation, pair: &mut (bool, bool), scan: &mut OrientationScan) {
    if (orientation == Orientation::Forward && pair.0)
        || (orientation == Orientation::Reversed && pair.1)
    {
        scan.bad += 1;
    } else if (orientation == Orientation::Forward && pair.1)
        || (orientation == Orientation::Reversed && pair.0)
    {
        scan.good += 1;
    }
    if pair.0 {
        pair.0 = false;
    } else if pair.1 {
        pair.1 = false;
    }
}
pub(super) fn empty_shell() -> Shell {
    let mut shell = Shell::default();
    BRepBuilder::default().make_shell(&mut shell);
    shell
}
