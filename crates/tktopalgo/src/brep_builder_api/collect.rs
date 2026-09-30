// Rust translation of BRepBuilderAPI_Collect.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepBuilderAPI_Collect`: the history of a chain of constructions, each shape of the
//! first input traced to its images (modifications and generations) through every step.
use super::make_shape::BRepBuilderAPIMakeShape;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkbrep::toptools::{MapOfShape, ShapeMapHasher};
use rustycad_tkernel::ncollection::{DataMap, List};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;

/// `NCollection_DataMap<TopoDS_Shape, NCollection_List<TopoDS_Shape>,
/// TopTools_ShapeMapHasher>`: each initial shape with its images.
pub type ShapeHistoryMap = DataMap<Shape, List<Shape>, ShapeMapHasher>;

/// `NCollection_DataMap<TopoDS_Shape, TopoDS_Shape, TopTools_ShapeMapHasher>`: each image
/// with its initial shape.
type ImageOrigins = DataMap<Shape, Shape, ShapeMapHasher>;

/// The shape types `Add` visits, in the native order.
const ADD_ORDER: [ShapeEnum; 8] = [
    ShapeEnum::Compound,
    ShapeEnum::CompSolid,
    ShapeEnum::Solid,
    ShapeEnum::Shell,
    ShapeEnum::Face,
    ShapeEnum::Wire,
    ShapeEnum::Edge,
    ShapeEnum::Vertex,
];

/// `BRepBuilderAPI_Collect` (docs/topalgo-design.md section 10): the modified and
/// generated images of the shapes of the first input. A later step that modifies an
/// image replaces it in the list of its initial shape; a generation from a modified
/// image is recorded on the initial shape.
///
/// ```
/// use rustycad_tkmath::gp::{Point3, Transform3};
/// use rustycad_tktopalgo::brep_builder_api::{
///     BRepBuilderAPICollect, BRepBuilderAPIMakeShape, BRepBuilderAPIMakeVertex, BRepBuilderAPITransform,
/// };
/// let vertex = BRepBuilderAPIMakeVertex::new(Point3::new(1.0, 2.0, 3.0))?.vertex()?;
/// let mut moved = BRepBuilderAPITransform::with_shape(&vertex, Transform3::translation([1.0, 0.0, 0.0]), true, false)?;
/// let mut collect = BRepBuilderAPICollect::new();
/// collect.add(&vertex, &mut moved)?;
/// assert_eq!(collect.modification().find(&vertex)?.extent(), 1);
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
#[derive(Default)]
pub struct BRepBuilderAPICollect {
    my_mod: ShapeHistoryMap,
    my_gen: ShapeHistoryMap,
}

impl BRepBuilderAPICollect {
    /// `BRepBuilderAPI_Collect()`: an empty history; `BRepBuilderAPICollect::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `Add(SI, MKS)`: records the images that `MKS` gives to the sub-shapes of `SI`,
    /// type by type from compounds to vertices, each sub-shape once;
    /// `collect.add(&input, &mut algorithm)?`.
    pub fn add(
        &mut self,
        input: &Shape,
        algorithm: &mut dyn BRepBuilderAPIMakeShape,
    ) -> Result<(), Failure> {
        // Vraiment pas optimum a Revoir
        let gen_back = build_back(&self.my_gen);
        let mod_back = build_back(&self.my_mod);
        let origins = (&mod_back, &gen_back);
        for shape_type in ADD_ORDER {
            self.update(origins, input, algorithm, shape_type)?;
        }
        Ok(())
    }

    /// The static `Update`: the modified and generated images of the sub-shapes of
    /// type `shape_type`.
    fn update(
        &mut self,
        (mod_back, gen_back): (&ImageOrigins, &ImageOrigins),
        input: &Shape,
        algorithm: &mut dyn BRepBuilderAPIMakeShape,
        shape_type: ShapeEnum,
    ) -> Result<(), Failure> {
        let mut deja_vu = MapOfShape::default();
        let mut explorer = Explorer::new(input, shape_type, ShapeEnum::Shape)?;
        while explorer.more() {
            let shape = explorer.current().clone();
            explorer.next()?;
            if !deja_vu.add(shape.clone()) {
                continue;
            }
            let modified = algorithm.modified(&shape)?.clone();
            self.store_modified(gen_back, mod_back, &shape, &modified)?;
            let generated = algorithm.generated(&shape)?.clone();
            self.store_generated(gen_back, mod_back, &shape, &generated)?;
        }
        Ok(())
    }

    /// The images `S` was modified into: a modification of a generated image is a
    /// generation of the initial shape.
    fn store_modified(
        &mut self,
        gen_back: &ImageOrigins,
        mod_back: &ImageOrigins,
        shape: &Shape,
        images: &List<Shape>,
    ) -> Result<(), Failure> {
        if images.is_empty() {
            return Ok(());
        }
        if gen_back.is_bound(shape) {
            // Modif de generation => generation du shape initial
            return store_image(&mut self.my_gen, shape, gen_back, images);
        }
        store_image(&mut self.my_mod, shape, mod_back, images)
    }

    /// The images generated from `S`: a generation from a modified image is a generation
    /// of its initial shape.
    fn store_generated(
        &mut self,
        gen_back: &ImageOrigins,
        mod_back: &ImageOrigins,
        shape: &Shape,
        images: &List<Shape>,
    ) -> Result<(), Failure> {
        if images.is_empty() {
            return Ok(());
        }
        // Generation de modif  => generation du shape initial
        let initial = mod_back.seek(shape).unwrap_or(shape);
        store_image(&mut self.my_gen, initial, gen_back, images)
    }

