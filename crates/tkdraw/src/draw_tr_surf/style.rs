// Rust translation of the display members that DrawTrSurf_Curve.hxx and
// DrawTrSurf_Curve2d.hxx both declare, at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The look of a drawable curve: color, orientation arrow and curvature radii.
use crate::draw::DrawColor;
use std::cell::Cell;

/// The display options of a curve beyond the discretisation: the optional arguments
/// of the `DrawTrSurf_Curve` and `DrawTrSurf_Curve2d` constructors.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CurveLook {
    /// `aColor`.
    pub color: DrawColor,
    /// `DispOrigin`: the orientation arrow at the end.
    pub display_origin: bool,
    /// `DispCurvRadius`: the curvature radii.
    pub display_curvature: bool,
    /// `RadiusMax`.
    pub radius_max: f64,
    /// `RatioOfRadius`.
    pub radius_ratio: f64,
}

impl CurveLook {
    /// `color` with the C++ defaults of the other options (origin shown, no radii,
    /// radii capped at 1000, drawn at 10%).
    ///
    /// ```
    /// use rustycad_tkdraw::draw::{DrawColor, DrawColorKind};
    /// use rustycad_tkdraw::draw_tr_surf::CurveLook;
    /// let look = CurveLook::new(DrawColor::new(DrawColorKind::Rouge));
    /// assert!(look.display_origin && !look.display_curvature);
    /// ```
    pub fn new(color: DrawColor) -> Self {
        Self {
            color,
            display_origin: true,
            display_curvature: false,
            radius_max: 1.0e3,
            radius_ratio: 0.1,
        }
    }
}

/// The mutable look of a drawable curve (the `look`, `disporigin`, `dispcurvradius`,
/// `radiusmax` and `radiusratio` members), shared by the 3D and 2D curves.
///
/// ```
/// use rustycad_tkdraw::draw::{DrawColor, DrawColorKind};
/// use rustycad_tkdraw::draw_tr_surf::{CurveLook, CurveStyle};
/// let style = CurveStyle::new(CurveLook::new(DrawColor::new(DrawColorKind::Vert)));
/// style.show_curvature();
/// assert!(style.curvature_shown());
/// ```
#[derive(Debug)]
pub struct CurveStyle {
    color: Cell<DrawColor>,
    display_origin: Cell<bool>,
    display_curvature: Cell<bool>,
    radius_max: Cell<f64>,
    radius_ratio: Cell<f64>,
}

impl CurveStyle {
    /// The style of a new curve.
    pub fn new(look: CurveLook) -> Self {
        Self {
            color: Cell::new(look.color),
            display_origin: Cell::new(look.display_origin),
            display_curvature: Cell::new(look.display_curvature),
            radius_max: Cell::new(look.radius_max),
            radius_ratio: Cell::new(look.radius_ratio),
        }
    }

    /// `SetColor`.
    pub fn set_color(&self, color: DrawColor) {
        self.color.set(color);
    }

    /// `Color`.
    pub fn color(&self) -> DrawColor {
        self.color.get()
    }

    /// `DisplayOrigin()`.
    pub fn display_origin(&self) -> bool {
        self.display_origin.get()
    }

    /// `DisplayOrigin(bool)`.
    pub fn set_display_origin(&self, display: bool) {
        self.display_origin.set(display);
    }

    /// `ShowCurvature`.
    pub fn show_curvature(&self) {
        self.display_curvature.set(true);
    }

    /// `ClearCurvature`.
    pub fn clear_curvature(&self) {
        self.display_curvature.set(false);
    }

    /// Whether the curvature radii are drawn (`dispcurvradius`).
    pub fn curvature_shown(&self) -> bool {
        self.display_curvature.get()
    }

    /// `SetRadiusMax`.
    pub fn set_radius_max(&self, radius: f64) {
        self.radius_max.set(radius);
    }

    /// `RadiusMax`.
    pub fn radius_max(&self) -> f64 {
        self.radius_max.get()
    }

    /// `SetRadiusRatio`.
    pub fn set_radius_ratio(&self, ratio: f64) {
        self.radius_ratio.set(ratio);
    }

    /// `RadiusRatio`.
    pub fn radius_ratio(&self) -> f64 {
        self.radius_ratio.get()
    }
}
