//! The XR input actions: `Aspect_XRAction`, `Aspect_XRActionSet`, the action data structs and
//! `Aspect_XRGenericAction`.
use super::{AspectTrackedDevicePose, AspectXRActionType};
use rustycad_tkernel::ncollection::Vec3;
use rustycad_tkernel::standard::{Handle, SharedState, StandardTransient};
use rustycad_tkernel::tcollection::AsciiString;

/// An XR input or output action; `AspectXRAction::new(&"/actions/menu".into(), AspectXRActionType::InputDigital)`.
pub struct AspectXRAction {
    transient: StandardTransient,
    id: AsciiString,
    action_type: AspectXRActionType,
    raw_handle: SharedState<u64>,
}
rustycad_tkernel::impl_transient!(
    AspectXRAction,
    "Aspect_XRAction",
    StandardTransient,
    transient
);

impl AspectXRAction {
    /// An action without a backend handle; `AspectXRAction::new(&id, AspectXRActionType::InputPose)`.
    pub fn new(id: &AsciiString, action_type: AspectXRActionType) -> Self {
        Self {
            transient: StandardTransient::default(),
            id: id.clone(),
            action_type,
            raw_handle: SharedState::new(0),
        }
    }
    /// The action id; `action.id()`.
    pub fn id(&self) -> &AsciiString {
        &self.id
    }
    /// The action type; `action.action_type()`.
    pub fn action_type(&self) -> AspectXRActionType {
        self.action_type
    }
    /// Whether a backend handle is set; `action.is_valid()`.
    pub fn is_valid(&self) -> bool {
        *self.raw_handle.read() != 0
    }
    /// The backend handle; `action.raw_handle()`.
    pub fn raw_handle(&self) -> u64 {
        *self.raw_handle.read()
    }
    /// Sets the backend handle; `action.set_raw_handle(7)`.
    pub fn set_raw_handle(&self, handle: u64) {
        *self.raw_handle.write() = handle;
    }
}

/// The fields of an action set.
struct XRActionSetState {
    raw_handle: u64,
    actions: Vec<(AsciiString, Handle<AspectXRAction>)>,
}

/// A set of XR actions; `AspectXRActionSet::new(&"/actions/main".into())`.
pub struct AspectXRActionSet {
    transient: StandardTransient,
    id: AsciiString,
    state: SharedState<XRActionSetState>,
}
rustycad_tkernel::impl_transient!(
    AspectXRActionSet,
    "Aspect_XRActionSet",
    StandardTransient,
    transient
);

impl AspectXRActionSet {
    /// An empty set; `AspectXRActionSet::new(&id)`.
    pub fn new(id: &AsciiString) -> Self {
        Self {
            transient: StandardTransient::default(),
            id: id.clone(),
            state: SharedState::new(XRActionSetState {
                raw_handle: 0,
                actions: Vec::new(),
            }),
        }
    }
    /// The set id; `set.id()`.
    pub fn id(&self) -> &AsciiString {
        &self.id
    }
    /// The backend handle; `set.raw_handle()`.
    pub fn raw_handle(&self) -> u64 {
        self.state.read().raw_handle
    }
    /// Sets the backend handle; `set.set_raw_handle(3)`.
    pub fn set_raw_handle(&self, handle: u64) {
        self.state.write().raw_handle = handle;
    }
    /// `AddAction`: adds an action by id (an existing id keeps its action, as the native
    /// indexed data map `Add` does); a null action is ignored (natively a null dereference).
    /// Example: `set.add_action(action)`.
    pub fn add_action(&self, action: Handle<AspectXRAction>) {
        let Some(id) = action.get().map(|a| a.id().clone()) else {
            return;
        };
        let mut state = self.state.write();
        if !state.actions.iter().any(|(key, _)| key.is_equal(&id)) {
            state.actions.push((id, action));
        }
    }
    /// The actions in insertion order; `set.actions()`.
    pub fn actions(&self) -> Vec<(AsciiString, Handle<AspectXRAction>)> {
        self.state.read().actions.clone()
    }
}

