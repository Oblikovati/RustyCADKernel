//! The grid and trihedron members of `V3d_View`: the CPU grid placement and snapping, the
//! shader grid, and the trihedron and graduated trihedron.
use super::{V3dTrihedron, V3dTypeOfVisualization, V3dView};
use rustycad_tkernel::quantity::Color;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::gp::{Axis3, Point3, Vector3, Xyz};
use rustycad_tkservice::aspect::{
    AspectCircularGrid, AspectCircularGridMembers as _, AspectGrid, AspectGridDrawMode,
    AspectGridMembers as _, AspectGridParams, AspectRectangularGrid,
    AspectRectangularGridMembers as _, AspectTypeOfTriedronPosition,
};
use rustycad_tkservice::graphic3d::{Graphic3dGraduatedTrihedron, Graphic3dVertex};
use std::f64::consts::PI;

/// `TrsPoint`: a vertex through a 4x4 matrix with the homogeneous division.
pub(crate) fn trs_point(vertex: &Graphic3dVertex, matrix: &[[f64; 4]; 4]) -> Xyz {
    let (x, y, z) = vertex.coord_f64();
    let row = |r: usize| {
        (matrix[r][3] + x * matrix[r][0] + y * matrix[r][1] + z * matrix[r][2]) / matrix[3][3]
    };
    Xyz::new(row(0), row(1), row(2))
}

/// The product `Trsf1 x Trsf2` of `SetGrid`, accumulated in the native order.
fn grid_matrix(plane: &Axis3, x_origin: f64, y_origin: f64, angle: f64) -> [[f64; 4]; 4] {
    let (l, xd, yd, d) = (
        plane.location(),
        plane.x_direction(),
        plane.y_direction(),
        plane.direction(),
    );
    let (cos_alpha, sin_alpha) = (angle.cos(), angle.sin());
    let trsf1 = [
        [xd.x(), yd.x(), d.x(), l.x()],
        [xd.y(), yd.y(), d.y(), l.y()],
        [xd.z(), yd.z(), d.z(), l.z()],
        [0.0, 0.0, 0.0, 1.0],
    ];
    let trsf2 = [
        [cos_alpha, sin_alpha, 0.0, -x_origin],
        [-sin_alpha, cos_alpha, 0.0, -y_origin],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ];
    let mut result = [[0.0; 4]; 4];
    for (i, row) in result.iter_mut().enumerate() {
        for (j, value) in row.iter_mut().enumerate() {
            for k in 0..4 {
                *value += trsf1[i][k] * trsf2[k][j];
            }
        }
    }
    result
}

