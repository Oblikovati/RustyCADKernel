// Rust translation of ModelingAlgorithms/TKShHealing/ShapeUpgrade/ShapeUpgrade.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `ShapeUpgrade`: the package functions that split a C0 B-spline at its C0 knots
//! (docs/shhealing-design.md section 8).
use rustycad_tkernel::ncollection::{Array1, HSequence};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{Geom2dBSplineCurve, Geom2dBoundedCurve, Geom2dCurve};
use rustycad_tkg3d::geom::{GeomBSplineCurve, GeomBoundedCurve, GeomCurve};
use rustycad_tkmath::bsplclib;
use rustycad_tkmath::gp::{self, Point2, Point3};

/// The C1 pieces of a 3D B-spline (`NCollection_HSequence<occ::handle<Geom_BoundedCurve>>`).
pub type BoundedCurveSequence = Handle<HSequence<Handle<dyn GeomBoundedCurve>>>;

/// The C1 pieces of a 2D B-spline (`NCollection_HSequence<occ::handle<Geom2d_BoundedCurve>>`).
pub type BoundedCurve2dSequence = Handle<HSequence<Handle<dyn Geom2dBoundedCurve>>>;

/// `ShapeUpgrade`: the package functions of ShapeUpgrade.
///
/// ```
/// use rustycad_tkernel::standard::Handle;
/// use rustycad_tkshhealing::shape_upgrade::ShapeUpgrade;
/// let pieces = ShapeUpgrade::c0_bspline_to_sequence_of_c1_bspline_curve(&Handle::default());
/// assert!(pieces.unwrap().is_none());
/// ```
#[derive(Clone, Copy, Debug, Default)]
pub struct ShapeUpgrade;

/// `array(i)` of a native array read by index, Standard_OutOfRange outside it.
fn at<T: Copy>(values: &Array1<'_, T>, index: i32) -> Result<T, Failure> {
    values.value(index).copied().map_err(|_| {
        Failure::with_type(
            "Standard_OutOfRange",
            &format!(
                "ShapeUpgrade::C0BSplineToSequenceOfC1BSplineCurve: index {index} outside [{}, {}]",
                values.lower(),
                values.upper()
            ),
            "",
        )
    })
}

/// The knots and multiplicities of one C1 piece, `TempKnots`/`TempMults` of the native
/// loop: the flat knots from `start_flat - deg` to `end_flat + deg`, equal ones (within
/// `gp::Resolution()`) merged, and the first and last knots of multiplicity 1 merged into
/// their neighbours.
fn piece_knots(
    knot_sequence: &Array1<'_, f64>,
    nb_knots: usize,
    deg: i32,
    (start_flat, end_flat): (i32, i32),
) -> Result<(Vec<f64>, Vec<i32>), Failure> {
    let mut temp_knots = vec![0.0; nb_knots];
    let mut temp_mults = vec![1; nb_knots];
    let mut index = 0usize;
    temp_knots[0] = at(knot_sequence, start_flat - deg)?;
    for j in start_flat - deg + 1..=end_flat + deg {
        let (current, previous) = (at(knot_sequence, j)?, at(knot_sequence, j - 1)?);
        if (current - previous).abs() <= gp::resolution() {
            *slot(&mut temp_mults, index)? += 1;
        } else {
            index += 1;
            *slot(&mut temp_knots, index)? = current;
        }
    }
    let (mut first, mut last) = (0usize, index);
    if *slot(&mut temp_mults, first)? == 1 {
        first += 1;
        *slot(&mut temp_mults, first)? += 1;
    }
    if *slot(&mut temp_mults, last)? == 1 {
        // A piece always spans two distinct knots; index 0 would be read before the array.
        last = last.checked_sub(1).ok_or_else(|| out_of_work_range(0))?;
        *slot(&mut temp_mults, last)? += 1;
    }
    let range = temp_knots
        .get(first..=last)
        .ok_or_else(|| out_of_work_range(first))?;
    Ok((range.to_vec(), temp_mults[first..=last].to_vec()))
}

/// The Standard_OutOfRange of a work-array index (0-based `index`).
fn out_of_work_range(index: usize) -> Failure {
    Failure::with_type(
        "Standard_OutOfRange",
        &format!("ShapeUpgrade: work index {index} outside the knots of the piece"),
        "",
    )
}

/// A mutable element of a work array, Standard_OutOfRange outside it.
fn slot<T>(values: &mut [T], index: usize) -> Result<&mut T, Failure> {
    let length = values.len();
    values.get_mut(index).ok_or_else(|| {
        Failure::with_type(
            "Standard_OutOfRange",
            &format!(
                "ShapeUpgrade: work index {} outside [1, {length}]",
                index + 1
            ),
            "",
        )
    })
}

