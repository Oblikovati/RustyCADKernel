// SPDX-License-Identifier: GPL-2.0-only
//! Shared exact construction bounds; extraction preserves the proved line expressions.
pub(super) mod convolution;
mod enclosure;
mod rational;
pub(super) mod rounding;
pub(super) use enclosure::{RatioInterval, bernstein_enclosure};
pub(super) use rational::ExactRatio;
pub(super) mod world;
