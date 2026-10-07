//! Virtual keys, key modifier flags and mouse buttons (`Aspect_VKey.hxx`,
//! `Aspect_VKeyFlags.hxx`).

/// A virtual key as an integer, so applications can extend the basic keys (`Aspect_VKey`).
pub type AspectVKey = u32;
/// Key modifier flags, combined with a general key from [`AspectVKey`] (`Aspect_VKeyFlags`).
pub type AspectVKeyFlags = u32;
/// Mouse buttons, combined with [`AspectVKey`] and [`AspectVKeyFlags`] (`Aspect_VKeyMouse`).
pub type AspectVKeyMouse = u32;

/// Enumeration defining virtual keys irrelevant to current keyboard layout for simplified hot-keys
/// management logic.
///
/// Native enumeration `Aspect_VKeyBasic` (`Aspect_VKey.hxx`).
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum AspectVKeyBasic {
    /// `Aspect_VKey_UNKNOWN`.
    Unknown = 0,
    /// `Aspect_VKey_A`.
    A = 1,
    /// `Aspect_VKey_B`.
    B = 2,
    /// `Aspect_VKey_C`.
    C = 3,
    /// `Aspect_VKey_D`.
    D = 4,
    /// `Aspect_VKey_E`.
    E = 5,
    /// `Aspect_VKey_F`.
    F = 6,
    /// `Aspect_VKey_G`.
    G = 7,
    /// `Aspect_VKey_H`.
    H = 8,
    /// `Aspect_VKey_I`.
    I = 9,
    /// `Aspect_VKey_J`.
    J = 10,
    /// `Aspect_VKey_K`.
    K = 11,
    /// `Aspect_VKey_L`.
    L = 12,
    /// `Aspect_VKey_M`.
    M = 13,
    /// `Aspect_VKey_N`.
    N = 14,
    /// `Aspect_VKey_O`.
    O = 15,
    /// `Aspect_VKey_P`.
    P = 16,
    /// `Aspect_VKey_Q`.
    Q = 17,
    /// `Aspect_VKey_R`.
    R = 18,
    /// `Aspect_VKey_S`.
    S = 19,
    /// `Aspect_VKey_T`.
    T = 20,
    /// `Aspect_VKey_U`.
    U = 21,
    /// `Aspect_VKey_V`.
    V = 22,
    /// `Aspect_VKey_W`.
    W = 23,
    /// `Aspect_VKey_X`.
    X = 24,
    /// `Aspect_VKey_Y`.
    Y = 25,
    /// `Aspect_VKey_Z`.
    Z = 26,
    /// `Aspect_VKey_0`.
    Digit0 = 27,
    /// `Aspect_VKey_1`.
    Digit1 = 28,
    /// `Aspect_VKey_2`.
    Digit2 = 29,
    /// `Aspect_VKey_3`.
    Digit3 = 30,
    /// `Aspect_VKey_4`.
    Digit4 = 31,
    /// `Aspect_VKey_5`.
    Digit5 = 32,
    /// `Aspect_VKey_6`.
    Digit6 = 33,
    /// `Aspect_VKey_7`.
    Digit7 = 34,
    /// `Aspect_VKey_8`.
    Digit8 = 35,
    /// `Aspect_VKey_9`.
    Digit9 = 36,
    /// `Aspect_VKey_F1`.
    F1 = 37,
    /// `Aspect_VKey_F2`.
    F2 = 38,
    /// `Aspect_VKey_F3`.
    F3 = 39,
    /// `Aspect_VKey_F4`.
    F4 = 40,
    /// `Aspect_VKey_F5`.
    F5 = 41,
    /// `Aspect_VKey_F6`.
    F6 = 42,
    /// `Aspect_VKey_F7`.
    F7 = 43,
    /// `Aspect_VKey_F8`.
    F8 = 44,
    /// `Aspect_VKey_F9`.
    F9 = 45,
    /// `Aspect_VKey_F10`.
    F10 = 46,
    /// `Aspect_VKey_F11`.
    F11 = 47,
    /// `Aspect_VKey_F12`.
    F12 = 48,
    /// `Aspect_VKey_Up`.
    Up = 49,
    /// `Aspect_VKey_Down`.
    Down = 50,
    /// `Aspect_VKey_Left`.
    Left = 51,
    /// `Aspect_VKey_Right`.
    Right = 52,
    /// '+'.
    Plus = 53,
    /// '-'.
    Minus = 54,
    /// '=+'.
    Equal = 55,
    /// `Aspect_VKey_PageUp`.
    PageUp = 56,
    /// `Aspect_VKey_PageDown`.
    PageDown = 57,
    /// `Aspect_VKey_Home`.
    Home = 58,
    /// `Aspect_VKey_End`.
    End = 59,
    /// `Aspect_VKey_Escape`.
    Escape = 60,
    /// `Aspect_VKey_Back`.
    Back = 61,
    /// `Aspect_VKey_Enter`.
    Enter = 62,
    /// `Aspect_VKey_Backspace`.
    Backspace = 63,
    /// `Aspect_VKey_Space`.
    Space = 64,
    /// `Aspect_VKey_Delete`.
    Delete = 65,
    /// `Aspect_VKey_Tilde`.
    Tilde = 66,
    /// `Aspect_VKey_Tab`.
    Tab = 67,
    /// ','.
    Comma = 68,
    /// '.'.
    Period = 69,
    /// ';:'.
    Semicolon = 70,
    /// '/?'.
    Slash = 71,
    /// '[{'.
    BracketLeft = 72,
    /// '\|'.
    Backslash = 73,
    /// ']}'.
    BracketRight = 74,
    /// ''"'.
    Apostrophe = 75,
    /// Num Lock key.
    Numlock = 76,
    /// Scroll Lock key.
    Scroll = 77,
    /// `Aspect_VKey_Numpad0`.
    Numpad0 = 78,
    /// `Aspect_VKey_Numpad1`.
    Numpad1 = 79,
    /// `Aspect_VKey_Numpad2`.
    Numpad2 = 80,
    /// `Aspect_VKey_Numpad3`.
    Numpad3 = 81,
    /// `Aspect_VKey_Numpad4`.
    Numpad4 = 82,
    /// `Aspect_VKey_Numpad5`.
    Numpad5 = 83,
    /// `Aspect_VKey_Numpad6`.
    Numpad6 = 84,
    /// `Aspect_VKey_Numpad7`.
    Numpad7 = 85,
    /// `Aspect_VKey_Numpad8`.
    Numpad8 = 86,
    /// `Aspect_VKey_Numpad9`.
    Numpad9 = 87,
    /// numpad '*'.
    NumpadMultiply = 88,
    /// numpad '+'.
    NumpadAdd = 89,
    /// numpad '-'.
    NumpadSubtract = 90,
    /// numpad '/'.
    NumpadDivide = 91,
    /// `Aspect_VKey_MediaNextTrack`.
    MediaNextTrack = 92,
    /// `Aspect_VKey_MediaPreviousTrack`.
    MediaPreviousTrack = 93,
    /// `Aspect_VKey_MediaStop`.
    MediaStop = 94,
    /// `Aspect_VKey_MediaPlayPause`.
    MediaPlayPause = 95,
    /// `Aspect_VKey_VolumeMute`.
    VolumeMute = 96,
    /// `Aspect_VKey_VolumeDown`.
    VolumeDown = 97,
    /// `Aspect_VKey_VolumeUp`.
    VolumeUp = 98,
    /// `Aspect_VKey_BrowserBack`.
    BrowserBack = 99,
    /// `Aspect_VKey_BrowserForward`.
    BrowserForward = 100,
    /// `Aspect_VKey_BrowserRefresh`.
    BrowserRefresh = 101,
    /// `Aspect_VKey_BrowserStop`.
    BrowserStop = 102,
    /// `Aspect_VKey_BrowserSearch`.
    BrowserSearch = 103,
    /// `Aspect_VKey_BrowserFavorites`.
    BrowserFavorites = 104,
    /// `Aspect_VKey_BrowserHome`.
    BrowserHome = 105,
    /// `Aspect_VKey_ViewTop`.
    ViewTop = 106,
    /// `Aspect_VKey_ViewBottom`.
    ViewBottom = 107,
    /// `Aspect_VKey_ViewLeft`.
    ViewLeft = 108,
    /// `Aspect_VKey_ViewRight`.
    ViewRight = 109,
    /// `Aspect_VKey_ViewFront`.
    ViewFront = 110,
    /// `Aspect_VKey_ViewBack`.
    ViewBack = 111,
    /// `Aspect_VKey_ViewAxoLeftProj`.
    ViewAxoLeftProj = 112,
    /// `Aspect_VKey_ViewAxoRightProj`.
    ViewAxoRightProj = 113,
    /// `Aspect_VKey_ViewFitAll`.
    ViewFitAll = 114,
    /// `Aspect_VKey_ViewRoll90CW`.
    ViewRoll90CW = 115,
    /// `Aspect_VKey_ViewRoll90CCW`.
    ViewRoll90CCW = 116,
    /// `Aspect_VKey_ViewSwitchRotate`.
    ViewSwitchRotate = 117,
    /// `Aspect_VKey_Shift`.
    Shift = 118,
    /// `Aspect_VKey_Control`.
    Control = 119,
    /// `Aspect_VKey_Alt`.
    Alt = 120,
    /// `Aspect_VKey_Menu`.
    Menu = 121,
    /// `Aspect_VKey_Meta`.
    Meta = 122,
    /// interact.
    NavInteract = 123,
    /// go forward.
    NavForward = 124,
    /// go backward.
    NavBackward = 125,
    /// sidewalk, left.
    NavSlideLeft = 126,
    /// sidewalk, right.
    NavSlideRight = 127,
    /// lift up.
    NavSlideUp = 128,
    /// fall down.
    NavSlideDown = 129,
    /// bank left  (roll counter-clockwise).
    NavRollCCW = 130,
    /// bank right (roll clockwise).
    NavRollCW = 131,
    /// look left  (yaw counter-clockwise).
    NavLookLeft = 132,
    /// look right (yaw clockwise).
    NavLookRight = 133,
    /// look up    (pitch clockwise).
    NavLookUp = 134,
    /// look down  (pitch counter-clockwise).
    NavLookDown = 135,
    /// crouch walking.
    NavCrouch = 136,
    /// jump.
    NavJump = 137,
    /// increase continuous velocity in forward  direction.
    NavThrustForward = 138,
    /// increase continuous velocity in reversed direction.
    NavThrustBackward = 139,
    /// reset continuous velocity.
    NavThrustStop = 140,
    /// increase navigation speed.
    NavSpeedIncrease = 141,
    /// decrease navigation speed.
    NavSpeedDecrease = 142,
}

