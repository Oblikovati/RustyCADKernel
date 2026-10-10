// Rust translation of ModelingAlgorithms/TKBool/BRepFill/BRepFill_CompatibleWires.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepFill_CompatibleWires`: sections reoriented, restarted and cut so that the
//! surface through them is not twisted and every section has the same edge count.
mod by_acr;
mod helpers;
mod intersect;
mod open_origin;
mod origin;
mod perform;
mod polar;
mod polar_regularize;

use super::{GeneratedFaces, ThruSectionErrorStatus};
use rustycad_tkbrep::topods::{Edge, Shape};
use rustycad_tkernel::ncollection::List;
use std::sync::OnceLock;

/// Constructs the sequence of sections (wires) with orientations and origins agreed
/// with each other (`BRepFill_CompatibleWires`).
///
/// ```ignore
/// let mut compatible = BRepFillCompatibleWires::with_sections(&[w1.into(), w2.into()]);
/// compatible.perform(true)?;
/// let wires = compatible.shape();
/// ```
pub struct BRepFillCompatibleWires {
    my_init: Vec<Shape>,
    my_work: Vec<Shape>,
    my_percent: f64,
    my_degen1: bool,
    my_degen2: bool,
    my_status: ThruSectionErrorStatus,
    my_map: GeneratedFaces,
}

impl Default for BRepFillCompatibleWires {
    fn default() -> Self {
        Self::new()
    }
}

impl BRepFillCompatibleWires {
    /// `BRepFill_CompatibleWires()`: no section and the NotDone status. The native
    /// constructor leaves the percent and the degenerated flags uninitialized; they
    /// start at the values `Init` and `Perform` give them.
    /// `BRepFillCompatibleWires::new()`.
    pub fn new() -> Self {
        Self {
            my_init: Vec::new(),
            my_work: Vec::new(),
            my_percent: 0.1,
            my_degen1: false,
            my_degen2: false,
            my_status: ThruSectionErrorStatus::NotDone,
            my_map: GeneratedFaces::default(),
        }
    }

    /// `BRepFill_CompatibleWires(Sections)`; `BRepFillCompatibleWires::with_sections(&wires)`.
    pub fn with_sections(sections: &[Shape]) -> Self {
        let mut compatible = Self::new();
        compatible.init(sections);
        compatible
    }

    /// `Init(Sections)`: the sections, a percent of 0.1, the NotDone status and an
    /// empty map; `compatible.init(&wires)`.
    pub fn init(&mut self, sections: &[Shape]) {
        self.my_init = sections.to_vec();
        self.my_work = sections.to_vec();
        self.my_percent = 0.1;
        self.my_status = ThruSectionErrorStatus::NotDone;
        self.my_map.clear(true);
    }

    /// `SetPercent(percent)`: kept only inside `(0, 1)`; `compatible.set_percent(0.01)`.
    pub fn set_percent(&mut self, percent: f64) {
        if 0. < percent && percent < 1. {
            self.my_percent = percent;
        }
    }

    /// `IsDone()`: the status is Done; `compatible.is_done()`.
    pub fn is_done(&self) -> bool {
        self.my_status == ThruSectionErrorStatus::Done
    }

    /// `GetStatus()`; `compatible.get_status()`.
    pub fn get_status(&self) -> ThruSectionErrorStatus {
        self.my_status
    }

    /// `Shape()`: the computed sections; `compatible.shape()`.
    pub fn shape(&self) -> &[Shape] {
        &self.my_work
    }

    /// `GeneratedShapes(SubSection)`: the edges made from an edge of a section, an
    /// empty list for an unknown edge; `compatible.generated_shapes(&edge)`.
    pub fn generated_shapes(&self, sub_section: &Edge) -> &List<Shape> {
        static EMPTY: OnceLock<List<Shape>> = OnceLock::new();
        match self.my_map.seek(sub_section) {
            Some(edges) => edges,
            None => EMPTY.get_or_init(List::default),
        }
    }

    /// `Generated()`: the map of each section edge to its new edges; `compatible.generated()`.
    pub fn generated(&self) -> &GeneratedFaces {
        &self.my_map
    }

    /// `IsDegeneratedFirstSection()`; `compatible.is_degenerated_first_section()`.
    pub fn is_degenerated_first_section(&self) -> bool {
        self.my_degen1
    }

    /// `IsDegeneratedLastSection()`; `compatible.is_degenerated_last_section()`.
    pub fn is_degenerated_last_section(&self) -> bool {
        self.my_degen2
    }
}
