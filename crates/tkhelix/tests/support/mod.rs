//! GoogleTest assertion helpers shared by the translated TKHelix suites.
#![allow(dead_code)]
use rustycad_tkernel::standard::Failure;

/// GoogleTest's biased representation of a double (sign and magnitude mapped onto an
/// ordered unsigned range), as `FloatingPoint::SignAndMagnitudeToBiased` does.
fn biased(value: f64) -> u64 {
    let bits: u64 = value.to_bits();
    let sign_mask: u64 = 1 << 63;
    if bits & sign_mask != 0 {
        (!bits).wrapping_add(1)
    } else {
        sign_mask | bits
    }
}

/// `EXPECT_DOUBLE_EQ(a, b)`: equal within 4 units in the last place, NaN never equal.
pub fn expect_double_eq(a: f64, b: f64) {
    let (x, y) = (biased(a), biased(b));
    let distance: u64 = x.max(y) - x.min(y);
    assert!(!a.is_nan() && !b.is_nan() && distance <= 4, "{a} != {b}");
}

/// `EXPECT_NEAR(a, b, t)`.
pub fn expect_near(a: f64, b: f64, t: f64) {
    assert!((a - b).abs() <= t, "|{a} - {b}| > {t}");
}

/// `EXPECT_THROW(statement, Type)`: the statement raised the native exception `kind`.
pub fn expect_throw<T>(result: Result<T, Failure>, kind: &str) {
    match result {
        Ok(_) => panic!("expected {kind}, got no exception"),
        Err(failure) => assert_eq!(failure.exception_type(), kind),
    }
}

/// Shape helpers of the HelixBRep fixtures (`TKHelixTest`, `HelixBRepTest`).
pub mod brep {
    use rustycad_tkbrep::topexp::Explorer;
    use rustycad_tkbrep::topods::Shape;
    use rustycad_tkernel::ncollection::Array1;
    use rustycad_tkg3d::top_abs::ShapeEnum;
    use rustycad_tkgeombase::gprop::GProps;
    use rustycad_tkmath::gp::{Axis3, Direction3, Point3};
    use rustycad_tktopalgo::brep_gprop::BRepGProp;

    /// The fixtures' `myAxis`: `gp_Ax3(gp_Pnt(0, 0, 0), gp_Dir(Z), gp_Dir(X))`.
    pub fn my_axis() -> Axis3 {
        Axis3::new(Point3::new(0., 0., 0.), Direction3::Z, Direction3::X).unwrap()
    }

    /// `NCollection_Array1<T>(1, n)` filled with `values`.
    pub fn array<T: Clone + Default>(values: &[T]) -> Array1<'static, T> {
        let mut array: Array1<'static, T> = Array1::with_bounds(1, values.len() as i32).unwrap();
        for (i, value) in values.iter().enumerate() {
            *array.change_value(i as i32 + 1).unwrap() = value.clone();
        }
        array
    }

    /// `CountShapes(theShape, theType)`: the TopExp_Explorer count.
    pub fn count_shapes(shape: &Shape, kind: ShapeEnum) -> i32 {
        let mut count = 0;
        let mut explorer = Explorer::new(shape, kind, ShapeEnum::Shape).unwrap();
        while explorer.more() {
            count += 1;
            explorer.next().unwrap();
        }
        count
    }

    /// `ComputeWireLength(theWire)`: `BRepGProp::LinearProperties` (default flags), then `Mass()`.
    pub fn compute_wire_length(wire: &Shape) -> f64 {
        let mut props = GProps::new();
        BRepGProp::linear_properties(wire, &mut props, false, false).unwrap();
        props.mass()
    }
}
