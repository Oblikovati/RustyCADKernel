// Rust translation of ShapeFix/ShapeFix_Wire.cxx reorder and gap entry points at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Ordered-wire state, reorder application and unchanged-gap passes.
use super::{FixRoot, FixRootData};
use crate::shape_analysis::{ShapeAnalysisWire, ShapeAnalysisWireOrder};
use crate::shape_extend::{ShapeExtend, ShapeExtendStatus, ShapeExtendWireData};
use rustycad_tkbrep::topods::{Edge, Face, Wire};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};

#[derive(Default)]
struct WireFixState {
    analyzer: Handle<ShapeAnalysisWire>,
    reorder_status: i32,
    gaps_3d_status: i32,
    gaps_2d_status: i32,
    closed_mode: bool,
}

/// The currently implemented `ShapeFix_Wire` reorder and no-gap pass.
pub struct ShapeFixWire {
    parent: StandardTransient,
    root: FixRootData,
    state: SharedState<WireFixState>,
}

impl_transient!(ShapeFixWire, "ShapeFix_Wire", StandardTransient, parent, interfaces: [dyn FixRoot]);

impl Default for ShapeFixWire {
    fn default() -> Self {
        Self {
            parent: StandardTransient::default(),
            root: FixRootData::default(),
            state: SharedState::new(WireFixState {
                analyzer: Handle::default(),
                reorder_status: 0,
                gaps_3d_status: 0,
                gaps_2d_status: 0,
                closed_mode: true,
            }),
        }
    }
}

impl ShapeFixWire {
    /// `ShapeFix_Wire()`: an unloaded fixer with closed-wire mode enabled.
    pub fn new() -> Self {
        Self::default()
    }

    /// `Init(analyzer)`: share an existing ShapeAnalysis_Wire instance.
    pub fn init(&self, analyzer: &Handle<ShapeAnalysisWire>) {
        self.state.write().analyzer = analyzer.clone();
        self.clear_statuses();
    }

    /// `Load(wire)`: replace the analyzer edge list and clear prior statuses.
    pub fn load(&self, wire: &Wire) -> Result<(), Failure> {
        let analyzer = self.ensure_analyzer();
        analyzer.load_wire(wire)?;
        self.clear_statuses();
        Ok(())
    }

    /// `SetFace(face)`: update the shared analyzer support face.
    pub fn set_face(&self, face: &Face) -> Result<(), Failure> {
        self.ensure_analyzer().set_face(face)
    }

    /// `FixReorder()`: analyze the loaded 3D endpoints, then apply the proposed order.
    pub fn fix_reorder(&self) -> Result<bool, Failure> {
        let mut order = ShapeAnalysisWireOrder::new();
        self.ensure_analyzer()
            .check_order_with(&mut order, self.closed_mode(), true, false)?;
        self.fix_reorder_with(&order)
    }

    /// `FixReorder(order)`: reorder existing wire data using its signed edge indices.
    pub fn fix_reorder_with(&self, order: &ShapeAnalysisWireOrder) -> Result<bool, Failure> {
        self.set_reorder_status(ShapeExtendStatus::Ok);
        let wire_data = self.wire_data()?;
        let count = wire_data.nb_edges();
        if count == 0 || order.status() == 0 {
            return Ok(false);
        }
        if order.status() <= -10 {
            self.set_reorder_status(ShapeExtendStatus::Fail1);
            return Ok(false);
        }
        if count != order.nb_edges() {
            self.set_reorder_status(ShapeExtendStatus::Fail2);
            return Ok(false);
        }
        self.apply_order(&wire_data, order, count)?;
        self.record_reorder_status(order.status());
        Ok(true)
    }

    /// `FixGaps3d()`: run the native adjacent-curve analysis when no gap needs repair.
    pub fn fix_gaps_3d(&self) -> Result<bool, Failure> {
        self.set_gap_status(true, ShapeExtendStatus::Ok);
        self.fix_gap_pass(true)
    }

    /// `FixGaps2d()`: run the native adjacent-pcurve analysis when no gap needs repair.
    pub fn fix_gaps_2d(&self) -> Result<bool, Failure> {
        self.set_gap_status(false, ShapeExtendStatus::Ok);
        self.fix_gap_pass(false)
    }

    /// `StatusReorder(status)`: query reorder flags.
    pub fn status_reorder(&self, status: ShapeExtendStatus) -> bool {
        ShapeExtend::decode_status(self.state.read().reorder_status, status)
    }

    /// `StatusGaps3d(status)`: query 3D gap-fixing flags.
    pub fn status_gaps_3d(&self, status: ShapeExtendStatus) -> bool {
        ShapeExtend::decode_status(self.state.read().gaps_3d_status, status)
    }

    /// `StatusGaps2d(status)`: query 2D gap-fixing flags.
    pub fn status_gaps_2d(&self, status: ShapeExtendStatus) -> bool {
        ShapeExtend::decode_status(self.state.read().gaps_2d_status, status)
    }

    fn ensure_analyzer(&self) -> Handle<ShapeAnalysisWire> {
        let mut state = self.state.write();
        if state.analyzer.is_null() {
            state.analyzer = Handle::new(ShapeAnalysisWire::new());
        }
        state.analyzer.clone()
    }

