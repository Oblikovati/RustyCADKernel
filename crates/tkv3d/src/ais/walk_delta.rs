//! Native walking deltas distinguish nonzero motion from a held navigation key.
use rustycad_tkernel::standard::real::real_small;
/// Walking translation axis; `AisWalkTranslation::Side` accesses lateral motion.
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AisWalkTranslation {
    /// Forward/backward motion.
    Forward = 0,
    /// Lateral motion.
    Side = 1,
    /// Vertical motion.
    Up = 2,
}
/// Walking rotation axis; `AisWalkRotation::Yaw` accesses heading changes.
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AisWalkRotation {
    /// Heading change.
    Yaw = 0,
    /// Pitch change.
    Pitch = 1,
    /// Roll change.
    Roll = 2,
}
/// One walking component; `AisWalkPart::new().pressure == 1.0`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AisWalkPart {
    /// Delta value.
    pub value: f64,
    /// Input key pressure.
    pub pressure: f64,
    /// Input duration.
    pub duration: f64,
}
impl Default for AisWalkPart {
    fn default() -> Self {
        Self {
            value: 0.,
            pressure: 1.,
            duration: 0.,
        }
    }
}
impl AisWalkPart {
    /// Start with no motion and full pressure; `let part = AisWalkPart::new()`.
    pub fn new() -> Self {
        Self::default()
    }
    /// Test the native DBL_MIN boundary; `AisWalkPart::new().is_empty()` is true.
    pub fn is_empty(&self) -> bool {
        // OCCT 3d097a0 uses <= RealSmall(); NaN remains nonempty, as in the native comparison.
        self.value.abs() <= real_small()
    }
}
/// Six-axis motion and independent stance flags; `AisWalkDelta::new()[AisWalkRotation::Yaw].value = 0.1`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct AisWalkDelta {
    translation: [AisWalkPart; 3],
    rotation: [AisWalkPart; 3],
    defined: bool,
    jumping: bool,
    crouching: bool,
    running: bool,
}
impl AisWalkDelta {
    /// Start with six empty components and no stance flags; `let delta = AisWalkDelta::new()`.
    pub fn new() -> Self {
        Self::default()
    }
    /// Borrow a translation axis; `delta.translation(AisWalkTranslation::Forward)`.
    pub fn translation(&self, axis: AisWalkTranslation) -> &AisWalkPart {
        &self.translation[axis as usize]
    }
    /// Edit a translation axis; `delta.translation_mut(AisWalkTranslation::Up).value = 1.0`.
    pub fn translation_mut(&mut self, axis: AisWalkTranslation) -> &mut AisWalkPart {
        &mut self.translation[axis as usize]
    }
    /// Borrow a rotation axis; `delta.rotation(AisWalkRotation::Pitch)`.
    pub fn rotation(&self, axis: AisWalkRotation) -> &AisWalkPart {
        &self.rotation[axis as usize]
    }
    /// Edit a rotation axis; `delta.rotation_mut(AisWalkRotation::Roll).value = 0.5`.
    pub fn rotation_mut(&mut self, axis: AisWalkRotation) -> &mut AisWalkPart {
        &mut self.rotation[axis as usize]
    }
    /// Test translation independently from key/stance flags; `delta.to_move()`.
    pub fn to_move(&self) -> bool {
        self.translation.iter().any(|part| !part.is_empty())
    }
    /// Test rotation independently from key/stance flags; `delta.to_rotate()`.
    pub fn to_rotate(&self) -> bool {
        self.rotation.iter().any(|part| !part.is_empty())
    }
    /// Test all six motion values; `AisWalkDelta::new().is_empty()` is true.
    pub fn is_empty(&self) -> bool {
        !self.to_move() && !self.to_rotate()
    }
    /// Detect held navigation keys or nonempty motion; `delta.set_defined(true); assert!(delta.is_defined())`.
    pub fn is_defined(&self) -> bool {
        self.defined || !self.is_empty()
    }
    /// Change held-key state without changing motion; `delta.set_defined(false)`.
    pub fn set_defined(&mut self, defined: bool) {
        self.defined = defined;
    }
}
macro_rules! walk_axis_index {
    ($axis:ty, $read:ident, $write:ident) => {
        impl std::ops::Index<$axis> for AisWalkDelta {
            type Output = AisWalkPart;
            fn index(&self, axis: $axis) -> &Self::Output {
                self.$read(axis)
            }
        }
        impl std::ops::IndexMut<$axis> for AisWalkDelta {
            fn index_mut(&mut self, axis: $axis) -> &mut Self::Output {
                self.$write(axis)
            }
        }
    };
}
walk_axis_index!(AisWalkTranslation, translation, translation_mut);
walk_axis_index!(AisWalkRotation, rotation, rotation_mut);
macro_rules! walk_stance_accessors {
    ($getter:ident, $setter:ident, $field:ident, $read_doc:literal, $write_doc:literal) => {
        impl AisWalkDelta {
            #[doc = $read_doc]
            pub fn $getter(&self) -> bool {
                self.$field
            }
            #[doc = $write_doc]
            pub fn $setter(&mut self, enabled: bool) {
                self.$field = enabled;
            }
        }
    };
}
walk_stance_accessors!(
    is_jumping,
    set_jumping,
    jumping,
    "Inspect jumping state; `delta.is_jumping()`.",
    "Change jumping without motion; `delta.set_jumping(true)`."
);
walk_stance_accessors!(
    is_crouching,
    set_crouching,
    crouching,
    "Inspect crouching state; `delta.is_crouching()`.",
    "Change crouching without motion; `delta.set_crouching(true)`."
);
walk_stance_accessors!(
    is_running,
    set_running,
    running,
    "Inspect running state; `delta.is_running()`.",
    "Change running without motion; `delta.set_running(true)`."
);
