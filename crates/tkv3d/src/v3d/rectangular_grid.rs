//! `V3d_RectangularGrid`: the rectangular grid of a viewer, drawn as lines or points in the
//! privileged plane by its own structure (CPU rendering, deprecated natively).
use super::V3dViewer;
use super::computed_structure::{V3dComputedStructure, V3dStructureComputer};
use super::grid_common::{V3dGridPlacement, finish_grid_group, should_rebuild};
use rustycad_tkernel::quantity::Color;
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient, WeakHandle};
use rustycad_tkmath::gp::Point3;
use rustycad_tkservice::aspect::{
    AspectGrid, AspectGridBase, AspectGridDrawMode, AspectGridMembers as _, AspectRectangularGrid,
    AspectRectangularGridBase, AspectRectangularGridMembers as _, AspectTypeOfLine,
    AspectTypeOfMarker,
};
use rustycad_tkservice::graphic3d::{
    Graphic3dArrayOfPoints, Graphic3dArrayOfSegments, Graphic3dAspectLine3d,
    Graphic3dAspectMarker3d, Graphic3dDisplayPriority, Graphic3dGroup, Graphic3dGroupMembers as _,
    Graphic3dStructureDisplay as _, Graphic3dStructureMembers as _, Graphic3dStructureSource,
};
use rustycad_tkservice::json_dump::{
    base_class, class_begin, field_dumped, field_integer, field_pointer, field_real,
};
use std::io;

/// `THE_DEFAULT_GRID_STEP`.
pub(crate) const DEFAULT_GRID_STEP: f64 = 10.0;
/// `THE_MYFACTOR`: the default Z offset is the step divided by it.
pub(crate) const GRID_OFFSET_FACTOR: f64 = 50.0;

/// The mutable drawing state of the grid.
#[derive(Clone)]
struct RectGridState {
    structure: Handle<dyn Graphic3dStructureSource>,
    group: Handle<dyn Graphic3dGroup>,
    placement: V3dGridPlacement,
    cur_are_defined: bool,
    to_compute_prs: bool,
    cur_draw_mode: AspectGridDrawMode,
    cur_x_step: f64,
    cur_y_step: f64,
    cur_x_size: f64,
    cur_y_size: f64,
    cur_off_set: f64,
}

/// The rectangular grid of a viewer.
///
/// Example: `let grid = V3dRectangularGrid::new(&viewer, gray50, gray70)?; grid.display();`.
pub struct V3dRectangularGrid {
    transient: StandardTransient,
    base: AspectRectangularGridBase,
    viewer: WeakHandle<V3dViewer>,
    state: SharedState<RectGridState>,
}
rustycad_tkernel::impl_transient!(V3dRectangularGrid, "V3d_RectangularGrid", dyn AspectRectangularGrid, transient,
    interfaces: [dyn AspectGrid, dyn AspectRectangularGrid, dyn V3dStructureComputer]);

