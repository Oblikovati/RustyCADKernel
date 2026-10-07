// Rust translation of Draw/TKDraw/DBRep/DBRep.hxx and DBRep::Set/getShape of
// DBRep.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The `DBRep` package of TKDraw: shapes in DRAW variables (`DBRep_DrawableShape`, its
//! iso builder and hidden-line data) and the basic shape commands.
mod commands;
mod drawable_shape;
mod edge_face;
mod fill_gaps;
mod hide_data;
mod iso_builder;
mod params;

pub use commands::basic_commands;
pub use drawable_shape::{
    DBRepDrawableShape, HlrSettings, LastShapePick, MeshNormals, NormalSegment, ShapeColors,
    color_orientation, last_pick,
};
pub use edge_face::{DBRepEdge, DBRepFace, FaceIso};
pub use hide_data::DBRepHideData;
pub use iso_builder::{DBRepIsoBuilder, EdgePCurves};
pub use params::{DBRepParams, parameters};

use crate::draw::{DrawConsole, DrawSession};
use params::current;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{self, ShapeEnum};
use std::rc::Rc;

/// `DBRep_WriteColorOrientation`: the legend of the orientation colors.
pub fn write_color_orientation(out: &dyn DrawConsole) {
    out.write_out("\nrouge  FORWARD\nbleu   REVERSED\nrose   EXTERNAL\norange INTERNAL\n");
}

impl DrawSession {
    /// `DBRep::Set(name, S)`: a drawable shape with the standard colors and the current
    /// `DBRep::Parameters()`.
    pub fn set_shape(&self, name: &str, shape: &Shape) -> Result<(), Failure> {
        let params = current(self);
        let drawable = DBRepDrawableShape::new(
            shape.clone(),
            ShapeColors::standard(),
            params.size,
            params.nb_isos,
            params.discretization,
        );
        drawable.set_display_triangulation(params.disp_triangles);
        drawable.set_display_polygons(params.display_polygons);
        drawable.display_hlr(HlrSettings {
            with_hlr: params.with_hlr,
            with_rg1: params.with_rg1,
            with_rgn: params.with_rgn,
            with_hid: params.with_hid,
            angle: params.hlr_angle,
        });
        self.set_drawable_auto(name, Some(Rc::new(drawable)))
    }

    /// `DBRep::Get(name, type, complain)` (`getShape`): the shape of a shape variable
    /// when it is of `kind` (`ShapeEnum::Shape` for any); a null shape otherwise, with
    /// a message on the console when `complain`. For a picked name (`.`) of the wrong
    /// type the sub-shape of the last pick is taken.
    pub fn get_shape(&self, name: &str, kind: ShapeEnum, complain: bool) -> Result<Shape, Failure> {
        let to_pick = name.starts_with('.');
        let Some(drawable) = self.get(name) else {
            return Ok(Shape::default());
        };
        let Some(drawable) = drawable.downcast_kind::<DBRepDrawableShape>() else {
            return Ok(Shape::default());
        };
        let mut shape = drawable.shape();
        if kind != ShapeEnum::Shape && kind != shape.shape_type()? && to_pick {
            shape = last_pick().shape;
        }
        if kind != ShapeEnum::Shape && kind != shape.shape_type()? {
            if complain {
                let (wanted, found) = (
                    top_abs::shape_type_to_string(kind),
                    top_abs::shape_type_to_string(shape.shape_type()?),
                );
                self.console()
                    .write_out(&format!("{name} is not a {wanted} but a {found}\n"));
            }
            return Ok(Shape::default());
        }
        Ok(shape)
    }

    /// `DBRep::GetExisting(name, type, complain)`: as [`Self::get_shape`] without
    /// picking (`.` gives a null shape).
    pub fn get_existing_shape(
        &self,
        name: &str,
        kind: ShapeEnum,
        complain: bool,
    ) -> Result<Shape, Failure> {
        if name == "." {
            return Ok(Shape::default());
        }
        self.get_shape(name, kind, complain)
    }

    /// `DBRep::Get(name)`: any shape.
    pub fn shape(&self, name: &str) -> Result<Shape, Failure> {
        self.get_shape(name, ShapeEnum::Shape, false)
    }
}

#[cfg(test)]
mod tests;
