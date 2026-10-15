// Rust translation of ModelingAlgorithms/TKOffset/BRepOffset/BRepOffset_MakeOffset_1.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871: the local class BRepOffset_BuildOffsetFaces
// (its members are split over the sibling modules) and its static helpers.

//! The splits of the offset faces of the complete intersection mode: the faces are split
//! by their intersection edges, the invalid splits (with inverted edges) are found,
//! removed or rebuilt by new intersections until the splits can form closed volumes.
mod bounds;
mod connections;
mod filter;
mod gaps;
mod inside;
mod intersect_faces;
mod invalid_edges;
mod invalid_faces;
mod inverted;
mod splits;
mod trim_edges;

use crate::brep_offset::BRepOffsetAnalyse;
use rustycad_tkbo::bop_algo::{BOPAlgoBuilder, BOPAlgoSplitter, BuilderFace};
use rustycad_tkbo::bop_tools;
use rustycad_tkbo::int_tools::Context;
use rustycad_tkbool::brep_algo::{BRepAlgoAsDes, BRepAlgoImage};
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_adaptor::BRepAdaptorCurve;
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Edge, Face, Shape};
use rustycad_tkbrep::toptools::{
    IndexedDataMapOfShapeListOfShape, IndexedMapOfShape, MapOfShape, ShapeMapHasher,
};
use rustycad_tkernel::message::ProgressRange;
use rustycad_tkernel::ncollection::{DataMap, List};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::geom_abs::CurveType;
use rustycad_tkmath::gp::Vector3;
use rustycad_tktopalgo::brep_lib;

/// A data map of shapes to shape lists (images, origins, ancestors).
pub(crate) type ShapeListMap = DataMap<Shape, List<Shape>, ShapeMapHasher>;
/// A data map of shapes to shapes.
pub(crate) type ShapeShapeMap = DataMap<Shape, Shape, ShapeMapHasher>;
/// A data map of shapes to shape maps.
pub(crate) type ShapeMapMap = DataMap<Shape, MapOfShape, ShapeMapHasher>;
/// `BRepOffset_DataMapOfShapeIndexedMapOfShape`.
pub(crate) type ShapeIndexedMapMap = DataMap<Shape, IndexedMapOfShape, ShapeMapHasher>;

/// A container of shapes for `AddToContainer` and `TakeModified`.
pub(crate) trait ShapeContainer {
    /// Adds the shape; true when it was added (always for lists and compounds).
    fn put(&mut self, s: &Shape) -> bool;
}

impl ShapeContainer for List<Shape> {
    fn put(&mut self, s: &Shape) -> bool {
        self.append(s.clone());
        true
    }
}

impl ShapeContainer for MapOfShape {
    fn put(&mut self, s: &Shape) -> bool {
        self.add(s.clone())
    }
}

impl ShapeContainer for IndexedMapOfShape {
    /// `AddToContainer` on an indexed map: true when the index is new.
    fn put(&mut self, s: &Shape) -> bool {
        let nb = self.extent();
        let ind = self.add(s.clone());
        ind > nb
    }
}

impl ShapeContainer for Shape {
    /// `BRep_Builder().Add(theSOut, theS)`.
    fn put(&mut self, s: &Shape) -> bool {
        let _ = BRepBuilder::default().add(self, s);
        true
    }
}

/// `AddToContainer(theKey, theValue, theMap)`: the value appended to the list of the key.
pub(crate) fn add_to_map_list(key: &Shape, value: &Shape, map: &mut ShapeListMap) {
    if let Some(l) = map.change_seek(key) {
        l.append(value.clone());
    } else {
        let mut l = List::default();
        l.append(value.clone());
        map.bind(key.clone(), l);
    }
}

