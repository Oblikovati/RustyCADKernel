//! `AIS_Manipulator::Axis`: one manipulator axis with its translation arrow, scaling cube,
//! rotation ring and plane-dragging sector.
use super::manipulator_quadrics::{flat_x_direction, local_system, null_part};
use super::{AisManipulatorCube, AisManipulatorDisk, AisManipulatorSector, AisManipulatorSkin};
use crate::prs_mgr::PrsMgrPresentationManager;
use crate::prs3d::{Prs3dArrow, Prs3dShadingAspect};
use rustycad_tkernel::quantity::Color;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::gp::{self, Axis1, Axis3, Point3};
use rustycad_tkservice::graphic3d::*;

/// `Graphic3d_ArrayFlags_VertexNormal`.
const VERTEX_NORMAL: Graphic3dArrayFlags = GRAPHIC3D_ARRAY_FLAGS_VERTEX_NORMAL;
/// `Graphic3d_ArrayFlags_VertexColor`.
const VERTEX_COLOR: Graphic3dArrayFlags = GRAPHIC3D_ARRAY_FLAGS_VERTEX_COLOR;

/// `AIS_Manipulator::Axis`: the parts of one manipulator axis with their sizes, groups and
/// highlight presentations.
#[derive(Clone)]
pub struct AisManipulatorAxis {
    /// `myReferenceAxis`.
    pub reference_axis: Axis1,
    /// `myPosition`: the axis with the local transformation.
    pub position: Axis1,
    /// `myColor`.
    pub color: Color,
    /// `myHasTranslation`.
    pub has_translation: bool,
    /// `myLength`: the length of the translation axis.
    pub length: f32,
    /// `myAxisRadius`.
    pub axis_radius: f32,
    /// `myHasScaling`.
    pub has_scaling: bool,
    /// `myBoxSize`: the size of the scaling cube.
    pub box_size: f32,
    /// `myHasRotation`.
    pub has_rotation: bool,
    /// `myInnerRadius`: the radius of the rotation ring.
    pub inner_radius: f32,
    /// `myDiskThickness`.
    pub disk_thickness: f32,
    /// `myIndent`: the gap between the parts.
    pub indent: f32,
    /// `myHasDragging`.
    pub has_dragging: bool,
    /// `myFacettesNumber`.
    pub facettes_number: i32,
    /// `myArrowTipPos`.
    pub arrow_tip_pos: Point3,
    /// `mySector`.
    pub sector: AisManipulatorSector,
    /// `myCircle`.
    pub circle: AisManipulatorDisk,
    /// `myCircleRadius`.
    pub circle_radius: f32,
    /// `myCube`.
    pub cube: AisManipulatorCube,
    /// `myCubePos`.
    pub cube_pos: Point3,
    /// `myTranslatorGroup`.
    pub translator_group: Handle<dyn Graphic3dGroup>,
    /// `myScalerGroup`.
    pub scaler_group: Handle<dyn Graphic3dGroup>,
    /// `myRotatorGroup`.
    pub rotator_group: Handle<dyn Graphic3dGroup>,
    /// `myDraggerGroup`.
    pub dragger_group: Handle<dyn Graphic3dGroup>,
    /// `myHighlightTranslator`.
    pub highlight_translator: Handle<dyn Graphic3dStructureSource>,
    /// `myHighlightScaler`.
    pub highlight_scaler: Handle<dyn Graphic3dStructureSource>,
    /// `myHighlightRotator`.
    pub highlight_rotator: Handle<dyn Graphic3dStructureSource>,
    /// `myHighlightDragger`.
    pub highlight_dragger: Handle<dyn Graphic3dStructureSource>,
    /// `myTriangleArray`: the translation arrow.
    pub triangle_array: Handle<Graphic3dArrayOfTriangles>,
}
impl Default for AisManipulatorAxis {
    fn default() -> Self {
        Self::new(&Axis1::default(), Color::default(), 10.0)
    }
}
impl AisManipulatorAxis {
    /// An axis of a color and length with every part enabled.
    /// Example: `AisManipulatorAxis::new(&gp::ox(), red, 10.0)`.
    pub fn new(axis: &Axis1, color: Color, length: f32) -> Self {
        let box_size = 2.0f32;
        Self {
            reference_axis: *axis,
            position: *axis,
            color,
            has_translation: true,
            length,
            axis_radius: 0.5,
            has_scaling: true,
            box_size,
            has_rotation: true,
            inner_radius: length + box_size,
            disk_thickness: box_size * 0.5,
            indent: 0.2,
            has_dragging: true,
            facettes_number: 20,
            arrow_tip_pos: Point3::default(),
            sector: AisManipulatorSector::default(),
            circle: AisManipulatorDisk::default(),
            circle_radius: length + box_size + box_size * 0.5 * 0.5,
            cube: AisManipulatorCube::default(),
            cube_pos: Point3::default(),
            translator_group: Handle::default(),
            scaler_group: Handle::default(),
            rotator_group: Handle::default(),
            dragger_group: Handle::default(),
            highlight_translator: Handle::default(),
            highlight_scaler: Handle::default(),
            highlight_rotator: Handle::default(),
            highlight_dragger: Handle::default(),
            triangle_array: Handle::default(),
        }
    }
    /// `ReferenceAxis`; `axis.reference_axis()`.
    pub fn reference_axis(&self) -> &Axis1 {
        &self.reference_axis
    }
    /// `SetPosition`; `axis.set_position(&position)`.
    pub fn set_position(&mut self, position: &Axis1) {
        self.position = *position;
    }
    /// `Position`; `axis.position()`.
    pub fn position(&self) -> &Axis1 {
        &self.position
    }
    /// `SetTransformPersistence`: the persistence of the highlight presentations.
    /// Example: `axis.set_transform_persistence(&pers)`.
    pub fn set_transform_persistence(&self, pers: &Handle<dyn Graphic3dTransformPersSource>) {
        for prs in self.highlight_presentations() {
            if let Some(prs) = prs.get() {
                prs.set_transform_persistence(pers);
            }
        }
    }
    /// `Transform`: the transformation of the highlight presentations.
    /// Example: `axis.transform(&datum)`.
    pub fn transform(&self, transformation: &Handle<rustycad_tkmath::toploc::Datum3>) {
        for prs in self.highlight_presentations() {
            if let Some(prs) = prs.get() {
                prs.set_transformation(transformation);
            }
        }
    }
    fn highlight_presentations(&self) -> [&Handle<dyn Graphic3dStructureSource>; 4] {
        [
            &self.highlight_translator,
            &self.highlight_scaler,
            &self.highlight_rotator,
            &self.highlight_dragger,
        ]
    }
    /// `HasTranslation`; `axis.has_translation()`.
    pub fn has_translation(&self) -> bool {
        self.has_translation
    }
    /// `HasRotation`; `axis.has_rotation()`.
    pub fn has_rotation(&self) -> bool {
        self.has_rotation
    }
    /// `HasScaling`; `axis.has_scaling()`.
    pub fn has_scaling(&self) -> bool {
        self.has_scaling
    }
    /// `HasDragging`; `axis.has_dragging()`.
    pub fn has_dragging(&self) -> bool {
        self.has_dragging
    }
    /// `SetTranslation`; `axis.set_translation(false)`.
    pub fn set_translation(&mut self, enabled: bool) {
        self.has_translation = enabled;
    }
    /// `SetRotation`; `axis.set_rotation(false)`.
    pub fn set_rotation(&mut self, enabled: bool) {
        self.has_rotation = enabled;
    }
    /// `SetScaling`; `axis.set_scaling(false)`.
    pub fn set_scaling(&mut self, enabled: bool) {
        self.has_scaling = enabled;
    }
    /// `SetDragging`; `axis.set_dragging(false)`.
    pub fn set_dragging(&mut self, enabled: bool) {
        self.has_dragging = enabled;
    }
    /// `Color`; `axis.color()`.
    pub fn color(&self) -> Color {
        self.color
    }
    /// `AxisLength`; `axis.axis_length()`.
    pub fn axis_length(&self) -> f32 {
        self.length
    }
    /// `BoxSize`; `axis.box_size()`.
    pub fn box_size(&self) -> f32 {
        self.box_size
    }
    /// `AxisRadius`; `axis.axis_radius()`.
    pub fn axis_radius(&self) -> f32 {
        self.axis_radius
    }
    /// `Indent`; `axis.indent()`.
    pub fn indent(&self) -> f32 {
        self.indent
    }
    /// `SetAxisRadius`; `axis.set_axis_radius(1.0)`.
    pub fn set_axis_radius(&mut self, value: f32) {
        self.axis_radius = value;
    }
    /// `TranslatorHighlightPrs`; `axis.translator_highlight_prs()`.
    pub fn translator_highlight_prs(&self) -> &Handle<dyn Graphic3dStructureSource> {
        &self.highlight_translator
    }
    /// `RotatorHighlightPrs`; `axis.rotator_highlight_prs()`.
    pub fn rotator_highlight_prs(&self) -> &Handle<dyn Graphic3dStructureSource> {
        &self.highlight_rotator
    }
    /// `ScalerHighlightPrs`; `axis.scaler_highlight_prs()`.
    pub fn scaler_highlight_prs(&self) -> &Handle<dyn Graphic3dStructureSource> {
        &self.highlight_scaler
    }
    /// `DraggerHighlightPrs`; `axis.dragger_highlight_prs()`.
    pub fn dragger_highlight_prs(&self) -> &Handle<dyn Graphic3dStructureSource> {
        &self.highlight_dragger
    }
    /// `TranslatorGroup`; `axis.translator_group()`.
    pub fn translator_group(&self) -> &Handle<dyn Graphic3dGroup> {
        &self.translator_group
    }
    /// `RotatorGroup`; `axis.rotator_group()`.
    pub fn rotator_group(&self) -> &Handle<dyn Graphic3dGroup> {
        &self.rotator_group
    }
    /// `ScalerGroup`; `axis.scaler_group()`.
    pub fn scaler_group(&self) -> &Handle<dyn Graphic3dGroup> {
        &self.scaler_group
    }
    /// `DraggerGroup`; `axis.dragger_group()`.
    pub fn dragger_group(&self) -> &Handle<dyn Graphic3dGroup> {
        &self.dragger_group
    }
    /// `TriangleArray`; `axis.triangle_array()`.
    pub fn triangle_array(&self) -> &Handle<Graphic3dArrayOfTriangles> {
        &self.triangle_array
    }
    /// `SetIndent`; `axis.set_indent(0.5)`.
    pub fn set_indent(&mut self, value: f32) {
        self.indent = value;
    }
    /// `Size`: the outer radius of the rotation ring; `axis.size()`.
    pub fn size(&self) -> f32 {
        self.inner_radius + self.disk_thickness + self.indent * 2.0
    }
    /// `InnerRadius`: the inner radius of the rotation ring; `axis.inner_radius()`.
    pub fn inner_radius(&self) -> f32 {
        self.inner_radius + self.indent * 2.0
    }
    /// `ScalerCenter`: the center of the scaling cube from a location.
    /// Example: `axis.scaler_center(&location)`.
    pub fn scaler_center(&self, location: &Point3) -> Point3 {
        let offset = f64::from(self.length + self.indent + self.box_size * 0.5);
        Point3::from(location.xyz() + self.position.direction().xyz() * offset)
    }
    /// `SetSize`: the part sizes of a side length (the indent kept when small enough).
    /// Example: `axis.set_size(100.0)`.
    pub fn set_size(&mut self, value: f32) {
        if self.indent > value * 0.1 {
            self.length = value * 0.7;
            self.box_size = value * 0.15;
            self.disk_thickness = value * 0.05;
            self.indent = value * 0.05;
        } else {
            let length = value - 2.0 * self.indent;
            self.length = length * 0.8;
            self.box_size = length * 0.15;
            self.disk_thickness = length * 0.05;
        }
        self.inner_radius = self.indent * 2.0 + self.box_size + self.length;
        self.axis_radius = self.box_size / 4.0;
    }
    /// `FacettesNumber`; `axis.facettes_number()`.
    pub fn facettes_number(&self) -> i32 {
        self.facettes_number
    }
    /// `TranslatorTipPosition`; `axis.translator_tip_position()`.
    pub fn translator_tip_position(&self) -> &Point3 {
        &self.arrow_tip_pos
    }
    /// `DraggerSector`; `axis.dragger_sector()`.
    pub fn dragger_sector(&self) -> &AisManipulatorSector {
        &self.sector
    }
    /// `RotatorDisk`; `axis.rotator_disk()`.
    pub fn rotator_disk(&self) -> &AisManipulatorDisk {
        &self.circle
    }
    /// `RotatorDiskRadius`; `axis.rotator_disk_radius()`.
    pub fn rotator_disk_radius(&self) -> f32 {
        self.circle_radius
    }
    /// `ScalerCube`; `axis.scaler_cube()`.
    pub fn scaler_cube(&self) -> &AisManipulatorCube {
        &self.cube
    }
    /// `ScalerCubePosition`; `axis.scaler_cube_position()`.
    pub fn scaler_cube_position(&self) -> &Point3 {
        &self.cube_pos
    }