    fn analyzer(&self) -> Result<Handle<ShapeAnalysisWire>, Failure> {
        let analyzer = self.state.read().analyzer.clone();
        if analyzer.is_null() {
            return Err(Failure::new(
                "ShapeFix_Wire requires Init(analyzer) or Load(wire)",
            ));
        }
        Ok(analyzer)
    }

    fn wire_data(&self) -> Result<Handle<ShapeExtendWireData>, Failure> {
        Ok(self.analyzer()?.wire_data())
    }

    fn closed_mode(&self) -> bool {
        self.state.read().closed_mode
    }

    fn apply_order(
        &self,
        wire_data: &Handle<ShapeExtendWireData>,
        order: &ShapeAnalysisWireOrder,
        count: i32,
    ) -> Result<(), Failure> {
        let old_edges = (1..=count)
            .map(|index| wire_data.edge(index))
            .collect::<Result<Vec<Edge>, _>>()?;
        for rank in 1..=count {
            let Some(edge) = self.reordered_edge(&old_edges, order, rank, count)? else {
                return Ok(());
            };
            wire_data.set(&edge, rank)?;
        }
        Ok(())
    }

    fn reordered_edge(
        &self,
        edges: &[Edge],
        order: &ShapeAnalysisWireOrder,
        rank: i32,
        count: i32,
    ) -> Result<Option<Edge>, Failure> {
        let signed_index = order.ordered(rank)?;
        if signed_index == 0 {
            self.set_reorder_status(ShapeExtendStatus::Fail3);
            return Ok(None);
        }
        let old_index = signed_index.abs();
        if old_index > count {
            return Err(invalid_order_index(signed_index, count));
        }
        let mut edge = edges[(old_index - 1) as usize].clone();
        if signed_index < 0 {
            edge.reverse();
        }
        Ok(Some(edge))
    }

    fn record_reorder_status(&self, order_status: i32) {
        self.set_reorder_status(ShapeExtendStatus::Done1);
        if matches!(order_status, 2 | -2) {
            self.or_reorder_status(ShapeExtendStatus::Done2);
        }
        if order_status < 0 {
            self.or_reorder_status(ShapeExtendStatus::Done3);
        }
        if order_status == 3 {
            self.or_reorder_status(ShapeExtendStatus::Done5);
        }
    }

    fn fix_gap_pass(&self, is_3d: bool) -> Result<bool, Failure> {
        let analyzer = self.analyzer()?;
        if !analyzer.is_loaded() {
            return Ok(false);
        }
        if !is_3d && !analyzer.is_ready() {
            self.set_gap_status(false, ShapeExtendStatus::Fail1);
            return Ok(false);
        }
        let start = if self.closed_mode() { 1 } else { 2 };
        for index in start..=analyzer.nb_edges() {
            self.fix_gap_pair(&analyzer, is_3d, index)?;
        }
        Ok(self.gap_status(is_3d, ShapeExtendStatus::Done))
    }

    fn fix_gap_pair(
        &self,
        analyzer: &ShapeAnalysisWire,
        is_3d: bool,
        index: i32,
    ) -> Result<(), Failure> {
        let changed = if is_3d {
            analyzer.check_gap_3d_at(index)?
        } else {
            analyzer.check_gap_2d_at(index)?
        };
        if analyzer.last_check_status(ShapeExtendStatus::Fail) {
            self.set_gap_status(is_3d, ShapeExtendStatus::Fail1);
            return Ok(());
        }
        if changed {
            return Err(unsupported_gap(index, is_3d, analyzer));
        }
        Ok(())
    }

    fn clear_statuses(&self) {
        let mut state = self.state.write();
        state.reorder_status = 0;
        state.gaps_3d_status = 0;
        state.gaps_2d_status = 0;
    }

    fn set_reorder_status(&self, status: ShapeExtendStatus) {
        self.state.write().reorder_status = ShapeExtend::encode_status(status);
    }

    fn or_reorder_status(&self, status: ShapeExtendStatus) {
        self.state.write().reorder_status |= ShapeExtend::encode_status(status);
    }

    fn set_gap_status(&self, is_3d: bool, status: ShapeExtendStatus) {
        let mut state = self.state.write();
        let flags = if is_3d {
            &mut state.gaps_3d_status
        } else {
            &mut state.gaps_2d_status
        };
        *flags |= ShapeExtend::encode_status(status);
    }

    fn gap_status(&self, is_3d: bool, status: ShapeExtendStatus) -> bool {
        let state = self.state.read();
        let flags = if is_3d {
            state.gaps_3d_status
        } else {
            state.gaps_2d_status
        };
        ShapeExtend::decode_status(flags, status)
    }
}

impl FixRoot for ShapeFixWire {
    fn fix_root_data(&self) -> &FixRootData {
        &self.root
    }
}

fn invalid_order_index(index: i32, count: i32) -> Failure {
    Failure::with_message(
        "Standard_OutOfRange",
        &format!(
            "ShapeFix_Wire::FixReorder: ordered index {index}; expected an index within 1..={count}"
        ),
    )
}

fn unsupported_gap(index: i32, is_3d: bool, analyzer: &ShapeAnalysisWire) -> Failure {
    let (space, distance) = if is_3d {
        ("3D", analyzer.max_distance_3d())
    } else {
        ("2D", analyzer.max_distance_2d())
    };
    Failure::with_message(
        "Standard_NotImplemented",
        &format!(
            "ShapeFix_Wire gap correction is not ported for {space} pair {index}, gap {distance}"
        ),
    )
}
