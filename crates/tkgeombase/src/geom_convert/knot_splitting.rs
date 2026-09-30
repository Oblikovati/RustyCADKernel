// Rust translation of ModelingData/TKGeomBase/GeomConvert/GeomConvert_BSplineCurveKnotSplitting.hxx/.cxx
// and GeomConvert_BSplineSurfaceKnotSplitting.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomConvert_BSplineCurveKnotSplitting` and
//! `GeomConvert_BSplineSurfaceKnotSplitting`: the knots where a B-spline must be
//! split to get arcs or patches of a required continuity.
use super::{live_bspline, live_bspline_surface};
use crate::geom2d_convert::split_knots;
use crate::native_failure::{array_failure, failure};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::arrays::one_based;
use rustycad_tkg3d::geom::{GeomBSplineCurve, GeomBSplineSurface};
use rustycad_tkmath::bsplclib;

/// The split knot indices of a B-spline curve, first and last knots included, in
/// increasing order.
///
/// `let split = BSplineCurveKnotSplitting::new(&curve, 1)?; let n = split.nb_splits();`
#[derive(Clone, Debug)]
pub struct BSplineCurveKnotSplitting {
    split_indexes: Array1<'static, i32>,
}

impl BSplineCurveKnotSplitting {
    /// The knots where `basis_curve` is less than C`continuity_range`: every inner
    /// knot with `Degree - Multiplicity < ContinuityRange`, between the first and last
    /// knots. Standard_RangeError for a negative range
    /// (`GeomConvert_BSplineCurveKnotSplitting(BasisCurve, ContinuityRange)`);
    /// `BSplineCurveKnotSplitting::new(&curve, 1)?`.
    pub fn new(
        basis_curve: &Handle<GeomBSplineCurve>,
        continuity_range: i32,
    ) -> Result<Self, Failure> {
        check_range(
            "GeomConvert_BSplineCurveKnotSplitting",
            "ContinuityRange",
            continuity_range,
        )?;
        let curve: &GeomBSplineCurve = live_bspline(basis_curve)?;
        let bounds = [curve.first_u_knot_index()?, curve.last_u_knot_index()?];
        let split_indexes = split_indexes(
            &curve.multiplicities(),
            bounds,
            curve.degree(),
            continuity_range,
        )?;
        Ok(Self { split_indexes })
    }

    /// The number of split knots, the ends included (`NbSplits()`); `split.nb_splits()`.
    pub fn nb_splits(&self) -> i32 {
        self.split_indexes.length()
    }

    /// Copies the split knot indices into `split_values` at `1..=NbSplits`; an index
    /// outside `split_values` raises Standard_OutOfRange (`Splitting(SplitValues)`);
    /// `split.splitting(&mut values)?`.
    pub fn splitting(&self, split_values: &mut Array1<'_, i32>) -> Result<(), Failure> {
        copy_indexes(&self.split_indexes, split_values)
    }

    /// The split knot index of rank `index`, Standard_RangeError outside
    /// `1..=NbSplits` (`SplitValue(Index)`); `split.split_value(1)?`.
    pub fn split_value(&self, index: i32) -> Result<i32, Failure> {
        indexed_value(
            &self.split_indexes,
            index,
            "GeomConvert_BSplineCurveKnotSplitting::SplitValue",
        )
    }
}

/// The split knot indices of a B-spline surface in each parametric direction.
///
/// Natively, when `VContinuityRange` is positive and the surface is already C
/// `VContinuityRange` in V, the constructor stores the V bounds into the U table
/// (`usplitIndexes`) and leaves the V table null, so every V accessor dereferences a
/// null handle; the port keeps both effects: the U table holds the two V bound
/// indices and the V accessors return Standard_NullObject.
///
/// `let split = BSplineSurfaceKnotSplitting::new(&surface, 1, 1)?;`
#[derive(Clone, Debug)]
pub struct BSplineSurfaceKnotSplitting {
    usplit_indexes: Array1<'static, i32>,
    vsplit_indexes: Option<Array1<'static, i32>>,
}

