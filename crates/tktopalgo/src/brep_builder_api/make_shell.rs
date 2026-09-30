// Rust translation of BRepBuilderAPI_MakeShell.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepBuilderAPI_MakeShell`: the shell of the C2 patches of a surface, built by the
//! wrapped `BRepLib_MakeShell`.
use super::errors::BRepBuilderAPIShellError;
use super::make_shape::{BRepBuilderAPIMakeShape, BRepBuilderAPIMakeShapeBase};
use super::wrap::{self, wrapped_constructors, wrapped_updates};
use crate::brep_lib::BRepLibMakeShell;
use rustycad_tkbrep::topods::Shell;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::GeomSurface;

/// `BRepBuilderAPI_MakeShell` (docs/topalgo-design.md section 10): each constructor and
/// Init runs the `BRepLib_MakeShell` member of the same Rust name; `IsDone` and `Error`
/// are those of the BRepLib maker. The parameter boxes are `(UMin, UMax, VMin, VMax)`.
///
/// ```
/// use rustycad_tkernel::standard::Handle;
/// use rustycad_tkg3d::geom::{GeomPlane, GeomSurface};
/// use rustycad_tkmath::gp::Axis3;
/// use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakeShape, BRepBuilderAPIMakeShell};
/// let plane: Handle<dyn GeomSurface> = Handle::new(GeomPlane::new(Axis3::default())).cast();
/// let maker = BRepBuilderAPIMakeShell::with_bounds(&plane, (0.0, 1.0, 0.0, 1.0), false)?;
/// assert!(maker.is_done());
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
#[derive(Clone)]
pub struct BRepBuilderAPIMakeShell {
    base: BRepBuilderAPIMakeShapeBase,
    my_make_shell: BRepLibMakeShell,
}

impl Default for BRepBuilderAPIMakeShell {
    fn default() -> Self {
        Self::new()
    }
}

impl BRepBuilderAPIMakeShape for BRepBuilderAPIMakeShell {
    fn make_shape_base(&self) -> &BRepBuilderAPIMakeShapeBase {
        &self.base
    }

    fn make_shape_base_mut(&mut self) -> &mut BRepBuilderAPIMakeShapeBase {
        &mut self.base
    }

    /// `IsDone()`: the flag of the BRepLib maker; `maker.is_done()`.
    fn is_done(&self) -> bool {
        self.my_make_shell.is_done()
    }
}

impl BRepBuilderAPIMakeShell {
    /// `BRepBuilderAPI_MakeShell()`: no shell, `BRepBuilderAPI_EmptyShell`;
    /// `BRepBuilderAPIMakeShell::new()`.
    pub fn new() -> Self {
        Self {
            base: BRepBuilderAPIMakeShapeBase::new(),
            my_make_shell: BRepLibMakeShell::new(),
        }
    }

    /// Wraps a constructed BRepLib maker and takes its shape when it is done.
    fn wrap(my_make_shell: BRepLibMakeShell) -> Result<Self, Failure> {
        let mut maker = Self {
            base: BRepBuilderAPIMakeShapeBase::new(),
            my_make_shell,
        };
        maker.adopt()?;
        Ok(maker)
    }

    /// The step after each BRepLib call: done and the shape when the BRepLib maker is.
    fn adopt(&mut self) -> Result<(), Failure> {
        wrap::adopt(&mut self.base, &self.my_make_shell)
    }

    wrapped_constructors!(BRepBuilderAPIMakeShell, BRepLibMakeShell;
        with_surface(surface: &Handle<dyn GeomSurface>, segment: bool)
            = "BRepBuilderAPI_MakeShell(S, Segment)";
        with_bounds(surface: &Handle<dyn GeomSurface>, bounds: (f64, f64, f64, f64), segment: bool)
            = "BRepBuilderAPI_MakeShell(S, UMin, UMax, VMin, VMax, Segment)";
    );

    wrapped_updates!(BRepBuilderAPIMakeShell, my_make_shell;
        init(surface: &Handle<dyn GeomSurface>, bounds: (f64, f64, f64, f64), segment: bool)
            = "Init(S, UMin, UMax, VMin, VMax, Segment)";
    );

    /// `Error()`: the status of the BRepLib maker, through the native switch;
    /// `maker.error()`.
    pub fn error(&self) -> BRepBuilderAPIShellError {
        BRepBuilderAPIShellError::from(self.my_make_shell.error())
    }

    /// `Shell()` and `operator TopoDS_Shell()`: the shell of the BRepLib maker (not
    /// checked, as natively); `maker.shell()?`.
    pub fn shell(&self) -> Result<Shell, Failure> {
        self.my_make_shell.shell()
    }
}