impl V3dRectangularGrid {
    /// A grid of steps 10 and half the default view size, offset step/50 under the plane.
    /// Example: `V3dRectangularGrid::new(&viewer, gray50, gray70)?`.
    pub fn new(
        viewer: &Handle<V3dViewer>,
        color: Color,
        tenth_color: Color,
    ) -> Result<Handle<Self>, Failure> {
        let Some(viewer_ref) = viewer.get() else {
            return Err(rustycad_tkernel::standard::exceptions::null_object(
                "V3d_RectangularGrid: null viewer; expected a viewer",
            ));
        };
        let base = AspectRectangularGridBase::new(1.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0)?;
        {
            let mut fields = base.grid().fields_mut();
            (fields.color, fields.tenth_color) = (color, tenth_color);
        }
        let state = RectGridState {
            structure: Handle::default(),
            group: Handle::default(),
            placement: V3dGridPlacement::default(),
            cur_are_defined: false,
            to_compute_prs: true,
            cur_draw_mode: AspectGridDrawMode::Lines,
            cur_x_step: 0.0,
            cur_y_step: 0.0,
            cur_x_size: 0.0,
            cur_y_size: 0.0,
            cur_off_set: 0.0,
        };
        let grid = Handle::new(Self {
            transient: StandardTransient::default(),
            base,
            viewer: viewer.downgrade(),
            state: SharedState::new(state),
        });
        let structure =
            V3dComputedStructure::create(&viewer_ref.structure_manager(), &grid.clone().cast())?;
        let group = structure
            .get()
            .map(|s| s.new_group())
            .transpose()?
            .unwrap_or_default();
        if let Some(value) = structure.get() {
            value.set_infinite_state(true);
        }
        (grid.state.write().structure, grid.state.write().group) = (structure, group);
        grid.set_x_step(DEFAULT_GRID_STEP)?;
        grid.set_y_step(DEFAULT_GRID_STEP)?;
        grid.set_size_x(0.5 * viewer_ref.default_view_size())?;
        grid.set_size_y(0.5 * viewer_ref.default_view_size())?;
        grid.set_z_offset(DEFAULT_GRID_STEP / GRID_OFFSET_FACTOR);
        Ok(grid)
    }
    fn structure(&self) -> Handle<dyn Graphic3dStructureSource> {
        self.state.read().structure.clone()
    }
    /// `GraphicValues`: `(XSize, YSize, OffSet)`; `grid.graphic_values()`.
    pub fn graphic_values(&self) -> (f64, f64, f64) {
        (self.size_x(), self.size_y(), self.z_offset())
    }
    /// `SetGraphicValues`; a negative size raises `Standard_NegativeValue`.
    /// Example: `grid.set_graphic_values(100.0, 100.0, 0.1)?`.
    pub fn set_graphic_values(
        &self,
        x_size: f64,
        y_size: f64,
        off_set: f64,
    ) -> Result<(), Failure> {
        self.set_size_x(x_size)?;
        self.set_size_y(y_size)?;
        self.set_z_offset(off_set);
        Ok(())
    }
    /// The parameters of a rebuild and whether they changed since the last one.
    fn rebuild_needed(&self, mode: AspectGridDrawMode) -> Option<[f64; 5]> {
        let values = [
            self.x_step(),
            self.y_step(),
            self.size_x(),
            self.size_y(),
            self.z_offset(),
        ];
        let mut state = self.state.write();
        let previous = [
            state.cur_x_step,
            state.cur_y_step,
            state.cur_x_size,
            state.cur_y_size,
            state.cur_off_set,
        ];
        let to_update = !state.cur_are_defined || state.cur_draw_mode != mode || values != previous;
        let structure = state.structure.clone();
        should_rebuild(to_update, &mut state.to_compute_prs, &structure).then_some(values)
    }
    /// Stores the parameters of the last rebuild and finishes the group.
    fn finish(&self, values: [f64; 5]) {
        let [x_step, y_step, x_size, y_size, off_set] = values;
        let state = {
            let mut state = self.state.write();
            (
                state.cur_x_step,
                state.cur_y_step,
                state.cur_x_size,
                state.cur_y_size,
                state.cur_off_set,
            ) = (x_step, y_step, x_size, y_size, off_set);
            state.clone()
        };
        let bounds = [-x_size, -y_size, -off_set, x_size, y_size, -off_set];
        finish_grid_group(
            &state.group,
            &state.structure,
            &self.viewer.upgrade(),
            bounds,
        );
    }
    /// `DefineLines`: the solid lines and every tenth line in the tenth color.
    fn define_lines(&self) -> Result<(), Failure> {
        let Some(values) = self.rebuild_needed(AspectGridDrawMode::Lines) else {
            return Ok(());
        };
        let group = self.state.read().group.clone();
        if let Some(g) = group.get() {
            g.clear(true);
        }
        let (lines, tenth) = rectangular_line_points(values);
        let colors = self.colors();
        add_segments(&group, &lines, colors.0)?;
        add_segments(&group, &tenth, colors.1)?;
        self.finish(values);
        Ok(())
    }
    /// `DefinePoints`: the grid nodes as point markers.
    fn define_points(&self) -> Result<(), Failure> {
        let Some(values) = self.rebuild_needed(AspectGridDrawMode::Points) else {
            return Ok(());
        };
        let group = self.state.read().group.clone();
        let Some(g) = group.get() else {
            return Ok(());
        };
        g.clear(true);
        let points = rectangular_grid_points(values);
        if !points.is_empty() {
            let array = Graphic3dArrayOfPoints::new(points.len() as i32, 0)?;
            points
                .iter()
                .try_for_each(|p| array.add_vertex_pnt(p).map(|_| ()))?;
            let marker =
                Graphic3dAspectMarker3d::new(AspectTypeOfMarker::Point, self.colors().0, 3.0)?;
            g.set_group_primitives_aspect(&Handle::new(marker).cast());
            g.add_primitive_array(&Handle::new(array).cast(), false);
        }
        self.finish(values);
        Ok(())
    }
    fn update_display_checked(&self) -> Result<(), Failure> {
        let Some(viewer) = self.viewer.upgrade().get().map(V3dViewer::privileged_plane) else {
            return Ok(());
        };
        let fields = *self.base.grid().fields();
        let wanted = V3dGridPlacement {
            view_plane: viewer,
            x_origin: fields.x_origin,
            y_origin: fields.y_origin,
            angle: fields.rotation_angle,
        };
        let (defined, current, structure) = {
            let state = self.state.read();
            (
                state.cur_are_defined,
                state.placement,
                state.structure.clone(),
            )
        };
        if current.needs_transform(defined, &viewer, &wanted) {
            wanted.apply(&structure)?;
            self.state.write().placement = wanted;
        }
        match fields.draw_mode {
            AspectGridDrawMode::Points => self.define_points()?,
            AspectGridDrawMode::Lines => self.define_lines()?,
            AspectGridDrawMode::None => {}
        }
        let mut state = self.state.write();
        (state.cur_draw_mode, state.cur_are_defined) = (fields.draw_mode, true);
        Ok(())
    }
}

