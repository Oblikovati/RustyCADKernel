//! The privileged plane members of `V3d_Viewer`.
use super::V3dViewer;
use rustycad_tkernel::quantity::{Color, NamedColor};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::gp::{Axis3, Point3};
use rustycad_tkservice::aspect::{AspectGridMembers as _, AspectTypeOfLine};
use rustycad_tkservice::graphic3d::{
    Graphic3dArrayOfSegments, Graphic3dAspectLine3d, Graphic3dAspectText3d,
    Graphic3dAspectsMembers as _, Graphic3dGroupMembers as _, Graphic3dStructure,
    Graphic3dStructureMembers as _, Graphic3dStructureSource, Graphic3dText,
};

/// The privileged plane.
impl V3dViewer {
    /// `PrivilegedPlane`; `viewer.privileged_plane()`.
    pub fn privileged_plane(&self) -> Axis3 {
        self.state.read().privileged_plane
    }
    /// `SetPrivilegedPlane`: moves the grid of every active view, redisplays the plane.
    /// Example: `viewer.set_privileged_plane(&plane)?`.
    pub fn set_privileged_plane(&self, plane: &Axis3) -> Result<(), Failure> {
        self.state.write().privileged_plane = *plane;
        let grid = self.grid(true)?;
        if let Some(value) = grid.get() {
            value.set_draw_mode(value.draw_mode());
        }
        let views = self.state.read().active_views.clone();
        views
            .iter()
            .filter_map(Handle::get)
            .for_each(|view| view.set_grid(plane, &grid));
        let (display, length) = {
            let state = self.state.read();
            (state.display_plane, state.display_plane_length)
        };
        if display {
            self.display_privileged_plane(true, length)?;
        }
        Ok(())
    }
    /// `DisplayPrivilegedPlane`: the three gray axes of the plane with their labels.
    /// Example: `viewer.display_privileged_plane(true, 100.0)?`.
    pub fn display_privileged_plane(&self, on_off: bool, size: f64) -> Result<(), Failure> {
        let existing = {
            let mut state = self.state.write();
            (state.display_plane, state.display_plane_length) = (on_off, size);
            state.plane_structure.clone()
        };
        if !on_off {
            if let Some(structure) = existing.get() {
                structure.erase();
            }
            return Ok(());
        }
        let structure = match existing.get() {
            Some(value) => {
                value.clear(true);
                existing.clone()
            }
            None => {
                let created: Handle<dyn Graphic3dStructureSource> =
                    Graphic3dStructure::new(&self.structure_manager(), &Handle::default())?.cast();
                if let Some(value) = created.get() {
                    value.set_infinite_state(true);
                    value.display();
                }
                self.state.write().plane_structure = created.clone();
                created
            }
        };
        self.fill_privileged_plane(&structure, size)?;
        if let Some(value) = structure.get() {
            value.display();
        }
        Ok(())
    }
    fn fill_privileged_plane(
        &self,
        structure: &Handle<dyn Graphic3dStructureSource>,
        length: f64,
    ) -> Result<(), Failure> {
        let Some(value) = structure.get() else {
            return Ok(());
        };
        let group = value.new_group()?;
        let Some(group) = group.get() else {
            return Ok(());
        };
        let line = Graphic3dAspectLine3d::new(
            Color::named(NamedColor::GRAY60),
            AspectTypeOfLine::Solid,
            1.0,
        )?;
        group.set_group_primitives_aspect(&Handle::new(line).cast());
        let text_aspect = Graphic3dAspectText3d::default();
        text_aspect.set_interior_color(Color::named(NamedColor::ROYALBLUE1));
        group.set_group_primitives_aspect(&Handle::new(text_aspect).cast());
        let prims = Graphic3dArrayOfSegments::new(6, 0, 0)?;
        let plane = self.privileged_plane();
        let p0 = plane.location();
        let axes = [
            (plane.x_direction(), "X"),
            (plane.y_direction(), "Y"),
            (plane.direction(), "Z"),
        ];
        for (direction, label) in axes {
            let end = Point3::from(p0.xyz() + length * direction.xyz());
            prims.add_vertex_pnt(&p0)?;
            prims.add_vertex_pnt(&end)?;
            let text = Graphic3dText::new(1.0f32 / 81.0f32);
            text.set_text_str(label);
            text.set_position(&end);
            group.add_text(&Handle::new(text), true);
        }
        group.add_primitive_array(&Handle::new(prims).cast(), true);
        Ok(())
    }
}
