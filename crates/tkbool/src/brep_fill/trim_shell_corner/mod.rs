// Rust translation of ModelingAlgorithms/TKBool/BRepFill/BRepFill_TrimShellCorner.hxx/.cxx
// (the class members) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepFill_TrimShellCorner`: the trimming of the two sets of faces met at a corner
//! of a pipe, by their intersection (or by the edges joining them when they do not
//! intersect).
mod choose_section;
mod ds_queries;
mod from_u_edge;
mod from_v_edge;
mod gap;
mod non_sec;
mod sec;
mod section_wire;

use super::shape_table::{ShapeTable, cell, copy_table};
use super::transition_style::TransitionStyle;
use rustycad_tkbo::bop_algo::BOPAlgoPaveFiller;
use rustycad_tkbo::bop_ds::DataStructure;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_tools::{BRepToolsReShape, ReShape};
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Shape, Shell};
use rustycad_tkbrep::toptools::ShapeMapHasher;
use rustycad_tkernel::ncollection::{DataMap, HArray2, List};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::gp::{Axis2, Vector3};

/// The history of the trimming: the new shapes of each old one.
pub(crate) type HistoryMap = DataMap<Shape, List<Shape>, ShapeMapHasher>;

/// The trimming of the faces at a corner of a pipe (`BRepFill_TrimShellCorner`);
/// `let mut trim = BRepFillTrimShellCorner::new(&faces, TransitionStyle::Right, &axe, &dir)?;`.
pub struct BRepFillTrimShellCorner {
    my_transition: TransitionStyle,
    my_axe_of_bis_plane: Axis2,
    my_int_point_cross_dir: Vector3,
    my_shape1: Shape,
    my_shape2: Shape,
    my_bounds: ShapeTable,
    my_u_edges: ShapeTable,
    my_v_edges: Vec<Shape>,
    my_v_edges_lower: i32,
    my_faces: ShapeTable,
    my_done: bool,
    my_has_section: bool,
    my_hist_map: HistoryMap,
}

impl BRepFillTrimShellCorner {
    /// `BRepFill_TrimShellCorner(theFaces, theTransition, theAxeOfBisPlane,
    /// theIntPointCrossDir)`: the faces to intersect (two columns), the transition and
    /// the bisector plane axis; the faces are copied.
    pub fn new(
        the_faces: &Handle<HArray2<Shape>>,
        the_transition: TransitionStyle,
        the_axe_of_bis_plane: &Axis2,
        the_int_point_cross_dir: &Vector3,
    ) -> Result<Self, Failure> {
        Ok(Self {
            my_transition: the_transition,
            my_axe_of_bis_plane: *the_axe_of_bis_plane,
            my_int_point_cross_dir: *the_int_point_cross_dir,
            my_shape1: Shape::default(),
            my_shape2: Shape::default(),
            my_bounds: ShapeTable::default(),
            my_u_edges: ShapeTable::default(),
            my_v_edges: Vec::new(),
            my_v_edges_lower: 1,
            my_faces: copy_table(the_faces, "BRepFill_TrimShellCorner")?,
            my_done: false,
            my_has_section: false,
            my_hist_map: HistoryMap::default(),
        })
    }

    /// `AddBounds(Bounds)`: the bound edges of the faces (copied);
    /// `trim.add_bounds(&bounds)?`.
    pub fn add_bounds(&mut self, bounds: &Handle<HArray2<Shape>>) -> Result<(), Failure> {
        self.my_bounds = copy_table(bounds, "BRepFill_TrimShellCorner")?;
        Ok(())
    }

    /// `AddUEdges(theUEdges)`: the u-edges of the faces (copied);
    /// `trim.add_u_edges(&u_edges)?`.
    pub fn add_u_edges(&mut self, the_u_edges: &Handle<HArray2<Shape>>) -> Result<(), Failure> {
        self.my_u_edges = copy_table(the_u_edges, "BRepFill_TrimShellCorner")?;
        Ok(())
    }