impl BSplineSurfaceKnotSplitting {
    /// The knots where `basis_surface` is less than C`u_continuity_range` in U and C
    /// `v_continuity_range` in V; Standard_RangeError when a range is negative
    /// (`GeomConvert_BSplineSurfaceKnotSplitting(BasisSurface, UContinuityRange,
    /// VContinuityRange)`); `BSplineSurfaceKnotSplitting::new(&surface, 1, 1)?`.
    pub fn new(
        basis_surface: &Handle<GeomBSplineSurface>,
        u_continuity_range: i32,
        v_continuity_range: i32,
    ) -> Result<Self, Failure> {
        if u_continuity_range < 0 || v_continuity_range < 0 {
            return Err(failure(
                "Standard_RangeError",
                &format!(
                    "GeomConvert_BSplineSurfaceKnotSplitting: ranges {u_continuity_range}, {v_continuity_range}; expected >= 0"
                ),
            ));
        }
        let surface: &GeomBSplineSurface = live_bspline_surface(basis_surface)?;
        let u_bounds = [surface.first_u_knot_index()?, surface.last_u_knot_index()?];
        let v_bounds = [surface.first_v_knot_index()?, surface.last_v_knot_index()?];
        let mut usplit_indexes: Array1<'static, i32> = split_indexes(
            &surface.u_multiplicities(),
            u_bounds,
            surface.u_degree(),
            u_continuity_range,
        )?;
        let vmults: Array1<'static, i32> = surface.v_multiplicities();
        let already_smooth: bool = v_continuity_range != 0
            && surface.v_degree() - bsplclib::max_knot_mult(&vmults, v_bounds[0], v_bounds[1])?
                >= v_continuity_range;
        if already_smooth {
            // The native copy-paste slip: the V bounds overwrite the U table.
            usplit_indexes = bounds_only(v_bounds)?;
            return Ok(Self {
                usplit_indexes,
                vsplit_indexes: None,
            });
        }
        let vsplit = split_indexes(&vmults, v_bounds, surface.v_degree(), v_continuity_range)?;
        Ok(Self {
            usplit_indexes,
            vsplit_indexes: Some(vsplit),
        })
    }

    /// The number of U split knots, the ends included (`NbUSplits()`);
    /// `split.nb_u_splits()`.
    pub fn nb_u_splits(&self) -> i32 {
        self.usplit_indexes.length()
    }

    /// The number of V split knots, the ends included; Standard_NullObject when the
    /// constructor left the V table null (`NbVSplits()`); `split.nb_v_splits()?`.
    pub fn nb_v_splits(&self) -> Result<i32, Failure> {
        Ok(self.v_table()?.length())
    }

    /// Copies the U and V split knot indices into `u_split` and `v_split` at
    /// `1..=NbSplits`; Standard_OutOfRange for an index outside an output table,
    /// Standard_NullObject (after the U copy) for a null V table
    /// (`Splitting(USplit, VSplit)`); `split.splitting(&mut u, &mut v)?`.
    pub fn splitting(
        &self,
        u_split: &mut Array1<'_, i32>,
        v_split: &mut Array1<'_, i32>,
    ) -> Result<(), Failure> {
        copy_indexes(&self.usplit_indexes, u_split)?;
        copy_indexes(self.v_table()?, v_split)
    }

    /// The U split knot index of rank `u_index`, Standard_RangeError outside
    /// `1..=NbUSplits` (`USplitValue(UIndex)`); `split.u_split_value(1)?`.
    pub fn u_split_value(&self, u_index: i32) -> Result<i32, Failure> {
        indexed_value(
            &self.usplit_indexes,
            u_index,
            "GeomConvert_BSplineSurfaceKnotSplitting::USplitValue",
        )
    }

    /// The V split knot index of rank `v_index`, Standard_RangeError outside
    /// `1..=NbVSplits` (`VSplitValue(VIndex)`); `split.v_split_value(1)?`.
    pub fn v_split_value(&self, v_index: i32) -> Result<i32, Failure> {
        indexed_value(
            self.v_table()?,
            v_index,
            "GeomConvert_BSplineSurfaceKnotSplitting::VSplitValue",
        )
    }

    fn v_table(&self) -> Result<&Array1<'static, i32>, Failure> {
        self.vsplit_indexes.as_ref().ok_or_else(|| {
            failure(
                "Standard_NullObject",
                "GeomConvert_BSplineSurfaceKnotSplitting: null vsplitIndexes (V already of the continuity); expected a table",
            )
        })
    }
}

fn check_range(class: &str, name: &str, range: i32) -> Result<(), Failure> {
    if range < 0 {
        return Err(failure(
            "Standard_RangeError",
            &format!("{class}: {name} = {range}; expected >= 0"),
        ));
    }
    Ok(())
}

// The split table of one direction: the bounds only for a null range or a curve
// already of the continuity, the native Split table otherwise.
fn split_indexes(
    mults: &Array1<'static, i32>,
    bounds: [i32; 2],
    degree: i32,
    continuity_range: i32,
) -> Result<Array1<'static, i32>, Failure> {
    if continuity_range == 0 {
        return bounds_only(bounds);
    }
    let mmax: i32 = bsplclib::max_knot_mult(mults, bounds[0], bounds[1])?;
    if degree - mmax >= continuity_range {
        return bounds_only(bounds);
    }
    let split: Vec<i32> = split_knots(mults, bounds, degree, continuity_range)?;
    let mut table: Array1<'static, i32> = one_based(split.len() as i32)?;
    for (i, index) in (1..).zip(split) {
        table[i] = index;
    }
    Ok(table)
}

// The first and last knot indices only.
fn bounds_only([first, last]: [i32; 2]) -> Result<Array1<'static, i32>, Failure> {
    let mut table: Array1<'static, i32> = one_based(2)?;
    (table[1], table[2]) = (first, last);
    Ok(table)
}

// SplitValues(i) = splitIndexes(i) for i in 1..=Length.
fn copy_indexes(table: &Array1<'static, i32>, target: &mut Array1<'_, i32>) -> Result<(), Failure> {
    for i in 1..=table.length() {
        target
            .set_value(i, table[i])
            .map_err(|error| array_failure(error, "GeomConvert knot splitting"))?;
    }
    Ok(())
}

// Standard_RangeError_Raise_if(Index < 1 || Index > Length).
fn indexed_value(table: &Array1<'static, i32>, index: i32, member: &str) -> Result<i32, Failure> {
    if index < 1 || index > table.length() {
        return Err(failure(
            "Standard_RangeError",
            &format!("{member}: Index = {index}; expected 1..={}", table.length()),
        ));
    }
    Ok(table[index])
}