impl V3dView {
    /// `SetGrid`: the plane and grid used by the grid snapping (erases the shader grid).
    /// Example: `view.set_grid(&plane, &grid)`.
    pub fn set_grid(&self, plane: &Axis3, grid: &Handle<dyn AspectGrid>) {
        if self.state.read().shader_grid_active {
            self.view().grid_erase();
            self.state.write().shader_grid_active = false;
        }
        let (x_origin, y_origin, angle) = grid.get().map_or((0.0, 0.0, 0.0), |g| {
            (g.x_origin(), g.y_origin(), g.rotation_angle())
        });
        let mut state = self.state.write();
        (state.plane, state.grid) = (*plane, grid.clone());
        state.trsf = grid_matrix(plane, x_origin, y_origin, angle);
    }
    /// `SetGridActivity`; `view.set_grid_activity(true)`.
    pub fn set_grid_activity(&self, flag: bool) {
        let grid = self.state.read().grid.clone();
        let Some(grid) = grid.get() else {
            return;
        };
        if flag {
            grid.activate();
        } else {
            grid.deactivate();
        }
    }
    /// `IsGridActive`: the viewer grid or the shader grid; `view.is_grid_active()?`.
    pub fn is_grid_active(&self) -> Result<bool, Failure> {
        let viewer_active = self
            .viewer()
            .get()
            .map(|v| v.is_grid_active())
            .transpose()?
            .unwrap_or(false);
        Ok(viewer_active || self.state.read().shader_grid_active)
    }
    /// `IsShaderGridActive`; `view.is_shader_grid_active()`.
    pub fn is_shader_grid_active(&self) -> bool {
        self.state.read().shader_grid_active
    }
    /// `GridDisplay(params)`: the shader grid in the privileged plane; `view.grid_display(&params)`.
    pub fn grid_display(&self, params: &AspectGridParams) {
        let plane = self
            .viewer()
            .get()
            .map(|v| v.privileged_plane())
            .unwrap_or_default();
        self.grid_display_in(params, &plane);
    }
    /// `GridDisplay(params, plane)`: the shader grid; the CPU grid is erased.
    /// Example: `view.grid_display_in(&params, &plane)`.
    pub fn grid_display_in(&self, params: &AspectGridParams, plane: &Axis3) {
        self.view().grid_display(params, plane);
        let grid = self.state.read().grid.clone();
        if let Some(grid) = grid.get().filter(|g| g.is_displayed()) {
            grid.erase();
        }
        self.state.write().shader_grid_active = params.draw_mode() != AspectGridDrawMode::None;
    }
    /// `GridErase`: removes the shader grid; `view.grid_erase()`.
    pub fn grid_erase(&self) {
        self.view().grid_erase();
        self.state.write().shader_grid_active = false;
    }
    /// `Compute` (private): the grid point closest to a vertex projected on the grid plane;
    /// a degenerate grid frame raises `gp_VectorWithNullMagnitude`.
    pub(crate) fn compute_grid_point(
        &self,
        vertex: &Graphic3dVertex,
    ) -> Result<Graphic3dVertex, Failure> {
        let camera = self.camera();
        let state = self.state.read().clone();
        let vpn = camera.direction().reversed();
        let gpn = state.plane.direction();
        let (vx, vy, vz) = vertex.coord_f64();
        // The native member projects the point first (unused) and compares with PI/180.
        let _ = self.project(vx, vy, vz);
        if (vpn.angle(&gpn) - PI / 2.0).abs() < PI / 180.0 {
            return Ok(*vertex);
        }
        let pnt0 = trs_point(&Graphic3dVertex::from_f64(0.0, 0.0, 0.0), &state.trsf);
        let pnt1 = trs_point(&Graphic3dVertex::from_f64(1.0, 0.0, 0.0), &state.trsf);
        let pnt2 = trs_point(&Graphic3dVertex::from_f64(0.0, 1.0, 0.0), &state.trsf);
        let (grid_x, grid_y) = (
            normalized_between(pnt0, pnt1)?,
            normalized_between(pnt0, pnt2)?,
        );
        let vertex_point = Point3::new(vx, vy, vz);
        let projection = if camera.is_orthographic() {
            Vector3::from(camera.direction())
        } else {
            normalized_between(camera.eye().xyz(), vertex_point.xyz())?
        };
        let point_origin = Vector3::between(vertex_point, Point3::from(pnt0));
        let normal = Vector3::from(state.plane.direction());
        let t = point_origin.dot(normal) / projection.dot(normal);
        let on_plane = Xyz::new(vx, vy, vz) + *projection.xyz() * t;
        Ok(snap_on_grid(&state.grid, pnt0, grid_x, grid_y, on_plane)
            .unwrap_or_else(|| Graphic3dVertex::from_f64(0.0, 0.0, 0.0)))
    }
    /// `ConvertToGrid(Xp, Yp, Vertex)`: the snapped grid point of a pixel, `None` without
    /// active grid. Example: `view.convert_to_grid_vertex(10, 20)?`.
    pub fn convert_to_grid_vertex(
        &self,
        xp: i32,
        yp: i32,
    ) -> Result<Option<Graphic3dVertex>, Failure> {
        if self.state.read().shader_grid_active {
            let mut point = Graphic3dVertex::default();
            return Ok(self
                .view()
                .shader_grid_echo(xp, yp, &mut point)
                .then_some(point));
        }
        let (x, y, z) = self.convert_to_world(xp, yp)?;
        let vrp = Graphic3dVertex::from_f64(x, y, z);
        let active = self
            .viewer()
            .get()
            .map(|v| v.is_grid_active())
            .transpose()?
            .unwrap_or(false);
        if !active {
            return Ok(None);
        }
        Ok(Some(self.compute_grid_point(&vrp)?))
    }
    /// `ConvertToGrid(Xp, Yp, Xg, Yg, Zg)`: the snapped point of a pixel, or its world point.
    /// Example: `view.convert_to_grid(10, 20)?`.
    pub fn convert_to_grid(&self, xp: i32, yp: i32) -> Result<(f64, f64, f64), Failure> {
        if let Some(point) = self.convert_to_grid_vertex(xp, yp)? {
            return Ok(point.coord_f64());
        }
        self.convert_to_world(xp, yp)
    }
    /// `ConvertToGridEcho`: the grid point and the echo point of a pixel.
    /// Example: `view.convert_to_grid_echo(10, 20)?`.
    pub fn convert_to_grid_echo(
        &self,
        xp: i32,
        yp: i32,
    ) -> Result<Option<(Graphic3dVertex, Graphic3dVertex)>, Failure> {
        if self.state.read().shader_grid_active {
            let (mut grid, mut echo) = (Graphic3dVertex::default(), Graphic3dVertex::default());
            let found = self
                .view()
                .shader_grid_echo_display(xp, yp, &mut grid, &mut echo);
            return Ok(found.then_some((grid, echo)));
        }
        Ok(self
            .convert_to_grid_vertex(xp, yp)?
            .map(|point| (point, point)))
    }
    /// `ConvertToGrid(X, Y, Z, Xg, Yg, Zg)`: the snapped point of a world point.
    /// Example: `view.convert_point_to_grid(1.0, 2.0, 0.0)?`.
    pub fn convert_point_to_grid(
        &self,
        x: f64,
        y: f64,
        z: f64,
    ) -> Result<(f64, f64, f64), Failure> {
        if self.state.read().shader_grid_active {
            let point = Graphic3dVertex::from_f64(x, y, z);
            let mut snapped = Graphic3dVertex::default();
            if self.view().shader_grid_snap_point(&point, &mut snapped) {
                return Ok(snapped.coord_f64());
            }
            return Ok((x, y, z));
        }
        let active = self
            .viewer()
            .get()
            .map(|v| v.is_grid_active())
            .transpose()?
            .unwrap_or(false);
        if active {
            return Ok(self
                .compute_grid_point(&Graphic3dVertex::from_f64(x, y, z))?
                .coord_f64());
        }
        Ok((x, y, z))
    }
}