/// First key of the auxiliary ranges (`Aspect_VKey_Lower`).
pub const ASPECT_VKEY_LOWER: AspectVKey = 0;
/// First modifier key (`Aspect_VKey_ModifiersLower`).
pub const ASPECT_VKEY_MODIFIERS_LOWER: AspectVKey = AspectVKeyBasic::Shift as AspectVKey;
/// Last modifier key (`Aspect_VKey_ModifiersUpper`).
pub const ASPECT_VKEY_MODIFIERS_UPPER: AspectVKey = AspectVKeyBasic::Meta as AspectVKey;
/// First virtual navigation key (`Aspect_VKey_NavigationKeysLower`).
pub const ASPECT_VKEY_NAVIGATION_KEYS_LOWER: AspectVKey =
    AspectVKeyBasic::NavInteract as AspectVKey;
/// Last virtual navigation key (`Aspect_VKey_NavigationKeysUpper`).
pub const ASPECT_VKEY_NAVIGATION_KEYS_UPPER: AspectVKey =
    AspectVKeyBasic::NavSpeedDecrease as AspectVKey;
/// Last basic key (`Aspect_VKey_Upper`).
pub const ASPECT_VKEY_UPPER: AspectVKey = AspectVKeyBasic::NavSpeedDecrease as AspectVKey;
/// Number of basic keys (`Aspect_VKey_NB`).
pub const ASPECT_VKEY_NB: AspectVKey = ASPECT_VKEY_UPPER - ASPECT_VKEY_LOWER + 1;
/// Largest key value; the upper bits carry modifier flags (`Aspect_VKey_MAX`).
pub const ASPECT_VKEY_MAX: AspectVKey = 255;

