//! `Font_GlyphOutline`: the vector outline of one glyph in font units (lines, quadratic and
//! cubic Bezier segments grouped in contours) and the identity of a resolved glyph.
use rustycad_tkernel::standard::hashing::{hash, hash_bytes};
use rustycad_tkmath::gp::Xy;
use std::sync::Arc;

/// `Font_GlyphOutline::Glyph::Identity`: the lifetime-safe owner of a font's glyph ids.
#[derive(Debug, Default)]
pub struct FontGlyphOutlineGlyphIdentity;

/// `Font_GlyphOutline::Glyph`: a glyph resolved by a font (owner, revision, face, index).
/// Example: `font.resolve_glyph('A' as u32)`.
#[derive(Clone, Debug)]
pub struct FontGlyphOutlineGlyph {
    owner: Arc<FontGlyphOutlineGlyphIdentity>,
    revision: u64,
    face: u32,
    index: u32,
}

impl PartialEq for FontGlyphOutlineGlyph {
    /// `operator==`: same owner object, revision, face and index.
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.owner, &other.owner)
            && self.revision == other.revision
            && self.face == other.face
            && self.index == other.index
    }
}
impl Eq for FontGlyphOutlineGlyph {}

impl FontGlyphOutlineGlyph {
    /// The glyph of a font (`Font_FTFont` only); `FontGlyphOutlineGlyph::new(owner, 1, 0, 36)`.
    pub(crate) fn new(
        owner: Arc<FontGlyphOutlineGlyphIdentity>,
        revision: u64,
        face: u32,
        index: u32,
    ) -> Self {
        Self {
            owner,
            revision,
            face,
            index,
        }
    }
    /// The identity of the font that resolved the glyph; `glyph.owner()`.
    pub fn owner(&self) -> &Arc<FontGlyphOutlineGlyphIdentity> {
        &self.owner
    }
    /// The geometry revision of the font at resolution; `glyph.revision()`.
    pub fn revision(&self) -> u64 {
        self.revision
    }
    /// 0 for the primary face, else 1 + the fallback Unicode subset; `glyph.face()`.
    pub fn face(&self) -> u32 {
        self.face
    }
    /// The glyph index in its face (0 for the missing glyph); `glyph.index()`.
    pub fn index(&self) -> u32 {
        self.index
    }
}

/// `Font_GlyphOutline::Glyph::Hasher`: the hash and equality of glyphs as map keys.
#[derive(Clone, Copy, Debug, Default)]
pub struct FontGlyphOutlineGlyphHasher;

impl FontGlyphOutlineGlyphHasher {
    /// The byte hash of the owner address, revision, face and index hashes.
    /// Example: `FontGlyphOutlineGlyphHasher.hash(&glyph)`.
    pub fn hash(&self, glyph: &FontGlyphOutlineGlyph) -> usize {
        let owner = Arc::as_ptr(&glyph.owner) as usize;
        let hashes: [usize; 4] = [
            hash::<usize, usize>(&owner),
            hash::<usize, u64>(&glyph.revision),
            hash::<usize, u32>(&glyph.face),
            hash::<usize, u32>(&glyph.index),
        ];
        let bytes: Vec<u8> = hashes.iter().flat_map(|h| h.to_ne_bytes()).collect();
        hash_bytes(&bytes)
    }
    /// The glyph equality; `FontGlyphOutlineGlyphHasher.equal(&a, &b)`.
    pub fn equal(&self, left: &FontGlyphOutlineGlyph, right: &FontGlyphOutlineGlyph) -> bool {
        left == right
    }
}

/// `Font_GlyphOutline::FillRule`.
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum FontGlyphOutlineFillRule {
    /// `NonZero`.
    #[default]
    NonZero = 0,
    /// `EvenOdd`.
    EvenOdd = 1,
}

/// `Font_GlyphOutline::Segment::Kind`.
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum FontGlyphOutlineSegmentKind {
    /// `Line`.
    #[default]
    Line = 0,
    /// `QuadraticBezier`.
    QuadraticBezier = 1,
    /// `CubicBezier`.
    CubicBezier = 2,
}

/// `Font_GlyphOutline::Segment`: start, two controls and end (unused controls repeat the
/// neighbouring point). Example: `FontGlyphOutlineSegment::line(a, b)`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct FontGlyphOutlineSegment {
    kind: FontGlyphOutlineSegmentKind,
    start: Xy,
    control1: Xy,
    control2: Xy,
    end: Xy,
}

