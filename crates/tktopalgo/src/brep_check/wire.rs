// Rust translation of ModelingAlgorithms/TKTopAlgo/BRepCheck/BRepCheck_Wire.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepCheck_Wire`: the checker of a wire (connected, closed in 3d and on a face,
//! oriented, not self-intersecting). `Closed` and `Closed2d` are in `wire_closed.rs`,
//! `Orientation` in `wire_orientation.rs`, `SelfIntersect` in `wire_self_intersect.rs`.
use super::explore::{holds_same, sub_shapes};
use super::result::{BRepCheckResult, ResultBase, add_status, close_status_list};
use super::status::BRepCheckStatus;
use rustycad_tkbrep::topods::{Edge, Shape, Wire, face};
use rustycad_tkbrep::toptools::{IndexedDataMapOfShapeListOfShape, MapOfShape};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::{Failure, SharedState, StandardTransient};
use rustycad_tkg3d::top_abs::ShapeEnum;

/// The own fields of `BRepCheck_Wire`.
pub(super) struct WireFields {
    /// `myCdone`: `Closed` has run.
    pub(super) cdone: bool,
    /// `myCstat`: the result of `Closed`.
    pub(super) cstat: BRepCheckStatus,
    /// `myMapVE`: each (oriented) vertex and the edges through it.
    pub(super) map_ve: IndexedDataMapOfShapeListOfShape,
    /// `myGctrl`: `SelfIntersect` is part of InContext.
    pub(super) gctrl: bool,
}

/// `BRepCheck_Wire`: checks a wire; in the context of a face it consequently checks
/// `SelfIntersect`, `Closed`, `Orientation` and `Closed2d` until one fails.
///
/// ```
/// use rustycad_tkbrep::brep::BRepBuilder;
/// use rustycad_tkbrep::topods::Wire;
/// use rustycad_tktopalgo::brep_check::{BRepCheckResult, BRepCheckStatus, BRepCheckWire};
/// let mut w = Wire::default();
/// BRepBuilder::default().make_wire(&mut w);
/// let checker = BRepCheckWire::new(&w).unwrap();
/// assert_eq!(checker.status().unwrap().first(), Some(&BRepCheckStatus::EmptyWire));
/// ```
pub struct BRepCheckWire {
    transient: StandardTransient,
    base: ResultBase,
    pub(super) fields: SharedState<WireFields>,
}
impl_transient!(BRepCheckWire, "BRepCheck_Wire", dyn BRepCheckResult, transient,
    interfaces: [dyn BRepCheckResult]);

impl BRepCheckWire {
    /// `BRepCheck_Wire(W)`: `Init(W)`, whose minimum check turns the geometric controls
    /// on; `BRepCheckWire::new(&wire)?`.
    pub fn new(wire: &Wire) -> Result<Self, Failure> {
        let checker = Self {
            transient: StandardTransient::default(),
            base: ResultBase::new(),
            fields: SharedState::new(WireFields {
                cdone: false,
                cstat: BRepCheckStatus::NoError,
                map_ve: IndexedDataMapOfShapeListOfShape::default(),
                gctrl: false,
            }),
        };
        checker.init(wire)?;
        Ok(checker)
    }

    /// `SetStatus(theStatus)`: adds `status` to the statuses of the wire;
    /// `checker.set_status(BRepCheckStatus::InvalidWire)?`.
    pub fn set_status(&self, status: BRepCheckStatus) -> Result<(), Failure> {
        add_status(&self.base.own_list()?, status);
        Ok(())
    }

    /// `GeometricControls(B)`: turning the controls on makes `Closed` run again;
    /// `checker.set_geometric_controls(false)`.
    pub fn set_geometric_controls(&self, controls: bool) {
        let mut fields = self.fields.write();
        if fields.gctrl != controls {
            if controls {
                fields.cdone = false;
            }
            fields.gctrl = controls;
        }
    }

    /// `GeometricControls()`; `checker.geometric_controls()`.
    pub fn geometric_controls(&self) -> bool {
        self.fields.read().gctrl
    }

    /// The wire being checked.
    pub(super) fn wire(&self) -> Wire {
        Wire::unchecked(self.base.shape())
    }