/// `TakeModified(theS, theImages, theContainer, theMFence)`: the images of the shape
/// (or the shape itself) put in the container when the fence admits them; true when
/// the shape has images.
pub(crate) fn take_modified(
    s: &Shape,
    images: &ShapeListMap,
    container: &mut dyn ShapeContainer,
    mut fence: Option<&mut dyn ShapeContainer>,
) -> bool {
    if let Some(ls_im) = images.seek(s) {
        for s_im in ls_im.iter() {
            if fence.as_mut().is_none_or(|f| f.put(s_im)) {
                container.put(s_im);
            }
        }
        true
    } else {
        if fence.as_mut().is_none_or(|f| f.put(s)) {
            container.put(s);
        }
        false
    }
}

/// `AppendToList(theList, theShape)`: the shape appended unless it is there (IsSame).
pub(crate) fn append_to_list(list: &mut List<Shape>, shape: &Shape) {
    if list.iter().any(|s| s.is_same(shape)) {
        return;
    }
    list.append(shape.clone());
}

/// `ProcessMicroEdge(theEdge, theCtx)`: whether the edge is a micro edge (a straight one
/// gets its vertex tolerances raised to half its length).
pub(crate) fn process_micro_edge(edge: &Edge, ctx: &Handle<Context>) -> Result<bool, Failure> {
    let (v1, v2) = topexp::vertices(edge, false);
    if v1.is_null() || v2.is_null() {
        return Ok(false);
    }
    let b_micro = bop_tools::is_micro_edge(edge, ctx, true)?;
    if b_micro && BRepAdaptorCurve::with_edge(edge)?.get_type()? == CurveType::Line {
        let len = BRepTool::pnt(&v1)?.distance(&BRepTool::pnt(&v2)?);
        BRepBuilder::default().update_vertex_tolerance(&v1, len / 2.)?;
        BRepBuilder::default().update_vertex_tolerance(&v2, len / 2.)?;
    }
    Ok(b_micro)
}

/// The modified images of a shape in a builder.
pub(crate) fn modified_of(gf: &BOPAlgoBuilder, s: &Shape) -> Result<List<Shape>, Failure> {
    Ok(gf.modified(s)?.into_iter().collect())
}

/// `UpdateOrigins(theLA, theOrigins, theGF)`: the origins of the arguments passed to
/// their images.
pub(crate) fn update_origins(
    la: &List<Shape>,
    origins: &mut ShapeListMap,
    gf: &BOPAlgoBuilder,
) -> Result<(), Failure> {
    for s in la.iter() {
        let ls_im = modified_of(gf, s)?;
        if ls_im.is_empty() {
            continue;
        }
        let ls = match origins.seek(s) {
            Some(l) => l.clone(),
            None => {
                let mut l = List::default();
                l.append(s.clone());
                l
            }
        };
        for s_im in ls_im.iter() {
            if let Some(ls_or) = origins.change_seek(s_im) {
                // merge two lists
                for x in ls.iter() {
                    append_to_list(ls_or, x);
                }
            } else {
                origins.bind(s_im.clone(), ls.clone());
            }
        }
    }
    Ok(())
}

/// `UpdateImages(theLA, theImages, theGF, theModified)`: the images of the arguments
/// replaced by their modifications in the builder.
pub(crate) fn update_images(
    la: &List<Shape>,
    images: &mut ShapeListMap,
    gf: &BOPAlgoBuilder,
    modified: &mut MapOfShape,
) -> Result<(), Failure> {
    for s in la.iter() {
        let Some(ls_im) = images.seek(s).cloned() else {
            let ls_im = modified_of(gf, s)?;
            if !ls_im.is_empty() {
                images.bind(s.clone(), ls_im);
                modified.add(s.clone());
            }
            continue;
        };
        let mut m_fence = MapOfShape::default();
        let mut ls_im_new = List::default();
        let mut b_modified = false;
        // check modifications of the images
        for s_im in ls_im.iter() {
            b_modified |= take_modified(s_im, gf.images(), &mut ls_im_new, Some(&mut m_fence));
        }
        if b_modified {
            if let Some(l) = images.change_seek(s) {
                *l = ls_im_new;
            }
            modified.add(s.clone());
        }
    }
    Ok(())
}

