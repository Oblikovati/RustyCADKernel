// Rust translation of BRepFill_CompatibleWires::Perform
// (ModelingAlgorithms/TKBool/BRepFill/BRepFill_CompatibleWires.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Perform`: the choice between the polar method and the origins for closed
//! sections, the origins and the curvilinear cuts for open ones.
use super::BRepFillCompatibleWires;
use super::helpers::{wire_closed, wire_continuity};
use crate::brep_fill::ThruSectionErrorStatus;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_tools::WireExplorer;
use rustycad_tkbrep::topods::{self, Shape, Wire};
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::geom_abs::Shape as Continuity;

/// Whether every edge met by the wire explorer is degenerated (true when none).
fn punctual(wire: &Shape) -> Result<bool, Failure> {
    let mut degenerated: bool = true;
    let mut explorer = WireExplorer::with_wire(&topods::wire(wire)?)?;
    while explorer.more() {
        degenerated = degenerated && BRepTool::degenerated(explorer.current())?;
        explorer.next()?;
    }
    Ok(degenerated)
}

/// The number of edges met by the wire explorer.
pub(super) fn explored_count(wire: &Wire) -> Result<usize, Failure> {
    let mut count: usize = 0;
    let mut explorer = WireExplorer::with_wire(wire)?;
    while explorer.more() {
        count += 1;
        explorer.next()?;
    }
    Ok(count)
}

impl BRepFillCompatibleWires {
    /// `Perform(WithRotation)`: computes the origins and orientations of the sections
    /// and cuts them to the same number of edges (by the polar method for closed
    /// sections with a report, by curvilinear abscissa for open ones); mixed open and
    /// closed sections give the NotSameTopology status; `compatible.perform(true)?`.
    pub fn perform(&mut self, with_rotation: bool) -> Result<(), Failure> {
        self.my_status = ThruSectionErrorStatus::Done;
        let report: bool = self.forward_sections_report()?;
        self.map_initial_edges()?;
        let nb: usize = self.my_work.len();
        let mut ideb: usize = 1;
        let mut ifin: usize = nb;
        self.my_degen1 = punctual(self.section(ideb)?)?;
        if self.my_degen1 {
            ideb += 1;
        }
        self.my_degen2 = punctual(self.section(ifin)?)?;
        if self.my_degen2 {
            ifin -= 1;
        }
        let (mut all_closed, mut all_open) = (true, true);
        for i in ideb..=ifin {
            let closed: bool = wire_closed(&self.my_work[i - 1])?;
            all_closed = all_closed && closed;
            all_open = all_open && !closed;
        }
        self.perform_by_topology(all_closed, all_open, report, with_rotation)
    }

    /// Section `i` (1-based); Standard_OutOfRange outside the sequence.
    pub(super) fn section(&self, i: usize) -> Result<&Shape, Failure> {
        self.my_work.get(i.wrapping_sub(1)).ok_or_else(|| {
            let message = format!(
                "BRepFill_CompatibleWires: section {i} of {}; expected 1..=count",
                self.my_work.len()
            );
            Failure::with_type("Standard_OutOfRange", &message, "BRepFill_CompatibleWires")
        })
    }

    /// Every section turned FORWARD; whether the sections need a report of cuts:
    /// different edge counts, or every section at least C1.
    fn forward_sections_report(&mut self) -> Result<bool, Failure> {
        let mut cont_s = Continuity::CN;
        let (mut nbmax, mut nbmin): (usize, usize) = (0, 0);
        for i in 0..self.my_work.len() {
            self.my_work[i] =
                topods::wire(&self.my_work[i].oriented(Orientation::Forward))?.into_shape();
            let wire: Wire = topods::wire(&self.my_work[i])?;
            let cont = wire_continuity(&wire)?;
            if cont < cont_s {
                cont_s = cont;
            }
            let count: usize = explored_count(&wire)?;
            if i == 0 {
                nbmin = count;
            }
            nbmax = nbmax.max(count);
            nbmin = nbmin.min(count);
        }
        Ok(nbmax != nbmin || cont_s >= Continuity::C1)
    }

    /// Each edge of each section mapped to itself.
    fn map_initial_edges(&mut self) -> Result<(), Failure> {
        for section in &self.my_work {
            let mut explorer = WireExplorer::with_wire(&topods::wire(section)?)?;
            while explorer.more() {
                let edge: Shape = explorer.current().as_ref().clone();
                let mut own: List<Shape> = List::default();
                own.append(edge.clone());
                self.my_map.bind(edge, own);
                explorer.next()?;
            }
        }
        Ok(())
    }

    /// The method for the topology of the sections.
    fn perform_by_topology(
        &mut self,
        all_closed: bool,
        all_open: bool,
        report: bool,
        with_rotation: bool,
    ) -> Result<(), Failure> {
        if all_closed {
            // All sections are closed.
            if report {
                return self.same_number_by_polar_method(with_rotation);
            }
            return self.compute_origin(false);
        }
        if all_open {
            // All sections are open.
            self.search_origin()?;
            if self.my_status != ThruSectionErrorStatus::Done {
                return Ok(());
            }
            if report {
                self.same_number_by_acr(report)?;
            }
            return Ok(());
        }
        // There are open and closed sections: not processed.
        self.my_status = ThruSectionErrorStatus::NotSameTopology;
        Ok(())
    }

    /// Whether the same number of edges is met in each section of `ideb..=ifin`.
    pub(super) fn same_edge_counts(&self, ideb: usize, ifin: usize) -> Result<bool, Failure> {
        let (mut nbmax, mut nbmin): (usize, usize) = (0, 0);
        for i in ideb..=ifin {
            let count: usize = explored_count(&topods::wire(&self.my_work[i - 1])?)?;
            if i == ideb {
                nbmin = count;
            }
            nbmax = nbmax.max(count);
            nbmin = nbmin.min(count);
        }
        Ok(nbmin == nbmax)
    }
}