/// The normalized vector from `a` to `b` (`gp_Vec(a, b).Normalized()`); equal points raise
/// `gp_VectorWithNullMagnitude` as natively.
fn normalized_between(a: Xyz, b: Xyz) -> Result<Vector3, Failure> {
    Vector3::between(Point3::from(a), Point3::from(b)).normalized()
}

/// The snapping of `Compute` on the rectangular or circular grid.
fn snap_on_grid(
    grid: &Handle<dyn AspectGrid>,
    pnt0: Xyz,
    grid_x: Vector3,
    grid_y: Vector3,
    on_plane: Xyz,
) -> Option<Graphic3dVertex> {
    let to_point = Vector3::between(Point3::from(pnt0), Point3::from(on_plane));
    if let Some(rect) = grid.clone().cast::<dyn AspectRectangularGrid>().get() {
        let x_steps = (grid_x.dot(to_point) / rect.x_step()).round();
        let y_steps = (grid_y.dot(to_point) / rect.y_step()).round();
        let result = grid_x * x_steps * rect.x_step()
            + grid_y * y_steps * rect.y_step()
            + Vector3::from(pnt0);
        return Some(Graphic3dVertex::from_f64(
            result.x(),
            result.y(),
            result.z(),
        ));
    }
    let circle = grid.clone().cast::<dyn AspectCircularGrid>();
    let circle = circle.get()?;
    let alpha = PI / f64::from(circle.division_number());
    let (local_x, local_y) = (grid_x.dot(to_point), grid_y.dot(to_point));
    let (r, phi) = (
        (local_x * local_x + local_y * local_y).sqrt(),
        local_y.atan2(local_x),
    );
    let r_steps = (r / circle.radius_step()).round();
    let phi_steps = (phi / alpha).round();
    let (rr, pp) = (r_steps * circle.radius_step(), phi_steps * alpha);
    let (local_x, local_y) = (rr * pp.cos(), rr * pp.sin());
    let result = grid_x * local_x + grid_y * local_y + Vector3::from(pnt0);
    Some(Graphic3dVertex::from_f64(
        result.x(),
        result.y(),
        result.z(),
    ))
}