/// The two segment lists of `DefineLines` (regular, tenth) for `[XStep, YStep, XSize, YSize, OffSet]`.
fn rectangular_line_points(values: [f64; 5]) -> (Vec<Point3>, Vec<Point3>) {
    let [x_step, y_step, x_size, y_size, off_set] = values;
    let zl = off_set;
    let (mut lines, mut tenth) = (Vec::new(), Vec::new());
    tenth.extend([
        Point3::new(0.0, -y_size, -zl),
        Point3::new(0.0, y_size, -zl),
    ]);
    let (mut nblines, mut xl) = (1, x_step);
    while xl < x_size {
        let target = if nblines % 10 != 0 {
            &mut lines
        } else {
            &mut tenth
        };
        target.extend([
            Point3::new(xl, -y_size, -zl),
            Point3::new(xl, y_size, -zl),
            Point3::new(-xl, -y_size, -zl),
            Point3::new(-xl, y_size, -zl),
        ]);
        (xl, nblines) = (xl + x_step, nblines + 1);
    }
    tenth.extend([
        Point3::new(-x_size, 0.0, -zl),
        Point3::new(x_size, 0.0, -zl),
    ]);
    let (mut nblines, mut yl) = (1, y_step);
    while yl < y_size {
        let target = if nblines % 10 != 0 {
            &mut lines
        } else {
            &mut tenth
        };
        target.extend([
            Point3::new(-x_size, yl, -zl),
            Point3::new(x_size, yl, -zl),
            Point3::new(-x_size, -yl, -zl),
            Point3::new(x_size, -yl, -zl),
        ]);
        (yl, nblines) = (yl + y_step, nblines + 1);
    }
    (lines, tenth)
}

/// The node list of `DefinePoints`.
fn rectangular_grid_points(values: [f64; 5]) -> Vec<Point3> {
    let [x_step, y_step, x_size, y_size, off_set] = values;
    let mut points = Vec::new();
    let mut xl = 0.0;
    while xl <= x_size {
        points.extend([
            Point3::new(xl, 0.0, -off_set),
            Point3::new(-xl, 0.0, -off_set),
        ]);
        let mut yl = y_step;
        while yl <= y_size {
            points.extend([
                Point3::new(xl, yl, -off_set),
                Point3::new(xl, -yl, -off_set),
                Point3::new(-xl, yl, -off_set),
                Point3::new(-xl, -yl, -off_set),
            ]);
            yl += y_step;
        }
        xl += x_step;
    }
    points
}

/// Adds the non-empty segment list in a solid line aspect of a color.
pub(crate) fn add_segments(
    group: &Handle<dyn Graphic3dGroup>,
    points: &[Point3],
    color: Color,
) -> Result<(), Failure> {
    let Some(g) = group.get().filter(|_| !points.is_empty()) else {
        return Ok(());
    };
    let aspect = Graphic3dAspectLine3d::new(color, AspectTypeOfLine::Solid, 1.0)?;
    g.set_primitives_aspect(&Handle::new(aspect).cast());
    let array = Graphic3dArrayOfSegments::new(points.len() as i32, 0, 0)?;
    points
        .iter()
        .try_for_each(|p| array.add_vertex_pnt(p).map(|_| ()))?;
    g.add_primitive_array(&Handle::new(array).cast(), false);
    Ok(())
}

