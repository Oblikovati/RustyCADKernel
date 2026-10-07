#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ClearMode {
    Same,
    Other,
    Null,
    Current,
}
impl ClearMode {
    pub const ALL: [Self; 4] = [Self::Same, Self::Other, Self::Null, Self::Current];
    pub fn name(self) -> &'static str {
        match self {
            Self::Same => "same",
            Self::Other => "other",
            Self::Null => "null",
            Self::Current => "self",
        }
    }
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ClearState {
    Empty,
    Filled,
    Retained,
}
impl ClearState {
    pub const ALL: [Self; 3] = [Self::Empty, Self::Filled, Self::Retained];
    pub fn name(self) -> &'static str {
        match self {
            Self::Empty => "empty",
            Self::Filled => "filled",
            Self::Retained => "retained",
        }
    }
}