/// No modifier (`Aspect_VKeyFlags_NONE`).
pub const ASPECT_VKEY_FLAGS_NONE: AspectVKeyFlags = 0;
/// Shift modifier; the first 8 bits are reserved for the key (`Aspect_VKeyFlags_SHIFT`).
pub const ASPECT_VKEY_FLAGS_SHIFT: AspectVKeyFlags = 1 << 8;
/// Control modifier (`Aspect_VKeyFlags_CTRL`).
pub const ASPECT_VKEY_FLAGS_CTRL: AspectVKeyFlags = 1 << 9;
/// Alt modifier (`Aspect_VKeyFlags_ALT`).
pub const ASPECT_VKEY_FLAGS_ALT: AspectVKeyFlags = 1 << 10;
/// Menu modifier (`Aspect_VKeyFlags_MENU`).
pub const ASPECT_VKEY_FLAGS_MENU: AspectVKeyFlags = 1 << 11;
/// Meta modifier (`Aspect_VKeyFlags_META`).
pub const ASPECT_VKEY_FLAGS_META: AspectVKeyFlags = 1 << 12;
/// Every modifier (`Aspect_VKeyFlags_ALL`).
pub const ASPECT_VKEY_FLAGS_ALL: AspectVKeyFlags = ASPECT_VKEY_FLAGS_SHIFT
    | ASPECT_VKEY_FLAGS_CTRL
    | ASPECT_VKEY_FLAGS_ALT
    | ASPECT_VKEY_FLAGS_MENU
    | ASPECT_VKEY_FLAGS_META;

