// Rust translation of BRepFill_CompatibleWires::SameNumberByACR
// (ModelingAlgorithms/TKBool/BRepFill/BRepFill_CompatibleWires.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `SameNumberByACR`: open sections cut at the union of their reduced curvilinear
//! abscissas, so that they have the same number of edges.
use super::BRepFillCompatibleWires;
use super::perform::explored_count;
use crate::brep_fill::ThruSectionErrorStatus;
use crate::brep_fill::package::{compute_acr, insert_acr};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_tools::WireExplorer;
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{self, Edge, Shape, Wire};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::arrays::one_based;
use rustycad_tkmath::gp::{self, Point3};

/// Standard_OutOfRange for a cut table that cannot be allocated.
fn table_error(error: rustycad_tkernel::ncollection::ArrayError) -> Failure {
    let message =
        format!("BRepFill_CompatibleWires::SameNumberByACR: {error}; expected a cut table");
    Failure::with_type("Standard_OutOfRange", &message, "BRepFill_CompatibleWires")
}

/// The point of the end of the current edge of the explorer away from its current
/// vertex (the origin when the vertex is not an end, as natively).
fn far_end(explorer: &WireExplorer, previous: Point3) -> Result<Point3, Failure> {
    let (vf, vr) = topexp::vertices(explorer.current(), true);
    let current = explorer.current_vertex();
    let mut point: Point3 = previous;
    if current.is_same(&vf) {
        point = BRepTool::pnt(&vr)?;
    }
    if current.is_same(&vr) {
        point = BRepTool::pnt(&vf)?;
    }
    Ok(point)
}

impl BRepFillCompatibleWires {
    /// `SameNumberByACR(report)`.
    pub(super) fn same_number_by_acr(&mut self, report: bool) -> Result<(), Failure> {
        let nb_sects: usize = self.my_work.len();
        let ideb: usize = if self.my_degen1 { 2 } else { 1 };
        let ifin: usize = if self.my_degen2 {
            nb_sects - 1
        } else {
            nb_sects
        };
        let v_closed: bool = !self.my_degen1
            && !self.my_degen2
            && self.my_work[ideb - 1].is_same(&self.my_work[ifin - 1]);
        let mut counts: Vec<usize> = Vec::with_capacity(nb_sects);
        for section in &self.my_work {
            counts.push(explored_count(&topods::wire(section)?)?);
        }
        let nbmax: usize = counts.iter().copied().max().unwrap_or(0);
        let nbmin: usize = counts
            .first()
            .map_or(0, |&first| counts.iter().copied().fold(first, usize::min));
        if nbmax > 1 && (report || nbmin < nbmax) {
            // Several edges: insertion of cuts.
            let (cuts, wire_len) = self.cut_table(nbmax)?;
            for i in 1..=nb_sects {
                self.insert_cuts(i, &cuts, wire_len[i - 1])?;
            }
        }
        // Blocking sections?
        if v_closed {
            self.my_work[nb_sects - 1] = self.my_work[0].clone();
        }
        if !self.same_edge_counts(ideb, ifin)? {
            self.my_status = ThruSectionErrorStatus::Failed;
        }
        Ok(())
    }

    /// The table of cuts `dec3` (the merged reduced abscissas of every section, below
    /// 1, without the cuts that make only null edges) and the length of each section.
    fn cut_table(&self, nbmax: usize) -> Result<(Array1<'static, f64>, Vec<f64>), Failure> {
        let nb_sects: usize = self.my_work.len();
        let nbdec: usize = (nbmax - 1) * nb_sects + 1;
        let mut dec: Vec<f64> = vec![0.; nbdec + 1];
        dec[2] = 1.;
        let mut wire_len: Vec<f64> = Vec::with_capacity(nb_sects);
        for section in &self.my_work {
            let wire1: Wire = topods::wire(section)?;
            let nb_e: usize = explored_count(&wire1)?;
            let mut acr: Array1<'static, f64> =
                Array1::with_bounds(0, nb_e as i32).map_err(table_error)?;
            acr.init(0.);
            compute_acr(&wire1, &mut acr)?;
            let values: &[f64] = acr.as_slice();
            wire_len.push(values[0]);
            for &abscissa in &values[1..values.len() - 1] {
                insert_cut(&mut dec, nbdec, abscissa);
            }
        }
        let mut k: usize = 1;
        while k <= nbdec && dec[k] < 1. {
            k += 1;
        }
        let dec2: Vec<f64> = dec[1..k].to_vec();
        Ok((without_null_cuts(&dec2, &wire_len)?, wire_len))
    }

