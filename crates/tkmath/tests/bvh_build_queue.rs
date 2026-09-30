//! Direct BVH_BuildQueue_Test.cxx translation at pinned OCCT 3d097a0.
//! `std::thread` workers become scoped threads borrowing the queue.
#![allow(non_snake_case, unused_mut)]
use rustycad_tkmath::bvh::BvhBuildQueue;
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};

include!("bvh_build_queue/suites_0.rs");
include!("bvh_build_queue/suites_1.rs");
