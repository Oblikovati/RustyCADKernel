//! `BOPAlgo_BOP::CheckData`, `TreatEmptyShape` and `BuildResult` (BOPAlgo_BOP.cxx).
use super::BOPAlgoBOP;
use crate::bop_algo::alerts::alert;
use crate::bop_algo::{
    AlertBOPNotAllowed, AlertBOPNotSet, AlertEmptyShape, AlertTooFewArguments, BOPAlgoPaveFiller,
    Operation,
};
use crate::bop_ds::DataStructure;
use crate::bop_tools::algo_tools3d::is_empty_shape;
use crate::bop_tools::dimensions;
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::message::AlertHandle;
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::{Failure, RuntimeTyped};
use rustycad_tkg3d::top_abs::ShapeEnum;
use std::sync::Arc;

/// The dimension bounds of the non-empty shapes of one group.
#[derive(Clone, Copy)]
struct GroupDimensions {
    min: i32,
    max: i32,
    valid: bool,
}

impl BOPAlgoBOP {
    /// Checks the operation and the groups (`CheckData`): a known operation, both groups
    /// non-empty, homogeneous dimensions for FUSE, and the dimension rules of CUT/CUT21;
    /// empty shapes are warned about and the result dimensions stored.
    pub(super) fn check_operation_data(
        &mut self,
        filler: &BOPAlgoPaveFiller,
    ) -> Result<(), Failure> {
        if !matches!(
            self.operation,
            Operation::Common | Operation::Fuse | Operation::Cut | Operation::Cut21
        ) {
            self.add_error(alert(AlertBOPNotSet));
            return Ok(());
        }
        if self.provider.builder.arguments.is_empty() || self.provider.tools().is_empty() {
            self.add_error(alert(AlertTooFewArguments));
            return Ok(());
        }
        self.provider.builder.check_filler(filler);
        if self.builder().options().has_errors() {
            return Ok(());
        }
        let groups = [
            self.provider.builder.arguments.clone(),
            self.provider.tools().clone(),
        ];
        let mut bounds = [GroupDimensions {
            min: 3,
            max: 0,
            valid: false,
        }; 2];
        for (side, group) in groups.iter().enumerate() {
            if !self.group_dimensions(group, &mut bounds[side])? {
                return Ok(());
            }
        }
        let [arguments, tools] = bounds;
        if arguments.valid && tools.valid && self.dimensions_not_allowed(arguments, tools) {
            self.add_error(alert(AlertBOPNotAllowed));
            return Ok(());
        }
        if arguments.valid || tools.valid {
            // A group of empty shapes only takes the dimension of the other group.
            self.dims[0] = if arguments.valid {
                arguments.min
            } else {
                tools.min
            };
            self.dims[1] = if tools.valid {
                tools.min
            } else {
                arguments.min
            };
        }
        Ok(())
    }

    /// Accumulates the dimensions of the non-empty shapes; false after an error.
    fn group_dimensions(
        &mut self,
        group: &List<Shape>,
        bounds: &mut GroupDimensions,
    ) -> Result<bool, Failure> {
        for shape in group.iter() {
            if is_empty_shape(shape)? {
                let warning: AlertHandle = Arc::new(AlertEmptyShape::new(shape));
                self.builder().options().add_warning(&warning);
                continue;
            }
            let (mut min, mut max) = (0, 0);
            dimensions(shape, &mut min, &mut max)?;
            bounds.min = bounds.min.min(min);
            bounds.max = bounds.max.max(max);
            if self.operation == Operation::Fuse && bounds.min != bounds.max {
                // Non-homogeneous argument.
                self.add_error(alert(AlertBOPNotAllowed));
                return Ok(false);
            }
            bounds.valid = true;
        }
        Ok(true)
    }

    fn dimensions_not_allowed(&self, arguments: GroupDimensions, tools: GroupDimensions) -> bool {
        match self.operation {
            Operation::Fuse => arguments.max != tools.max,
            Operation::Cut => arguments.max > tools.min,
            Operation::Cut21 => arguments.min < tools.max,
            _ => false,
        }
    }

    fn add_error(&self, error: AlertHandle) {
        self.builder().options().add_error(&error);
    }

    /// Whether `CheckData` warned about an empty shape.
    pub(super) fn has_empty_shape_warning(&self) -> bool {
        self.builder()
            .options()
            .report()
            .has_alert(AlertEmptyShape::static_type())
    }

    /// Builds the result right away when a group has only empty shapes and the other needs
    /// no splitting (`TreatEmptyShape`); true when the result is complete.
    pub(super) fn treat_empty_shape(&mut self) -> Result<bool, Failure> {
        let objects = non_empty(&self.provider.builder.arguments)?;
        let tools = non_empty(self.provider.tools())?;
        let (has_objects, has_tools) = (!objects.is_empty(), !tools.is_empty());
        if has_objects && has_tools {
            return Ok(false);
        }
        if !has_objects && !has_tools {
            // All shapes are empty: the result is empty.
            return Ok(true);
        }
        let result = match self.operation {
            Operation::Fuse if objects.len() + tools.len() > 1 => return Ok(false),
            Operation::Fuse => Some(if has_objects { objects } else { tools }),
            Operation::Cut if objects.len() > 1 => return Ok(false),
            Operation::Cut => Some(objects),
            Operation::Cut21 if tools.len() > 1 => return Ok(false),
            Operation::Cut21 => Some(tools),
            _ => None,
        };
        let builder = BRepBuilder::default();
        for shape in result.unwrap_or_default() {
            builder.add(&mut self.provider.builder.shape, &shape)?;
        }
        Ok(true)
    }

    /// Adds the images of the DS arguments of `kind` (or the arguments) once each
    /// (`BuildResult`); `bop.build_result_of_arguments(ShapeEnum::Solid, &ds)?`.
    pub fn build_result_of_arguments(
        &mut self,
        kind: ShapeEnum,
        structure: &DataStructure,
    ) -> Result<(), Failure> {
        self.provider
            .builder
            .add_argument_images(structure.arguments(), kind)
    }
}

fn non_empty(shapes: &List<Shape>) -> Result<Vec<Shape>, Failure> {
    let mut valid = Vec::new();
    for shape in shapes.iter() {
        if !is_empty_shape(shape)? {
            valid.push(shape.clone());
        }
    }
    Ok(valid)
}
