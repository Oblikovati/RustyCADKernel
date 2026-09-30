// Rust translation of BRepTopAdaptor_Tool.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepTopAdaptor_Tool`: a surface adaptor and the topology tool initialized on it.
use super::topol_tool::BRepTopAdaptorTopolTool;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorSurface;
use rustycad_tkbrep::topods::Face;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;

/// `BRepTopAdaptor_Tool` (docs/topalgo-design.md section 4): a
/// `BRepTopAdaptor_TopolTool` handle and the surface it was initialized on. The 2d
/// tolerance arguments are unused natively.
///
/// ```
/// use rustycad_tktopalgo::brep_top_adaptor::BRepTopAdaptorTool;
/// let tool = BRepTopAdaptorTool::new();
/// assert!(tool.get_surface().is_null());
/// assert!(!tool.get_topol_tool().is_null());
/// ```
pub struct BRepTopAdaptorTool {
    myloaded: bool,
    my_topol_tool: Handle<BRepTopAdaptorTopolTool>,
    my_h_surface: Handle<dyn Adaptor3dSurface>,
}

impl Default for BRepTopAdaptorTool {
    fn default() -> Self {
        Self::new()
    }
}

impl BRepTopAdaptorTool {
    /// `BRepTopAdaptor_Tool()`: a new, uninitialized topology tool and no surface;
    /// `BRepTopAdaptorTool::new()`.
    pub fn new() -> Self {
        Self {
            myloaded: false,
            my_topol_tool: Handle::new(BRepTopAdaptorTopolTool::new()),
            my_h_surface: Handle::default(),
        }
    }

    /// `BRepTopAdaptor_Tool(F, Tol2d)`: the tool on the face restricted to its UV
    /// bounds; `BRepTopAdaptorTool::with_face(&face, 1e-9)?`.
    pub fn with_face(face: &Face, tol_2d: f64) -> Result<Self, Failure> {
        let mut tool = Self::new();
        tool.init_face(face, tol_2d)?;
        Ok(tool)
    }

    /// `BRepTopAdaptor_Tool(Surface, Tol2d)`: the tool on `surface`, which must be a
    /// BRepAdaptor_Surface (Standard_ConstructionError otherwise);
    /// `BRepTopAdaptorTool::with_surface(&surface, 1e-9)?`.
    pub fn with_surface(
        surface: &Handle<dyn Adaptor3dSurface>,
        tol_2d: f64,
    ) -> Result<Self, Failure> {
        let mut tool = Self::new();
        tool.init_surface(surface, tol_2d)?;
        Ok(tool)
    }

    /// `Init(F, Tol2d)`: reinitializes the topology tool on a new restricted adaptor
    /// of the face; `tool.init_face(&face, 1e-9)?`.
    pub fn init_face(&mut self, face: &Face, tol_2d: f64) -> Result<(), Failure> {
        let surface = BRepAdaptorSurface::with_face(face, true)?;
        let surface: Handle<dyn Adaptor3dSurface> = Handle::new(surface).cast();
        self.init_surface(&surface, tol_2d)
    }

    /// `Init(Surface, Tol2d)`: reinitializes the topology tool on `surface`;
    /// `tool.init_surface(&surface, 1e-9)?`.
    pub fn init_surface(
        &mut self,
        surface: &Handle<dyn Adaptor3dSurface>,
        _tol_2d: f64,
    ) -> Result<(), Failure> {
        let tool = self.my_topol_tool.get().ok_or_else(|| {
            super::errors::null_object(
                "BRepTopAdaptor_Tool::Init: null topology tool; expected SetTopolTool with a tool",
            )
        })?;
        tool.initialize_surface(surface)?;
        self.my_h_surface = surface.clone();
        self.myloaded = true;
        Ok(())
    }

    /// `GetTopolTool()`: the topology tool, loaded or not; `tool.get_topol_tool()`.
    pub fn get_topol_tool(&self) -> Handle<BRepTopAdaptorTopolTool> {
        self.my_topol_tool.clone()
    }

    /// `SetTopolTool(TT)`; `tool.set_topol_tool(&topol_tool)`.
    pub fn set_topol_tool(&mut self, topol_tool: &Handle<BRepTopAdaptorTopolTool>) {
        self.my_topol_tool = topol_tool.clone();
    }

    /// `GetSurface()`: the surface, loaded or not (null before an Init);
    /// `tool.get_surface()`.
    pub fn get_surface(&self) -> Handle<dyn Adaptor3dSurface> {
        self.my_h_surface.clone()
    }

    /// `Destroy()`: does nothing; `tool.destroy()`.
    pub fn destroy(&mut self) {}

    /// Whether an Init ran (`myloaded`; GetTopolTool and GetSurface only print a debug
    /// message natively when not); `tool.is_loaded()`.
    pub fn is_loaded(&self) -> bool {
        self.myloaded
    }
}