    /// `Compute`: the enabled parts of the axis in new groups of a presentation (with the
    /// aspect), each with its highlight presentation.
    /// Example: `axis.compute(&manager, &prs, &aspect, AisManipulatorSkin::Shaded)?`.
    pub fn compute(
        &mut self,
        manager: &Handle<PrsMgrPresentationManager>,
        prs: &Handle<dyn Graphic3dStructureSource>,
        aspect: &Prs3dShadingAspect,
        skin: AisManipulatorSkin,
    ) -> Result<(), Failure> {
        let manager = manager.get().ok_or_else(null_part)?;
        let structures = manager.structure_manager();
        let fill = aspect.aspect().cast::<dyn Graphic3dAspectsSource>();
        let direction = self.reference_axis.direction();
        if self.has_translation {
            self.compute_translator(prs, &fill, skin)?;
            let highlight = highlight_presentation(&mut self.highlight_translator, &structures)?;
            let group = highlight.current_group()?;
            group.set_group_primitives_aspect(&fill);
            group.add_primitive_array(&self.triangle_array.clone().cast(), true);
        }
        if self.has_scaling {
            self.cube_pos = Point3::from(direction.xyz() * f64::from(self.length + self.indent));
            let box_size = if skin == AisManipulatorSkin::Shaded {
                self.box_size
            } else {
                self.box_size * 0.5 + self.indent
            };
            self.cube
                .init(&Axis1::new(self.cube_pos, direction), box_size, skin)?;
            let group = prs.new_group()?;
            group.set_closed(skin == AisManipulatorSkin::Shaded);
            group.set_group_primitives_aspect(&fill);
            group.add_primitive_array(&self.cube.array.clone().cast(), true);
            self.scaler_group = group;
            let highlight = highlight_presentation(&mut self.highlight_scaler, &structures)?;
            let group = highlight.current_group()?;
            group.set_group_primitives_aspect(&fill);
            group.add_primitive_array(&self.cube.array.clone().cast(), true);
        }
        if self.has_rotation {
            self.circle_radius = self.inner_radius + self.indent * 2.0 + self.disk_thickness * 0.5;
            // `M_PI * 2.0f` is a double product.
            let angle = if skin == AisManipulatorSkin::Shaded {
                std::f64::consts::PI * 2.0
            } else {
                std::f64::consts::PI
            };
            let size = self.size();
            self.circle.init(
                self.inner_radius + self.indent * 2.0,
                size,
                &Axis1::new(gp::origin(), direction),
                angle,
                self.facettes_number * 2,
                20,
            )?;
            let group = prs.new_group()?;
            group.set_group_primitives_aspect(&fill);
            group.add_primitive_array(&self.circle.array().clone().cast(), true);
            self.rotator_group = group;
            let highlight = highlight_presentation(&mut self.highlight_rotator, &structures)?;
            let group = highlight.current_group()?;
            group.set_group_primitives_aspect(&fill);
            group.add_primitive_array(&self.circle.array().clone().cast(), true);
        }
        if self.has_dragging {
            let x_direction = flat_x_direction(&direction);
            let position = if skin == AisManipulatorSkin::Flat {
                Point3::from(direction.reversed().xyz() * f64::from(self.axis_radius))
            } else {
                gp::origin()
            };
            let radius = if skin == AisManipulatorSkin::Flat {
                self.length
            } else {
                self.inner_radius + self.indent * 2.0
            };
            self.sector.init(
                radius,
                &Axis1::new(position, direction),
                &x_direction,
                skin,
                self.facettes_number * 2,
                5,
            )?;
            let group = prs.new_group()?;
            let fill_area: Handle<dyn Graphic3dAspectsSource> = if skin == AisManipulatorSkin::Flat
            {
                fill.clone()
            } else {
                Handle::new(Graphic3dAspectFillArea3d::default()).cast()
            };
            group.set_group_primitives_aspect(&fill_area);
            group.add_primitive_array(&self.sector.array().clone().cast(), true);
            self.dragger_group = group;
            let highlight = highlight_presentation(&mut self.highlight_dragger, &structures)?;
            let group = highlight.current_group()?;
            group.set_group_primitives_aspect(&fill_area);
            group.add_primitive_array(&self.sector.array().clone().cast(), true);
        }
        Ok(())
    }