    /// Section `i` cut at the abscissas, and the generated map updated: each cut edge
    /// is replaced, in the list that holds it, by its pieces.
    fn insert_cuts(
        &mut self,
        i: usize,
        cuts: &Array1<'static, f64>,
        wire_len: f64,
    ) -> Result<(), Failure> {
        let oldwire: Wire = topods::wire(&self.my_work[i - 1])?;
        let mut tol: f64 = precision::confusion();
        if wire_len > gp::resolution() {
            tol /= wire_len;
        }
        let newwire: Wire = insert_acr(&oldwire, cuts, tol)?;
        let mut exp1 = WireExplorer::with_wire(&oldwire)?;
        let mut exp2 = WireExplorer::with_wire(&newwire)?;
        while exp1.more() {
            let ecur: Edge = exp1.current().clone();
            if ecur.is_same(exp2.current()) {
                exp2.next()?;
                exp1.next()?;
                continue;
            }
            let pieces: Vec<Shape> = pieces_of(&exp1, &mut exp2)?;
            self.replace_in_map(&ecur, pieces);
            exp1.next()?;
        }
        self.my_work[i - 1] = newwire.into_shape();
        Ok(())
    }

    /// The first list of the map (in map order) that holds `edge`: the edge replaced by
    /// `pieces` (`InsertBefore` then `Remove`).
    fn replace_in_map(&mut self, edge: &Edge, pieces: Vec<Shape>) {
        let found: Option<(Shape, usize)> = self.my_map.items().find_map(|(key, list)| {
            list.iter()
                .position(|e| e.is_same(edge))
                .map(|at| (key.clone(), at))
        });
        let Some((ancestor, at)) = found else {
            return;
        };
        if let Some(list) = self.my_map.change_seek(&ancestor) {
            let mut items: Vec<Shape> = list.iter().cloned().collect();
            items.splice(at..=at, pieces);
            list.clear();
            for item in items {
                list.append(item);
            }
        }
    }
}

/// `dec(k) = ACR(j)` inserted in order when strictly between two cuts.
fn insert_cut(dec: &mut [f64], nbdec: usize, abscissa: f64) {
    let mut k: usize = 1;
    while dec[k] < abscissa {
        k += 1;
        if k > nbdec {
            break;
        }
    }
    // `dec(k)` past the table reads the slot after it natively (dec(nbdec + 1)); the
    // strict comparison then fails as for any value not above the abscissa.
    let above: f64 = dec.get(k).copied().unwrap_or(f64::NEG_INFINITY);
    if dec[k - 1] < abscissa && abscissa < above {
        let mut l: usize = nbdec - 1;
        while l >= k {
            dec[l + 1] = dec[l];
            l -= 1;
        }
        dec[k] = abscissa;
    }
}

/// The cuts without those whose edges are null in every section (shorter than
/// Precision::Confusion()), as an array 1..n.
fn without_null_cuts(dec2: &[f64], wire_len: &[f64]) -> Result<Array1<'static, f64>, Failure> {
    let nbdec: usize = dec2.len();
    let mut kept: Vec<f64> = Vec::new();
    for k in 1..=nbdec {
        let knot1: f64 = dec2[k - 1];
        let knot2: f64 = if k == nbdec { 1. } else { dec2[k] };
        let all_lengths_null: bool = !wire_len
            .iter()
            .any(|&len| (knot2 - knot1) * len > precision::confusion());
        if !all_lengths_null {
            kept.push(knot1);
        }
    }
    let mut dec3: Array1<'static, f64> = one_based(kept.len() as i32)?;
    dec3.as_mut_slice().copy_from_slice(&kept);
    Ok(dec3)
}

/// The edges of the new wire that replace the current edge of the old one: taken until
/// their far end meets the far end of the old edge within 1e-3.
fn pieces_of(exp1: &WireExplorer, exp2: &mut WireExplorer) -> Result<Vec<Shape>, Failure> {
    let p1: Point3 = far_end(exp1, Point3::default())?;
    let mut p2: Point3 = far_end(exp2, Point3::default())?;
    let mut pieces: Vec<Shape> = Vec::new();
    while p1.distance(&p2) > 1.0e-3 {
        pieces.push(exp2.current().as_ref().clone());
        exp2.next()?;
        p2 = far_end(exp2, p2)?;
        if p1.distance(&p2) <= 1.0e-3 {
            pieces.push(exp2.current().as_ref().clone());
            exp2.next()?;
        }
    }
    Ok(pieces)
}