    /// `AddVEdges(theVEdges, theIndex)`: column `the_index` of the v-edges;
    /// `trim.add_v_edges(&v_edges, 2)?`.
    pub fn add_v_edges(
        &mut self,
        the_v_edges: &Handle<HArray2<Shape>>,
        the_index: i32,
    ) -> Result<(), Failure> {
        let table = copy_table(the_v_edges, "BRepFill_TrimShellCorner")?;
        self.my_v_edges_lower = table.lower_row();
        self.my_v_edges = (table.lower_row()..=table.upper_row())
            .map(|i| cell(&table, i, the_index).cloned())
            .collect::<Result<Vec<Shape>, Failure>>()?;
        Ok(())
    }

    /// `IsDone()`: whether `perform` succeeded; `trim.is_done()`.
    pub fn is_done(&self) -> bool {
        self.my_done
    }

    /// `HasSection()`: always false, as natively (the flag is never raised);
    /// `trim.has_section()`.
    pub fn has_section(&self) -> bool {
        self.my_has_section
    }

    /// `Modified(S, theModified)`: the new shapes of `s` (empty when it is unchanged);
    /// `let new_shapes = trim.modified(&face);`.
    pub fn modified(&self, s: &Shape) -> List<Shape> {
        self.my_hist_map.seek(s).cloned().unwrap_or_default()
    }

    /// `Perform()`: intersects the two shells of faces and trims each pair of faces by
    /// the section (or joins them when they do not intersect); `trim.perform()?`.
    pub fn perform(&mut self) -> Result<(), Failure> {
        self.my_done = false;
        self.my_hist_map.clear(true);
        if self.my_faces.row_length() != 2 {
            return Ok(());
        }
        self.build_shells()?;
        let Some(filler) = self.intersect_shells()? else {
            return Ok(());
        };
        let Some(the_ds) = filler.data_structure() else {
            return Ok(());
        };
        if !ds_queries::split_u_edges(
            &self.my_u_edges,
            the_ds,
            &self.my_int_point_cross_dir,
            &mut self.my_hist_map,
        )? {
            return Ok(());
        }
        for ii in self.my_faces.lower_row()..=self.my_faces.upper_row() {
            if !self.trim_face_pair(ii, the_ds)? {
                self.my_hist_map.clear(true);
                return Ok(());
            }
        }
        self.my_done = true;
        Ok(())
    }

    /// The two shells of the face columns (closed when they are).
    fn build_shells(&mut self) -> Result<(), Failure> {
        let builder = BRepBuilder::default();
        for jj in self.my_faces.lower_col()..=self.my_faces.upper_col() {
            let mut a_shell = Shell::default();
            builder.make_shell(&mut a_shell);
            for ii in self.my_faces.lower_row()..=self.my_faces.upper_row() {
                builder.add(&mut a_shell, cell(&self.my_faces, ii, jj)?)?;
            }
            let closed: bool = BRepTool::is_closed(&a_shell)?;
            a_shell.set_closed(closed)?;
            if jj == self.my_faces.lower_col() {
                self.my_shape1 = a_shell.into_shape();
            } else {
                self.my_shape2 = a_shell.into_shape();
            }
        }
        Ok(())
    }

    /// The largest vertex tolerance of a shape.
    fn max_vertex_tolerance(shape: &Shape, mut max_tol: f64) -> Result<f64, Failure> {
        for v in Explorer::new(shape, ShapeEnum::Vertex, ShapeEnum::Shape)? {
            max_tol = max_tol.max(BRepTool::vertex_tolerance(&topods::vertex(&v)?)?);
        }
        Ok(max_tol)
    }

    /// The pave filler of the two shells (fuzzy when the vertices are tight); `None`
    /// when it fails.
    fn intersect_shells(&self) -> Result<Option<BOPAlgoPaveFiller>, Failure> {
        let a_max_tol: f64 = Self::max_vertex_tolerance(
            &self.my_shape2,
            Self::max_vertex_tolerance(&self.my_shape1, 0.)?,
        )?;
        let mut a_fuzzy: f64 = 4. * precision::confusion();
        let mut a_pf = BOPAlgoPaveFiller::new();
        let mut a_ls: List<Shape> = List::default();
        a_ls.append(self.my_shape1.clone());
        a_ls.append(self.my_shape2.clone());
        a_pf.set_arguments(&a_ls);
        if a_max_tol < 1.005 * precision::confusion() {
            a_fuzzy = a_pf.options().fuzzy_value().max(a_fuzzy);
            a_pf.options_mut().set_fuzzy_value(a_fuzzy);
        }
        a_pf.perform();
        if a_pf.options().has_errors() {
            return Ok(None);
        }
        Ok(Some(a_pf))
    }