/// No mouse button (`Aspect_VKeyMouse_NONE`).
pub const ASPECT_VKEY_MOUSE_NONE: AspectVKeyMouse = 0;
/// Left mouse button (`Aspect_VKeyMouse_LeftButton`).
pub const ASPECT_VKEY_MOUSE_LEFT_BUTTON: AspectVKeyMouse = 1 << 13;
/// Middle mouse button (scroll) (`Aspect_VKeyMouse_MiddleButton`).
pub const ASPECT_VKEY_MOUSE_MIDDLE_BUTTON: AspectVKeyMouse = 1 << 14;
/// Right mouse button (`Aspect_VKeyMouse_RightButton`).
pub const ASPECT_VKEY_MOUSE_RIGHT_BUTTON: AspectVKeyMouse = 1 << 15;
/// The three main buttons (`Aspect_VKeyMouse_MainButtons`).
pub const ASPECT_VKEY_MOUSE_MAIN_BUTTONS: AspectVKeyMouse = ASPECT_VKEY_MOUSE_LEFT_BUTTON
    | ASPECT_VKEY_MOUSE_MIDDLE_BUTTON
    | ASPECT_VKEY_MOUSE_RIGHT_BUTTON;

/// The modifier flag of a modifier key, 0 for any other key (`Aspect_VKey2Modifier`).
///
/// Example: `aspect_vkey_to_modifier(AspectVKeyBasic::Shift as u32) == ASPECT_VKEY_FLAGS_SHIFT`.
pub fn aspect_vkey_to_modifier(key: AspectVKey) -> AspectVKeyFlags {
    const SHIFT: AspectVKey = AspectVKeyBasic::Shift as AspectVKey;
    const CONTROL: AspectVKey = AspectVKeyBasic::Control as AspectVKey;
    const ALT: AspectVKey = AspectVKeyBasic::Alt as AspectVKey;
    const MENU: AspectVKey = AspectVKeyBasic::Menu as AspectVKey;
    const META: AspectVKey = AspectVKeyBasic::Meta as AspectVKey;
    match key {
        SHIFT => ASPECT_VKEY_FLAGS_SHIFT,
        CONTROL => ASPECT_VKEY_FLAGS_CTRL,
        ALT => ASPECT_VKEY_FLAGS_ALT,
        MENU => ASPECT_VKEY_FLAGS_MENU,
        META => ASPECT_VKEY_FLAGS_META,
        _ => 0,
    }
}
