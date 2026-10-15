// Rust translation of ModelingAlgorithms/TKOffset/BiTgte/BiTgte_Blend.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871 (the computations are in the sibling modules).

//! `BiTgte_Blend`: the rolling-ball blend of a shape along its edges, by offsets of
//! the faces and tubes on the edges.
mod centers;
mod shape;
mod statics;
mod surfaces;

use super::BiTgteContactType;
use crate::brep_offset::BRepOffsetAnalyse;
use crate::brep_offset::OffsetFaces;
use rustycad_tkbool::brep_algo::{BRepAlgoAsDes, BRepAlgoImage};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_tools::Quilt;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Edge, Face, Shape};
use rustycad_tkbrep::toptools::{
    IndexedDataMapOfShapeListOfShape, IndexedMapOfShape, MapOfShape, ShapeMapHasher,
};
use rustycad_tkernel::ncollection::{DataMap, List};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::geom::{GeomCurve, GeomSurface, GeomTrimmedCurve};
use rustycad_tkg3d::top_abs::ShapeEnum;

/// The edges created on the original shapes for each cut center line (`myCreated`).
pub(super) type CreatedMap =
    DataMap<Shape, DataMap<Shape, List<Shape>, ShapeMapHasher>, ShapeMapHasher>;

/// `BiTgte_Blend`: the blend of radius `Radius` rolling along the edges and between
/// the faces given, with tubes on the edges and spheres at the vertices of three
/// tubes.
///
/// ```
/// use rustycad_tkoffset::bi_tgte::BiTgteBlend;
/// let blend = BiTgteBlend::new();
/// assert!(!blend.is_done());
/// assert_eq!(blend.nb_surfaces(), 0);
/// ```
pub struct BiTgteBlend {
    my_radius: f64,
    my_tol: f64,
    my_nubs: bool,
    my_shape: Shape,
    my_result: Shape,
    my_build_shape: bool,
    my_ancestors: IndexedDataMapOfShapeListOfShape,
    my_created: CreatedMap,
    my_cut_edges: DataMap<Shape, List<Shape>, ShapeMapHasher>,
    my_faces: IndexedMapOfShape,
    my_edges: IndexedMapOfShape,
    my_stop_faces: MapOfShape,
    my_analyse: BRepOffsetAnalyse,
    my_centers: IndexedMapOfShape,
    my_map_sf: OffsetFaces,
    my_init_offset_face: BRepAlgoImage,
    my_image: BRepAlgoImage,
    my_image_offset: BRepAlgoImage,
    my_as_des: Handle<BRepAlgoAsDes>,
    my_nb_branches: i32,
    my_indices: Vec<i32>,
    my_done: bool,
}

impl Default for BiTgteBlend {
    fn default() -> Self {
        Self::new()
    }
}

impl BiTgteBlend {
    /// `BiTgte_Blend()`: no shape (the radius, tolerance and flags uninitialized
    /// natively are 0 and false); `BiTgteBlend::new()`.
    pub fn new() -> Self {
        Self {
            my_radius: 0.,
            my_tol: 0.,
            my_nubs: false,
            my_shape: Shape::default(),
            my_result: Shape::default(),
            my_build_shape: false,
            my_ancestors: IndexedDataMapOfShapeListOfShape::default(),
            my_created: CreatedMap::default(),
            my_cut_edges: DataMap::default(),
            my_faces: IndexedMapOfShape::default(),
            my_edges: IndexedMapOfShape::default(),
            my_stop_faces: MapOfShape::default(),
            my_analyse: BRepOffsetAnalyse::new(),
            my_centers: IndexedMapOfShape::default(),
            my_map_sf: OffsetFaces::default(),
            my_init_offset_face: BRepAlgoImage::new(),
            my_image: BRepAlgoImage::new(),
            my_image_offset: BRepAlgoImage::new(),
            my_as_des: Handle::new(BRepAlgoAsDes::new()),
            my_nb_branches: -1,
            my_indices: Vec::new(),
            my_done: false,
        }
    }

    /// `BiTgte_Blend(S, Radius, Tol, NUBS)`; `BiTgteBlend::with_shape(&shape, 1.0, 1e-4, false)`.
    pub fn with_shape(s: &Shape, radius: f64, tol: f64, nubs: bool) -> Self {
        let mut blend = Self::new();
        blend.init(s, radius, tol, nubs);
        blend
    }

