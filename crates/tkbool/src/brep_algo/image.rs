//! Translation of `BRepAlgo_Image` at OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
use rustycad_tkbrep::{topexp::Explorer, topods::Shape, toptools::ShapeMapHasher};
use rustycad_tkernel::{ncollection::DataMap, standard::Failure};
use rustycad_tkg3d::top_abs::ShapeEnum;
use std::sync::{Mutex, OnceLock};

static UNBOUND_IMAGE_CACHE: OnceLock<Mutex<Vec<Shape>>> = OnceLock::new();

/// Links each source shape to its direct and recursively final images.
/// `let mut images = BRepAlgoImage::new(); images.bind(&source, &image)?;`.
pub struct BRepAlgoImage {
    roots: Vec<Shape>,
    up: DataMap<Shape, Shape, ShapeMapHasher>,
    down: DataMap<Shape, Vec<Shape>, ShapeMapHasher>,
}

impl Default for BRepAlgoImage {
    fn default() -> Self {
        Self::new()
    }
}

impl BRepAlgoImage {
    /// Creates an empty shape-image graph; `BRepAlgoImage::new()`.
    pub fn new() -> Self {
        Self {
            roots: Vec::new(),
            up: DataMap::default(),
            down: DataMap::default(),
        }
    }

    /// Adds a root in insertion order, including repeated roots; `images.set_root(&shape)`.
    pub fn set_root(&mut self, shape: &Shape) {
        self.roots.push(shape.clone());
    }

    /// Links one direct image, raising `Standard_ConstructionError` on duplicate bindings.
    /// `images.bind(&source, &image)?`.
    pub fn bind(&mut self, source: &Shape, image: &Shape) -> Result<(), Failure> {
        self.check_bind(source)?;
        self.down.bind(source.clone(), vec![image.clone()]);
        // NCollection_DataMap::Bind replaces the image's prior direct generator.
        self.up.bind(image.clone(), source.clone());
        Ok(())
    }

    /// Links several direct images; an empty image sequence leaves the source unbound.
    /// `images.bind_many(&source, &[first, second])?`.
    pub fn bind_many(&mut self, source: &Shape, images: &[Shape]) -> Result<(), Failure> {
        if self.has_image(source) {
            return Err(construction_error(
                "Bind source already has an image",
                source,
            ));
        }
        for (index, image) in images.iter().enumerate() {
            if index == 0 {
                self.bind(source, image)?;
            } else {
                self.add(source, image)?;
            }
        }
        Ok(())
    }

    /// Adds an image to an existing source link; `images.add(&source, &image)?`.
    pub fn add(&mut self, source: &Shape, image: &Shape) -> Result<(), Failure> {
        if !self.has_image(source) {
            return Err(construction_error(
                "Add requires a source with an image",
                source,
            ));
        }
        self.down.change_find(source)?.push(image.clone());
        self.up.bind(image.clone(), source.clone());
        Ok(())
    }

    /// Adds a sequence of images to an existing link; `images.add_many(&source, &images)?`.
    pub fn add_many(&mut self, source: &Shape, images: &[Shape]) -> Result<(), Failure> {
        for image in images {
            self.add(source, image)?;
        }
        Ok(())
    }

    /// Clears roots and both directions of the image graph; `images.clear()`.
    pub fn clear(&mut self) {
        self.roots.clear();
        self.up.clear(true);
        self.down.clear(true);
    }

    /// Removes a generated shape from its direct generator; `images.remove(&image)?`.
    pub fn remove(&mut self, image: &Shape) -> Result<(), Failure> {
        let source = self.image_from(image)?;
        let images = self.down.change_find(&source)?;
        remove_first_same(images, image);
        if images.is_empty() {
            self.down.unbind(&source);
        }
        self.up.unbind(image);
        Ok(())
    }

    /// Removes the first same root and its direct root-to-image links; `images.remove_root(&root)`.
    pub fn remove_root(&mut self, root: &Shape) {
        let Some(index) = self.roots.iter().position(|entry| entry.is_same(root)) else {
            return;
        };
        self.roots.remove(index);
        self.unbind_root_images(root);
    }

    /// Replaces a linked root and preserves its final images; `images.replace_root(&old, &new)?`.
    pub fn replace_root(&mut self, old_root: &Shape, new_root: &Shape) -> Result<(), Failure> {
        if !self.has_image(old_root) {
            return Ok(());
        }
        let images = self.image(old_root);
        self.link_root_images(new_root, &images)?;
        self.set_root(new_root);
        self.remove_root(old_root);
        Ok(())
    }

    /// Borrows roots in insertion order; `images.roots()`.
    pub fn roots(&self) -> &[Shape] {
        &self.roots
    }

    /// Checks whether a shape is the image of another shape; `images.is_image(&shape)`.
    pub fn is_image(&self, shape: &Shape) -> bool {
        self.up.is_bound(shape)
    }

    /// Returns its immediate generator, or `Standard_ConstructionError` if it is not an image.
    /// `images.image_from(&generated)?`.
    pub fn image_from(&self, shape: &Shape) -> Result<Shape, Failure> {
        self.up
            .find(shape)
            .cloned()
            .map_err(|_| construction_error("ImageFrom requires a generated shape", shape))
    }

