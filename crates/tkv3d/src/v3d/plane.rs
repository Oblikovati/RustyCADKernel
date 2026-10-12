//! `V3d_Plane`: a clipping plane with an optional hatched presentation.
use crate::v3d::V3dViewSource;
use rustycad_tkernel::quantity::Color;
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkmath::gp::{Plane3, Transform3};
use rustycad_tkmath::toploc::Datum3;
use rustycad_tkservice::aspect::{AspectHatchStyle, AspectInteriorStyle};
use rustycad_tkservice::graphic3d::{
    Graphic3dArrayOfQuadrangles, Graphic3dAspectFillArea3d, Graphic3dAspectsEdgeMembers as _,
    Graphic3dAspectsMembers as _, Graphic3dClipPlane, Graphic3dDisplayPriority,
    Graphic3dGroupMembers as _, Graphic3dHatchStyle, Graphic3dMaterialAspect,
    Graphic3dNameOfMaterial, Graphic3dStructure, Graphic3dStructureDisplay as _,
    Graphic3dStructureMembers as _, Graphic3dStructureSource,
};

/// A clipping plane `Ax + By + Cz + D = 0` that a view can display.
///
/// Example: `let plane = V3dPlane::new(0.0, 0.0, 1.0, 0.0)?; plane.display(&view, gray)?;`.
pub struct V3dPlane {
    transient: StandardTransient,
    graphic_structure: SharedState<Handle<dyn Graphic3dStructureSource>>,
    plane: Handle<Graphic3dClipPlane>,
}
rustycad_tkernel::impl_transient!(V3dPlane, "V3d_Plane", StandardTransient, transient);

impl V3dPlane {
    /// A plane of equation coefficients; a null normal raises `Standard_ConstructionError`.
    /// Example: `V3dPlane::new(0.0, 0.0, 1.0, 0.0)?`.
    pub fn new(a: f64, b: f64, c: f64, d: f64) -> Result<Self, Failure> {
        let clip = Graphic3dClipPlane::from_plane(&Plane3::from_equation(a, b, c, d)?)?;
        Ok(Self {
            transient: StandardTransient::default(),
            graphic_structure: SharedState::new(Handle::default()),
            plane: Handle::new(clip),
        })
    }
    /// `SetPlane`: changes the equation and updates a displayed presentation.
    /// Example: `plane.set_plane(1.0, 0.0, 0.0, -2.0)?`.
    pub fn set_plane(&self, a: f64, b: f64, c: f64, d: f64) -> Result<(), Failure> {
        if let Some(clip) = self.plane.get() {
            clip.set_plane(&Plane3::from_equation(a, b, c, d)?)?;
        }
        if self.is_displayed() {
            self.update()?;
        }
        Ok(())
    }
    /// `Display`: a hatched transparent square of half the default view size, at the bottom.
    /// Example: `plane.display(&view, gray)?`.
    pub fn display(&self, view: &Handle<dyn V3dViewSource>, color: Color) -> Result<(), Failure> {
        let Some(viewer) = view
            .get()
            .map(|view| view.viewer())
            .filter(|v| !v.is_null())
        else {
            return Ok(());
        };
        if let Some(previous) = self.structure().get() {
            previous.clear(true);
        }
        let structure: Handle<dyn Graphic3dStructureSource> =
            Graphic3dStructure::new(&viewer.structure_manager(), &Handle::default())?.cast();
        let size = (0.5 * viewer.default_view_size()) as f32;
        fill_plane_group(&structure, color, size)?;
        if let Some(value) = structure.get() {
            value.set_display_priority(Graphic3dDisplayPriority::Bottom)?;
            value.display();
        }
        *self.graphic_structure.write() = structure;
        self.update()
    }
    /// `Erase`: hides the presentation; `plane.erase()`.
    pub fn erase(&self) {
        if let Some(structure) = self.structure().get() {
            structure.erase();
        }
    }
    /// `Plane`: the equation coefficients `[A, B, C, D]`; `plane.plane()`.
    pub fn plane(&self) -> [f64; 4] {
        self.plane.get().map_or([0.0; 4], |clip| {
            let equation = clip.get_equation();
            [equation.x, equation.y, equation.z, equation.w]
        })
    }
    /// `IsDisplayed`; `plane.is_displayed()`.
    pub fn is_displayed(&self) -> bool {
        self.structure().get().is_some_and(|s| s.is_displayed())
    }
    /// `ClipPlane`: the clip plane; `plane.clip_plane()`.
    pub fn clip_plane(&self) -> Handle<Graphic3dClipPlane> {
        self.plane.clone()
    }
    /// The presentation structure (`myGraphicStructure`, null before the first display).
    pub fn structure(&self) -> Handle<dyn Graphic3dStructureSource> {
        self.graphic_structure.read().clone()
    }
    /// `Update`: places the presentation in the plane frame.
    fn update(&self) -> Result<(), Failure> {
        let structure = self.structure();
        let (Some(value), Some(clip)) = (structure.get(), self.plane.get()) else {
            return Ok(());
        };
        let mut transform = Transform3::default();
        transform.set_transformation_to(clip.to_plane().position());
        transform.invert()?;
        value.set_transformation(&Handle::new(Datum3::new(transform)));
        Ok(())
    }
}

/// The hatched, half transparent plastic quadrangle of `Display`.
fn fill_plane_group(
    structure: &Handle<dyn Graphic3dStructureSource>,
    color: Color,
    size: f32,
) -> Result<(), Failure> {
    let Some(value) = structure.get() else {
        return Ok(());
    };
    let group = value.new_group()?;
    let aspect = Graphic3dAspectFillArea3d::default();
    let mut plastic = Graphic3dMaterialAspect::new(Graphic3dNameOfMaterial::Plastified);
    plastic.set_color(color)?;
    plastic.set_transparency(0.5)?;
    aspect.set_front_material(plastic);
    aspect.set_interior_style(AspectInteriorStyle::Hatch);
    aspect.set_hatch_style(Handle::new(Graphic3dHatchStyle::predefined(
        AspectHatchStyle::GridDiagonalWide,
    )));
    let Some(group) = group.get() else {
        return Ok(());
    };
    group.set_group_primitives_aspect(&Handle::new(aspect).cast());
    let offset = size / 5000.0f32;
    let prims = Graphic3dArrayOfQuadrangles::new(4, 0, 0)?;
    for (x, y) in [(-size, -size), (-size, size), (size, size), (size, -size)] {
        prims.add_vertex(x, y, offset)?;
    }
    group.add_primitive_array(&Handle::new(prims).cast(), true);
    Ok(())
}
