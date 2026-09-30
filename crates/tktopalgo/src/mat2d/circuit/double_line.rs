// Rust translation of ModelingAlgorithms/TKTopAlgo/MAT2d/MAT2d_Circuit.cxx (DoubleLine and Side) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The doubling of an open line: its items are followed back in the reverse order,
//! the connexions move to the side of the circuit they are on, and the re-entrant
//! corner points are removed.
use super::corners::cross_prod;
use super::{CircuitState, trimmed_item};
use crate::mat2d::connexion::Mat2dConnexion;
use crate::mat2d::{Mat2dLine, seq};
use rustycad_tkernel::ncollection::Sequence;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{
    Geom2dCartesianPoint, Geom2dCurve, Geom2dGeometry, Geom2dTrimmedCurve, errors,
};
use rustycad_tkmath::gp::Vector2;

type Connexions = Sequence<Handle<Mat2dConnexion>>;

fn is_point(geometry: &Handle<dyn Geom2dGeometry>) -> bool {
    geometry.cast::<Geom2dCartesianPoint>().is_some()
}

/// `Side(C1, Line)`: -1 when the connexion leaves its item to the left of the curve,
/// 1 otherwise.
fn side(c1: &Mat2dConnexion, line: &Mat2dLine) -> Result<f64, Failure> {
    let (first, second) = (c1.point_on_first(), c1.point_on_second());
    let vect1 = Vector2::new(second.x() - first.x(), second.y() - first.y());
    let curve = trimmed_item(&seq::at(line, c1.index_item_on_first())?)?;
    let vect2: Vector2 = curve.dn(c1.parameter_on_first(), 1)?;
    Ok(if vect1.crossed(vect2) > 0. { -1. } else { 1. })
}

// The item of `line` at `index` and whether a point there is a corner to flip: a
// point that is not an end and whose neighbours turn to the side `side_ref`.
fn flipped_point(
    line: &Mat2dLine,
    index: i32,
    nb_items: i32,
    side_ref: f64,
) -> Result<bool, Failure> {
    if index == nb_items || index == 1 {
        return Ok(false);
    }
    let (pro_vec, _) = cross_prod(&seq::at(line, index - 1)?, &seq::at(line, index + 1)?)?;
    Ok(pro_vec * side_ref > 0.)
}

impl CircuitState {
    /// `DoubleLine(Line, ConnexionFrom, ConnexionFather, SideRef)`.
    pub(super) fn double_line(
        &self,
        line: &mut Mat2dLine,
        connexion_from: &mut Connexions,
        connexion_father: &Handle<Mat2dConnexion>,
        side_ref: f64,
    ) -> Result<(), Failure> {
        let nb_items: i32 = line.length();
        if !self.is_open_result {
            complete_line(line, nb_items)?;
        }
        let nb_connexions: i32 = connexion_from.length();
        move_connexions(line, connexion_from, nb_items, side_ref)?;
        if let Some(father) = connexion_father.get() {
            move_father(line, father, nb_items, side_ref)?;
        }
        let corres: Vec<i32> = self.remove_reentrant_corners(line, nb_items, side_ref)?;
        if self.is_open_result {
            return Ok(());
        }
        let corres: Vec<i32> = fill_corres(corres, nb_items)?;
        for i in 1..=nb_connexions {
            let cc: Handle<Mat2dConnexion> = seq::at(connexion_from, i)?;
            cc.set_index_item_on_first(corres_at(&corres, cc.index_item_on_first())?);
        }
        if let Some(father) = connexion_father.get() {
            father.set_index_item_on_second(corres_at(&corres, father.index_item_on_second())?);
        }
        Ok(())
    }

    // The suppression of the re-entrant corners: a point (not at an end of the first
    // half) whose neighbours make no sharp corner is removed. The table maps each old
    // index to the new one, 0 for a removed point; it has one slot per item of the
    // line, of which the scan writes the first ones.
    fn remove_reentrant_corners(
        &self,
        line: &mut Mat2dLine,
        nb_items: i32,
        side_ref: f64,
    ) -> Result<Vec<i32>, Failure> {
        let mut corres: Vec<i32> = vec![0; line.length().max(0) as usize];
        let (mut ind_line, mut i_corres) = (1, 1);
        while seq::at(line, ind_line)? != seq::last(line)? {
            set_corres(&mut corres, i_corres, ind_line)?;
            let point: bool = is_point(&seq::at(line, ind_line)?);
            if point && i_corres != 1 && i_corres != nb_items {
                let previous = seq::at(line, ind_line - 1)?;
                let next = seq::at(line, ind_line + 1)?;
                if !self.is_sharp_corner(&previous, &next, side_ref)? {
                    seq::remove(line, ind_line)?;
                    ind_line -= 1;
                    set_corres(&mut corres, i_corres, 0)?;
                }
            }
            ind_line += 1;
            i_corres += 1;
        }
        set_corres(&mut corres, i_corres, ind_line)?;
        Ok(corres)
    }
}

