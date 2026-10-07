//! The privileged plane, grid and grid echo members of `V3d_Viewer`.
use super::{V3dCircularGrid, V3dRectangularGrid, V3dView, V3dViewer};
use rustycad_tkernel::quantity::{Color, NamedColor};
use rustycad_tkernel::standard::short_real::short_real_last;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkservice::aspect::{
    AspectCircularGridMembers as _, AspectGrid, AspectGridDrawMode, AspectGridMembers as _,
    AspectGridType, AspectRectangularGridMembers as _, AspectTypeOfMarker,
};
use rustycad_tkservice::graphic3d::{
    GRAPHIC3D_ZLAYER_ID_TOPMOST, Graphic3dArrayOfPoints, Graphic3dAspectMarker3d,
    Graphic3dCViewMembers as _, Graphic3dGroupMembers as _, Graphic3dStructure,
    Graphic3dStructureMembers as _, Graphic3dStructureSource, Graphic3dVertex,
    Graphic3dViewAffinity,
};

impl V3dViewer {
    /// `Grid(type, toCreate)`: the grid of a type, created on demand (gray 50 and gray 70).
    /// Example: `viewer.grid_of_type(AspectGridType::Circular, true)?`.
    pub fn grid_of_type(
        &self,
        grid_type: AspectGridType,
        to_create: bool,
    ) -> Result<Handle<dyn AspectGrid>, Failure> {
        let (gray50, gray70) = (
            Color::named(NamedColor::GRAY50),
            Color::named(NamedColor::GRAY70),
        );
        match grid_type {
            AspectGridType::Circular => {
                if self.state.read().c_grid.is_null() && to_create {
                    let grid = V3dCircularGrid::new(&self.this(), gray50, gray70)?;
                    self.state.write().c_grid = grid;
                }
                Ok(self.state.read().c_grid.clone().cast())
            }
            AspectGridType::Rectangular => {
                if self.state.read().r_grid.is_null() && to_create {
                    let grid = V3dRectangularGrid::new(&self.this(), gray50, gray70)?;
                    self.state.write().r_grid = grid;
                }
                Ok(self.state.read().r_grid.clone().cast())
            }
        }
    }
    /// `Grid(toCreate)`: the grid of the current type; `viewer.grid(false)?`.
    pub fn grid(&self, to_create: bool) -> Result<Handle<dyn AspectGrid>, Failure> {
        let grid_type = self.state.read().grid_type;
        self.grid_of_type(grid_type, to_create)
    }
    /// `GridType`; `viewer.grid_type()`.
    pub fn grid_type(&self) -> AspectGridType {
        self.state.read().grid_type
    }
    /// `GridDrawMode`: the draw mode of the grid, lines without grid; `viewer.grid_draw_mode()?`.
    pub fn grid_draw_mode(&self) -> Result<AspectGridDrawMode, Failure> {
        Ok(self
            .grid(false)?
            .get()
            .map_or(AspectGridDrawMode::Lines, |g| g.draw_mode()))
    }
    /// `ActivateGrid`: erases the old grid, then activates (and displays) the grid of a type.
    /// Example: `viewer.activate_grid(AspectGridType::Rectangular, AspectGridDrawMode::Lines)?`.
    pub fn activate_grid(
        &self,
        grid_type: AspectGridType,
        mode: AspectGridDrawMode,
    ) -> Result<(), Failure> {
        if let Some(old) = self.grid(false)?.get() {
            old.erase();
        }
        self.state.write().grid_type = grid_type;
        let grid = self.grid(true)?;
        if let Some(value) = grid.get() {
            value.set_draw_mode(mode);
            if mode != AspectGridDrawMode::None {
                value.display();
            }
            value.activate();
        }
        let plane = self.privileged_plane();
        let views = self.state.read().active_views.clone();
        views
            .iter()
            .filter_map(Handle::get)
            .for_each(|view| view.set_grid(&plane, &grid));
        Ok(())
    }
    /// `DeactivateGrid`: erases and deactivates the grid, back to the rectangular type.
    /// Example: `viewer.deactivate_grid()?`.
    pub fn deactivate_grid(&self) -> Result<(), Failure> {
        let grid = self.grid(false)?;
        let Some(value) = grid.get() else {
            return Ok(());
        };
        value.erase();
        value.deactivate();
        let (views, echo, echo_structure) = {
            let mut state = self.state.write();
            state.grid_type = AspectGridType::Rectangular;
            (
                state.active_views.clone(),
                state.grid_echo,
                state.grid_echo_structure.clone(),
            )
        };
        for view in views.iter().filter_map(Handle::get) {
            view.set_grid_activity(false);
            if let Some(structure) = echo_structure.get().filter(|_| echo) {
                structure.erase();
            }
        }
        Ok(())
    }
    /// `IsGridActive`; `viewer.is_grid_active()?`.
    pub fn is_grid_active(&self) -> Result<bool, Failure> {
        Ok(self.grid(false)?.get().is_some_and(|g| g.is_active()))
    }
    /// Deprecated `IsActive`, the same as `IsGridActive`; `viewer.is_active()?`.
    pub fn is_active(&self) -> Result<bool, Failure> {
        self.is_grid_active()
    }
    fn rectangular(&self) -> Result<Handle<V3dRectangularGrid>, Failure> {
        self.grid_of_type(AspectGridType::Rectangular, true)?;
        Ok(self.state.read().r_grid.clone())
    }
    fn circular(&self) -> Result<Handle<V3dCircularGrid>, Failure> {
        self.grid_of_type(AspectGridType::Circular, true)?;
        Ok(self.state.read().c_grid.clone())
    }
    /// `RectangularGridValues`: `(XOrigin, YOrigin, XStep, YStep, RotationAngle)`.
    /// Example: `viewer.rectangular_grid_values()?`.
    pub fn rectangular_grid_values(&self) -> Result<(f64, f64, f64, f64, f64), Failure> {
        let grid = self.rectangular()?;
        Ok(grid.get().map_or((0.0, 0.0, 0.0, 0.0, 0.0), |g| {
            (
                g.x_origin(),
                g.y_origin(),
                g.x_step(),
                g.y_step(),
                g.rotation_angle(),
            )
        }))
    }
    /// `SetRectangularGridValues`, then the grid of every active view.
    /// Example: `viewer.set_rectangular_grid_values(0.0, 0.0, 5.0, 5.0, 0.0)?`.
    pub fn set_rectangular_grid_values(
        &self,
        x_origin: f64,
        y_origin: f64,
        x_step: f64,
        y_step: f64,
        rotation_angle: f64,
    ) -> Result<(), Failure> {
        let grid = self.rectangular()?;
        if let Some(value) = grid.get() {
            value.set_grid_values(x_origin, y_origin, x_step, y_step, rotation_angle)?;
        }
        self.set_grid_in_active_views(&grid.cast());
        Ok(())
    }
    /// `CircularGridValues`: `(XOrigin, YOrigin, RadiusStep, DivisionNumber, RotationAngle)`.
    /// Example: `viewer.circular_grid_values()?`.
    pub fn circular_grid_values(&self) -> Result<(f64, f64, f64, i32, f64), Failure> {
        let grid = self.circular()?;
        Ok(grid.get().map_or((0.0, 0.0, 0.0, 0, 0.0), |g| {
            (
                g.x_origin(),
                g.y_origin(),
                g.radius_step(),
                g.division_number(),
                g.rotation_angle(),
            )
        }))
    }
    /// `SetCircularGridValues`, then the grid of every active view.
    /// Example: `viewer.set_circular_grid_values(0.0, 0.0, 5.0, 8, 0.0)?`.
    pub fn set_circular_grid_values(
        &self,
        x_origin: f64,
        y_origin: f64,
        radius_step: f64,
        division_number: i32,
        rotation_angle: f64,
    ) -> Result<(), Failure> {
        let grid = self.circular()?;
        if let Some(value) = grid.get() {
            value.set_grid_values(
                x_origin,
                y_origin,
                radius_step,
                division_number,
                rotation_angle,
            )?;
        }
        self.set_grid_in_active_views(&grid.cast());
        Ok(())
    }
    fn set_grid_in_active_views(&self, grid: &Handle<dyn AspectGrid>) {
        let plane = self.privileged_plane();
        let views = self.state.read().active_views.clone();
        views
            .iter()
            .filter_map(Handle::get)
            .for_each(|view| view.set_grid(&plane, grid));
    }
    /// `RectangularGridGraphicValues`: `(XSize, YSize, OffSet)`; `viewer.rectangular_grid_graphic_values()?`.
    pub fn rectangular_grid_graphic_values(&self) -> Result<(f64, f64, f64), Failure> {
        Ok(self
            .rectangular()?
            .get()
            .map_or((0.0, 0.0, 0.0), V3dRectangularGrid::graphic_values))
    }
    /// `SetRectangularGridGraphicValues`; `viewer.set_rectangular_grid_graphic_values(100.0, 100.0, 0.0)?`.
    pub fn set_rectangular_grid_graphic_values(
        &self,
        x_size: f64,
        y_size: f64,
        off_set: f64,
    ) -> Result<(), Failure> {
        match self.rectangular()?.get() {
            Some(grid) => grid.set_graphic_values(x_size, y_size, off_set),
            None => Ok(()),
        }
    }
    /// `CircularGridGraphicValues`: `(Radius, OffSet)`; `viewer.circular_grid_graphic_values()?`.
    pub fn circular_grid_graphic_values(&self) -> Result<(f64, f64), Failure> {
        Ok(self
            .circular()?
            .get()
            .map_or((0.0, 0.0), V3dCircularGrid::graphic_values))
    }
    /// `SetCircularGridGraphicValues`; `viewer.set_circular_grid_graphic_values(100.0, 0.0)?`.
    pub fn set_circular_grid_graphic_values(
        &self,
        radius: f64,
        off_set: f64,
    ) -> Result<(), Failure> {
        match self.circular()?.get() {
            Some(grid) => grid.set_graphic_values(radius, off_set),
            None => Ok(()),
        }
    }
}