/// `FindCommonParts(theLS1, theLS2, theLSC, theType)`: the sub-shapes of the type in
/// both lists.
pub(crate) fn find_common_parts(
    ls1: &List<Shape>,
    ls2: &List<Shape>,
    lsc: &mut List<Shape>,
    ty: ShapeEnum,
) -> Result<(), Failure> {
    // map shapes in the first list
    let mut ms1 = IndexedMapOfShape::default();
    for s in ls1.iter() {
        topexp::map_shapes_of_type(s, ty, &mut ms1)?;
    }
    if ms1.is_empty() {
        return Ok(());
    }
    // check for such shapes in the other list
    let mut m_fence = MapOfShape::default();
    for s in ls2.iter() {
        for st in Explorer::new(s, ty, ShapeEnum::Shape)? {
            if ms1.contains(&st) && m_fence.add(st.clone()) {
                lsc.append(st);
            }
        }
    }
    Ok(())
}

/// `NbPoints(theEdge)`: 1 sample point for a line, 11 otherwise.
pub(crate) fn nb_points(edge: &Edge) -> Result<i32, Failure> {
    Ok(match BRepAdaptorCurve::with_edge(edge)?.get_type()? {
        CurveType::Line => 1,
        _ => 11,
    })
}

/// `FindShape(theSWhat, theSWhere, theAnalyse, theRes)`: the sub-shape of the shape (or
/// of its descendants in the analysis) same as the one searched.
pub(crate) fn find_shape(
    s_what: &Shape,
    s_where: &Shape,
    analyse: Option<&BRepOffsetAnalyse>,
) -> Result<Option<Shape>, Failure> {
    let ty = s_what.shape_type()?;
    for s in Explorer::new(s_where, ty, ShapeEnum::Shape)? {
        if s.is_same(s_what) {
            return Ok(Some(s));
        }
    }
    if let Some(analyse) = analyse
        && let Some(ld) = analyse.descendants(s_where, false)
    {
        for s in ld {
            if s.is_same(s_what) {
                return Ok(Some(s.clone()));
            }
        }
    }
    Ok(None)
}

/// `BuildSplitsOfTrimmedFace(theFace, theEdges, theLFImages, Range)`: the faces of the
/// face split by the edges (BOPAlgo_Splitter), nothing on errors.
pub(crate) fn build_splits_of_trimmed_face(
    face: &Face,
    edges: &Shape,
    lf_images: &mut List<Shape>,
) -> Result<(), Failure> {
    let mut splitter = BOPAlgoSplitter::new();
    splitter.add_argument(face);
    splitter.add_argument(edges);
    splitter.set_to_fill_history(false);
    splitter.perform();
    if splitter.options().has_errors() {
        return Ok(());
    }
    // splits of the offset shape
    for f in Explorer::new(splitter.shape(), ShapeEnum::Face, ShapeEnum::Shape)? {
        lf_images.append(f);
    }
    Ok(())
}

/// `BuildSplitsOfFace(theFace, theEdges, theFacesOrigins, theLFImages)`: the areas of the
/// planar face bounded by the edges (both orientations, with their p-curves on the
/// plane), oriented as the face and bound to it as their origin.
pub(crate) fn build_splits_of_face(
    face: &Face,
    edges: &Shape,
    faces_origins: &mut ShapeShapeMap,
    lf_images: &mut List<Shape>,
) -> Result<(), Failure> {
    lf_images.clear();
    // take edges to split the face
    let mut le = List::default();
    for e in Explorer::new(edges, ShapeEnum::Edge, ShapeEnum::Shape)? {
        le.append(e.oriented(Orientation::Forward));
        le.append(e.oriented(Orientation::Reversed));
    }
    let an_or = face.orientation();
    let ff = topods::face(&face.oriented(Orientation::Forward))?;
    // build pcurves for edges on the face
    brep_lib::build_pcurve_for_edges_on_plane(le.iter(), &ff)?;
    // build splits of faces
    let mut bf = BuilderFace::new();
    bf.set_face(&ff);
    bf.set_shapes(&le);
    if bf.perform().is_err() || bf.options().has_errors() {
        return Ok(());
    }
    for f_sp in bf.areas().iter() {
        let f_sp = f_sp.oriented(an_or);
        lf_images.append(f_sp.clone());
        faces_origins.bind(f_sp, face.clone().into_shape());
    }
    Ok(())
}