// The completion of the line: its items from the one before the last to the second,
// the points shared and the curves reversed copies.
fn complete_line(line: &mut Mat2dLine, nb_items: i32) -> Result<(), Failure> {
    let mut i: i32 = nb_items - 1;
    while i > 1 {
        let item: Handle<dyn Geom2dGeometry> = seq::at(line, i)?;
        if is_point(&item) {
            line.append(item);
        } else {
            let curve: Handle<Geom2dTrimmedCurve> = trimmed_item(&item.copy()?)?;
            curve.reverse()?;
            line.append(curve.cast());
        }
        i -= 1;
    }
    Ok(())
}

// The connexions from the line that leave it on the other side move to the doubled
// items (a corner point, or the reversed curve with its reversed parameter), and to
// the end of the sequence.
fn move_connexions(
    line: &Mat2dLine,
    connexion_from: &mut Connexions,
    nb_items: i32,
    side_ref: f64,
) -> Result<(), Failure> {
    let mut i_after: i32 = connexion_from.length();
    let mut i: i32 = 1;
    while i <= i_after {
        let cc: Handle<Mat2dConnexion> = seq::at(connexion_from, i)?;
        let ind_cof: i32 = cc.index_item_on_first();
        let item: Handle<dyn Geom2dGeometry> = seq::at(line, ind_cof)?;
        let flip: bool = if is_point(&item) {
            flipped_point(line, ind_cof, nb_items, side_ref)?
        } else {
            side(&cc, line)? != side_ref
        };
        if flip {
            if !is_point(&item) {
                let curve = trimmed_item(&item)?;
                cc.set_parameter_on_first(curve.reversed_parameter(cc.parameter_on_first())?);
            }
            cc.set_index_item_on_first(2 * nb_items - ind_cof);
            seq::insert_after(connexion_from, i_after, cc)?;
            seq::remove(connexion_from, i)?;
            i_after -= 1;
            i -= 1;
        }
        i += 1;
    }
    Ok(())
}

// The connexion of the father arrives on the doubled item when it arrives on the
// other side.
fn move_father(
    line: &Mat2dLine,
    father: &Mat2dConnexion,
    nb_items: i32,
    side_ref: f64,
) -> Result<(), Failure> {
    let cc: Handle<Mat2dConnexion> = father.reverse();
    let ind_cof: i32 = cc.index_item_on_first();
    let item: Handle<dyn Geom2dGeometry> = seq::at(line, ind_cof)?;
    if is_point(&item) {
        if flipped_point(line, ind_cof, nb_items, side_ref)? {
            father.set_index_item_on_second(2 * nb_items - ind_cof);
        }
        return Ok(());
    }
    if side(&cc, line)? != side_ref {
        let curve = trimmed_item(&item)?;
        father.set_index_item_on_second(2 * nb_items - ind_cof);
        father.set_parameter_on_second(curve.reversed_parameter(father.parameter_on_second())?);
    }
    Ok(())
}

// The removed points of the first half take the index of their double in the
// second half.
fn fill_corres(mut corres: Vec<i32>, nb_items: i32) -> Result<Vec<i32>, Failure> {
    for i in 1..2 * nb_items - 2 {
        if corres_at(&corres, i)? == 0 {
            let double: i32 = corres_at(&corres, 2 * nb_items - i)?;
            set_corres(&mut corres, i, double)?;
        }
    }
    Ok(corres)
}

fn corres_at(corres: &[i32], index: i32) -> Result<i32, Failure> {
    usize::try_from(index - 1)
        .ok()
        .and_then(|offset| corres.get(offset).copied())
        .ok_or_else(|| out_of_table(index, corres.len()))
}

fn set_corres(corres: &mut [i32], index: i32, value: i32) -> Result<(), Failure> {
    let len: usize = corres.len();
    let slot: &mut i32 = usize::try_from(index - 1)
        .ok()
        .and_then(|offset| corres.get_mut(offset))
        .ok_or_else(|| out_of_table(index, len))?;
    *slot = value;
    Ok(())
}

// Standard_OutOfRange of the table `Corres` (NCollection_Array1).
fn out_of_table(index: i32, len: usize) -> Failure {
    errors::out_of_range(&format!(
        "MAT2d_Circuit::DoubleLine: Corres({index}); expected 1..={len}"
    ))
}