    fn compute_translator(
        &mut self,
        prs: &Handle<dyn Graphic3dStructureSource>,
        fill: &Handle<dyn Graphic3dAspectsSource>,
        skin: AisManipulatorSkin,
    ) -> Result<(), Failure> {
        let direction = self.reference_axis.direction();
        let arrow_length = 0.25 * f64::from(self.length);
        let cylinder_length = f64::from(self.length) - arrow_length;
        self.arrow_tip_pos = Point3::from(direction.xyz() * cylinder_length);
        let group = prs.new_group()?;
        group.set_closed(skin == AisManipulatorSkin::Shaded);
        group.set_group_primitives_aspect(fill);
        self.translator_group = group.clone();
        if skin != AisManipulatorSkin::Flat {
            self.triangle_array = Prs3dArrow::draw_shaded(
                &Axis1::new(gp::origin(), direction),
                f64::from(self.axis_radius),
                f64::from(self.length),
                f64::from(self.axis_radius) * 1.5,
                arrow_length,
                self.facettes_number,
            )?;
            group.add_primitive_array(&self.triangle_array.clone().cast(), true);
            return Ok(());
        }
        const STRIPS: i32 = 14;
        let array = Handle::new(Graphic3dArrayOfTriangles::new(
            STRIPS * 4,
            STRIPS * 6,
            VERTEX_NORMAL,
        )?);
        let colorless = Handle::new(Graphic3dArrayOfTriangles::new(
            STRIPS * 2,
            STRIPS * 3,
            VERTEX_NORMAL,
        )?);
        let colored = Handle::new(Graphic3dArrayOfTriangles::new(
            STRIPS * 2,
            STRIPS * 3,
            VERTEX_NORMAL | VERTEX_COLOR,
        )?);
        let trsf = local_system(Axis3::from_normal(gp::origin(), direction)?);
        let normal = gp::dx().transformed(trsf);
        let length = f64::from(self.length + self.indent * 4.0);
        let step_v = f64::from(1.0f32 / STRIPS as f32);
        for u in 0..=1 {
            for v in 0..=STRIPS {
                let vertex = Point3::new(
                    0.0,
                    f64::from(self.axis_radius * (1.5 * u as f32 - 0.75)),
                    length * f64::from(v) * step_v,
                )
                .transformed(trsf);
                array.add_vertex_pnt_normal(&vertex, &normal)?;
                if v != 0 {
                    colorless.add_vertex_pnt_normal(&vertex, &normal)?;
                }
                if v != STRIPS {
                    colored.add_vertex_pnt_normal_color(&vertex, &normal, self.color)?;
                }
                if u != 0 && v != 0 {
                    let id = array.vertex_number();
                    array.add_triangle_edges(id, id - STRIPS - 2, id - 1)?;
                    array.add_triangle_edges(id - STRIPS - 2, id, id - STRIPS - 1)?;
                    let squares = if v % 2 == 0 { &colorless } else { &colored };
                    let id = squares.vertex_number();
                    squares.add_triangle_edges(id, id - STRIPS - 1, id - 1)?;
                    squares.add_triangle_edges(id - STRIPS - 1, id, id - STRIPS)?;
                }
            }
        }
        self.triangle_array = array;
        group.add_primitive_array(&colored.cast(), true);
        group.add_primitive_array(&colorless.cast(), true);
        Ok(())
    }
}

/// A highlight presentation of the structure manager, created once then cleared.
fn highlight_presentation(
    slot: &mut Handle<dyn Graphic3dStructureSource>,
    manager: &Handle<Graphic3dStructureManager>,
) -> Result<Handle<dyn Graphic3dStructureSource>, Failure> {
    if let Some(prs) = slot.get() {
        prs.clear(true);
    } else {
        *slot = Graphic3dStructure::new(manager, &Handle::default())?.cast();
    }
    Ok(slot.clone())
}