/// `GetAverageTangent(theS, theNbP)`: the sum of the unit tangents sampled on the edges.
pub(crate) fn get_average_tangent(s: &Shape, nb_p: i32) -> Result<Vector3, Failure> {
    let mut va = Vector3::default();
    for e in Explorer::new(s, ShapeEnum::Edge, ShapeEnum::Shape)? {
        let e = topods::edge(&e)?;
        let (c, t1, t2) = BRepTool::curve(&e)?;
        let c = c.get().ok_or_else(|| {
            Failure::with_message("Standard_NullObject", "BRepOffset_MakeOffset_1: no curve")
        })?;
        let mut v_sum = Vector3::default();
        let mut t = t1;
        let dt = (t2 - t1) / f64::from(nb_p);
        while t <= t2 {
            let (_, v) = c.d1(t)?;
            v_sum += v.normalized()?;
            t += dt;
        }
        if e.orientation() == Orientation::Reversed {
            v_sum.reverse();
        }
        va += v_sum;
    }
    Ok(va)
}

/// `BRepOffset_BuildOffsetFaces`: the tool building the splits of the offset faces,
/// with the input data of the offset algorithm, the intermediate maps and the history.
pub(crate) struct BuildOffsetFaces<'a> {
    // Input data
    my_faces: Option<&'a List<Shape>>,
    my_as_des: Option<&'a BRepAlgoAsDes>,
    my_analyzer: Option<&'a BRepOffsetAnalyse>,
    my_edges_origins: Option<&'a mut ShapeListMap>,
    my_faces_origins: Option<&'a mut ShapeShapeMap>,
    my_e_trim_e_inf: Option<&'a mut ShapeShapeMap>,
    // The local origins of BuildSplitsOfTrimmedFaces without given ones.
    my_local_edges_origins: ShapeListMap,

    // Intermediate data
    my_oe_images: ShapeListMap,
    my_oe_origins: ShapeListMap,
    my_of_images: IndexedDataMapOfShapeListOfShape,

    my_invalid_edges: IndexedMapOfShape,
    my_valid_edges: IndexedMapOfShape,
    my_inverted_edges: IndexedMapOfShape,
    my_edges_to_avoid: IndexedMapOfShape,
    my_last_inv_edges: MapOfShape,
    my_modified_edges: MapOfShape,
    my_inside_edges: IndexedMapOfShape,

    my_invalid_faces: IndexedDataMapOfShapeListOfShape,
    my_art_invalid_faces: ShapeIndexedMapMap,
    my_already_inv_faces: DataMap<Shape, i32, ShapeMapHasher>,
    my_f_new_holes: ShapeListMap,

    my_ss_interfs: ShapeListMap,
    my_ss_interfs_art: ShapeListMap,
    my_intersection_pairs: DataMap<Shape, ShapeMapMap, ShapeMapHasher>,

    my_faces_to_rebuild: IndexedDataMapOfShapeListOfShape,
    my_f_self_reb_avoid: MapOfShape,

    my_solids: Shape,

    // Auxiliary tools
    my_context: Handle<Context>,

    // Output
    my_image: &'a mut BRepAlgoImage,
}

