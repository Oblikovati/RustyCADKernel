// Rust translation of BRepLib_MakeShell.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepLib_MakeShell`: the shell of the C2 patches of a surface.
mod grid;

use super::errors::BRepLibShellError;
use super::make_shape::BRepLibMakeShape;
use rustycad_tkbrep::topods::{self, Shell};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::GeomSurface;
use std::ops::{Deref, DerefMut};

/// `BRepLib_MakeShell` (docs/topalgo-design.md section 7): a shell of one face per C2
/// patch of a surface (each face on a copy of the surface, segmented to its patch for a
/// B-spline when asked), with shared edges and vertices, the 3d curves built, the
/// regularities encoded and the degenerated edges flagged.
///
/// ```
/// use rustycad_tkmath::gp::Axis3;
/// use rustycad_tkernel::standard::Handle;
/// use rustycad_tkg3d::geom::{GeomPlane, GeomSurface};
/// use rustycad_tktopalgo::brep_lib::{BRepLibMakeShell, BRepLibShellError};
/// let plane: Handle<dyn GeomSurface> = Handle::new(GeomPlane::new(Axis3::default())).cast();
/// let maker = BRepLibMakeShell::with_bounds(&plane, (0.0, 1.0, 0.0, 1.0), false)?;
/// assert_eq!(maker.error(), BRepLibShellError::ShellDone);
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
#[derive(Clone)]
pub struct BRepLibMakeShell {
    base: BRepLibMakeShape,
    my_error: BRepLibShellError,
}

impl Default for BRepLibMakeShell {
    fn default() -> Self {
        Self::new()
    }
}

impl Deref for BRepLibMakeShell {
    type Target = BRepLibMakeShape;
    fn deref(&self) -> &BRepLibMakeShape {
        &self.base
    }
}

impl DerefMut for BRepLibMakeShell {
    fn deref_mut(&mut self) -> &mut BRepLibMakeShape {
        &mut self.base
    }
}

impl BRepLibMakeShell {
    /// `BRepLib_MakeShell()`: not done, `BRepLib_EmptyShell`; `BRepLibMakeShell::new()`.
    pub fn new() -> Self {
        Self {
            base: BRepLibMakeShape::new(),
            my_error: BRepLibShellError::EmptyShell,
        }
    }

    /// `BRepLib_MakeShell(S, Segment)`: the whole surface; Standard_NullObject for a null
    /// surface; `BRepLibMakeShell::with_surface(&s, false)?`.
    pub fn with_surface(surface: &Handle<dyn GeomSurface>, segment: bool) -> Result<Self, Failure> {
        let s = surface
            .get()
            .ok_or_else(|| super::errors::null_object("BRepLib_MakeShell: null surface"))?;
        let (umin, umax, vmin, vmax) = s.bounds();
        Self::with_bounds(surface, (umin, umax, vmin, vmax), segment)
    }

    /// `BRepLib_MakeShell(S, UMin, UMax, VMin, VMax, Segment)`;
    /// `BRepLibMakeShell::with_bounds(&s, (0.0, 1.0, 0.0, 1.0), false)?`.
    pub fn with_bounds(
        surface: &Handle<dyn GeomSurface>,
        bounds: (f64, f64, f64, f64),
        segment: bool,
    ) -> Result<Self, Failure> {
        let mut maker = Self::new();
        maker.init(surface, bounds, segment)?;
        Ok(maker)
    }

    /// `Init(S, UMin, UMax, VMin, VMax, Segment)`: the shell of the C2 patches of the
    /// surface (of the basis of a rectangular trimmed surface) over the bounds;
    /// `BRepLib_EmptyShell` and not done when a direction has no C2 interval;
    /// `maker.init(&s, (0.0, 1.0, 0.0, 1.0), false)?`.
    pub fn init(
        &mut self,
        surface: &Handle<dyn GeomSurface>,
        bounds: (f64, f64, f64, f64),
        segment: bool,
    ) -> Result<(), Failure> {
        self.my_error = BRepLibShellError::EmptyShell;
        if !grid::shell_of_patches(&mut self.base.my_shape, surface, bounds, segment)? {
            return Ok(());
        }
        grid::finish_shell(&self.base.my_shape)?;
        self.my_error = BRepLibShellError::ShellDone;
        self.set_done();
        Ok(())
    }

    /// `Error()`; `maker.error()`.
    pub fn error(&self) -> BRepLibShellError {
        self.my_error
    }

    /// `Shell()` and `operator TopoDS_Shell()`: the shell under construction (not
    /// checked, as natively); `maker.shell()?`.
    pub fn shell(&self) -> Result<Shell, Failure> {
        topods::shell(&self.base.my_shape)
    }
}
