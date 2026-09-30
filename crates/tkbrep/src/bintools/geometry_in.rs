// Rust translation of the static `operator>>` for gp_Pnt, gp_Dir, gp_Pnt2d and
// gp_Dir2d repeated in BinTools_CurveSet.cxx, BinTools_Curve2dSet.cxx and
// BinTools_SurfaceSet.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The value readers of the geometry records: every coordinate is a `GetReal`, which
//! raises Storage_StreamTypeMismatchError on a short read.
use super::scalars::{get_ext_char, get_integer, get_real};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::storage::IoStream;
use rustycad_tkmath::gp::{Direction2, Direction3, Point2, Point3};

/// `IS >> gp_Pnt`: three `GetReal`, then `SetCoord`.
pub(crate) fn read_point3(stream: &mut IoStream) -> Result<Point3, Failure> {
    let (x, y, z) = (get_real(stream)?, get_real(stream)?, get_real(stream)?);
    Ok(Point3::new(x, y, z))
}

/// `IS >> gp_Dir`: three `GetReal`, then `SetCoord`, which raises
/// Standard_ConstructionError for a null vector.
pub(crate) fn read_direction3(stream: &mut IoStream) -> Result<Direction3, Failure> {
    let (x, y, z) = (get_real(stream)?, get_real(stream)?, get_real(stream)?);
    let mut direction = Direction3::default();
    direction.set_coordinates(x, y, z)?;
    Ok(direction)
}

/// `IS >> gp_Pnt2d`: two `GetReal`, then `SetCoord`.
pub(crate) fn read_point2(stream: &mut IoStream) -> Result<Point2, Failure> {
    let (x, y) = (get_real(stream)?, get_real(stream)?);
    Ok(Point2::new(x, y))
}

/// `IS >> gp_Dir2d`: two `GetReal`, then `SetCoord` (Standard_ConstructionError for a
/// null vector).
pub(crate) fn read_direction2(stream: &mut IoStream) -> Result<Direction2, Failure> {
    let (x, y) = (get_real(stream)?, get_real(stream)?);
    let mut direction = Direction2::default();
    direction.set_coordinates(x, y)?;
    Ok(direction)
}

/// `GetExtChar` of a degree: the `char16_t` widened to `int`.
pub(crate) fn read_degree(stream: &mut IoStream) -> Result<i32, Failure> {
    get_ext_char(stream).map(i32::from)
}

/// `NCollection_Array1<T> array(1, upper)`: a negative extent is refused as the native
/// allocation refuses it.
pub(crate) fn array1<T: Default>(upper: i32) -> Result<Array1<'static, T>, Failure> {
    Array1::with_bounds(1, upper).map_err(|error| error.into_failure("NCollection_Array1"))
}

/// `array(index) = value` inside the bounds the reader allocated.
pub(crate) fn store<T>(array: &mut Array1<'_, T>, index: i32, value: T) -> Result<(), Failure> {
    array
        .set_value(index, value)
        .map_err(|error| error.into_failure("NCollection_Array1::ChangeValue"))
}

/// The pole loop of the curve readers: `count` poles, each followed by its weight when
/// `rational` (the weights stay 0 otherwise and are not used).
pub(crate) fn read_poles<P: Default>(
    stream: &mut IoStream,
    count: i32,
    rational: bool,
    read_point: fn(&mut IoStream) -> Result<P, Failure>,
) -> Result<(Array1<'static, P>, Array1<'static, f64>), Failure> {
    let mut poles: Array1<'static, P> = array1(count)?;
    let mut weights: Array1<'static, f64> = array1(count)?;
    for index in 1..=count {
        store(&mut poles, index, read_point(stream)?)?;
        if rational {
            store(&mut weights, index, get_real(stream)?)?;
        }
    }
    Ok((poles, weights))
}

/// The knot loop: `count` pairs of `GetReal(knot)` and `GetInteger(mult)`.
pub(crate) fn read_knots(
    stream: &mut IoStream,
    count: i32,
) -> Result<(Array1<'static, f64>, Array1<'static, i32>), Failure> {
    let mut knots: Array1<'static, f64> = array1(count)?;
    let mut mults: Array1<'static, i32> = array1(count)?;
    for index in 1..=count {
        store(&mut knots, index, get_real(stream)?)?;
        store(&mut mults, index, get_integer(stream)?)?;
    }
    Ok((knots, mults))
}