impl<'a> BuildOffsetFaces<'a> {
    /// `BRepOffset_BuildOffsetFaces(theImage)`.
    pub(crate) fn new(image: &'a mut BRepAlgoImage) -> Self {
        Self {
            my_faces: None,
            my_as_des: None,
            my_analyzer: None,
            my_edges_origins: None,
            my_faces_origins: None,
            my_e_trim_e_inf: None,
            my_local_edges_origins: ShapeListMap::default(),
            my_oe_images: ShapeListMap::default(),
            my_oe_origins: ShapeListMap::default(),
            my_of_images: IndexedDataMapOfShapeListOfShape::default(),
            my_invalid_edges: IndexedMapOfShape::default(),
            my_valid_edges: IndexedMapOfShape::default(),
            my_inverted_edges: IndexedMapOfShape::default(),
            my_edges_to_avoid: IndexedMapOfShape::default(),
            my_last_inv_edges: MapOfShape::default(),
            my_modified_edges: MapOfShape::default(),
            my_inside_edges: IndexedMapOfShape::default(),
            my_invalid_faces: IndexedDataMapOfShapeListOfShape::default(),
            my_art_invalid_faces: ShapeIndexedMapMap::default(),
            my_already_inv_faces: DataMap::default(),
            my_f_new_holes: ShapeListMap::default(),
            my_ss_interfs: ShapeListMap::default(),
            my_ss_interfs_art: ShapeListMap::default(),
            my_intersection_pairs: DataMap::default(),
            my_faces_to_rebuild: IndexedDataMapOfShapeListOfShape::default(),
            my_f_self_reb_avoid: MapOfShape::default(),
            my_solids: Shape::default(),
            my_context: Handle::new(Context::default()),
            my_image: image,
        }
    }

    /// `SetFaces(theFaces)`.
    pub(crate) fn set_faces(&mut self, faces: &'a List<Shape>) {
        self.my_faces = Some(faces);
    }

    /// `SetAsDesInfo(theAsDes)`.
    pub(crate) fn set_as_des_info(&mut self, as_des: &'a BRepAlgoAsDes) {
        self.my_as_des = Some(as_des);
    }

    /// `SetAnalysis(theAnalyse)`.
    pub(crate) fn set_analysis(&mut self, analyse: &'a BRepOffsetAnalyse) {
        self.my_analyzer = Some(analyse);
    }

    /// `SetEdgesOrigins(theEdgesOrigins)`.
    pub(crate) fn set_edges_origins(&mut self, origins: &'a mut ShapeListMap) {
        self.my_edges_origins = Some(origins);
    }

    /// `SetFacesOrigins(theFacesOrigins)`.
    pub(crate) fn set_faces_origins(&mut self, origins: &'a mut ShapeShapeMap) {
        self.my_faces_origins = Some(origins);
    }

    /// `SetInfEdges(theETrimEInf)`.
    pub(crate) fn set_inf_edges(&mut self, e_trim_e_inf: &'a mut ShapeShapeMap) {
        self.my_e_trim_e_inf = Some(e_trim_e_inf);
    }

    /// The intersection context.
    fn context(&self) -> Result<&Context, Failure> {
        self.my_context.get().ok_or_else(|| {
            Failure::with_message(
                "Standard_NullObject",
                "BRepOffset_BuildOffsetFaces: no context",
            )
        })
    }

    /// The input faces (empty when not set).
    fn faces(&self) -> List<Shape> {
        self.my_faces.cloned().unwrap_or_default()
    }

