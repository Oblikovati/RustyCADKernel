//! `Draw_ColorKind`, `Draw_Color` and `Draw_MarkerShape`.

/// The fifteen colors of DRAW's palette (`Draw_ColorKind`, French names as upstream).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(i32)]
pub enum DrawColorKind {
    /// `Draw_blanc`: white (index 0).
    #[default]
    Blanc,
    /// `Draw_rouge`: red.
    Rouge,
    /// `Draw_vert`: green.
    Vert,
    /// `Draw_bleu`: blue.
    Bleu,
    /// `Draw_cyan`: cyan.
    Cyan,
    /// `Draw_or`: gold.
    Or,
    /// `Draw_magenta`: magenta.
    Magenta,
    /// `Draw_marron`: maroon.
    Marron,
    /// `Draw_orange`: orange.
    Orange,
    /// `Draw_rose`: pink.
    Rose,
    /// `Draw_saumon`: salmon.
    Saumon,
    /// `Draw_violet`: violet.
    Violet,
    /// `Draw_jaune`: yellow.
    Jaune,
    /// `Draw_kaki`: khaki.
    Kaki,
    /// `Draw_corail`: coral (index 14).
    Corail,
}

/// `MAXCOLOR`: the number of palette entries.
pub const MAXCOLOR: usize = 15;

/// The X11 color names DRAW allocates for the palette (`ColorNames` in `Draw.cxx`).
pub const COLOR_NAMES: [&str; MAXCOLOR] = [
    "White", "Red", "Green", "Blue", "Cyan", "Gold", "Magenta", "Maroon", "Orange", "Pink",
    "Salmon", "Violet", "Yellow", "Khaki", "Coral",
];

impl DrawColorKind {
    /// Every kind, in index order.
    pub const ALL: [Self; MAXCOLOR] = [
        Self::Blanc,
        Self::Rouge,
        Self::Vert,
        Self::Bleu,
        Self::Cyan,
        Self::Or,
        Self::Magenta,
        Self::Marron,
        Self::Orange,
        Self::Rose,
        Self::Saumon,
        Self::Violet,
        Self::Jaune,
        Self::Kaki,
        Self::Corail,
    ];

    /// The palette index (the C++ enum value).
    pub fn index(self) -> usize {
        self as usize
    }

    /// The kind with palette index `index`; `None` outside `0..MAXCOLOR`.
    ///
    /// ```
    /// use rustycad_tkdraw::draw::DrawColorKind;
    /// assert_eq!(DrawColorKind::from_index(3), Some(DrawColorKind::Bleu));
    /// assert_eq!(DrawColorKind::from_index(15), None);
    /// ```
    pub fn from_index(index: usize) -> Option<Self> {
        Self::ALL.get(index).copied()
    }
}

/// `Draw_Color`: a palette color (white by default).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct DrawColor {
    kind: DrawColorKind,
}

impl DrawColor {
    /// `Draw_Color(Draw_ColorKind)`.
    ///
    /// ```
    /// use rustycad_tkdraw::draw::{DrawColor, DrawColorKind};
    /// assert_eq!(DrawColor::new(DrawColorKind::Vert).id(), DrawColorKind::Vert);
    /// assert_eq!(DrawColor::default().id(), DrawColorKind::Blanc);
    /// ```
    pub fn new(kind: DrawColorKind) -> Self {
        Self { kind }
    }

    /// `ID`: the palette entry.
    pub fn id(self) -> DrawColorKind {
        self.kind
    }
}

/// `Draw_MarkerShape`. `Circle` keeps its screen size under zoom; `CircleZoom` scales.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DrawMarkerShape {
    /// `Draw_Square`.
    Square,
    /// `Draw_Losange`: a diamond.
    Losange,
    /// `Draw_X`.
    X,
    /// `Draw_Plus`.
    Plus,
    /// `Draw_Circle`.
    Circle,
    /// `Draw_CircleZoom`.
    CircleZoom,
}
