//! Recursive area checking preserves native working-face and context mutations.
use super::{FaceDivider, ShapeUpgradeFaceDivideArea, ShapeUpgradeSplitSurfaceArea};
use crate::shape_extend::ShapeExtendStatus;
use rustycad_tkbrep::{
    brep::{BRepBuilder, BRepTool},
    brep_tools::ReShape,
    topexp::Explorer,
    topods::{self, Shape},
};
use rustycad_tkernel::{
    precision,
    standard::{Failure, Handle, real::real_to_int},
};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkgeombase::gprop::GProps;
use rustycad_tktopalgo::brep_gprop::BRepGProp;

impl ShapeUpgradeFaceDivideArea {
    pub(super) fn perform_area_division(&self) -> Result<bool, Failure> {
        self.face_divide_data().with_mut(|fields| fields.status = 0);
        let area = self.measure_area()?;
        let Some(_surface_tool) = self.configure_area_tool(area)? else {
            return Ok(false);
        };
        if !super::super::face_divide::perform_face_division(self, area)? {
            return Ok(false);
        }
        let result = self.result();
        if result.shape_type()? == ShapeEnum::Face {
            return Ok(false);
        }
        self.finish_area_result(&result)
    }
    fn finish_area_result(&self, result: &Shape) -> Result<bool, Failure> {
        let status = self.face_divide_data().with(|fields| fields.status);
        if !self.settings.read().by_number {
            self.refine_area_faces(result)?;
        }
        self.face_divide_data()
            .with_mut(|fields| fields.status |= status);
        let applied = self.context().apply(result, ShapeEnum::Shape)?;
        self.face_divide_data()
            .with_mut(|fields| fields.result = applied);
        Ok(self.status_flag(ShapeExtendStatus::Done))
    }
    fn measure_area(&self) -> Result<f64, Failure> {
        let face = self
            .face_divide_data()
            .with(|fields| fields.face.clone().into_shape());
        let mut properties = GProps::default();
        BRepGProp::surface_properties_eps(&face, &mut properties, self.precision(), false)?;
        Ok(properties.mass())
    }
    fn requested_parts(&self, area: f64) -> Option<i32> {
        let mut settings = self.settings.write();
        let parts = if settings.max_area == -1. {
            settings.max_area = area / f64::from(settings.parts);
            settings.parts
        } else {
            0
        };
        if area - settings.max_area < precision::confusion() {
            return None;
        }
        Some(if parts == 0 {
            real_to_int((area / settings.max_area).ceil())
        } else {
            parts
        })
    }
    fn configure_area_tool(
        &self,
        area: f64,
    ) -> Result<Option<Handle<ShapeUpgradeSplitSurfaceArea>>, Failure> {
        let Some(parts) = self.requested_parts(area) else {
            return Ok(None);
        };
        let surface = self
            .split_surface_tool()
            .cast::<ShapeUpgradeSplitSurfaceArea>();
        if surface.is_null() {
            return Ok(None);
        }
        surface.set_nb_parts(parts);
        let settings = *self.settings.read();
        if settings.by_number {
            surface.set_splitting_into_squares(true);
            surface.set_numbers_uv_splits(settings.u_splits, settings.v_splits)?;
        }
        Ok(Some(surface))
    }
    fn refine_area_faces(&self, result: &Shape) -> Result<(), Failure> {
        let mut faces = Explorer::new(result, ShapeEnum::Face, ShapeEnum::Shape)?;
        let mut copy = result.empty_copied()?;
        let mut modified = false;
        while faces.more() {
            modified |= self.refine_area_face(faces.current(), &mut copy)?;
            faces.next()?;
        }
        if !modified {
            return Ok(());
        }
        if matches!(copy.shape_type()?, ShapeEnum::Wire | ShapeEnum::Shell) {
            copy.set_closed(BRepTool::is_closed(&copy)?)?;
        }
        self.context().replace(result, &copy)
    }
    fn refine_area_face(&self, original: &Shape, copy: &mut Shape) -> Result<bool, Failure> {
        let applied = self.context().apply(original, ShapeEnum::Shape)?;
        self.init(&topods::face(&applied)?);
        if !self.perform(0.)? {
            BRepBuilder::default().add(copy, &applied)?;
            return Ok(false);
        }
        let mut faces = Explorer::new(&self.result(), ShapeEnum::Face, ShapeEnum::Shape)?;
        while faces.more() {
            BRepBuilder::default().add(copy, faces.current())?;
            faces.next()?;
        }
        Ok(true)
    }
}