impl AspectGrid for V3dRectangularGrid {
    fn grid_base(&self) -> &AspectGridBase {
        self.base.grid()
    }
    fn compute(&self, x: f64, y: f64) -> (f64, f64) {
        self.base.compute(x, y)
    }
    /// `Display`: the structure above the bottom priority, then the update.
    fn display(&self) {
        if let Some(value) = self.structure().get() {
            // The almost-bottom priority is always valid.
            let _ = value.set_display_priority(Graphic3dDisplayPriority::AlmostBottom);
            value.display();
        }
        self.update_display();
    }
    fn erase(&self) {
        if let Some(value) = self.structure().get() {
            value.erase();
        }
    }
    fn is_displayed(&self) -> bool {
        self.structure().get().is_some_and(|s| s.is_displayed())
    }
    fn init(&self) {
        self.base.init();
    }
    /// `UpdateDisplay`; the transformation failures of a degenerate plane are not reported
    /// (natively `gp_Trsf::SetValues` raises inside a `void` member).
    fn update_display(&self) {
        let _ = self.update_display_checked();
    }
    /// `SetColors`: redraws only when a color changes.
    fn set_colors(&self, color: Color, tenth_color: Color) {
        {
            let mut fields = self.base.grid().fields_mut();
            if fields.color == color && fields.tenth_color == tenth_color {
                return;
            }
            (fields.color, fields.tenth_color) = (color, tenth_color);
        }
        self.state.write().cur_are_defined = false;
        self.update_display();
    }
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        class_begin(stream, "V3d_RectangularGrid")?;
        base_class(stream, depth, &|s, d| self.base.dump_json(s, d))?;
        let state = self.state.read().clone();
        dump_grid_state(
            stream,
            depth,
            &state.structure,
            &state.group,
            &state.placement,
            self.viewer.object_address() as usize,
        )?;
        field_integer(stream, "myCurAreDefined", i32::from(state.cur_are_defined))?;
        field_integer(stream, "myToComputePrs", i32::from(state.to_compute_prs))?;
        field_integer(stream, "myCurDrawMode", state.cur_draw_mode as i32)?;
        let reals = [
            ("myCurXo", state.placement.x_origin),
            ("myCurYo", state.placement.y_origin),
            ("myCurAngle", state.placement.angle),
            ("myCurXStep", state.cur_x_step),
            ("myCurYStep", state.cur_y_step),
            ("myCurXSize", state.cur_x_size),
            ("myCurYSize", state.cur_y_size),
            ("myCurOffSet", state.cur_off_set),
        ];
        reals
            .iter()
            .try_for_each(|(name, value)| field_real(stream, name, *value))
    }
}

/// The structure, group, view plane and viewer fields that both grids dump.
pub(crate) fn dump_grid_state(
    stream: &mut DumpStream,
    depth: i32,
    structure: &Handle<dyn Graphic3dStructureSource>,
    group: &Handle<dyn Graphic3dGroup>,
    placement: &V3dGridPlacement,
    viewer_address: usize,
) -> io::Result<()> {
    if let Some(value) = structure.get() {
        field_dumped(stream, depth, "myStructure.get()", &|s, d| {
            value.dump_json(s, d)
        })?;
    }
    if let Some(value) = group.get() {
        field_dumped(stream, depth, "myGroup.get()", &|s, d| {
            value.dump_json(s, d)
        })?;
    }
    let plane = placement.view_plane;
    field_dumped(stream, depth, "&myCurViewPlane", &|s, _| plane.dump_json(s))?;
    field_pointer(stream, "myViewer", viewer_address)
}

impl AspectRectangularGrid for V3dRectangularGrid {
    fn rectangular_base(&self) -> &AspectRectangularGridBase {
        &self.base
    }
}

impl V3dStructureComputer for V3dRectangularGrid {
    /// `RectangularGridStructure::Compute`: clears the structure and rebuilds a new group.
    fn compute_structure(&self) {
        let structure = self.structure();
        let Some(value) = structure.get() else {
            return;
        };
        value.graphic_clear(false);
        let group = value.new_group().unwrap_or_default();
        {
            let mut state = self.state.write();
            (state.group, state.cur_are_defined) = (group, false);
        }
        self.update_display();
    }
}

impl Drop for V3dRectangularGrid {
    /// The destructor erases the structure.
    fn drop(&mut self) {
        let structure = self.state.read().structure.clone();
        if let Some(value) = structure.get() {
            value.erase();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn line_lists_split_every_tenth_line() {
        let (lines, tenth) = rectangular_line_points([1.0, 1.0, 12.0, 3.0, 0.2]);
        // X: lines at 1..=11 (10 is a tenth line), Y: lines at 1 and 2.
        assert_eq!(tenth.len(), 2 + 4 + 2);
        assert_eq!(lines.len(), 4 * 10 + 4 * 2);
        assert_eq!(tenth[2], Point3::new(10.0, -3.0, -0.2));
        let points = rectangular_grid_points([1.0, 1.0, 1.0, 1.0, 0.0]);
        assert_eq!(points.len(), 2 * (2 + 4));
    }
}
