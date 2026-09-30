// Rust translation of ModelingAlgorithms/TKTopAlgo/BRepCheck/BRepCheck_Analyzer.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepCheck_Analyzer`: the checkers of a shape and of all its sub-shapes, run in
//! their contexts (the functor is in `analyzer_contexts.rs`), and the validity queries.
use super::analyzer_contexts::check_in_contexts;
use super::edge::BRepCheckEdge;
use super::errors::null_object;
use super::explore::sub_shapes;
use super::face::BRepCheckFace;
use super::result::BRepCheckResult;
use super::shell::BRepCheckShell;
use super::solid::BRepCheckSolid;
use super::status::BRepCheckStatus;
use super::vertex::BRepCheckVertex;
use super::wire::BRepCheckWire;
use rustycad_tkbrep::topods::{Shape, ShapeIterator, edge, face, shell, solid, vertex, wire};
use rustycad_tkernel::ncollection::IndexedDataMap;
use rustycad_tkernel::osd::ThreadPool;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::ShapeEnum;

/// `NCollection_IndexedDataMap<TopoDS_Shape, Handle(BRepCheck_Result)>` with the default
/// hasher: a sub-shape met with two orientations has two checkers.
pub(super) type CheckerMap = IndexedDataMap<Shape, Handle<dyn BRepCheckResult>>;

/// `BRepCheck_Analyzer`: checks the validity of a shape and of its sub-shapes; the
/// geometric controls can be switched off, the checks run in parallel and the edge
/// distances computed exactly.
///
/// ```
/// use rustycad_tkbrep::brep::BRepBuilder;
/// use rustycad_tkbrep::topods::Compound;
/// use rustycad_tktopalgo::brep_check::BRepCheckAnalyzer;
/// let mut c = Compound::default();
/// BRepBuilder::default().make_compound(&mut c);
/// let analyzer = BRepCheckAnalyzer::new(&c, true, false, false).unwrap();
/// assert!(analyzer.is_valid().unwrap());
/// ```
pub struct BRepCheckAnalyzer {
    shape: Shape,
    map: CheckerMap,
    is_parallel: bool,
    is_exact: bool,
}

impl BRepCheckAnalyzer {
    /// `BRepCheck_Analyzer(S, GeomControls, theIsParallel, theIsExact)`: `Init(S,
    /// GeomControls)`; Standard_NullObject for a null shape;
    /// `BRepCheckAnalyzer::new(&shape, true, false, false)?`.
    pub fn new(
        shape: &Shape,
        geom_controls: bool,
        is_parallel: bool,
        is_exact: bool,
    ) -> Result<Self, Failure> {
        let mut analyzer = Self {
            shape: Shape::default(),
            map: CheckerMap::default(),
            is_parallel,
            is_exact,
        };
        analyzer.init(shape, geom_controls)?;
        Ok(analyzer)
    }

    /// `Init(S, GeomControls)`: the checkers of `shape` and its sub-shapes, then their
    /// checks in context; Standard_NullObject for a null shape;
    /// `analyzer.init(&shape, true)?`.
    pub fn init(&mut self, shape: &Shape, geom_controls: bool) -> Result<(), Failure> {
        if shape.is_null() {
            return Err(null_object("BRepCheck_Analyzer::Init() - NULL shape"));
        }
        self.shape = shape.clone();
        self.map.clear(false);
        self.put(shape, geom_controls)?;
        self.perform()
    }

    /// `SetExactMethod(theIsExact)`; `analyzer.set_exact_method(true)`.
    pub fn set_exact_method(&mut self, is_exact: bool) {
        self.is_exact = is_exact;
    }

    /// `IsExactMethod()`; `analyzer.is_exact_method()`.
    pub fn is_exact_method(&self) -> bool {
        self.is_exact
    }

    /// `SetParallel(theIsParallel)`; `analyzer.set_parallel(true)`.
    pub fn set_parallel(&mut self, is_parallel: bool) {
        self.is_parallel = is_parallel;
    }

    /// `IsParallel()`; `analyzer.is_parallel()`.
    pub fn is_parallel(&self) -> bool {
        self.is_parallel
    }

    /// `Result(theSubS)`: the checker of a sub-shape (null for a compound or a
    /// compsolid); Standard_NoSuchObject for a shape that is not a sub-shape (with its
    /// orientation); `analyzer.result(&sub)?`.
    pub fn result(&self, sub_shape: &Shape) -> Result<Handle<dyn BRepCheckResult>, Failure> {
        self.map.find_from_key(sub_shape).cloned()
    }

    /// `IsValid()`: no defect on the shape or its sub-shapes; `analyzer.is_valid()?`.
    pub fn is_valid(&self) -> Result<bool, Failure> {
        self.is_valid_shape(&self.shape)
    }

