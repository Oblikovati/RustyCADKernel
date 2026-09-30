// Rust translation of FoundationClasses/TKMath/PLib/PLib.cxx (SetPoles/GetPoles) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Packing of 2D/3D poles into flat real arrays and back.
use super::access::{array_cell, array_value};
use crate::gp::{Point2, Point3};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;

mod sealed {
    pub trait Sealed {}
    impl Sealed for crate::gp::Point2 {}
    impl Sealed for crate::gp::Point3 {}
}

/// A pole type PLib flattens coordinate by coordinate (`gp_Pnt` or `gp_Pnt2d`);
/// `<Point3 as PolePoint>::DIMENSION == 3`.
pub trait PolePoint: sealed::Sealed + Copy + Default + 'static {
    /// Number of Cartesian coordinates.
    const DIMENSION: usize;
    /// Reads the zero-based coordinate; `point.pole_coordinate(0)`.
    fn pole_coordinate(&self, index: usize) -> f64;
    /// Writes the zero-based coordinate; `point.set_pole_coordinate(0, 1.0)`.
    fn set_pole_coordinate(&mut self, index: usize, value: f64);
}
impl PolePoint for Point3 {
    const DIMENSION: usize = 3;
    fn pole_coordinate(&self, index: usize) -> f64 {
        self.coordinates[index]
    }
    fn set_pole_coordinate(&mut self, index: usize, value: f64) {
        self.coordinates[index] = value;
    }
}
impl PolePoint for Point2 {
    const DIMENSION: usize = 2;
    fn pole_coordinate(&self, index: usize) -> f64 {
        [self.x(), self.y()][index]
    }
    fn set_pole_coordinate(&mut self, index: usize, value: f64) {
        match index {
            0 => self.set_x(value),
            _ => self.set_y(value),
        }
    }
}

/// Copies pole coordinates into `fp` from its lower bound (`PLib::SetPoles`);
/// `plib::set_poles(&poles, &mut flat)?`.
pub fn set_poles<P: PolePoint>(
    poles: &Array1<'_, P>,
    fp: &mut Array1<'_, f64>,
) -> Result<(), Failure> {
    let mut j: i32 = fp.lower();
    for i in poles.lower()..=poles.upper() {
        let pole: P = array_value(poles, i)?;
        for coordinate in 0..P::DIMENSION {
            *array_cell(fp, j)? = pole.pole_coordinate(coordinate);
            j += 1;
        }
    }
    Ok(())
}

/// Copies weighted coordinates followed by each weight (`PLib::SetPoles` with weights);
/// `plib::set_rational_poles(&poles, &weights, &mut flat)?`.
pub fn set_rational_poles<P: PolePoint>(
    poles: &Array1<'_, P>,
    weights: &Array1<'_, f64>,
    fp: &mut Array1<'_, f64>,
) -> Result<(), Failure> {
    let mut j: i32 = fp.lower();
    for i in poles.lower()..=poles.upper() {
        let w: f64 = array_value(weights, i)?;
        let pole: P = array_value(poles, i)?;
        for coordinate in 0..P::DIMENSION {
            *array_cell(fp, j)? = pole.pole_coordinate(coordinate) * w;
            j += 1;
        }
        *array_cell(fp, j)? = w;
        j += 1;
    }
    Ok(())
}

/// Reads poles back from consecutive coordinates (`PLib::GetPoles`);
/// `plib::get_poles(&flat, &mut poles)?`.
pub fn get_poles<P: PolePoint>(
    fp: &Array1<'_, f64>,
    poles: &mut Array1<'_, P>,
) -> Result<(), Failure> {
    let mut j: i32 = fp.lower();
    for i in poles.lower()..=poles.upper() {
        for coordinate in 0..P::DIMENSION {
            let value: f64 = array_value(fp, j)?;
            array_cell(poles, i)?.set_pole_coordinate(coordinate, value);
            j += 1;
        }
    }
    Ok(())
}

/// Divides weighted coordinates by the trailing weight (`PLib::GetPoles` with weights);
/// `plib::get_rational_poles(&flat, &mut poles, &mut weights)?`.
pub fn get_rational_poles<P: PolePoint>(
    fp: &Array1<'_, f64>,
    poles: &mut Array1<'_, P>,
    weights: &mut Array1<'_, f64>,
) -> Result<(), Failure> {
    let mut j: i32 = fp.lower();
    for i in poles.lower()..=poles.upper() {
        // The weight is read and stored before any coordinate, as in the native loop.
        let w: f64 = array_value(fp, j + P::DIMENSION as i32)?;
        *array_cell(weights, i)? = w;
        for coordinate in 0..P::DIMENSION {
            let value: f64 = array_value(fp, j)? / w;
            array_cell(poles, i)?.set_pole_coordinate(coordinate, value);
            j += 1;
        }
        j += 1;
    }
    Ok(())
}
