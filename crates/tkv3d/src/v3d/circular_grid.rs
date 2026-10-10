//! `V3d_CircularGrid`: the circular grid of a viewer, drawn as radial lines and circles or as
//! points in the privileged plane by its own structure (CPU rendering, deprecated natively).
use super::V3dViewer;
use super::computed_structure::{V3dComputedStructure, V3dStructureComputer};
use super::grid_common::{V3dGridPlacement, finish_grid_group, should_rebuild};
use super::rectangular_grid::{DEFAULT_GRID_STEP, GRID_OFFSET_FACTOR, dump_grid_state};
use rustycad_tkernel::message::{Gravity, Message};
use rustycad_tkernel::quantity::Color;
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkernel::standard::exceptions::null_object;
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient, WeakHandle};
use rustycad_tkernel::tcollection::AsciiString;
use rustycad_tkmath::gp::Point3;
use rustycad_tkservice::aspect::{
    AspectCircularGrid, AspectCircularGridBase, AspectCircularGridMembers as _, AspectGrid,
    AspectGridBase, AspectGridDrawMode, AspectGridMembers as _, AspectTypeOfLine,
    AspectTypeOfMarker,
};
use rustycad_tkservice::graphic3d::{
    Graphic3dArrayOfPoints, Graphic3dArrayOfPolylines, Graphic3dArrayOfSegments,
    Graphic3dAspectLine3d, Graphic3dAspectMarker3d, Graphic3dAspectsMembers as _,
    Graphic3dDisplayPriority, Graphic3dGroup, Graphic3dGroupMembers as _,
    Graphic3dStructureDisplay as _, Graphic3dStructureMembers as _, Graphic3dStructureSource,
};
use rustycad_tkservice::json_dump::{base_class, class_begin, field_integer, field_real};
use std::f64::consts::PI;
use std::io;

/// `THE_DEFAULT_DIVISION`.
const DEFAULT_DIVISION: i32 = 8;

/// The mutable drawing state of the grid.
#[derive(Clone)]
struct CircGridState {
    structure: Handle<dyn Graphic3dStructureSource>,
    group: Handle<dyn Graphic3dGroup>,
    placement: V3dGridPlacement,
    cur_are_defined: bool,
    to_compute_prs: bool,
    cur_draw_mode: AspectGridDrawMode,
    cur_step: f64,
    cur_divi: i32,
    cur_radius: f64,
    cur_off_set: f64,
    arc_warned: bool,
}

/// The circular grid of a viewer.
///
/// Example: `let grid = V3dCircularGrid::new(&viewer, gray50, gray70)?; grid.display();`.
pub struct V3dCircularGrid {
    transient: StandardTransient,
    base: AspectCircularGridBase,
    viewer: WeakHandle<V3dViewer>,
    state: SharedState<CircGridState>,
}
rustycad_tkernel::impl_transient!(V3dCircularGrid, "V3d_CircularGrid", dyn AspectCircularGrid, transient,
    interfaces: [dyn AspectGrid, dyn AspectCircularGrid, dyn V3dStructureComputer]);