/// The grid echo members.
impl V3dViewer {
    /// `SetGridEcho(bool)`: shows or hides the echo marker; `viewer.set_grid_echo(false)`.
    pub fn set_grid_echo(&self, to_show_grid: bool) {
        let structure = {
            let mut state = self.state.write();
            if state.grid_echo == to_show_grid {
                return;
            }
            state.grid_echo = to_show_grid;
            state.grid_echo_structure.clone()
        };
        if to_show_grid {
            return;
        }
        if let Some(value) = structure.get() {
            value.erase();
        }
    }
    /// `SetGridEcho(marker)`: the marker of the echo; `viewer.set_grid_echo_marker(&marker)?`.
    pub fn set_grid_echo_marker(
        &self,
        marker: &Handle<Graphic3dAspectMarker3d>,
    ) -> Result<(), Failure> {
        self.ensure_echo_structure()?;
        let group = {
            let mut state = self.state.write();
            state.grid_echo_aspect = marker.clone();
            state.grid_echo_group.clone()
        };
        if let Some(group) = group.get() {
            group.set_primitives_aspect(&marker.clone().cast());
        }
        Ok(())
    }
    /// Creates the echo structure and group when missing; true when created.
    fn ensure_echo_structure(&self) -> Result<bool, Failure> {
        if !self.state.read().grid_echo_structure.is_null() {
            return Ok(false);
        }
        let structure: Handle<dyn Graphic3dStructureSource> =
            Graphic3dStructure::new(&self.structure_manager(), &Handle::default())?.cast();
        let group = structure
            .get()
            .map(|s| s.new_group())
            .transpose()?
            .unwrap_or_default();
        let mut state = self.state.write();
        (state.grid_echo_structure, state.grid_echo_group) = (structure, group);
        Ok(true)
    }
    /// `GridEcho`; `viewer.grid_echo()`.
    pub fn grid_echo(&self) -> bool {
        self.state.read().grid_echo
    }
    /// `ShowGridEcho`: the echo marker at a grid point, visible in one view only.
    /// Example: `viewer.show_grid_echo(&view, &vertex)?`.
    pub fn show_grid_echo(
        &self,
        view: &Handle<V3dView>,
        vertex: &Graphic3dVertex,
    ) -> Result<(), Failure> {
        if !self.state.read().grid_echo {
            return Ok(());
        }
        if self.ensure_echo_structure()? {
            let marker = Graphic3dAspectMarker3d::new(
                AspectTypeOfMarker::Star,
                Color::named(NamedColor::GRAY90),
                3.0,
            )?;
            let marker = Handle::new(marker);
            let group = {
                let mut state = self.state.write();
                state.grid_echo_aspect = marker.clone();
                state.grid_echo_group.clone()
            };
            if let Some(group) = group.get() {
                group.set_primitives_aspect(&marker.cast());
            }
        }
        let Some(view_ref) = view.get() else {
            return Ok(());
        };
        let last = self.state.read().grid_echo_last_vert;
        if !view_ref.is_shader_grid_active()
            && vertex.x() == last.x()
            && vertex.y() == last.y()
            && vertex.z() == last.z()
        {
            return Ok(());
        }
        let (structure, group, aspect) = {
            let mut state = self.state.write();
            state.grid_echo_last_vert = *vertex;
            (
                state.grid_echo_structure.clone(),
                state.grid_echo_group.clone(),
                state.grid_echo_aspect.clone(),
            )
        };
        if let Some(group) = group.get() {
            group.clear(true);
            group.set_primitives_aspect(&aspect.cast());
            let points = Graphic3dArrayOfPoints::new(1, 0)?;
            let (x, y, z) = vertex.coord();
            points.add_vertex(x, y, z)?;
            group.add_primitive_array(&Handle::new(points).cast(), true);
        }
        show_echo_structure(&structure, view_ref);
        Ok(())
    }
    /// `HideGridEcho`: erases the echo when it is visible in a view; `viewer.hide_grid_echo(&view)`.
    pub fn hide_grid_echo(&self, view: &Handle<V3dView>) {
        let structure = self.state.read().grid_echo_structure.clone();
        let Some(value) = structure.get() else {
            return;
        };
        let last = short_real_last();
        self.state.write().grid_echo_last_vert = Graphic3dVertex::new(last, last, last);
        let Some(view_id) = view
            .get()
            .and_then(|v| v.view().get().map(|c| c.identification()))
        else {
            return;
        };
        let affinity = value
            .cstructure()
            .get()
            .map(|c| c.cstructure_base().fields.read().view_affinity.clone())
            .unwrap_or_default();
        if affinity.get().is_some_and(|a| a.is_visible(view_id)) {
            value.erase();
        }
    }
}

/// Displays the echo structure in the topmost layer, visible in one view only.
fn show_echo_structure(structure: &Handle<dyn Graphic3dStructureSource>, view: &V3dView) {
    let Some(value) = structure.get() else {
        return;
    };
    value.set_z_layer(GRAPHIC3D_ZLAYER_ID_TOPMOST);
    value.set_infinite_state(true);
    if let Some(cstructure) = value.cstructure().get() {
        let affinity = Graphic3dViewAffinity::default();
        affinity.set_visible(false);
        if let Some(cview) = view.view().get() {
            affinity.set_view_visible(cview.identification(), true);
        }
        cstructure.cstructure_base().fields.write().view_affinity = Handle::new(affinity);
    }
    value.display();
}
