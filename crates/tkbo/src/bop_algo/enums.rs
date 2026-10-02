//! Public option values shared by the intersection, builder, and Boolean algorithms.

/// `BOPAlgo_GlueEnum`: gluing assumption passed to algorithms that support coinciding faces.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
#[repr(i32)]
pub enum Glue {
    /// `BOPAlgo_GlueOff` (the native default).
    #[default]
    Off = 0,
    /// `BOPAlgo_GlueShift`: partially coinciding faces without real intersections.
    Shift = 1,
    /// `BOPAlgo_GlueFull`: fully coinciding shapes without partial overlap.
    Full = 2,
}

/// `BOPAlgo_Operation`: operation requested from the Boolean builder.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
#[repr(i32)]
pub enum Operation {
    /// `BOPAlgo_COMMON`.
    Common = 0,
    /// `BOPAlgo_FUSE`.
    Fuse = 1,
    /// `BOPAlgo_CUT`.
    Cut = 2,
    /// `BOPAlgo_CUT21` (subtract the first operand from the second).
    Cut21 = 3,
    /// `BOPAlgo_SECTION`.
    Section = 4,
    /// `BOPAlgo_UNKNOWN` (the native default).
    #[default]
    Unknown = 5,
}