impl V3dCircularGrid {
    /// A grid of radius step 10, 8 divisions and half the default view size as radius.
    /// Example: `V3dCircularGrid::new(&viewer, gray50, gray70)?`.
    pub fn new(
        viewer: &Handle<V3dViewer>,
        color: Color,
        tenth_color: Color,
    ) -> Result<Handle<Self>, Failure> {
        let Some(viewer_ref) = viewer.get() else {
            return Err(null_object(
                "V3d_CircularGrid: null viewer; expected a viewer",
            ));
        };
        let base = AspectCircularGridBase::new(1.0, DEFAULT_DIVISION, 0.0, 0.0, 0.0);
        {
            let mut fields = base.grid().fields_mut();
            (fields.color, fields.tenth_color) = (color, tenth_color);
        }
        let state = CircGridState {
            structure: Handle::default(),
            group: Handle::default(),
            placement: V3dGridPlacement::default(),
            cur_are_defined: false,
            to_compute_prs: false,
            cur_draw_mode: AspectGridDrawMode::Lines,
            cur_step: 0.0,
            cur_divi: 0,
            cur_radius: 0.0,
            cur_off_set: 0.0,
            arc_warned: false,
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
        grid.set_radius_step(DEFAULT_GRID_STEP)?;
        grid.set_radius(0.5 * viewer_ref.default_view_size())?;
        grid.set_z_offset(DEFAULT_GRID_STEP / GRID_OFFSET_FACTOR);
        Ok(grid)
    }
    fn structure(&self) -> Handle<dyn Graphic3dStructureSource> {
        self.state.read().structure.clone()
    }
    /// `GraphicValues`: `(Radius, OffSet)`; `grid.graphic_values()`.
    pub fn graphic_values(&self) -> (f64, f64) {
        (self.radius(), self.z_offset())
    }
    /// `SetGraphicValues`; a negative radius raises `Standard_NegativeValue`.
    /// Example: `grid.set_graphic_values(100.0, 0.1)?`.
    pub fn set_graphic_values(&self, radius: f64, off_set: f64) -> Result<(), Failure> {
        self.set_radius(radius)?;
        self.set_z_offset(off_set);
        Ok(())
    }
    /// The parameters `[Step, Division, Radius, OffSet]` of a rebuild when one is needed.
    fn rebuild_needed(&self, mode: AspectGridDrawMode) -> Option<[f64; 4]> {
        let values = [
            self.radius_step(),
            f64::from(self.division_number()),
            self.radius(),
            self.z_offset(),
        ];
        let mut state = self.state.write();
        let to_update = !state.cur_are_defined
            || state.cur_draw_mode != mode
            || values[1] != f64::from(state.cur_divi)
            || values[0] != state.cur_step
            || values[2] != state.cur_radius
            || values[3] != state.cur_off_set;
        let structure = state.structure.clone();
        should_rebuild(to_update, &mut state.to_compute_prs, &structure).then_some(values)
    }
    fn finish(&self, values: [f64; 4]) {
        let [step, division, radius, off_set] = values;
        let state = {
            let mut state = self.state.write();
            (
                state.cur_step,
                state.cur_divi,
                state.cur_radius,
                state.cur_off_set,
            ) = (step, division as i32, radius, off_set);
            state.clone()
        };
        let bounds = [-radius, -radius, -off_set, radius, radius, -off_set];
        finish_grid_group(
            &state.group,
            &state.structure,
            &self.viewer.upgrade(),
            bounds,
        );
    }
    /// `DefineLines`: the radial segments, then the circles (every tenth in the tenth color).
    fn define_lines(&self) -> Result<(), Failure> {
        let Some(values) = self.rebuild_needed(AspectGridDrawMode::Lines) else {
            return Ok(());
        };
        let group = self.state.read().group.clone();
        let Some(g) = group.get() else {
            return Ok(());
        };
        g.clear(true);
        let (color, tenth_color) = self.colors();
        add_radial_segments(g, values, tenth_color)?;
        let (lines, tenth, nbpnts) = circle_points(values);
        if !tenth.is_empty() {
            let aspect = Graphic3dAspectLine3d::new(tenth_color, AspectTypeOfLine::Solid, 1.0)?;
            g.set_group_primitives_aspect(&Handle::new(aspect).cast());
            add_polylines(g, &tenth, nbpnts)?;
        }
        if !lines.is_empty() {
            let aspect = Graphic3dAspectLine3d::new(color, AspectTypeOfLine::Solid, 1.0)?;
            g.set_primitives_aspect(&Handle::new(aspect).cast());
            add_polylines(g, &lines, nbpnts)?;
        }
        self.finish(values);
        Ok(())
    }
    /// `DefinePoints`: the center and the points of every circle.
    fn define_points(&self) -> Result<(), Failure> {
        let Some(values) = self.rebuild_needed(AspectGridDrawMode::Points) else {
            return Ok(());
        };
        let group = self.state.read().group.clone();
        let Some(g) = group.get() else {
            return Ok(());
        };
        g.clear(true);
        let marker = Graphic3dAspectMarker3d::default();
        marker.set_interior_color(self.colors().0);
        marker.set_marker_kind(AspectTypeOfMarker::Point);
        marker.set_scale_f64(3.0)?;
        let points = circular_grid_points(values);
        g.set_group_primitives_aspect(&Handle::new(marker).cast());
        if !points.is_empty() {
            let array = Graphic3dArrayOfPoints::new(points.len() as i32, 0)?;
            points
                .iter()
                .try_for_each(|p| array.add_vertex_pnt(p).map(|_| ()))?;
            g.add_primitive_array(&Handle::new(array).cast(), false);
        }
        self.finish(values);
        Ok(())
    }
    /// The one-time warning that the CPU path ignores the arc range.
    fn warn_arc(&self) {
        if !self.is_arc() || std::mem::replace(&mut self.state.write().arc_warned, true) {
            return;
        }
        let text = "V3d_CircularGrid: arc range (AngleStart/AngleEnd) is not supported by the CPU rendering path and will be ignored. Use V3d_View::GridDisplay with Aspect_GridParams::SetArcRange for arc rendering.";
        Message::send_text(&AsciiString::from(text), Gravity::Warning);
    }
    fn update_display_checked(&self) -> Result<(), Failure> {
        self.warn_arc();
        let Some(plane) = self.viewer.upgrade().get().map(V3dViewer::privileged_plane) else {
            return Ok(());
        };
        let fields = *self.base.grid().fields();
        let wanted = V3dGridPlacement {
            view_plane: plane,
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
        if current.needs_transform(defined, &plane, &wanted) {
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

/// The radial segments of `DefineLines`, in the tenth color.
fn add_radial_segments(
    g: &dyn Graphic3dGroup,
    values: [f64; 4],
    tenth_color: Color,
) -> Result<(), Failure> {
    let [_, division, radius, off_set] = values;
    let clamped = (if division >= f64::from(DEFAULT_DIVISION) {
        division
    } else {
        f64::from(DEFAULT_DIVISION)
    }) as i32;
    let nbpnts = 2 * clamped;
    // Native quirk: the angle step uses the unclamped division number.
    let alpha = PI / division;
    let aspect = Graphic3dAspectLine3d::new(tenth_color, AspectTypeOfLine::Solid, 1.0)?;
    g.set_group_primitives_aspect(&Handle::new(aspect).cast());
    let prims = Graphic3dArrayOfSegments::new(2 * nbpnts, 0, 0)?;
    let p0 = Point3::new(0.0, 0.0, -off_set);
    for i in 1..=nbpnts {
        prims.add_vertex_pnt(&p0)?;
        let angle = alpha * f64::from(i);
        prims.add_vertex_f64(angle.cos() * radius, angle.sin() * radius, -off_set)?;
    }
    g.add_primitive_array(&Handle::new(prims).cast(), false);
    Ok(())
}

/// The circle points of `DefineLines` (regular, tenth) and the points per circle.
fn circle_points(values: [f64; 4]) -> (Vec<Point3>, Vec<Point3>, i32) {
    let [step, division, radius, off_set] = values;
    let clamped = (if division >= f64::from(DEFAULT_DIVISION) {
        division
    } else {
        f64::from(DEFAULT_DIVISION)
    }) as i32;
    let nbpnts = 2 * clamped + 1;
    let alpha = PI / f64::from(clamped);
    let (mut lines, mut tenth) = (Vec::new(), Vec::new());
    let (mut r, mut nblines) = (step, 0);
    while r <= radius {
        let target = if nblines % 10 == 0 {
            &mut tenth
        } else {
            &mut lines
        };
        for i in 0..nbpnts {
            let angle = alpha * f64::from(i);
            target.push(Point3::new(angle.cos() * r, angle.sin() * r, -off_set));
        }
        (r, nblines) = (r + step, nblines + 1);
    }
    (lines, tenth, nbpnts)
}

/// Adds the circles of a point list as bounded polylines of `nbpnts` vertices each.
fn add_polylines(g: &dyn Graphic3dGroup, points: &[Point3], nbpnts: i32) -> Result<(), Failure> {
    let size = points.len() as i32;
    let nbl = size / nbpnts;
    let prims = Graphic3dArrayOfPolylines::new(size, nbl, 0, 0)?;
    let mut np = 0usize;
    for _ in 0..nbl {
        prims.add_bound(nbpnts)?;
        for _ in 0..nbpnts {
            prims.add_vertex_pnt(&points[np])?;
            np += 1;
        }
    }
    g.add_primitive_array(&Handle::new(prims).cast(), false);
    Ok(())
}

/// The point list of `DefinePoints`.
fn circular_grid_points(values: [f64; 4]) -> Vec<Point3> {
    let [step, division, radius, off_set] = values;
    let nbpnts = (2.0 * division) as i32;
    let alpha = PI / division;
    let mut points = vec![Point3::new(0.0, 0.0, -off_set)];
    let mut r = step;
    while r <= radius {
        for i in 0..nbpnts {
            let angle = alpha * f64::from(i);
            points.push(Point3::new(angle.cos() * r, angle.sin() * r, -off_set));
        }
        r += step;
    }
    points
}

impl AspectGrid for V3dCircularGrid {
    fn grid_base(&self) -> &AspectGridBase {
        self.base.grid()
    }
    fn compute(&self, x: f64, y: f64) -> (f64, f64) {
        self.base.compute(x, y)
    }
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
    /// `UpdateDisplay`; transformation failures of a degenerate plane are not reported, as
    /// in the native `void` member.
    fn update_display(&self) {
        let _ = self.update_display_checked();
    }
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
        class_begin(stream, "V3d_CircularGrid")?;
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
        field_real(stream, "myCurXo", state.placement.x_origin)?;
        field_real(stream, "myCurYo", state.placement.y_origin)?;
        field_real(stream, "myCurAngle", state.placement.angle)?;
        field_real(stream, "myCurStep", state.cur_step)?;
        field_integer(stream, "myCurDivi", state.cur_divi)?;
        field_real(stream, "myCurRadius", state.cur_radius)?;
        field_real(stream, "myCurOffSet", state.cur_off_set)
    }
}

impl AspectCircularGrid for V3dCircularGrid {
    fn circular_base(&self) -> &AspectCircularGridBase {
        &self.base
    }
}

impl V3dStructureComputer for V3dCircularGrid {
    /// `CircularGridStructure::Compute`: clears the structure and rebuilds a new group.
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

impl Drop for V3dCircularGrid {
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
    fn circles_and_points_follow_the_native_counts() {
        let (lines, tenth, nbpnts) = circle_points([1.0, 8.0, 3.0, 0.0]);
        assert_eq!(nbpnts, 17);
        assert_eq!(tenth.len(), 17);
        assert_eq!(lines.len(), 2 * 17);
        let points = circular_grid_points([1.0, 4.0, 2.0, 0.5]);
        assert_eq!(points.len(), 1 + 2 * 8);
        assert_eq!(points[0], Point3::new(0.0, 0.0, -0.5));
    }
}