/// `Aspect_XRAnalogActionData`: the state of an analog action.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AspectXRAnalogActionData {
    /// `ActiveOrigin`.
    pub active_origin: u64,
    /// `UpdateTime` (negative for a past event).
    pub update_time: f32,
    /// `VecXYZ`: the current state.
    pub vec_xyz: Vec3<f32>,
    /// `DeltaXYZ`: the change since the previous update.
    pub delta_xyz: Vec3<f32>,
    /// `IsActive`.
    pub is_active: bool,
}
impl Default for AspectXRAnalogActionData {
    fn default() -> Self {
        let zero = Vec3::new(0.0, 0.0, 0.0);
        Self {
            active_origin: 0,
            update_time: 0.0,
            vec_xyz: zero,
            delta_xyz: zero,
            is_active: false,
        }
    }
}
impl AspectXRAnalogActionData {
    /// `IsChanged`: a non-zero delta; `data.is_changed()`.
    pub fn is_changed(&self) -> bool {
        self.delta_xyz != Vec3::new(0.0, 0.0, 0.0)
    }
}

/// `Aspect_XRDigitalActionData`: the state of a digital action.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct AspectXRDigitalActionData {
    /// `ActiveOrigin`.
    pub active_origin: u64,
    /// `UpdateTime`.
    pub update_time: f32,
    /// `IsActive`.
    pub is_active: bool,
    /// `IsPressed`.
    pub is_pressed: bool,
    /// `IsChanged`.
    pub is_changed: bool,
}

/// `Aspect_XRHapticActionData`: a vibration request.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct AspectXRHapticActionData {
    /// `Delay` in seconds.
    pub delay: f32,
    /// `Duration` in seconds.
    pub duration: f32,
    /// `Frequency`.
    pub frequency: f32,
    /// `Amplitude`.
    pub amplitude: f32,
}
impl AspectXRHapticActionData {
    /// `IsValid`: positive duration, amplitude and frequency, non-negative delay.
    pub fn is_valid(&self) -> bool {
        self.duration > 0.0 && self.amplitude > 0.0 && self.frequency > 0.0 && self.delay >= 0.0
    }
}

/// `Aspect_XRPoseActionData`: the state of a pose action.
#[derive(Clone, Copy, Debug, Default)]
pub struct AspectXRPoseActionData {
    /// `Pose`.
    pub pose: AspectTrackedDevicePose,
    /// `ActiveOrigin`.
    pub active_origin: u64,
    /// `IsActive`.
    pub is_active: bool,
}

/// `Aspect_XRGenericAction`: the generic actions of a tracked device role.
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AspectXRGenericAction {
    /// `IsHeadsetOn`.
    IsHeadsetOn = 0,
    /// `InputAppMenu`.
    InputAppMenu = 1,
    /// `InputSysMenu`.
    InputSysMenu = 2,
    /// `InputTriggerPull`.
    InputTriggerPull = 3,
    /// `InputTriggerClick`.
    InputTriggerClick = 4,
    /// `InputGripClick`.
    InputGripClick = 5,
    /// `InputTrackPadPosition`.
    InputTrackPadPosition = 6,
    /// `InputTrackPadTouch`.
    InputTrackPadTouch = 7,
    /// `InputTrackPadClick`.
    InputTrackPadClick = 8,
    /// `InputThumbstickPosition`.
    InputThumbstickPosition = 9,
    /// `InputThumbstickTouch`.
    InputThumbstickTouch = 10,
    /// `InputThumbstickClick`.
    InputThumbstickClick = 11,
    /// `InputPoseBase`.
    InputPoseBase = 12,
    /// `InputPoseFront`.
    InputPoseFront = 13,
    /// `InputPoseHandGrip`.
    InputPoseHandGrip = 14,
    /// `InputPoseFingerTip`.
    InputPoseFingerTip = 15,
    /// `OutputHaptic`.
    OutputHaptic = 16,
}

/// `Aspect_XRGenericAction_NB`.
pub const ASPECT_XR_GENERIC_ACTION_NB: i32 = AspectXRGenericAction::OutputHaptic as i32 + 1;