    /// `AddGenerated(S, Gen)`: records `Gen` as generated from `S` (from the initial
    /// shape of `S` when `S` is a modified image); `collect.add_generated(&s, &gen)?`.
    pub fn add_generated(&mut self, shape: &Shape, generated: &Shape) -> Result<(), Failure> {
        let gen_back = build_back(&self.my_gen);
        let mod_back = build_back(&self.my_mod);
        let images = single(generated);
        self.store_generated(&gen_back, &mod_back, shape, &images)
    }

    /// `AddModif(S, Mod)`: records `Mod` as the modification of `S` (a generation when
    /// `S` is a generated image); `collect.add_modif(&s, &modified)?`.
    pub fn add_modif(&mut self, shape: &Shape, modified: &Shape) -> Result<(), Failure> {
        let gen_back = build_back(&self.my_gen);
        let mod_back = build_back(&self.my_mod);
        let images = single(modified);
        self.store_modified(&gen_back, &mod_back, shape, &images)
    }

    /// `Filter(SF)`: removes every image that is not a face (an edge, a vertex, once the
    /// images hold one) of `SF`; `collect.filter(&result)?`.
    pub fn filter(&mut self, final_shape: &Shape) -> Result<(), Failure> {
        filter_by_shape(&mut self.my_gen, final_shape)?;
        filter_by_shape(&mut self.my_mod, final_shape)
    }

    /// `Modification()`: the modified images; `collect.modification()`.
    pub fn modification(&self) -> &ShapeHistoryMap {
        &self.my_mod
    }

    /// `Generated()`: the generated images; `collect.generated()`.
    pub fn generated(&self) -> &ShapeHistoryMap {
        &self.my_gen
    }
}

/// A list of one shape.
fn single(shape: &Shape) -> List<Shape> {
    let mut images = List::default();
    images.append(shape.clone());
    images
}

/// The static `BuildBack`: each image bound to its initial shape (a later binding of the
/// same image wins, in the iteration order of the map, as natively).
fn build_back(history: &ShapeHistoryMap) -> ImageOrigins {
    let mut origins = ImageOrigins::default();
    for (initial, images) in history.items() {
        for image in images.iter() {
            origins.bind(image.clone(), initial.clone());
        }
    }
    origins
}

/// The static `Replace`: removes the first shape IsSame to `old` from the list, then
/// appends the new images.
fn replace(images: &mut List<Shape>, old: &Shape, new: &List<Shape>) {
    // Suppression de Old dans la liste.
    let mut removed = false;
    images.retain(|image| {
        if removed || !image.is_same(old) {
            return true;
        }
        removed = true;
        false
    });
    // Ajout de New a L.
    for image in new.iter() {
        images.append(image.clone());
    }
}

/// The static `StoreImage`: `shape` is an image of an earlier step (it is replaced by
/// its images in the list of its initial shape), or an initial shape whose list gets
/// the images appended.
fn store_image(
    history: &mut ShapeHistoryMap,
    shape: &Shape,
    back: &ImageOrigins,
    images: &List<Shape>,
) -> Result<(), Failure> {
    if images.is_empty() {
        return Ok(());
    }
    if let Some(initial) = back.seek(shape) {
        replace(history.change_find(initial)?, shape, images);
        return Ok(());
    }
    if !history.is_bound(shape) {
        history.bind(shape.clone(), List::default());
    }
    // Dans tous les cas on copie la liste pour eviter les pb de
    // const& dans BRepBuilderAPI.
    let list = history.change_find(shape)?;
    for image in images.iter() {
        list.append(image.clone());
    }
    Ok(())
}

/// The static `FilterByShape`: keeps the images that are faces of `SF`, and edges or
/// vertices of `SF` from the first image of that type on.
fn filter_by_shape(history: &mut ShapeHistoryMap, final_shape: &Shape) -> Result<(), Failure> {
    let mut kept = MapOfShape::default();
    add_sub_shapes(&mut kept, final_shape, ShapeEnum::Face)?;
    let mut has_edge = false;
    let mut has_vertex = false;
    for (_, images) in history.items_mut() {
        let mut failure: Option<Failure> = None;
        images.retain(|image| {
            let widened = widen_kept(
                &mut kept,
                image,
                final_shape,
                &mut has_edge,
                &mut has_vertex,
            );
            if let Err(error) = widened {
                failure.get_or_insert(error);
            }
            kept.contains(image)
        });
        if let Some(error) = failure {
            return Err(error);
        }
    }
    Ok(())
}

/// The first edge (vertex) image adds the edges (vertices) of `SF` to the kept shapes.
fn widen_kept(
    kept: &mut MapOfShape,
    image: &Shape,
    final_shape: &Shape,
    has_edge: &mut bool,
    has_vertex: &mut bool,
) -> Result<(), Failure> {
    let image_type = image.shape_type()?;
    // Images contiennet des edges => ajout des edges resultat dans MSF.
    if !*has_edge && image_type == ShapeEnum::Edge {
        add_sub_shapes(kept, final_shape, ShapeEnum::Edge)?;
        *has_edge = true;
    }
    // Images contiennet des vertex => ajout des vertex resultat dans MSF.
    if !*has_vertex && image_type == ShapeEnum::Vertex {
        add_sub_shapes(kept, final_shape, ShapeEnum::Vertex)?;
        *has_vertex = true;
    }
    Ok(())
}

/// Adds the sub-shapes of `shape` of one type to `found`.
fn add_sub_shapes(
    found: &mut MapOfShape,
    shape: &Shape,
    shape_type: ShapeEnum,
) -> Result<(), Failure> {
    let mut explorer = Explorer::new(shape, shape_type, ShapeEnum::Shape)?;
    while explorer.more() {
        found.add(explorer.current().clone());
        explorer.next()?;
    }
    Ok(())
}
