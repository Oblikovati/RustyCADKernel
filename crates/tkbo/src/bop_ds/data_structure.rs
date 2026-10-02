//! Shape storage and the currently ported source-vertex portion of `BOPDS_DS::Init`.
use super::{IndexRange, ShapeInfo};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkbrep::topods::{ShapeIterator, Vertex};
use rustycad_tkbrep::toptools::ShapeMapHasher;
use rustycad_tkernel::ncollection::DataMap;
use rustycad_tkernel::standard::Failure;

/// The shape and argument storage shared by Boolean operation algorithms.
/// `let mut ds = DataStructure::default(); ds.append_shape(shape);`.
#[derive(Default)]
pub struct DataStructure {
    arguments: Vec<Shape>,
    ranges: Vec<IndexRange>,
    lines: Vec<ShapeInfo>,
    shape_indices: Option<DataMap<Shape, i32, ShapeMapHasher>>,
    source_shape_count: i32,
}

impl DataStructure {
    /// Initializes argument topology and source vertex boxes; remaining edge/face/solid
    /// preparation is not ported yet. `ds.init(1e-5)?`.
    pub fn init(&mut self, fuzz: f64) -> Result<(), Failure> {
        if self.arguments.is_empty() {
            return Ok(());
        }
        self.append_unindexed_arguments()?;
        self.source_shape_count = self.nb_shapes();
        self.prepare_vertex_bounds(additional_tolerance(fuzz))
    }

    /// Replaces operation arguments with a copy of `shapes`.
    /// `ds.set_arguments(&[shape.clone()]);`.
    pub fn set_arguments(&mut self, shapes: &[Shape]) {
        self.arguments = shapes.to_vec();
    }

    /// Borrows the ordered operation arguments; `ds.arguments().len()`.
    pub fn arguments(&self) -> &[Shape] {
        &self.arguments
    }

    /// Returns the number of stored shape records; `ds.nb_shapes()`.
    pub fn nb_shapes(&self) -> i32 {
        self.lines.len() as i32
    }

    /// Returns the number of source records initialized from arguments.
    /// Before `Init`, OCCT reports zero; `ds.nb_source_shapes()`.
    pub fn nb_source_shapes(&self) -> i32 {
        self.source_shape_count
    }

    /// Returns the number of source index ranges; `ds.nb_ranges()`.
    pub fn nb_ranges(&self) -> i32 {
        self.ranges.len() as i32
    }

    /// Borrows a zero-based range, reporting the offending index on failure.
    /// `let range = ds.range(0)?;`.
    pub fn range(&self, index: i32) -> Result<&IndexRange, Failure> {
        let slot = checked_slot(index, self.ranges.len(), "BOPDS_DS::Range")?;
        Ok(&self.ranges[slot])
    }

    /// Returns the first range containing `index`, or `-1` when none does.
    /// `ds.rank(2) == -1` for an uninitialized structure.
    pub fn rank(&self, index: i32) -> i32 {
        self.ranges
            .iter()
            .position(|range| range.contains(index))
            .map_or(-1, |rank| rank as i32)
    }

    /// Reports whether the index is at or beyond the initialized source count.
    /// `ds.is_new_shape(0)` is true before `Init`.
    pub fn is_new_shape(&self, index: i32) -> bool {
        index >= self.source_shape_count
    }

    /// Appends a shape-info copy and binds its shape to the new zero-based index.
    /// `let index = ds.append_shape_info(&info);`.
    pub fn append_shape_info(&mut self, info: &ShapeInfo) -> i32 {
        let index = self.nb_shapes();
        self.lines.push(info.clone());
        self.bind_shape_index(info.shape(), index);
        index
    }

    /// Appends default metadata for `shape` and returns its zero-based index.
    /// `let index = ds.append_shape(&shape);`.
    pub fn append_shape(&mut self, shape: &Shape) -> i32 {
        let mut info = ShapeInfo::default();
        info.set_shape(shape);
        self.append_shape_info(&info)
    }

    /// Borrows shape metadata by zero-based index.
    /// `let info = ds.shape_info(0)?;`.
    pub fn shape_info(&self, index: i32) -> Result<&ShapeInfo, Failure> {
        let slot = checked_slot(index, self.lines.len(), "BOPDS_DS::ShapeInfo")?;
        Ok(&self.lines[slot])
    }

