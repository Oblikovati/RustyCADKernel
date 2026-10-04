//! Stateful BeanFace facade; analytic stages precede the remaining general solver port.
use super::{Context, Range};
use rustycad_tkbrep::{
    brep::BRepTool,
    brep_adaptor::{BRepAdaptorCurve, BRepAdaptorSurface},
    topods::{Edge, Face},
};
use rustycad_tkernel::{
    ncollection::Sequence,
    precision,
    standard::{Failure, Handle},
};
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkmath::geom_abs::{CurveType, SurfaceType};
mod line_plane;

/// Curve/face intersection ranges; `BeanFaceIntersector::from_shapes(&edge, &face)?`.
pub struct BeanFaceIntersector {
    curve: Handle<BRepAdaptorCurve>,
    surface: Handle<BRepAdaptorSurface>,
    context: Handle<Context>,
    range: Range,
    bounds: [f64; 4],
    tolerances: [f64; 2],
    criteria: f64,
    curve_resolution: f64,
    done: bool,
    minimum_square_distance: f64,
    results: Sequence<Range>,
}

impl Default for BeanFaceIntersector {
    fn default() -> Self {
        Self {
            curve: Handle::new(BRepAdaptorCurve::default()),
            surface: Handle::new(BRepAdaptorSurface::default()),
            context: Handle::default(),
            range: Range::new(0., 0.),
            bounds: [0.; 4],
            tolerances: [0.; 2],
            criteria: precision::confusion(),
            curve_resolution: precision::p_confusion(),
            done: false,
            minimum_square_distance: f64::MAX,
            results: Sequence::default(),
        }
    }
}

impl BeanFaceIntersector {
    /// Creates native empty state; `BeanFaceIntersector::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Initializes geometry without assigning the curve range; `BeanFaceIntersector::from_shapes(&edge,&face)?`.
    pub fn from_shapes(edge: &Edge, face: &Face) -> Result<Self, Failure> {
        let mut solver = Self::new();
        solver.init_shapes(edge, face)?;
        Ok(solver)
    }

    /// Copies adaptors and tolerances; `BeanFaceIntersector::from_adaptors(&curve,&surface,1e-7,1e-7)?`.
    pub fn from_adaptors(
        curve: &BRepAdaptorCurve,
        surface: &BRepAdaptorSurface,
        bean_tol: f64,
        face_tol: f64,
    ) -> Result<Self, Failure> {
        let mut solver = Self::new();
        solver.init_adaptors(curve, surface, bean_tol, face_tol)?;
        Ok(solver)
    }

    /// Reinitializes topology while retaining range and completion state; `solver.init_shapes(&edge,&face)?`.
    pub fn init_shapes(&mut self, edge: &Edge, face: &Face) -> Result<(), Failure> {
        self.ensure_context();
        self.curve = Handle::new(BRepAdaptorCurve::with_edge(edge)?);
        let cached = self
            .context
            .get()
            .expect("initialized bean context")
            .surface_adaptor(face)?;
        self.surface = cached.read().shallow_copy()?.cast();
        self.tolerances = [
            BRepTool::edge_tolerance(edge)?,
            BRepTool::face_tolerance(face)?,
        ];
        self.criteria = self.tolerances[0] + self.tolerances[1] + precision::confusion();
        self.finish_initialization()
    }

    /// Copies geometry and clears ranges without resetting IsDone; `solver.init_adaptors(&curve,&surface,1e-7,1e-7)?`.
    pub fn init_adaptors(
        &mut self,
        curve: &BRepAdaptorCurve,
        surface: &BRepAdaptorSurface,
        bean_tol: f64,
        face_tol: f64,
    ) -> Result<(), Failure> {
        self.curve = curve.shallow_copy()?.cast();
        self.surface = surface.shallow_copy()?.cast();
        self.tolerances = [bean_tol, face_tol];
        self.criteria = bean_tol + face_tol;
        self.finish_initialization()
    }

    fn finish_initialization(&mut self) -> Result<(), Failure> {
        self.curve_resolution = self
            .curve
            .get()
            .expect("bean curve")
            .resolution(self.criteria)?;
        let surface = self.surface.get().expect("bean surface");
        self.bounds = [
            surface.first_u_parameter()?,
            surface.last_u_parameter()?,
            surface.first_v_parameter()?,
            surface.last_v_parameter()?,
        ];
        self.results.clear(None);
        Ok(())
    }

    /// Replaces the shared context, including a null handle; `solver.set_context(&context)`.
    pub fn set_context(&mut self, context: &Handle<Context>) {
        self.context = context.clone();
    }
    /// Borrows the context handle; `solver.context().is_null()`.
    pub fn context(&self) -> &Handle<Context> {
        &self.context
    }
    /// Assigns the explicit curve interval; `solver.set_bean_parameters(0.,1.)`.
    pub fn set_bean_parameters(&mut self, first: f64, last: f64) {
        self.range = Range::new(first, last);
    }
    /// Assigns UV boundaries; `solver.set_surface_parameters(-1.,1.,-1.,1.)`.
    pub fn set_surface_parameters(&mut self, umin: f64, umax: f64, vmin: f64, vmax: f64) {
        self.bounds = [umin, umax, vmin, vmax];
    }
    /// Reads completion state; `solver.is_done()`.
    pub fn is_done(&self) -> bool {
        self.done
    }
    /// Borrows all retained result ranges; `solver.result().length()`.
    pub fn result(&self) -> &Sequence<Range> {
        &self.results
    }
    /// Replaces the destination sequence by copy; `solver.result_into(&mut ranges)`.
    pub fn result_into(&self, ranges: &mut Sequence<Range>) {
        ranges.assign(&self.results);
    }
    /// Reads the native distance sentinel or computed value; `solver.minimal_square_distance()`.
    pub fn minimal_square_distance(&self) -> f64 {
        self.minimum_square_distance
    }

    /// Resets results and runs the available analytic stage; `solver.perform()?`.
    pub fn perform(&mut self) -> Result<(), Failure> {
        self.done = false;
        self.results.clear(None);
        self.ensure_context();
        let curve_type = self.curve.get().expect("bean curve").get_type()?;
        let surface_type = self.surface.get().expect("bean surface").get_type()?;
        if curve_type == CurveType::Line && surface_type == SurfaceType::Plane {
            return self.compute_line_plane();
        }
        Err(Failure::new(&format!(
            "BeanFaceIntersector::Perform unsupported pair {curve_type:?}/{surface_type:?}; expected ported Line/Plane stage"
        )))
    }

    /// Appends a line/plane result without clearing earlier ranges; `solver.compute_line_plane()?`.
    pub fn compute_line_plane(&mut self) -> Result<(), Failure> {
        let line = self.curve.get().expect("bean curve").line()?;
        let plane = self.surface.get().expect("bean surface").plane()?;
        self.done = true;
        let result = if self.criteria == self.tolerances[0] + self.tolerances[1] {
            line_plane::intersect_line_plane(line, plane, self.range, self.bounds, self.tolerances)?
        } else {
            line_plane::intersect_with_criteria(
                line,
                plane,
                self.range,
                self.bounds,
                self.tolerances,
                self.criteria,
            )?
        };
        if let Some(range) = result {
            self.results.append(range);
        }
        Ok(())
    }

    fn ensure_context(&mut self) {
        if self.context.is_null() {
            self.context = Handle::new(Context::default());
        }
    }
}