    /// The trimming of the faces of row `ii` by their face/face interference (the first
    /// one of the pair); false when it fails.
    fn trim_face_pair(&mut self, ii: i32, the_ds: &DataStructure) -> Result<bool, Failure> {
        let a_f1: Shape = cell(&self.my_faces, ii, self.my_faces.lower_col())?.clone();
        let a_f2: Shape = cell(&self.my_faces, ii, self.my_faces.upper_col())?.clone();
        let (an_index1, an_index2) = (the_ds.index(&a_f1), the_ds.index(&a_f2));
        if an_index1 == -1 || an_index2 == -1 {
            return Ok(true);
        }
        let Some((i, bhassec)) = ds_queries::face_pair_interference(the_ds, an_index1, an_index2)?
        else {
            return Ok(true);
        };
        if bhassec {
            return self.make_faces_sec(ii, the_ds, an_index1, an_index2, i);
        }
        self.make_faces_non_sec(ii, the_ds, an_index1, an_index2)
    }

    /// The v-edge of the row `the_index`.
    fn v_edge(&self, the_index: i32) -> Result<&Shape, Failure> {
        usize::try_from(the_index - self.my_v_edges_lower)
            .ok()
            .and_then(|k| self.my_v_edges.get(k))
            .ok_or_else(|| {
                Failure::with_type(
                    "Standard_OutOfRange",
                    &format!(
                        "BRepFill_TrimShellCorner: v-edge index {the_index}; expected {}..={}",
                        self.my_v_edges_lower,
                        self.my_v_edges_lower + self.my_v_edges.len() as i32 - 1
                    ),
                    "NCollection_Array1",
                )
            })
    }

    /// The substitutions applied to the face (the result of Apply is dropped and the
    /// face replaced by its own value, as natively): the new face and the edges of
    /// every replaced edge not yet in the history are recorded.
    fn record_face_history(
        &mut self,
        a_substitutor: &BRepToolsReShape,
        a_face: &Shape,
        a_face_ori: Orientation,
    ) -> Result<(), Failure> {
        a_substitutor.apply(a_face, ShapeEnum::Shape)?;
        let mut a_new_face: Shape = a_substitutor.value(a_face)?;
        a_new_face.set_orientation(a_face_ori);
        self.my_hist_map
            .bind(a_face.clone(), ds_queries::single(a_new_face));
        for an_edge in Explorer::new(a_face, ShapeEnum::Edge, ShapeEnum::Shape)? {
            let a_new_value: Shape = a_substitutor.value(&an_edge)?;
            if a_new_value.is_null()
                || a_new_value.is_same(&an_edge)
                || self.my_hist_map.is_bound(&an_edge)
            {
                continue;
            }
            let mut a_list_of_new_edge: List<Shape> = List::default();
            for e in Explorer::new(&a_new_value, ShapeEnum::Edge, ShapeEnum::Shape)? {
                a_list_of_new_edge.append(e);
            }
            self.my_hist_map.bind(an_edge, a_list_of_new_edge);
        }
        Ok(())
    }

    /// The u-edge at `(row, col)`, replaced by its first new edge when it has one.
    fn current_u_edge(&self, row: i32, col: i32) -> Result<Shape, Failure> {
        let a_shape: &Shape = cell(&self.my_u_edges, row, col)?;
        Ok(ds_queries::first_new(&self.my_hist_map, a_shape).unwrap_or_else(|| a_shape.clone()))
    }

    /// `StoreVedgeInHistMap(theVEdges, theIndex, theNewVshape, theHistMap)`: the new
    /// shape appended to the history of the v-edge of the row.
    fn store_v_edge_in_hist_map(
        &mut self,
        the_index: i32,
        the_new_vshape: &Shape,
    ) -> Result<(), Failure> {
        let a_v_edge: Shape = self.v_edge(the_index)?.clone();
        self.my_hist_map
            .bound(a_v_edge, List::default())
            .append(the_new_vshape.clone());
        Ok(())
    }
}
