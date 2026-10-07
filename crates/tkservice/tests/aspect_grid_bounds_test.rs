//! Direct translation of Aspect_Grid_Bounds_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::standard::StandardTransient;
use rustycad_tkservice::aspect::{
    AspectCircularGrid, AspectCircularGridBase, AspectCircularGridMembers, AspectGrid,
    AspectGridBase, AspectRectangularGrid, AspectRectangularGridBase, AspectRectangularGridMembers,
};
use std::f64::consts::PI;

// Concrete shims: Aspect_RectangularGrid / Aspect_CircularGrid are abstract because
// Display/Erase/IsDisplayed/UpdateDisplay are pure virtual on the base. V3d_RectangularGrid
// supplies them in production; these tests only exercise the bounds accessors.
struct RectGridStub {
    transient: StandardTransient,
    base: AspectRectangularGridBase,
}
impl_transient!(RectGridStub, "RectGridStub", dyn AspectRectangularGrid, transient,
    interfaces: [dyn AspectGrid, dyn AspectRectangularGrid]);
impl RectGridStub {
    fn new(theXStep: f64, theYStep: f64) -> Self {
        let base = AspectRectangularGridBase::new(theXStep, theYStep, 0.0, 0.0, 0.0, 0.0, 0.0)
            .expect("valid steps");
        Self {
            transient: StandardTransient::default(),
            base,
        }
    }
}
impl AspectGrid for RectGridStub {
    fn grid_base(&self) -> &AspectGridBase {
        self.base.grid()
    }
    fn compute(&self, x: f64, y: f64) -> (f64, f64) {
        self.base.compute(x, y)
    }
    fn display(&self) {}
    fn erase(&self) {}
    fn is_displayed(&self) -> bool {
        false
    }
    fn init(&self) {
        self.base.init();
    }
    fn update_display(&self) {}
}
impl AspectRectangularGrid for RectGridStub {
    fn rectangular_base(&self) -> &AspectRectangularGridBase {
        &self.base
    }
}

struct CircGridStub {
    transient: StandardTransient,
    base: AspectCircularGridBase,
}
impl_transient!(CircGridStub, "CircGridStub", dyn AspectCircularGrid, transient,
    interfaces: [dyn AspectGrid, dyn AspectCircularGrid]);
impl CircGridStub {
    fn new(theRadiusStep: f64, theDivisions: i32) -> Self {
        let base = AspectCircularGridBase::new(theRadiusStep, theDivisions, 0.0, 0.0, 0.0);
        Self {
            transient: StandardTransient::default(),
            base,
        }
    }
}
impl AspectGrid for CircGridStub {
    fn grid_base(&self) -> &AspectGridBase {
        self.base.grid()
    }
    fn compute(&self, x: f64, y: f64) -> (f64, f64) {
        self.base.compute(x, y)
    }
    fn display(&self) {}
    fn erase(&self) {}
    fn is_displayed(&self) -> bool {
        false
    }
    fn init(&self) {
        self.base.init();
    }
    fn update_display(&self) {}
}
impl AspectCircularGrid for CircGridStub {
    fn circular_base(&self) -> &AspectCircularGridBase {
        &self.base
    }
}

#[test]
fn Aspect_GridBoundsTest__Rectangular_Defaults() {
    let aGrid = RectGridStub::new(1.0, 1.0);
    assert_eq!(0.0, aGrid.size_x());
    assert_eq!(0.0, aGrid.size_y());
    assert_eq!(0.0, aGrid.z_offset());
}

#[test]
fn Aspect_GridBoundsTest__Rectangular_RoundTrip() {
    let aGrid = RectGridStub::new(1.0, 1.0);
    aGrid.set_size_x(8.0).unwrap();
    aGrid.set_size_y(4.0).unwrap();
    aGrid.set_z_offset(-0.005);
    assert_eq!(8.0, aGrid.size_x());
    assert_eq!(4.0, aGrid.size_y());
    assert_eq!(-0.005, aGrid.z_offset());
}

#[test]
fn Aspect_GridBoundsTest__Circular_Defaults() {
    let aGrid = CircGridStub::new(1.0, 8);
    assert_eq!(0.0, aGrid.radius());
    assert_eq!(0.0, aGrid.z_offset());
    assert_eq!(0.0, aGrid.angle_start());
    assert_eq!(0.0, aGrid.angle_end());
    assert!(!aGrid.is_arc());
}

#[test]
fn Aspect_GridBoundsTest__Circular_RoundTrip() {
    let aGrid = CircGridStub::new(1.0, 8);
    aGrid.set_radius(2.5).unwrap();
    aGrid.set_z_offset(-0.005);
    assert_eq!(2.5, aGrid.radius());
    assert_eq!(-0.005, aGrid.z_offset());
}

#[test]
fn Aspect_GridBoundsTest__Circular_ArcRange() {
    let aGrid = CircGridStub::new(1.0, 8);
    assert!(!aGrid.is_arc());

    aGrid.set_arc_range(0.0, PI);
    assert!(aGrid.is_arc());
    assert_eq!(0.0, aGrid.angle_start());
    assert_eq!(PI, aGrid.angle_end());

    aGrid.set_arc_range(0.0, 0.0);
    assert!(
        !aGrid.is_arc(),
        "equal start/end is the full-circle sentinel"
    );

    aGrid.set_arc_range(2.356, -2.356);
    assert!(aGrid.is_arc(), "wraparound (Start > End) is valid");
}