    /// `Init(S, Radius, Tol, NUBS)`: clears and sets the shape and parameters;
    /// `blend.init(&shape, 1.0, 1e-4, false)`.
    pub fn init(&mut self, s: &Shape, radius: f64, tol: f64, nubs: bool) {
        self.clear();
        self.my_shape = s.clone();
        self.my_tol = tol;
        self.my_nubs = nubs;
        self.my_radius = radius;
        self.my_nb_branches = -1;
    }

    /// `Clear()`: the images, stop faces, analysis and ascendants/descendants (the
    /// faces, edges, centers and offsets are kept, as natively); `blend.clear()`.
    pub fn clear(&mut self) {
        self.my_init_offset_face.clear();
        self.my_image.clear();
        self.my_image_offset.clear();
        self.my_stop_faces.clear(true);
        self.my_analyse.clear();
        if let Some(as_des) = self.my_as_des.get() {
            as_des.clear_shared();
        }
        self.my_nb_branches = -1;
        self.my_done = false;
    }

    /// `SetStoppingFace(Face)`: a face on which the blend stops; `blend.set_stopping_face(&f)`.
    pub fn set_stopping_face(&mut self, face: &Face) {
        self.my_stop_faces.add(face.clone().into_shape());
    }

    /// `SetFaces(F1, F2)`: two faces to blend between; `blend.set_faces(&f1, &f2)`.
    pub fn set_faces(&mut self, f1: &Face, f2: &Face) {
        self.my_faces.add(f1.clone().into_shape());
        self.my_faces.add(f2.clone().into_shape());
    }

    /// `SetEdge(Edge)`: an edge to blend; `blend.set_edge(&e)`.
    pub fn set_edge(&mut self, edge: &Edge) {
        self.my_edges.add(edge.clone().into_shape());
    }

    /// `IsDone()`; `blend.is_done()`.
    pub fn is_done(&self) -> bool {
        self.my_done
    }

    /// `Shape()`: the blend result; `blend.shape()`.
    pub fn shape(&self) -> &Shape {
        &self.my_result
    }

    /// `NbSurfaces()`: the number of center lines (tubes and spheres);
    /// `blend.nb_surfaces()`.
    pub fn nb_surfaces(&self) -> i32 {
        self.my_centers.extent() as i32
    }

    /// The center line of an index (Standard_OutOfRange outside 1..=NbSurfaces).
    fn center(&self, index: i32) -> Result<Shape, Failure> {
        usize::try_from(index)
            .ok()
            .filter(|i| *i >= 1)
            .map(|i| self.my_centers.find_key(i).cloned())
            .unwrap_or_else(|| {
                Err(Failure::with_message(
                    "Standard_OutOfRange",
                    "NCollection_IndexedMap::FindKey",
                ))
            })
    }

    /// `Surface(Index)`; `blend.surface(1)?`.
    pub fn surface(&self, index: i32) -> Result<Handle<dyn GeomSurface>, Failure> {
        self.surface_of(&self.center(index)?)
    }

    /// `Face(Index)`; `blend.face(1)?`.
    pub fn face(&self, index: i32) -> Result<Face, Failure> {
        self.face_of(&self.center(index)?)
    }

    /// `CenterLines(LC)`: the center lines; `blend.center_lines()`.
    pub fn center_lines(&self) -> List<Shape> {
        let mut lc = List::default();
        for i in 1..=self.my_centers.extent() {
            if let Ok(c) = self.my_centers.find_key(i) {
                lc.append(c.clone());
            }
        }
        lc
    }

    /// The offset of a center line (Standard_NoSuchObject when it is not one).
    fn offset_of(
        &self,
        center_line: &Shape,
    ) -> Result<&crate::brep_offset::BRepOffsetOffset, Failure> {
        self.my_map_sf.seek(center_line).ok_or_else(|| {
            Failure::with_message("Standard_NoSuchObject", "NCollection_DataMap::Find")
        })
    }

    /// `Surface(CenterLine)`: the surface of the blend face;
    /// `blend.surface_of(&center)?`.
    pub fn surface_of(&self, center_line: &Shape) -> Result<Handle<dyn GeomSurface>, Failure> {
        BRepTool::surface(self.offset_of(center_line)?.face())
    }