/// The C1 piece between the knots `start` and `end` of the non-periodic `bs`.
fn c1_piece(bs: &GeomBSplineCurve, start: i32, end: i32) -> Result<GeomBSplineCurve, Failure> {
    let deg = bs.degree();
    let mults = bs.multiplicities();
    let start_flat = bsplclib::flat_index(deg, start, &mults, false)?;
    let end_flat = bsplclib::flat_index(deg, end, &mults, false)? - (at(&mults, end)? - 1);
    let (knots, piece_mults) = piece_knots(
        &bs.knot_sequence(),
        bs.nb_knots() as usize,
        deg,
        (start_flat, end_flat),
    )?;
    let new_mults = Array1::from_shared(&piece_mults, 1);
    let nb_poles = bsplclib::nb_poles(deg, false, &new_mults)?;
    // "Index of starting pole when splitting B-Spline is an index of starting knot".
    let pole_index = start_flat - deg;
    let poles = bs.poles();
    let (new_poles, new_weights) = bs.weights_array(|weights| {
        (0..nb_poles)
            .map(|j| Ok((at(&poles, j + pole_index)?, at(weights, j + pole_index)?)))
            .collect::<Result<(Vec<Point3>, Vec<f64>), Failure>>()
    })?;
    GeomBSplineCurve::with_weights(
        &Array1::from_shared(&new_poles, 1),
        &Array1::from_shared(&new_weights, 1),
        &Array1::from_shared(&knots, 1),
        &new_mults,
        deg,
        false,
    )
}

impl ShapeUpgrade {
    /// `C0BSplineToSequenceOfC1BSplineCurve(BS, seqBS)` for a 3D B-spline: the pieces
    /// between the knots of multiplicity at least the degree (the curve is made
    /// non-periodic first, in place, "to have equation NbPoles = NbKnots with
    /// Multiplicities - degree - 1"); `None` (native false, the output left unchanged) for
    /// a null or C1 curve;
    /// `ShapeUpgrade::c0_bspline_to_sequence_of_c1_bspline_curve(&bspline)?`.
    pub fn c0_bspline_to_sequence_of_c1_bspline_curve(
        bs: &Handle<GeomBSplineCurve>,
    ) -> Result<Option<BoundedCurveSequence>, Failure> {
        let Some(curve) = bs.get() else {
            return Ok(None);
        };
        if curve.is_cn(1)? {
            return Ok(None);
        }
        let sequence = Handle::new(HSequence::default());
        curve.set_not_periodic()?;
        let deg = curve.degree();
        let mults = curve.multiplicities();
        let last_index = curve.last_u_knot_index()?;
        let mut start = curve.first_u_knot_index()?;
        for end in start + 1..=last_index {
            if at(&mults, end)? < deg && end < last_index {
                continue;
            }
            let piece = c1_piece(curve, start, end)?;
            sequence.append(Handle::new(piece).cast::<dyn GeomBoundedCurve>());
            start = end;
        }
        Ok(Some(sequence))
    }

    /// `C0BSplineToSequenceOfC1BSplineCurve(BS, seqBS)` for a 2D B-spline: the 3D pieces
    /// of the curve lifted to z = 0 (a copy: the 2D curve keeps its periodicity), brought
    /// back to 2D; `None` for a null or C1 curve;
    /// `ShapeUpgrade::c0_bspline2d_to_sequence_of_c1_bspline_curve(&bspline)?`.
    pub fn c0_bspline2d_to_sequence_of_c1_bspline_curve(
        bs: &Handle<Geom2dBSplineCurve>,
    ) -> Result<Option<BoundedCurve2dSequence>, Failure> {
        let Some(curve) = bs.get() else {
            return Ok(None);
        };
        if curve.is_cn(1)? {
            return Ok(None);
        }
        let lifted = Handle::new(bspline_2d_to_3d(curve)?);
        let Some(pieces) = Self::c0_bspline_to_sequence_of_c1_bspline_curve(&lifted)? else {
            return Ok(None);
        };
        let sequence = Handle::new(HSequence::default());
        pieces.sequence(|items| {
            items.iter().try_for_each(|piece| {
                let piece = piece.cast::<GeomBSplineCurve>();
                let piece = piece.get().ok_or_else(null_piece)?;
                sequence
                    .append(Handle::new(bspline_3d_to_2d(piece)?).cast::<dyn Geom2dBoundedCurve>());
                Ok::<(), Failure>(())
            })
        })?;
        Ok(Some(sequence))
    }
}

/// The null dereference of `occ::down_cast<Geom_BSplineCurve>(seqBS3d->Value(i))`, which
/// never happens (every piece is a B-spline).
fn null_piece() -> Failure {
    Failure::with_type(
        "Standard_NullObject",
        "ShapeUpgrade::C0BSplineToSequenceOfC1BSplineCurve: a piece is not a B-spline",
        "",
    )
}

/// The static `BSplineCurve2dTo3d`: the curve with its poles at z = 0.
fn bspline_2d_to_3d(bs: &Geom2dBSplineCurve) -> Result<GeomBSplineCurve, Failure> {
    let poles: Vec<Point3> = bs
        .poles()
        .iter()
        .map(|p| Point3::new(p.x(), p.y(), 0.0))
        .collect();
    bs.weights_array(|weights| {
        GeomBSplineCurve::with_weights(
            &Array1::from_shared(&poles, 1),
            weights,
            &bs.knots(),
            &bs.multiplicities(),
            bs.degree(),
            bs.is_periodic(),
        )
    })
}

/// The static `BSplineCurve3dTo2d`: the curve with its poles projected on z = 0.
fn bspline_3d_to_2d(bs: &GeomBSplineCurve) -> Result<Geom2dBSplineCurve, Failure> {
    let poles: Vec<Point2> = bs
        .poles()
        .iter()
        .map(|p| Point2::new(p.x(), p.y()))
        .collect();
    bs.weights_array(|weights| {
        Geom2dBSplineCurve::with_weights(
            &Array1::from_shared(&poles, 1),
            weights,
            &bs.knots(),
            &bs.multiplicities(),
            bs.degree(),
            bs.is_periodic(),
        )
    })
}