    /// Mutably borrows shape metadata by zero-based index.
    /// `ds.change_shape_info(0)?.set_flag(1);`.
    pub fn change_shape_info(&mut self, index: i32) -> Result<&mut ShapeInfo, Failure> {
        let slot = checked_slot(index, self.lines.len(), "BOPDS_DS::ChangeShapeInfo")?;
        Ok(&mut self.lines[slot])
    }

    /// Borrows the shape stored at `index`.
    /// `let shape = ds.shape(0)?;`.
    pub fn shape(&self, index: i32) -> Result<&Shape, Failure> {
        Ok(self.shape_info(index)?.shape())
    }

    /// Finds a shape using OCCT `IsSame` identity, returning `-1` when absent.
    /// `ds.index(&shape) == 0` after appending it.
    pub fn index(&self, shape: &Shape) -> i32 {
        self.shape_indices
            .as_ref()
            .and_then(|indices| indices.find(shape).ok())
            .copied()
            .unwrap_or(-1)
    }

    /// Clears arguments and all currently implemented DS storage.
    /// `ds.clear(); assert_eq!(ds.nb_shapes(), 0);`.
    pub fn clear(&mut self) {
        self.arguments.clear();
        self.ranges.clear();
        self.lines.clear();
        self.source_shape_count = 0;
        if let Some(indices) = &mut self.shape_indices {
            indices.clear(false);
        }
    }

    fn bind_shape_index(&mut self, shape: &Shape, index: i32) {
        self.shape_indices
            .get_or_insert_with(|| DataMap::with_hasher(ShapeMapHasher, 100))
            .bind(shape.clone(), index);
    }

    fn append_unindexed_arguments(&mut self) -> Result<(), Failure> {
        let mut first_index = 0;
        for shape in self.arguments.clone() {
            if self.index(&shape) >= 0 {
                continue;
            }
            let shape_index = self.append_shape(&shape);
            self.init_shape(shape_index, &shape)?;
            let last_index = self.nb_shapes() - 1;
            self.ranges.push(IndexRange::new(first_index, last_index));
            first_index = last_index + 1;
        }
        Ok(())
    }

    fn init_shape(&mut self, shape_index: i32, shape: &Shape) -> Result<(), Failure> {
        let shape_type = shape.shape_type()?;
        self.change_shape_info(shape_index)?
            .set_shape_type(shape_type);
        let mut iterator = ShapeIterator::new(shape, true, true);
        while iterator.more() {
            let child = iterator.value()?.clone();
            let child_index = self.append_or_find(&child)?;
            self.init_shape(child_index, &child)?;
            self.append_unique_child(shape_index, child_index)?;
            iterator.next();
        }
        Ok(())
    }

    fn append_or_find(&mut self, shape: &Shape) -> Result<i32, Failure> {
        let found = self.index(shape);
        if found >= 0 {
            return Ok(found);
        }
        Ok(self.append_shape(shape))
    }

    fn append_unique_child(&mut self, parent: i32, child: i32) -> Result<(), Failure> {
        let children = self.change_shape_info(parent)?.change_sub_shapes();
        if !children.contains(&child) {
            children.push(child);
        }
        Ok(())
    }

    fn prepare_vertex_bounds(&mut self, added_gap: f64) -> Result<(), Failure> {
        for index in 0..self.source_shape_count {
            let info = self.shape_info(index)?;
            if info.shape_type() != rustycad_tkg3d::top_abs::ShapeEnum::Vertex {
                continue;
            }
            let vertex = Vertex::unchecked(self.shape(index)?.clone());
            let point = BRepTool::pnt(&vertex)?;
            let gap = BRepTool::vertex_tolerance(&vertex)? + added_gap;
            let mut bounds = info.box_bounds().clone();
            bounds.set_gap(gap);
            bounds.add_point(point);
            self.change_shape_info(index)?.set_box(&bounds);
        }
        Ok(())
    }
}

fn additional_tolerance(fuzz: f64) -> f64 {
    let confusion = rustycad_tkernel::precision::confusion();
    if fuzz < confusion {
        confusion * 0.5
    } else {
        fuzz * 0.5
    }
}

fn checked_slot(index: i32, length: usize, operation: &str) -> Result<usize, Failure> {
    let slot = usize::try_from(index).ok().filter(|slot| *slot < length);
    slot.ok_or_else(|| {
        Failure::with_type(
            "Standard_OutOfRange",
            &format!("{operation}: index {index}; expected 0..{length}"),
            "",
        )
    })
}
