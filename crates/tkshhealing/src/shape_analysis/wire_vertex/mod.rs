// ShapeAnalysis_WireVertex at OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
mod analyze;
mod records;
use crate::shape_extend::ShapeExtendWireData;
use rustycad_tkbrep::topods::Wire;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle, SharedState};
use rustycad_tkmath::gp::Xyz;
use std::sync::Arc;

#[derive(Clone, Copy, Default)]
struct VertexRecord {
    status: i32,
    position: Xyz,
    previous: f64,
    following: f64,
}

/// Analyze consecutive edge vertices; `ShapeAnalysisWireVertex::new()`.
/// Copies share the native record arrays, while precision and done flags are copied.
#[derive(Clone)]
pub struct ShapeAnalysisWireVertex {
    wire: Handle<ShapeExtendWireData>,
    records: Arc<SharedState<Vec<VertexRecord>>>,
    precision: f64,
    done: bool,
}
impl Default for ShapeAnalysisWireVertex {
    fn default() -> Self {
        Self {
            wire: Handle::default(),
            records: Arc::new(SharedState::new(Vec::new())),
            precision: precision::confusion(),
            done: false,
        }
    }
}
impl ShapeAnalysisWireVertex {
    /// Construct unloaded analysis; `ShapeAnalysisWireVertex::new()`.
    pub fn new() -> Self {
        Self::default()
    }
    /// Initialize from a wire; `tool.init(&wire, 1e-4)?`.
    /// Native initialization ignores precision; use `set_precision` to change it.
    pub fn init(&mut self, wire: &Wire, precision: f64) -> Result<(), Failure> {
        let wire = Handle::new(ShapeExtendWireData::from_wire(wire, true, true)?);
        self.init_wire_data(&wire, precision);
        Ok(())
    }
    /// Share a wire and allocate records; `tool.init_wire_data(&wire, 1e-4)`.
    pub fn init_wire_data(&mut self, wire: &Handle<ShapeExtendWireData>, _precision: f64) {
        let count = wire.nb_edges();
        if count == 0 {
            return;
        }
        self.done = false;
        self.wire = wire.clone();
        self.records = Arc::new(SharedState::new(vec![
            VertexRecord::default();
            count as usize
        ]));
    }
    /// Load a wire preserving precision; `tool.load(&wire)?`.
    pub fn load(&mut self, wire: &Wire) -> Result<(), Failure> {
        self.init(wire, self.precision)
    }
    /// Load shared wire data; `tool.load_wire_data(&wire)`.
    pub fn load_wire_data(&mut self, wire: &Handle<ShapeExtendWireData>) {
        self.init_wire_data(wire, self.precision);
    }
    /// Set analysis precision and invalidate done; `tool.set_precision(1e-4)`.
    pub fn set_precision(&mut self, precision: f64) {
        self.precision = precision;
        self.done = false;
    }
    /// Query whether analysis ran; `tool.is_done()`.
    pub fn is_done(&self) -> bool {
        self.done
    }
    /// Return working precision; `tool.precision()`.
    pub fn precision(&self) -> f64 {
        self.precision
    }
    /// Return the loaded edge count; `tool.nb_edges()`.
    pub fn nb_edges(&self) -> i32 {
        self.wire.nb_edges()
    }
    /// Share the loaded wire; `tool.wire_data()`.
    pub fn wire_data(&self) -> Handle<ShapeExtendWireData> {
        self.wire.clone()
    }
    fn record_index(&self, num: i32) -> Result<usize, Failure> {
        let count = self.records.read().len();
        if num > 0 && num as usize <= count {
            return Ok(num as usize - 1);
        }
        Err(Failure::with_message(
            "Standard_OutOfRange",
            &format!("wire vertex index {num}; expected 1..={count}"),
        ))
    }
    fn read_record(&self, num: i32) -> Result<VertexRecord, Failure> {
        let index = self.record_index(num)?;
        Ok(self.records.read()[index])
    }
    fn update_record(&self, num: i32, edit: impl FnOnce(&mut VertexRecord)) -> Result<(), Failure> {
        let index = self.record_index(num)?;
        edit(&mut self.records.write()[index]);
        Ok(())
    }
}