    /// `IsValid(S)`: the first status of `shape` is `NoError`, its children are valid,
    /// and the vertices of an edge, the wires, edges and vertices of a face and the
    /// shells of a solid have no defect in its context; `analyzer.is_valid_shape(&sub)?`.
    pub fn is_valid_shape(&self, shape: &Shape) -> Result<bool, Failure> {
        let checker = self.map.find_from_key(shape)?.clone();
        if let Some(c) = checker.get() {
            // Only the first status is read ("a voir").
            let first = c.status()?.first().copied();
            if first != Some(BRepCheckStatus::NoError) {
                return Ok(false);
            }
        }
        let mut iterator = ShapeIterator::new(shape, true, true);
        while let Ok(child) = iterator.value() {
            if !self.is_valid_shape(child)? {
                return Ok(false);
            }
            iterator.next();
        }
        Ok(match shape.shape_type()? {
            ShapeEnum::Edge => self.valid_sub(shape, ShapeEnum::Vertex)?,
            ShapeEnum::Face => {
                self.valid_sub(shape, ShapeEnum::Wire)?
                    && self.valid_sub(shape, ShapeEnum::Edge)?
                    && self.valid_sub(shape, ShapeEnum::Vertex)?
            }
            ShapeEnum::Solid => self.valid_sub(shape, ShapeEnum::Shell)?,
            _ => true,
        })
    }

    /// `ValidSub(S, SubType)`: the sub-shapes of `kind` have no defect in the context of
    /// `shape`; the scan stops at the first sub-shape not checked in that context.
    fn valid_sub(&self, shape: &Shape, kind: ShapeEnum) -> Result<bool, Failure> {
        for sub in sub_shapes(shape, kind)? {
            let checker = self.map.find_from_key(&sub)?.clone();
            let rv = checker
                .get()
                .ok_or_else(|| null_object("BRepCheck_Analyzer::ValidSub: null checker"))?;
            rv.init_context_iterator()?;
            while rv.more_shape_in_context() && !rv.contextual_shape()?.is_same(shape) {
                rv.next_shape_in_context();
            }
            if !rv.more_shape_in_context() {
                break;
            }
            if rv
                .status_on_shape()?
                .iter()
                .any(|s| *s != BRepCheckStatus::NoError)
            {
                return Ok(false);
            }
        }
        Ok(true)
    }

    /// `Put(S, Gctrl)`: the checker of `shape` (none for a compound or a compsolid),
    /// then those of its children, depth first.
    fn put(&mut self, shape: &Shape, geom_controls: bool) -> Result<(), Failure> {
        if self.map.contains(shape) {
            return Ok(());
        }
        let checker = self.make_checker(shape, geom_controls)?;
        if let Some(c) = checker.get() {
            c.set_parallel(self.is_parallel);
        }
        self.map.add(shape.clone(), checker);
        let mut iterator = ShapeIterator::new(shape, true, true);
        while let Ok(child) = iterator.value() {
            let child = child.clone();
            iterator.next();
            self.put(&child, geom_controls)?;
        }
        Ok(())
    }

    /// The checker of one shape by its type, with the analyzer options.
    fn make_checker(
        &self,
        shape: &Shape,
        geom_controls: bool,
    ) -> Result<Handle<dyn BRepCheckResult>, Failure> {
        Ok(match shape.shape_type()? {
            ShapeEnum::Vertex => Handle::new(BRepCheckVertex::new(&vertex(shape)?)?).cast(),
            ShapeEnum::Edge => {
                let checker = BRepCheckEdge::new(&edge(shape)?)?;
                checker.set_geometric_controls(geom_controls);
                checker.set_exact_method(self.is_exact);
                Handle::new(checker).cast()
            }
            ShapeEnum::Wire => {
                let checker = BRepCheckWire::new(&wire(shape)?)?;
                checker.set_geometric_controls(geom_controls);
                Handle::new(checker).cast()
            }
            ShapeEnum::Face => {
                let checker = BRepCheckFace::new(&face(shape)?)?;
                checker.set_geometric_controls(geom_controls);
                Handle::new(checker).cast()
            }
            ShapeEnum::Shell => Handle::new(BRepCheckShell::new(&shell(shape)?)?).cast(),
            ShapeEnum::Solid => Handle::new(BRepCheckSolid::new(&solid(shape)?)?).cast(),
            _ => Handle::default(),
        })
    }

    /// `Perform()`: the shapes dealt to tasks of at least 10 shapes (ten tasks per
    /// thread of the default pool), each task checking its shapes in their contexts;
    /// in parallel with the parallel flag, else in order.
    fn perform(&self) -> Result<(), Failure> {
        let size = self.map.extent();
        let threads = ThreadPool::default_pool(-1).nb_threads().max(1) as usize;
        let mut tasks = threads * 10;
        let mut task_size = size.div_ceil(tasks);
        if task_size < 10 {
            task_size = 10;
            tasks = size.div_ceil(task_size);
        }
        let mut arrays: Vec<Vec<Shape>> = vec![Vec::new(); tasks];
        for index in 1..=size {
            arrays[(index - 1) / task_size].push(self.map.find_key(index)?.clone());
        }
        check_in_contexts(&arrays, &self.map, self.is_parallel)
    }
}