/// The trihedron members.
impl V3dView {
    /// `Trihedron(toCreate)`: the view trihedron, created on demand; `view.trihedron(true)?`.
    pub fn trihedron(&self, to_create: bool) -> Result<Handle<V3dTrihedron>, Failure> {
        if self.state.read().trihedron.is_null() && to_create {
            let trihedron = V3dTrihedron::new()?;
            self.state.write().trihedron = trihedron;
        }
        Ok(self.state.read().trihedron.clone())
    }
    /// `ZBufferTriedronSetup`: the arrow colors, size ratio, facets and diameter.
    /// Example: `view.z_buffer_triedron_setup(red, green, blue1, 0.8, 0.05, 12)?`.
    #[allow(clippy::too_many_arguments)]
    pub fn z_buffer_triedron_setup(
        &self,
        x_color: Color,
        y_color: Color,
        z_color: Color,
        size_ratio: f64,
        axis_diametr: f64,
        nb_facettes: i32,
    ) -> Result<(), Failure> {
        let trihedron = self.trihedron(true)?;
        let Some(value) = trihedron.get() else {
            return Ok(());
        };
        value.set_arrows_color(x_color, y_color, z_color)?;
        value.set_size_ratio(size_ratio);
        value.set_nb_facets(nb_facettes);
        value.set_arrow_diameter(axis_diametr);
        Ok(())
    }
    /// `TriedronDisplay`: shows the trihedron at a corner; `view.triedron_display(corner, white, 0.02, V3dTypeOfVisualization::Wireframe)?`.
    pub fn triedron_display(
        &self,
        position: AspectTypeOfTriedronPosition,
        color: Color,
        scale: f64,
        mode: V3dTypeOfVisualization,
    ) -> Result<(), Failure> {
        let trihedron = self.trihedron(true)?;
        let Some(value) = trihedron.get() else {
            return Ok(());
        };
        value.set_labels_color(color);
        value.set_scale(scale);
        value.set_position(position)?;
        value.set_wireframe(mode == V3dTypeOfVisualization::Wireframe);
        value.display(self)
    }
    /// `TriedronErase`; `view.triedron_erase()`.
    pub fn triedron_erase(&self) {
        let trihedron = self.state.read().trihedron.clone();
        if let Some(value) = trihedron.get() {
            value.erase();
        }
    }
    /// `GetGraduatedTrihedron`; `view.get_graduated_trihedron()`.
    pub fn get_graduated_trihedron(&self) -> Graphic3dGraduatedTrihedron {
        self.view().get_graduated_trihedron()
    }
    /// `GraduatedTrihedronDisplay`; `view.graduated_trihedron_display(&data)`.
    pub fn graduated_trihedron_display(&self, data: &Graphic3dGraduatedTrihedron) {
        self.view().graduated_trihedron_display(data);
    }
    /// `GraduatedTrihedronErase`; `view.graduated_trihedron_erase()`.
    pub fn graduated_trihedron_erase(&self) {
        self.view().graduated_trihedron_erase();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grid_matrix_composes_plane_and_rotation() {
        let matrix = grid_matrix(&Axis3::default(), 1.0, 2.0, 0.0);
        assert_eq!(matrix[0], [1.0, 0.0, 0.0, -1.0]);
        assert_eq!(matrix[1], [0.0, 1.0, 0.0, -2.0]);
        let point = trs_point(&Graphic3dVertex::from_f64(3.0, 3.0, 0.0), &matrix);
        assert_eq!((point.x(), point.y()), (2.0, 1.0));
    }
}
