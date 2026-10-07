//! The drawing state that `Draw_Viewer.cxx` keeps in file-scope statics and that every
//! `Draw_Display` shares: the current view, the segment buffer, colors, bounds, the
//! picking query and the PostScript transformation.
use crate::draw::color::{DrawColorKind, MAXCOLOR};
use crate::draw::ostream::DrawOStream;
use rustycad_tkmath::gp::{Point2, Point3, Transform3};

/// `MAXSEGMENT`: segments buffered before they are sent to the window.
pub const MAXSEGMENT: usize = 1000;

/// `DRAWINFINITE`: the initial (empty) drawing bounds.
pub const DRAWINFINITE: f64 = 1e50;

/// `Draw_XSegment`: one window segment in 16-bit pixel coordinates (as `XSegment`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DrawXSegment {
    /// Start and end points, `[x, y]` each.
    pub points: [[i16; 2]; 2],
}

impl DrawXSegment {
    /// `Init`: the coordinates are cast to `short`, wrapping as the C++ cast does.
    ///
    /// ```
    /// use rustycad_tkdraw::draw::DrawXSegment;
    /// assert_eq!(DrawXSegment::new(1, 2, 70000, 4).points, [[1, 2], [4464, 4]]);
    /// ```
    pub fn new(x_start: i32, y_start: i32, x_end: i32, y_end: i32) -> Self {
        Self {
            points: [
                [x_start as i16, y_start as i16],
                [x_end as i16, y_end as i16],
            ],
        }
    }

    /// `operator[](i) const`: point 0 (start) or 1 (end); `None` for another index
    /// (the C++ reads outside the array).
    ///
    /// ```
    /// use rustycad_tkdraw::draw::DrawXSegment;
    /// assert_eq!(DrawXSegment::new(1, 2, 3, 4).point(1), Some([3, 4]));
    /// ```
    pub fn point(&self, index: usize) -> Option<[i16; 2]> {
        self.points.get(index).copied()
    }

    /// `operator[](i)`: the point for modification.
    pub fn point_mut(&mut self, index: usize) -> Option<&mut [i16; 2]> {
        self.points.get_mut(index)
    }
}

/// What a `Draw_Display` draws on: the geometry of the current `Draw_View` and its
/// window. `Draw_View` implements it; tests use a recording fake.
pub trait DisplaySurface {
    /// `GetZoom`.
    fn zoom(&self) -> f64;
    /// `GetMatrix`: the view transformation.
    fn matrix(&self) -> Transform3;
    /// `IsPerspective`.
    fn is_perspective(&self) -> bool;
    /// `GetFocalDistance`.
    fn focal_distance(&self) -> f64;
    /// `GetDx`: horizontal offset of the view origin in pixels.
    fn dx(&self) -> i32;
    /// `GetDy`: vertical offset of the view origin in pixels.
    fn dy(&self) -> i32;
    /// `GetFrame`: the visible frame `(x0, y0, x1, y1)` in view pixels (cached).
    fn frame(&mut self) -> (i32, i32, i32, i32);
    /// `Draw_Viewer::GetFrame`: the frame computed from the window size and pan.
    fn viewer_frame(&self) -> (i32, i32, i32, i32);
    /// `Draw_Viewer::GetType`: the view type (`+X+Y`, `-2D-`, `AXON`, ...).
    fn view_type(&self) -> String;
    /// `Draw_Window::SetColor`.
    fn set_color(&mut self, color: DrawColorKind);
    /// `Draw_Window::SetMode` (3 = copy, 6 = xor).
    fn set_mode(&mut self, mode: i32);
    /// `Draw_Window::DrawSegments`.
    fn draw_segments(&mut self, segments: &[DrawXSegment]);
    /// `Draw_Window::DrawString`.
    fn draw_string(&mut self, x: i32, y: i32, text: &str);
}

/// `DrawingMode`: what the display operations do.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DrawingMode {
    /// Draw into the window.
    #[default]
    Draw,
    /// Hit-test against the pick point.
    Pick,
    /// Write PostScript.
    PostScript,
}

/// The bounds of what was drawn, in zoomed view coordinates.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DrawBounds {
    /// `xmin`.
    pub x_min: f64,
    /// `xmax`.
    pub x_max: f64,
    /// `ymin`.
    pub y_min: f64,
    /// `ymax`.
    pub y_max: f64,
}

