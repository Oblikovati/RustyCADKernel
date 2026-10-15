// Rust translation of ModelingAlgorithms/TKFeat/BRepFeat/BRepFeat_MakeCylindricalHole.cxx
// and .lxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepFeat_MakeCylindricalHole`: a cylindrical hole along an axis, through all, up to
//! the next face, up to the end, between two parameters or blind of a given depth, cut
//! by the feature Boolean builder keeping the right parts of the cylinder.
mod perform;
mod statics;

use super::{BRepFeatBuilder, Status};
use crate::errors::construction_error;
use crate::loc_ope::CurveShapeIntersector;
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{Face, Shape, Solid};
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::Axis1;
use statics::{HoleCylinder, baryc};

/// `BRepFeat_MakeCylindricalHole`: the hole of a shape along an axis.
pub struct BRepFeatMakeCylindricalHole {
    builder: BRepFeatBuilder,
    axis: Axis1,
    ax_def: bool,
    status: Status,
    is_blind: bool,
    validate: bool,
    top_face: Face,
    bot_face: Face,
}

impl Default for BRepFeatMakeCylindricalHole {
    fn default() -> Self {
        Self::new()
    }
}

impl BRepFeatMakeCylindricalHole {
    /// `BRepFeat_MakeCylindricalHole()`: no axis yet; `BRepFeatMakeCylindricalHole::new()`.
    pub fn new() -> Self {
        Self {
            builder: BRepFeatBuilder::new(),
            axis: Axis1::default(),
            ax_def: false,
            status: Status::NoError,
            is_blind: false,
            validate: false,
            top_face: Face::default(),
            bot_face: Face::default(),
        }
    }

    /// `Init(Axis)`: the axis of the hole; `hole.init_axis(axis)`.
    pub fn init_axis(&mut self, axis: Axis1) {
        self.axis = axis;
        self.ax_def = true;
    }

    /// `Init(S, Axis)`: the shape to drill and the axis; `hole.init(&shape, axis)`.
    pub fn init(&mut self, s: &Shape, axis: Axis1) {
        self.builder.init(s);
        self.axis = axis;
        self.ax_def = true;
    }

    /// `Status()`: why the hole failed (NoError when built); `hole.status()`.
    pub fn status(&self) -> Status {
        self.status
    }

    /// The feature Boolean builder (`BRepFeat_Builder` base): its result and history;
    /// `hole.builder().shape()`.
    pub fn builder(&self) -> &BRepFeatBuilder {
        &self.builder
    }

    /// `Shape()`: the drilled shape after `build`; `hole.shape()`.
    pub fn shape(&self) -> &Shape {
        self.builder.shape()
    }

    /// The shape to drill, checked with the axis (Standard_ConstructionError without),
    /// the flags of the new hole set (the validation kept when `cont` is None), and the
    /// intersections of the axis with the shape.
    fn start(&mut self, blind: bool, cont: Option<bool>) -> Result<CurveShapeIntersector, Failure> {
        let a_object = self
            .builder
            .arguments()
            .first()
            .cloned()
            .unwrap_or_default();
        if a_object.is_null() || !self.ax_def {
            return Err(construction_error(
                "BRepFeat_MakeCylindricalHole::Perform",
                "no shape or no axis",
            ));
        }
        self.is_blind = blind;
        if let Some(cont) = cont {
            self.validate = cont;
        }
        self.status = Status::NoError;
        CurveShapeIntersector::with_axis(&self.axis, &a_object)
    }

    /// The tool solid of the cylinder cut from the shape, its faces kept.
    fn cut_by(&mut self, cyl: HoleCylinder, keep_bottom: bool) -> Result<(), Failure> {
        let b = BRepBuilder::default();
        let mut the_tool = Solid::default();
        b.make_solid(&mut the_tool);
        b.add(&mut the_tool, &cyl.shell)?;
        self.top_face = cyl.top;
        if keep_bottom {
            self.bot_face = cyl.bottom;
        } else {
            self.bot_face.nullify();
        }
        self.builder.add_tool(&the_tool);
        self.builder.set_operation(0);
        self.builder.perform();
        Ok(())
    }

    /// The parts of the cylinder in the result with the parameters of their barycentres
    /// along the axis when there are several (`nbparts >= 2`); InvalidPlacement (None)
    /// without part.
    fn parts(&mut self) -> Result<Option<Vec<(Shape, f64)>>, Failure> {
        let mut parts = List::default();
        self.builder.parts_of_tool(&mut parts)?;
        if parts.is_empty() {
            self.status = Status::InvalidPlacement;
            return Ok(None);
        }
        let mut with_par = Vec::new();
        if parts.len() >= 2 {
            for p in parts.iter() {
                with_par.push((p.clone(), elclib::line_parameter(self.axis, baryc(p)?)));
            }
        }
        Ok(Some(with_par))
    }

    /// Keeps the part `tokeep` (the first one the same) when not null.
    fn keep_one(&mut self, parts: &[(Shape, f64)], tokeep: &Shape) -> Result<(), Failure> {
        if let Some((p, _)) = parts.iter().find(|(p, _)| tokeep.is_same(p)) {
            self.builder.keep_part(p)?;
        }
        Ok(())
    }

    /// `Build()`: the result of the hole when it was placed, validated on request (its
    /// status then telling why it failed); `hole.build()?`.
    pub fn build(&mut self) -> Result<(), Failure> {
        if self.status != Status::NoError {
            return Ok(());
        }
        self.builder.perform_result()?;
        if self.builder.has_errors() {
            self.status = Status::InvalidPlacement; // why not
            return Ok(());
        }
        self.status = if self.validate {
            self.validate_hole()?
        } else {
            Status::NoError
        };
        Ok(())
    }

    /// `Validate()`: a blind hole must keep the top face of its cylinder
    /// (HoleTooLong), another one neither its top nor its bottom face (InvalidPlacement).
    fn validate_hole(&self) -> Result<Status, Failure> {
        let faces: Vec<Shape> =
            Explorer::new(self.builder.shape(), ShapeEnum::Face, ShapeEnum::Shape)?.collect();
        if self.is_blind {
            // limit of the hole
            if !faces.iter().any(|f| f.is_same(&self.top_face)) {
                return Ok(Status::HoleTooLong);
            }
            return Ok(Status::NoError);
        }
        if faces.iter().any(|f| f.is_same(&self.top_face))
            || faces.iter().any(|f| f.is_same(&self.bot_face))
        {
            return Ok(Status::InvalidPlacement);
        }
        Ok(Status::NoError)
    }
}