    /// `Face(CenterLine)`: the blend face; Standard_DomainError for a shape that is not a
    /// center line; `blend.face_of(&center)?`.
    pub fn face_of(&self, center_line: &Shape) -> Result<Face, Failure> {
        match self.my_map_sf.seek(center_line) {
            Some(of) => Ok(of.face().clone()),
            None => Err(Failure::with_message(
                "Standard_DomainError",
                "BiTgte_Blend::Face",
            )),
        }
    }

    /// `ContactType(Index)`: the kinds of the two support shapes.
    pub fn contact_type(&self, index: i32) -> Result<BiTgteContactType, Failure> {
        let s1 = self.support_shape1(index)?;
        let s2 = self.support_shape2(index)?;

        let mut type1 = s1.shape_type()?;
        let mut type2 = s2.shape_type()?;

        if (type2 as i32) < (type1 as i32) {
            std::mem::swap(&mut type1, &mut type2);
        }
        let ty = match (type1, type2) {
            (ShapeEnum::Vertex, ShapeEnum::Vertex) => BiTgteContactType::VertexVertex,
            (ShapeEnum::Vertex, ShapeEnum::Edge) => BiTgteContactType::EdgeVertex,
            (ShapeEnum::Vertex, ShapeEnum::Face) => BiTgteContactType::FaceVertex,
            (ShapeEnum::Edge, ShapeEnum::Edge) => BiTgteContactType::EdgeEdge,
            (ShapeEnum::Edge, ShapeEnum::Face) => BiTgteContactType::FaceEdge,
            // A face-face contact is reported as FaceEdge natively.
            (ShapeEnum::Face, ShapeEnum::Face) => BiTgteContactType::FaceEdge,
            _ => BiTgteContactType::VertexVertex,
        };
        Ok(ty)
    }

    /// The ascendants of a center line in the AsDes.
    fn ascendants(&self, s: &Shape) -> Vec<Shape> {
        self.my_as_des
            .get()
            .map_or_else(Vec::new, |a| a.ascendant(s).collect())
    }

    /// `SupportShape1(Index)`: the shape whose offset is the first face of the center
    /// line.
    pub fn support_shape1(&self, index: i32) -> Result<Shape, Failure> {
        let cur_e = topods::edge(&self.center(index)?)?;
        let l = self.ascendants(&cur_e);
        // F1 and F2 = 2 parallel faces intersecting at CurE.
        let f1 = topods::face(l.first().ok_or_else(empty_list)?)?;
        self.my_init_offset_face.image_from(&f1)
    }

    /// `SupportShape2(Index)`: the shape whose offset is the second face of the center
    /// line.
    pub fn support_shape2(&self, index: i32) -> Result<Shape, Failure> {
        let cur_e = topods::edge(&self.center(index)?)?;
        let l = self.ascendants(&cur_e);
        let f2 = topods::face(l.last().ok_or_else(empty_list)?)?;
        self.my_init_offset_face.image_from(&f2)
    }

    /// The trimmed curve of the edge of rank `rank` of the blend face (null when
    /// degenerated).
    fn curve_on_shape(&self, index: i32, rank: usize) -> Result<Handle<dyn GeomCurve>, Failure> {
        let cur_e = topods::edge(&self.center(index)?)?;
        let f = self.offset_of(&cur_e)?.face().clone();

        // somewhat brutal method based ONLY on the construction of the fillet:
        // the first edge of the tube is exactly the edge on Shape1.
        let e = Explorer::new(&f, ShapeEnum::Edge, ShapeEnum::Shape)?
            .nth(rank)
            .ok_or_else(|| {
                Failure::with_message("Standard_NoSuchObject", "TopExp_Explorer::Current")
            })?;
        let e = topods::edge(&e)?;
        let mut c: Handle<dyn GeomCurve> = Handle::default();
        if !BRepTool::degenerated(&e)? {
            let (curve, f, l) = BRepTool::curve(&e)?;
            c = Handle::new(GeomTrimmedCurve::new(&curve, f, l, true, true)?).cast();
        }
        Ok(c)
    }