impl FontGlyphOutlineSegment {
    /// `Line`; `FontGlyphOutlineSegment::line(a, b)`.
    pub fn line(start: Xy, end: Xy) -> Self {
        Self {
            kind: FontGlyphOutlineSegmentKind::Line,
            start,
            control1: start,
            control2: end,
            end,
        }
    }
    /// `Quadratic`; `FontGlyphOutlineSegment::quadratic(a, c, b)`.
    pub fn quadratic(start: Xy, control: Xy, end: Xy) -> Self {
        let kind = FontGlyphOutlineSegmentKind::QuadraticBezier;
        Self {
            kind,
            start,
            control1: control,
            control2: control,
            end,
        }
    }
    /// `Cubic`; `FontGlyphOutlineSegment::cubic(a, c1, c2, b)`.
    pub fn cubic(start: Xy, control1: Xy, control2: Xy, end: Xy) -> Self {
        Self {
            kind: FontGlyphOutlineSegmentKind::CubicBezier,
            start,
            control1,
            control2,
            end,
        }
    }
    /// `Type`; `segment.kind()`.
    pub fn kind(&self) -> FontGlyphOutlineSegmentKind {
        self.kind
    }
    /// `Start`; `segment.start()`.
    pub fn start(&self) -> Xy {
        self.start
    }
    /// `Control1`; `segment.control1()`.
    pub fn control1(&self) -> Xy {
        self.control1
    }
    /// `Control2`; `segment.control2()`.
    pub fn control2(&self) -> Xy {
        self.control2
    }
    /// `End`; `segment.end()`.
    pub fn end(&self) -> Xy {
        self.end
    }
}

/// `Font_GlyphOutline::Contour`: a run of segments; `FontGlyphOutlineContour::new(0, 4)`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FontGlyphOutlineContour {
    first_segment: u32,
    nb_segments: u32,
}

impl FontGlyphOutlineContour {
    /// A contour of `count` segments from `first`; `FontGlyphOutlineContour::new(0, 4)`.
    pub fn new(first: u32, count: u32) -> Self {
        Self {
            first_segment: first,
            nb_segments: count,
        }
    }
    /// The index of the first segment; `contour.first_segment()`.
    pub fn first_segment(&self) -> u32 {
        self.first_segment
    }
    /// The number of segments; `contour.nb_segments()`.
    pub fn nb_segments(&self) -> u32 {
        self.nb_segments
    }
}

/// A glyph outline; `FontGlyphOutline::create(2048.0, FontGlyphOutlineFillRule::NonZero, false, false, segments, contours)`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct FontGlyphOutline {
    pub(crate) units_per_em: f64,
    pub(crate) fill_rule: FontGlyphOutlineFillRule,
    pub(crate) has_possible_overlaps: bool,
    pub(crate) is_single_stroke: bool,
    pub(crate) segments: Vec<FontGlyphOutlineSegment>,
    pub(crate) contours: Vec<FontGlyphOutlineContour>,
}

impl FontGlyphOutline {
    /// `Create`: an outline from its parts (the vectors are moved in).
    /// Example: `FontGlyphOutline::create(1000.0, rule, false, false, segments, contours)`.
    pub fn create(
        units_per_em: f64,
        fill_rule: FontGlyphOutlineFillRule,
        has_possible_overlaps: bool,
        is_single_stroke: bool,
        segments: Vec<FontGlyphOutlineSegment>,
        contours: Vec<FontGlyphOutlineContour>,
    ) -> Self {
        Self {
            units_per_em,
            fill_rule,
            has_possible_overlaps,
            is_single_stroke,
            segments,
            contours,
        }
    }
    /// The font units per EM; `outline.units_per_em()`.
    pub fn units_per_em(&self) -> f64 {
        self.units_per_em
    }
    /// The fill rule; `outline.rule()`.
    pub fn rule(&self) -> FontGlyphOutlineFillRule {
        self.fill_rule
    }
    /// Whether contours may overlap; `outline.has_possible_overlaps()`.
    pub fn has_possible_overlaps(&self) -> bool {
        self.has_possible_overlaps
    }
    /// Whether the outline is single-stroke (open contours); `outline.is_single_stroke()`.
    pub fn is_single_stroke(&self) -> bool {
        self.is_single_stroke
    }
    /// Whether there is no segment; `outline.is_empty()`.
    pub fn is_empty(&self) -> bool {
        self.segments.is_empty()
    }
    /// The segments; `outline.segments()`.
    pub fn segments(&self) -> &[FontGlyphOutlineSegment] {
        &self.segments
    }
    /// The contours; `outline.contours()`.
    pub fn contours(&self) -> &[FontGlyphOutlineContour] {
        &self.contours
    }
}