    /// The AsDes (Standard_NullObject when not set, a null dereference natively).
    fn as_des(&self) -> Result<&'a BRepAlgoAsDes, Failure> {
        self.my_as_des.ok_or_else(|| {
            Failure::with_message(
                "Standard_NullObject",
                "BRepOffset_BuildOffsetFaces: no AsDes",
            )
        })
    }

    /// The descendants of a shape in the AsDes.
    fn descendants(&self, s: &Shape) -> Result<Vec<Shape>, Failure> {
        Ok(self.as_des()?.descendant(s).collect())
    }

    /// The origins of the offset edges (given, or the local map).
    fn edges_origins(&mut self) -> &mut ShapeListMap {
        match self.my_edges_origins.as_deref_mut() {
            Some(m) => m,
            None => &mut self.my_local_edges_origins,
        }
    }

    /// The origins of the offset faces (Standard_NullObject when not set).
    fn faces_origins(&mut self) -> Result<&mut ShapeShapeMap, Failure> {
        self.my_faces_origins.as_deref_mut().ok_or_else(|| {
            Failure::with_message(
                "Standard_NullObject",
                "BRepOffset_BuildOffsetFaces: no faces origins",
            )
        })
    }

    /// The origins of the offset faces, to read.
    fn faces_origins_ref(&self) -> Result<&ShapeShapeMap, Failure> {
        self.my_faces_origins.as_deref().ok_or_else(|| {
            Failure::with_message(
                "Standard_NullObject",
                "BRepOffset_BuildOffsetFaces: no faces origins",
            )
        })
    }

    /// The infinite edges of the trimmed ones (Standard_NullObject when not set).
    fn e_trim_e_inf(&mut self) -> Result<&mut ShapeShapeMap, Failure> {
        self.my_e_trim_e_inf.as_deref_mut().ok_or_else(|| {
            Failure::with_message(
                "Standard_NullObject",
                "BRepOffset_BuildOffsetFaces: no infinite edges",
            )
        })
    }

    /// The infinite edges of the trimmed ones, to read.
    fn e_trim_e_inf_ref(&self) -> Result<&ShapeShapeMap, Failure> {
        self.my_e_trim_e_inf.as_deref().ok_or_else(|| {
            Failure::with_message(
                "Standard_NullObject",
                "BRepOffset_BuildOffsetFaces: no infinite edges",
            )
        })
    }

    /// `BuildSplitsOfTrimmedFaces(theRange)`: the faces split by their intersected
    /// edges, then the history.
    pub(crate) fn build_splits_of_trimmed_faces(
        &mut self,
        _range: &ProgressRange,
    ) -> Result<(), Failure> {
        if self.my_faces.is_none_or(|f| f.is_empty()) {
            return Ok(());
        }

        // Fuse all edges
        self.intersect_trimmed_edges()?;

        for a_f in self.faces().iter() {
            let a_f = topods::face(a_f)?;
            let mut a_ce = Shape::default();
            let b_found = self.get_edges(&a_f, &mut a_ce, None)?;

            // split the face by the edges
            if !b_found {
                if !self.my_image.has_image(&a_f) {
                    let i = self
                        .my_of_images
                        .add(a_f.clone().into_shape(), List::default());
                    self.my_of_images
                        .change_from_index(i)?
                        .append(a_f.into_shape());
                }
                continue;
            }

            let mut a_lf_images = List::default();
            build_splits_of_trimmed_face(&a_f, &a_ce, &mut a_lf_images)?;

            self.my_of_images.add(a_f.into_shape(), a_lf_images);
        }
        // Fill history for faces and edges
        self.fill_history()
    }

    /// `BuildSplitsOfExtendedFaces(theRange)`: the faces split by their intersected
    /// edges, the invalid splits found and rebuilt (at most ten times), the gaps filled,
    /// then the history.
    pub(crate) fn build_splits_of_extended_faces(
        &mut self,
        _range: &ProgressRange,
    ) -> Result<(), Failure> {
        // Check input data
        if self.my_faces.is_none_or(|f| f.is_empty())
            || self
                .my_edges_origins
                .as_deref()
                .is_none_or(|m| m.is_empty())
            || self
                .my_faces_origins
                .as_deref()
                .is_none_or(|m| m.is_empty())
            || self.my_e_trim_e_inf.as_deref().is_none_or(|m| m.is_empty())
        {
            return Ok(());
        }

        // Fusing all trimmed offset edges to avoid self-intersections in the splits
        self.intersect_trimmed_edges()?;
        // vertices to avoid
        let mut verts_to_avoid = MapOfShape::default();

        // Limit total number of attempts by 10. This should be extra, as each invalid face can be
        // rebuilt only three times. So, in general, there are about 2-5 loops done.
        let nb_max_attempts = 10;
        for _ in 1..=nb_max_attempts {
            // Clear the data before further faces construction
            self.my_invalid_faces.clear(true);
            self.my_art_invalid_faces.clear(true);
            self.my_invalid_edges.clear(true);
            self.my_inverted_edges.clear(true);
            self.my_ss_interfs.clear(true);
            self.my_ss_interfs_art.clear(true);
            self.my_intersection_pairs.clear(true);
            self.my_solids.nullify();
            self.my_faces_to_rebuild.clear(true);
            self.my_f_self_reb_avoid.clear(true);

            // Build splits of the faces having new intersection edges
            self.build_splits_of_faces()?;
            if self.my_invalid_faces.is_empty() {
                break;
            }

            // Find faces to rebuild
            self.find_faces_to_rebuild()?;
            if self.my_faces_to_rebuild.is_empty() {
                break;
            }

            // Perform new intersections
            self.my_modified_edges.clear(true);
            self.intersect_faces(&mut verts_to_avoid)?;
        }

        // Fill possible gaps in the splits of offset faces to increase possibility of
        // creating closed volume from these splits
        self.fill_gaps()?;

        // Fill history for faces and edges
        self.fill_history()
    }

    /// `UpdateIntersectedEdges(theLA, theGF)`: the infinite edges of the trimmed edges
    /// passed to their images.
    fn update_intersected_edges(
        &mut self,
        la: &List<Shape>,
        gf: &BOPAlgoBuilder,
    ) -> Result<(), Failure> {
        for s in la.iter() {
            let Some(e_inf) = self.e_trim_e_inf_ref()?.seek(s).cloned() else {
                continue;
            };
            let ls_im = modified_of(gf, s)?;
            if ls_im.is_empty() {
                continue;
            }
            let map = self.e_trim_e_inf()?;
            for e_im in ls_im.iter() {
                if !map.is_bound(e_im) {
                    map.bind(e_im.clone(), e_inf.clone());
                }
            }
        }
        Ok(())
    }

    /// `IntersectTrimmedEdges(theRange)`: the edges of the offset faces (not micro
    /// edges) fused with each other; their images and origins recorded.
    fn intersect_trimmed_edges(&mut self) -> Result<(), Failure> {
        // get edges to intersect from descendants of the offset faces
        let mut ls = List::default();
        for a_f in self.faces().iter() {
            for e in self.descendants(a_f)? {
                let e = topods::edge(&e)?;
                if process_micro_edge(&e, &self.my_context)? {
                    continue;
                }
                if self.my_modified_edges.add(e.clone().into_shape()) {
                    ls.append(e.into_shape());
                }
            }
        }
        if ls.len() < 2 {
            // nothing to intersect
            return Ok(());
        }
        // perform intersection of the edges
        let mut gfe = BOPAlgoBuilder::new();
        gfe.set_arguments(&ls);
        gfe.perform();
        if gfe.options().has_errors() {
            return Ok(());
        }
        let mut la = List::default();
        // fill map with edges images
        for e in ls.iter() {
            let le_im = modified_of(&gfe, e)?;
            if le_im.is_empty() {
                continue;
            }
            la.append(e.clone());
            // save images
            self.my_oe_images.bind(e.clone(), le_im.clone());
            // save origins
            for e_im in le_im.iter() {
                if let Some(le_or) = self.my_oe_origins.change_seek(e_im) {
                    append_to_list(le_or, e);
                } else {
                    let mut l = List::default();
                    l.append(e.clone());
                    self.my_oe_origins.bind(e_im.clone(), l);
                }
            }
        }
        update_origins(&la, self.edges_origins(), &gfe)?;
        self.update_intersected_edges(&la, &gfe)
    }
}