    /// `CurveOnShape1(Index)`: the curve of the first edge of the blend face.
    pub fn curve_on_shape1(&self, index: i32) -> Result<Handle<dyn GeomCurve>, Failure> {
        self.curve_on_shape(index, 0)
    }

    /// `CurveOnShape2(Index)`: the curve of the second edge of the blend face.
    pub fn curve_on_shape2(&self, index: i32) -> Result<Handle<dyn GeomCurve>, Failure> {
        self.curve_on_shape(index, 1)
    }

    /// `PCurveOnFace1(Index)`: a null curve, as natively.
    pub fn pcurve_on_face1(&self, _index: i32) -> Handle<dyn Geom2dCurve> {
        Handle::default()
    }

    /// `PCurve1OnFillet(Index)`: a null curve, as natively.
    pub fn pcurve1_on_fillet(&self, _index: i32) -> Handle<dyn Geom2dCurve> {
        Handle::default()
    }

    /// `PCurveOnFace2(Index)`: a null curve, as natively.
    pub fn pcurve_on_face2(&self, _index: i32) -> Handle<dyn Geom2dCurve> {
        Handle::default()
    }

    /// `PCurve2OnFillet(Index)`: a null curve, as natively.
    pub fn pcurve2_on_fillet(&self, _index: i32) -> Handle<dyn Geom2dCurve> {
        Handle::default()
    }

    /// `NbBranches()`: the number of connected sets of blend faces (computed once; the
    /// center lines are reordered branch by branch); `blend.nb_branches()?`.
    pub fn nb_branches(&mut self) -> Result<i32, Failure> {
        if self.my_nb_branches != -1 {
            return Ok(self.my_nb_branches);
        }

        // else, compute the Branches.
        let mut glue = Quilt::new();

        let nb_faces = self.my_centers.extent();

        if nb_faces == 0 {
            return Ok(0);
        }

        for i in 1..=nb_faces {
            let center_line = self.my_centers.find_key(i)?.clone();
            glue.add(&self.offset_of(&center_line)?.face().clone().into_shape())?;
        }

        let shells = glue.shells()?;

        // Reorder Map myCenters.
        // The method is brutal and unpolished,
        // it is possible to refine it.
        self.my_nb_branches = 0;
        let mut tmp_map = IndexedMapOfShape::default();

        self.my_nb_branches =
            Explorer::new(&shells, ShapeEnum::Shell, ShapeEnum::Shape)?.count() as i32;

        self.my_indices = vec![0; self.my_nb_branches as usize + 1];

        self.my_indices[0] = 0;
        let mut count = 0;

        for (index, cur_s) in (2..).zip(Explorer::new(&shells, ShapeEnum::Shell, ShapeEnum::Shape)?)
        {
            // CurS = the current Shell.
            for cur_f in Explorer::new(&cur_s, ShapeEnum::Face, ShapeEnum::Shape)? {
                // CurF = the current face of the current Shell.
                for i in 1..=nb_faces {
                    let center = self.my_centers.find_key(i)?.clone();
                    let rakk = self.offset_of(&center)?.face().clone();
                    // Rakk = the ith generated connection face
                    if cur_f.is_equal(&rakk) {
                        tmp_map.add(center);
                        count += 1;
                        break;
                    }
                }
            }
            self.my_indices[index - 1usize] = count;
        }

        self.my_centers = tmp_map;
        Ok(self.my_nb_branches)
    }

    /// `IndicesOfBranche(Index, From, To)`: the range of the center lines of a branch
    /// (`(From, To)`); Standard_OutOfRange outside the branches.
    pub fn indices_of_branche(&self, index: i32) -> Result<(i32, i32), Failure> {
        // Attention to the ranking in myIndices:
        // If the branches are  1-4 5-9 10-12, it is ranked in myIndices:
        //                      0 4   9    12
        let at = |i: i32| -> Result<i32, Failure> {
            usize::try_from(i - 1)
                .ok()
                .and_then(|k| self.my_indices.get(k).copied())
                .ok_or_else(|| {
                    Failure::with_message("Standard_OutOfRange", "NCollection_Array1::Value")
                })
        };
        Ok((at(index)? + 1, at(index + 1)?))
    }
}

/// Standard_NoSuchObject of the first or last item of an empty list.
fn empty_list() -> Failure {
    Failure::with_message("Standard_NoSuchObject", "NCollection_List::First")
}
