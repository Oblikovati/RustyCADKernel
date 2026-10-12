//! Martensite context defaults control future isoline and triangulation presentations.
use super::AisInteractiveContext;

/// Native isoparametric direction; `AisTypeOfIso::Both`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum AisTypeOfIso {
    /// Constant-U isoparameters; `context.iso_number(AisTypeOfIso::U)`.
    U = 0,
    /// Constant-V isoparameters; `context.iso_number(AisTypeOfIso::V)`.
    V = 1,
    /// Both directions, reporting -1 when their counts differ; `AisTypeOfIso::Both`.
    Both = 2,
    /// The representable unnamed native value follows the switch's default path.
    Unnamed = 3,
}

impl AisInteractiveContext {
    /// Set selected default isoline counts without scheduling redraw; `context.set_iso_number(3, AisTypeOfIso::U)`.
    pub fn set_iso_number(&self, count: i32, direction: AisTypeOfIso) {
        if direction == AisTypeOfIso::Unnamed {
            return;
        }
        let drawer = self.default_drawer();
        if matches!(direction, AisTypeOfIso::U | AisTypeOfIso::Both) {
            drawer.u_iso_aspect().set_number(count);
        }
        if matches!(direction, AisTypeOfIso::V | AisTypeOfIso::Both) {
            drawer.v_iso_aspect().set_number(count);
        }
    }

    /// Read the default count, or -1 for unequal U/V counts; `context.iso_number(AisTypeOfIso::Both)`.
    pub fn iso_number(&self, direction: AisTypeOfIso) -> i32 {
        if direction == AisTypeOfIso::Unnamed {
            return 0;
        }
        let drawer = self.default_drawer();
        match direction {
            AisTypeOfIso::U => drawer.u_iso_aspect().number(),
            AisTypeOfIso::V => drawer.v_iso_aspect().number(),
            AisTypeOfIso::Both => {
                let u = drawer.u_iso_aspect().number();
                let v = drawer.v_iso_aspect().number();
                if u == v { u } else { -1 }
            }
            AisTypeOfIso::Unnamed => 0,
        }
    }
}

macro_rules! context_drawer_property {
    ($get:ident, $set:ident, $type:ty, $intent:literal, $sample:literal) => {
        impl AisInteractiveContext {
            #[doc = concat!($intent, "; `context.", stringify!($get), "()`.")]
            pub fn $get(&self) -> $type {
                let drawer = self.default_drawer();
                drawer.$get()
            }
            #[doc = concat!($intent, "; `context.", stringify!($set), "(", $sample, ")`.")]
            pub fn $set(&self, value: $type) {
                let drawer = self.default_drawer();
                drawer.$set(value);
            }
        }
    };
}
context_drawer_property!(
    iso_on_plane,
    set_iso_on_plane,
    bool,
    "Default planar isoline policy",
    "true"
);
context_drawer_property!(
    iso_on_triangulation,
    set_iso_on_triangulation,
    bool,
    "Default triangulated isoline policy",
    "false"
);
context_drawer_property!(
    deviation_coefficient,
    set_deviation_coefficient,
    f64,
    "Default relative meshing deviation",
    "0.01"
);
context_drawer_property!(
    deviation_angle,
    set_deviation_angle,
    f64,
    "Default angular meshing deviation",
    "0.2"
);
