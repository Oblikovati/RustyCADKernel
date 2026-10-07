//! `BOPAlgo_MakeConnected` origins and material associations, and their update after the
//! periodicity operations.
use super::{BOPAlgoMakeConnected, null_history};
use crate::bop_tools::treat_compound;
use rustycad_tkbrep::brep_tools::History;
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkbrep::toptools::MapOfShape;
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};

impl BOPAlgoMakeConnected {
    /// The input shapes on the side of `shape` its forward orientation points away from
    /// (`MaterialsOnPositiveSide`); `maker.materials_on_positive_side(&face)`.
    pub fn materials_on_positive_side(&self, shape: &Shape) -> &List<Shape> {
        self.materials_of(&shape.oriented(Orientation::Forward))
    }

    /// The input shapes on the other side of `shape` (`MaterialsOnNegativeSide`);
    /// `maker.materials_on_negative_side(&face)`.
    pub fn materials_on_negative_side(&self, shape: &Shape) -> &List<Shape> {
        self.materials_of(&shape.oriented(Orientation::Reversed))
    }

    fn materials_of(&self, element: &Shape) -> &List<Shape> {
        self.materials.seek(element).unwrap_or(&self.no_shapes)
    }

    /// The shapes `shape` became; empty before `perform` (`GetModified`);
    /// `maker.get_modified(&face)?`.
    pub fn get_modified(&self, shape: &Shape) -> Result<Vec<Shape>, Failure> {
        match self.history.get() {
            Some(history) => history.modified(shape),
            None => Ok(Vec::new()),
        }
    }

    /// The input shapes a result shape comes from (`GetOrigins`); `maker.get_origins(&face)`.
    pub fn get_origins(&self, shape: &Shape) -> &List<Shape> {
        self.origins.seek(shape).unwrap_or(&self.no_shapes)
    }

    /// Each modified or generated image of a supported input sub-shape gets that input
    /// as an origin (`FillOrigins`).
    pub(super) fn fill_origins(&mut self) -> Result<(), Failure> {
        self.origins.clear(false);
        if self.all_inputs.is_empty() {
            for argument in self.arguments.iter() {
                topexp::map_shapes(argument, &mut self.all_inputs, true, true);
            }
        }
        let history = self.history.get().ok_or_else(null_history)?;
        for position in 1..=self.all_inputs.extent() {
            let input = self.all_inputs.find_key(position)?;
            if !History::is_supported_type(input)? {
                continue;
            }
            let images = history
                .modified(input)?
                .into_iter()
                .chain(history.generated(input)?);
            for image in images {
                add_origin(&mut self.origins, image, input);
            }
        }
        Ok(())
    }

    /// Each boundary element of each result shape gets the origin of that shape as a
    /// material, by element orientation (`AssociateMaterials`).
    pub(super) fn associate_materials(&mut self) -> Result<(), Failure> {
        self.materials.clear(false);
        let (mut shapes, mut fence) = (Vec::new(), MapOfShape::default());
        treat_compound(&self.periodic, &mut shapes, Some(&mut fence))?;
        let Some(first) = shapes.first() else {
            return Ok(());
        };
        let Some(element_type) = boundary_type(first.shape_type()?) else {
            return Ok(());
        };
        for shape in &shapes {
            let material = self.get_origins(shape).first().unwrap_or(shape).clone();
            let mut explorer = Explorer::new(shape, element_type, ShapeEnum::Shape)?;
            while explorer.more() {
                let element = explorer.current().clone();
                match self.materials.change_seek(&element) {
                    Some(list) => {
                        list.append(material.clone());
                    }
                    None => {
                        self.materials
                            .bound(element, List::default())
                            .append(material.clone());
                    }
                }
                explorer.next()?;
            }
        }
        Ok(())
    }

    /// The history rebuilt from the gluing and periodicity histories, then the origins and
    /// materials of the current shape (`Update`).
    pub(super) fn update(&mut self) -> Result<(), Failure> {
        let history = self.history.get().ok_or_else(null_history)?;
        history.clear();
        history.merge_handle(&self.glue_history)?;
        history.merge_handle(self.periodicity_maker.history())?;
        self.fill_origins()?;
        self.associate_materials()
    }
}

/// Adds `input` to the origins of `image` unless already there.
fn add_origin(origins: &mut super::OriginMap, image: Shape, input: &Shape) {
    let Some(list) = origins.change_seek(&image) else {
        origins.bound(image, List::default()).append(input.clone());
        return;
    };
    if !list.contains(input) {
        list.append(input.clone());
    }
}

/// The type of the elements bounding a material of the type: faces of solids, edges of
/// faces, vertices of edges; None for other types.
fn boundary_type(material: ShapeEnum) -> Option<ShapeEnum> {
    match material {
        ShapeEnum::Solid | ShapeEnum::CompSolid => Some(ShapeEnum::Face),
        ShapeEnum::Face | ShapeEnum::Shell => Some(ShapeEnum::Edge),
        ShapeEnum::Edge | ShapeEnum::Wire => Some(ShapeEnum::Vertex),
        _ => None,
    }
}