    /// The wire InContext of a face: the first failing check among `SelfIntersect`
    /// (with the geometric controls), `Closed`, `Orientation` and `Closed2d`.
    fn check_in_face(&self, context: &Shape) -> Result<BRepCheckStatus, Failure> {
        let f = face(context)?;
        if self.geometric_controls() {
            let (mut e1, mut e2) = (Edge::default(), Edge::default());
            let status = self.self_intersect(&f, &mut e1, &mut e2, true)?;
            if status != BRepCheckStatus::NoError {
                return Ok(status);
            }
        }
        let status = self.closed(false)?;
        if status != BRepCheckStatus::NoError {
            return Ok(status);
        }
        let status = self.orientation(&f, false)?;
        if status != BRepCheckStatus::NoError {
            return Ok(status);
        }
        self.closed2d(&f, false)
    }
}

/// `Propagate(mapVE, edg, mapE)`: fills `map_e` with the edges connected to `edge`
/// through the vertices of `map_ve`, breadth first.
pub(super) fn propagate(
    map_ve: &IndexedDataMapOfShapeListOfShape,
    edge: &Shape,
    map_e: &mut MapOfShape,
) -> Result<(), Failure> {
    let mut current: Vec<Shape> = vec![edge.clone()];
    while !current.is_empty() {
        let mut next: Vec<Shape> = Vec::new();
        for edge in &current {
            if !map_e.contains(edge) {
                map_e.add(edge.clone());
            }
            for vertex in sub_shapes(edge, ShapeEnum::Vertex)? {
                let index = map_ve.find_index(&vertex);
                if index == 0 {
                    continue;
                }
                for other in map_ve.find_from_index(index)? {
                    if !edge.is_same(other) && !map_e.contains(other) {
                        map_e.add(other.clone());
                        next.push(other.clone());
                    }
                }
            }
        }
        current = next;
    }
    Ok(())
}

/// Appends `value` to the list of `key` in `map` (a vertex and its edges, an edge and
/// its faces), adding the key first.
pub(super) fn append_under_key(
    map: &mut IndexedDataMapOfShapeListOfShape,
    key: &Shape,
    value: &Shape,
) -> Result<(), Failure> {
    let mut index = map.find_index(key);
    if index == 0 {
        index = map.add(key.clone(), List::default());
    }
    map.change_from_index(index)?.append(value.clone());
    Ok(())
}

impl BRepCheckResult for BRepCheckWire {
    fn result_base(&self) -> &ResultBase {
        &self.base
    }

    /// `InContext(S)`: `SubshapeNotInShape` when the wire is not in `context`, else in a
    /// face the first failing check (see [`BRepCheckWire`]).
    fn in_context(&self, context: &Shape) -> Result<(), Failure> {
        let Some(list) = self.base.bind_context_list(context) else {
            return Ok(());
        };
        if !holds_same(context, ShapeEnum::Wire, &self.base.shape())? {
            add_status(&list, BRepCheckStatus::SubshapeNotInShape);
            return Ok(());
        }
        let status = match context.shape_type()? {
            ShapeEnum::Face => self.check_in_face(context)?,
            _ => BRepCheckStatus::NoError,
        };
        if status != BRepCheckStatus::NoError {
            add_status(&list, status);
        }
        close_status_list(&list);
        Ok(())
    }

    /// `Minimum()`: `EmptyWire` without edges, `NotConnected` when the edges are not all
    /// connected through their vertices; always turns the geometric controls on and
    /// resets `Closed`.
    fn minimum(&self) -> Result<(), Failure> {
        {
            let mut fields = self.fields.write();
            fields.cdone = false;
            fields.gctrl = true;
        }
        if self.base.is_minimum() {
            return Ok(());
        }
        let list = self.base.bind_own_list();
        let edges = sub_shapes(&self.base.shape(), ShapeEnum::Edge)?;
        let mut map_ve = IndexedDataMapOfShapeListOfShape::default();
        for edge in &edges {
            for vertex in sub_shapes(edge, ShapeEnum::Vertex)? {
                append_under_key(&mut map_ve, &vertex, edge)?;
            }
        }
        if edges.is_empty() {
            add_status(&list, BRepCheckStatus::EmptyWire);
        } else if edges.len() >= 2 {
            let mut map_e = MapOfShape::default();
            propagate(&map_ve, &edges[0], &mut map_e)?;
            if edges.iter().any(|edge| !map_e.contains(edge)) {
                add_status(&list, BRepCheckStatus::NotConnected);
            }
        }
        close_status_list(&list);
        self.fields.write().map_ve.clear(true);
        self.base.set_minimum(true);
        Ok(())
    }

    /// `Blind()`: nothing more than the minimum.
    fn blind(&self) {
        self.base.set_blind(true);
    }
}
