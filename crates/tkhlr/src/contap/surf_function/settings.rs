use super::super::ContapTFunction;
use super::ContapSurfFunction;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::h_surface_tool::Adaptor3dSurfaceHandle;
use rustycad_tkmath::gp::{Direction3, Point3};
impl ContapSurfFunction {
    /// Set(S): the surface and the mean normal length over its sample points (the
    /// process-wide sampling of `Contap_HContTool`); `f.set_surface(&s)?`.
    pub fn set_surface(&mut self, s: &Adaptor3dSurfaceHandle) -> Result<(), Failure> {
        self.my_surf = s.clone();
        super::super::normal_sampling::sample_normal_scale(s, &mut self.my_mean, &mut self.solpt)?;
        self.computed = false;
        self.derived = false;
        Ok(())
    }

    /// Set(Eye): the silhouette from `eye`; `f.set_eye(p)`.
    pub fn set_eye(&mut self, eye: Point3) {
        self.my_type = ContapTFunction::ContourPrs;
        self.my_eye = eye;
        self.my_ang = 0.;
    }

    /// Set(Dir): the silhouette along `dir`; `f.set_direction(d)`.
    pub fn set_direction(&mut self, dir: Direction3) {
        self.my_type = ContapTFunction::ContourStd;
        self.my_dir = dir;
        self.my_ang = 0.;
    }

    /// Set(Dir, Angle): the draft line of `dir` at `angle`; `f.set_direction_angle(d, 0.1)`.
    pub fn set_direction_angle(&mut self, dir: Direction3, angle: f64) {
        self.my_type = ContapTFunction::DraftStd;
        self.my_dir = dir;
        self.my_ang = angle;
        self.my_cos_ang = (std::f64::consts::PI / 2. + angle).cos();
    }

    /// Set(Eye, Angle): the draft line of `eye` at `angle` (not evaluated natively);
    /// `f.set_eye_angle(p, 0.1)`.
    pub fn set_eye_angle(&mut self, eye: Point3, angle: f64) {
        self.my_type = ContapTFunction::DraftPrs;
        self.my_eye = eye;
        self.my_ang = angle;
        self.my_cos_ang = (std::f64::consts::PI / 2. + angle).cos();
    }

    /// Set(Tolerance): at least 1e-12; `f.set_tolerance(1e-7)`.
    pub fn set_tolerance(&mut self, tolerance: f64) {
        self.tol = if tolerance < 1.0e-12 {
            1.0e-12
        } else {
            tolerance
        };
    }

    /// NbVariables: 2; `f.nb_variables()`.
    pub fn nb_variables(&self) -> i32 {
        2
    }

    /// NbEquations: 1; `f.nb_equations()`.
    pub fn nb_equations(&self) -> i32 {
        1
    }

    /// Root: `F` at the last Value/Values; `f.root()`.
    pub fn root(&self) -> f64 {
        self.valf
    }

    /// Tolerance; `f.tolerance()`.
    pub fn tolerance(&self) -> f64 {
        self.tol
    }

    /// Point: the surface point of the last evaluation; `f.point()`.
    pub fn point(&self) -> Point3 {
        self.solpt
    }

    /// FunctionType; `f.function_type()`.
    pub fn function_type(&self) -> ContapTFunction {
        self.my_type
    }

    /// Eye; `f.eye()`.
    pub fn eye(&self) -> Point3 {
        self.my_eye
    }

    /// Direction; `f.direction()`.
    pub fn direction(&self) -> Direction3 {
        self.my_dir
    }

    /// Angle; `f.angle()`.
    pub fn angle(&self) -> f64 {
        self.my_ang
    }

    /// Surface: the surface (shared handle); `f.surface()`.
    pub fn surface(&self) -> Adaptor3dSurfaceHandle {
        self.my_surf.clone()
    }

    /// PSurface: the surface, as `Surface`; `f.p_surface()`.
    pub fn p_surface(&self) -> Adaptor3dSurfaceHandle {
        self.surface()
    }
}