    /// Returns the highest linked generator, or `Standard_ConstructionError` for a non-image.
    /// `images.root(&generated)?`.
    pub fn root(&self, shape: &Shape) -> Result<Shape, Failure> {
        let mut current = shape.clone();
        let mut parent = self.image_from(&current)?;
        while self.is_image(&parent) && !parent.is_same(&current) {
            current = parent;
            parent = self.image_from(&current)?;
        }
        Ok(parent)
    }

    /// Checks whether a source has a direct image link; `images.has_image(&source)`.
    pub fn has_image(&self, source: &Shape) -> bool {
        self.down.is_bound(source)
    }

    /// Returns direct images or the source itself when no link exists; `images.image(&shape)`.
    pub fn image(&self, source: &Shape) -> Vec<Shape> {
        if let Ok(images) = self.down.find(source) {
            return images.clone();
        }
        append_unbound_image(source)
    }

    /// Returns terminal images by following each direct image recursively.
    /// `images.last_image(&source)`.
    pub fn last_image(&self, source: &Shape) -> Vec<Shape> {
        let mut result = Vec::new();
        append_last_images(self, source, &mut result);
        result
    }

    /// Keeps only direct links from roots to terminal images; `images.compact()`.
    pub fn compact(&mut self) -> Result<(), Failure> {
        let compacted = compact_roots(self);
        self.up.clear(true);
        self.down.clear(true);
        bind_compacted_roots(self, &compacted)?;
        Ok(())
    }

    /// Deletes links whose generator type is absent from `shape`; `images.filter(&shape, ShapeEnum::Edge)?`.
    pub fn filter(&mut self, shape: &Shape, kind: ShapeEnum) -> Result<(), Failure> {
        let present = explored_shapes(shape, kind)?;
        remove_filtered_images(self, kind, &present)
    }

    fn check_bind(&self, source: &Shape) -> Result<(), Failure> {
        if self.has_image(source) {
            return Err(construction_error(
                "Bind source already has an image",
                source,
            ));
        }
        Ok(())
    }

    fn unbind_root_images(&mut self, root: &Shape) {
        let direct = self.down.find(root).cloned().unwrap_or_default();
        for image in direct {
            if self
                .up
                .find(&image)
                .is_ok_and(|source| source.is_same(root))
            {
                self.up.unbind(&image);
            }
        }
        self.down.unbind(root);
    }

    fn link_root_images(&mut self, root: &Shape, images: &[Shape]) -> Result<(), Failure> {
        if self.has_image(root) {
            self.add_many(root, images)
        } else {
            self.bind_many(root, images)
        }
    }
}

fn remove_first_same(images: &mut Vec<Shape>, target: &Shape) {
    if let Some(index) = images.iter().position(|image| image.is_same(target)) {
        images.remove(index);
    }
}

fn append_unbound_image(source: &Shape) -> Vec<Shape> {
    let cache = UNBOUND_IMAGE_CACHE.get_or_init(|| Mutex::new(Vec::new()));
    let mut images = cache
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    images.push(source.clone());
    images.clone()
}

fn append_last_images(graph: &BRepAlgoImage, source: &Shape, result: &mut Vec<Shape>) {
    for image in graph.image(source) {
        if image.is_same(source) || !graph.has_image(&image) {
            result.push(image);
        } else {
            append_last_images(graph, &image, result);
        }
    }
}

fn compact_roots(graph: &BRepAlgoImage) -> Vec<(Shape, Vec<Shape>)> {
    graph
        .roots()
        .iter()
        .filter(|root| graph.has_image(root))
        .map(|root| (root.clone(), graph.last_image(root)))
        .collect()
}

fn bind_compacted_roots(
    graph: &mut BRepAlgoImage,
    roots: &[(Shape, Vec<Shape>)],
) -> Result<(), Failure> {
    for (root, images) in roots {
        graph.bind_many(root, images)?;
    }
    Ok(())
}

fn explored_shapes(shape: &Shape, kind: ShapeEnum) -> Result<Vec<Shape>, Failure> {
    Explorer::new(shape, kind, ShapeEnum::Shape).map(Iterator::collect)
}

fn remove_filtered_images(
    graph: &mut BRepAlgoImage,
    kind: ShapeEnum,
    present: &[Shape],
) -> Result<(), Failure> {
    loop {
        let removed = filtered_key(graph, kind, present)?;
        let Some(shape) = removed else { return Ok(()) };
        graph.remove(&shape)?;
    }
}

fn filtered_key(
    graph: &BRepAlgoImage,
    kind: ShapeEnum,
    present: &[Shape],
) -> Result<Option<Shape>, Failure> {
    for (image, _) in graph.up.items() {
        if image.shape_type()? == kind && !present.iter().any(|shape| shape.is_same(image)) {
            return Ok(Some(image.clone()));
        }
    }
    Ok(None)
}

fn construction_error(operation: &str, shape: &Shape) -> Failure {
    let message = format!("{operation}: {shape:?}; expected a linked TopoDS_Shape");
    Failure::with_type("Standard_ConstructionError", &message, "BRepAlgo_Image")
}