impl DrawBounds {
    /// Empty bounds (`xmin = ymin = DRAWINFINITE`, `xmax = ymax = -DRAWINFINITE`).
    pub fn empty() -> Self {
        Self {
            x_min: DRAWINFINITE,
            x_max: -DRAWINFINITE,
            y_min: DRAWINFINITE,
            y_max: -DRAWINFINITE,
        }
    }

    /// Grows the bounds to `(x, y)` (the repeated `if (x > xmax) xmax = x` blocks).
    pub fn include(&mut self, x: f64, y: f64) {
        if x > self.x_max {
            self.x_max = x;
        }
        if x < self.x_min {
            self.x_min = x;
        }
        if y > self.y_max {
            self.y_max = y;
        }
        if y < self.y_min {
            self.y_min = y;
        }
    }
}

/// The picking query and its result (`xpick`, `ypick`, `precpick`, `found`,
/// `lastPickP1`, `lastPickP2`, `lastPickParam`).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct PickState {
    /// Pick point x in view pixels.
    pub x: i32,
    /// Pick point y in view pixels.
    pub y: i32,
    /// Pick tolerance in pixels.
    pub precision: i32,
    /// True once a segment passed near the pick point.
    pub found: bool,
    /// Start of the last picked 3D segment.
    pub last_p1: Point3,
    /// End of the last picked 3D segment.
    pub last_p2: Point3,
    /// Parameter of the pick point on the picked segment.
    pub last_param: f64,
}

/// The PostScript mapping and output (`ps_vx`, `ps_kx`, `ps_px`, ..., `ps_width`,
/// `ps_gray`, `ps_stream`).
#[derive(Debug, Clone, PartialEq)]
pub struct PostScriptState {
    /// View origin x.
    pub vx: i32,
    /// View origin y.
    pub vy: i32,
    /// Horizontal scale.
    pub kx: f64,
    /// Vertical scale.
    pub ky: f64,
    /// Paper origin x.
    pub px: i32,
    /// Paper origin y.
    pub py: i32,
    /// Line width per palette color (`PostColor`).
    pub widths: [i32; MAXCOLOR],
    /// Gray level per palette color.
    pub grays: [f64; MAXCOLOR],
    /// The PostScript text being written.
    pub stream: DrawOStream,
}

impl Default for PostScriptState {
    fn default() -> Self {
        Self {
            vx: 0,
            vy: 0,
            kx: 0.0,
            ky: 0.0,
            px: 0,
            py: 0,
            widths: [1; MAXCOLOR],
            grays: [0.0; MAXCOLOR],
            stream: DrawOStream::new(),
        }
    }
}

/// The shared drawing state of the viewer and its displays.
#[derive(Debug, Clone, PartialEq)]
pub struct DisplayState {
    /// `CurrentMode`.
    pub mode: DrawingMode,
    /// `curviewId`.
    pub view_id: i32,
    /// `segm`/`nbseg`: buffered segments.
    pub segments: Vec<DrawXSegment>,
    /// `currentcolor`.
    pub current_color: DrawColorKind,
    /// `highlight`/`highlightcol`: the color that overrides drawing while highlighting.
    pub highlight: Option<DrawColorKind>,
    /// `xmin`/`xmax`/`ymin`/`ymax`.
    pub bounds: DrawBounds,
    /// `Draw_Bounds`: whether drawing grows the bounds.
    pub draw_bounds: bool,
    /// `PtCur`: the current 2D point, zoomed.
    pub current_point: Point2,
    /// `PtPers`: the current 3D point in view space.
    pub perspective_point: Point3,
    /// The picking query.
    pub pick: PickState,
    /// The PostScript output.
    pub postscript: PostScriptState,
}

impl Default for DisplayState {
    fn default() -> Self {
        Self {
            mode: DrawingMode::Draw,
            view_id: 0,
            segments: Vec::with_capacity(MAXSEGMENT),
            current_color: DrawColorKind::Blanc,
            highlight: None,
            bounds: DrawBounds {
                x_min: 0.0,
                x_max: 0.0,
                y_min: 0.0,
                y_max: 0.0,
            },
            draw_bounds: true,
            current_point: Point2::new(0.0, 0.0),
            perspective_point: Point3::new(0.0, 0.0, 0.0),
            pick: PickState::default(),
            postscript: PostScriptState::default(),
        }
    }
}

impl DisplayState {
    /// `Draw_Flush`: sends the buffered segments (in the highlight color, if any).
    pub fn flush_segments(&mut self, surface: &mut dyn DisplaySurface) {
        if let Some(color) = self.highlight {
            surface.set_color(color);
        }
        surface.draw_segments(&self.segments);
        self.segments.clear();
    }
}
