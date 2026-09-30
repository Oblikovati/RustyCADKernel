//! Complete Bnd_Box legacy suite, preserving original assertions.
#![allow(non_snake_case, unused_mut, unused_assignments, unused_variables)]
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkmath::{
    bnd::BoundingBox3,
    gp::{Axis1, Direction3, Line3, Plane3, Point3, Transform3},
};
include!("bnd_box/suites_01.rs");
include!("bnd_box/suites_09.rs");
include!("bnd_box/suites_17.rs");
include!("bnd_box/suites_25.rs");
include!("bnd_box/suites_33.rs");
include!("bnd_box/suites_41.rs");
