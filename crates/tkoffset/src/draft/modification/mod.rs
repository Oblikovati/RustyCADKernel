// Rust translation of ModelingAlgorithms/TKOffset/Draft/Draft_Modification.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871 (the private members of Draft_Modification_1.cxx
// are in the sibling modules).

//! `Draft_Modification`: the draft angle modification of faces of a shape, as a
//! `BRepTools_Modification`.
mod add;
mod geometry;
mod params;
mod perform;

use super::info::{DraftEdgeInfo, DraftErrorStatus, DraftFaceInfo, DraftVertexInfo};
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_tools::{self, BRepToolsModification, NewCurve, NewSurface};
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{Edge, Face, Shape, Vertex};
use rustycad_tkbrep::toptools::{IndexedDataMapOfShapeListOfShape, ShapeMapHasher};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::ncollection::{IndexedDataMap, List};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::geom::GeomSweptSurface;
use rustycad_tkg3d::geom::{
    GeomCurve, GeomRectangularTrimmedSurface, GeomSurface, GeomSurfaceOfLinearExtrusion,
    GeomTrimmedCurve,
};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkgeombase::geom_proj_lib;
use rustycad_tkmath::geom_abs::Shape as Continuity;
use rustycad_tkmath::gp::Point2;
use rustycad_tkmath::gp::{Direction3, Plane3, Point3, Vector2};
use std::f64::consts::PI;

/// The face records of the draft, by face.
pub(super) type FaceMap = IndexedDataMap<Shape, DraftFaceInfo, ShapeMapHasher>;
/// The edge records of the draft, by edge.
pub(super) type EdgeMap = IndexedDataMap<Shape, DraftEdgeInfo, ShapeMapHasher>;
/// The vertex records of the draft, by vertex.
pub(super) type VertexMap = IndexedDataMap<Shape, DraftVertexInfo, ShapeMapHasher>;

/// The fields of `Draft_Modification`.
#[derive(Default)]
pub(super) struct DraftState {
    pub(super) my_f_map: FaceMap,
    pub(super) my_e_map: EdgeMap,
    pub(super) my_v_map: VertexMap,
    pub(super) my_comp: bool,
    pub(super) my_shape: Shape,
    pub(super) bad_shape: Shape,
    pub(super) err_stat: DraftErrorStatus,
    pub(super) cur_face: Face,
    pub(super) conne_f: List<Shape>,
    pub(super) my_ef_map: IndexedDataMapOfShapeListOfShape,
}

/// The object of a live handle; Standard_NullObject otherwise (a null dereference
/// natively).
pub(super) fn live<'a, T: ?Sized + rustycad_tkernel::standard::RuntimeTyped>(
    handle: &'a Handle<T>,
    what: &str,
) -> Result<&'a T, Failure> {
    handle.get().ok_or_else(|| {
        Failure::with_message(
            "Standard_NullObject",
            &format!("Draft_Modification: null {what}"),
        )
    })
}

/// Standard_DomainError of a query before a successful `Perform`.
fn not_done_domain() -> Failure {
    Failure::with_message("Standard_DomainError", "Draft_Modification: not done")
}

/// The basis of a rectangular trimmed surface (`DynamicType() ==`), else the surface.
pub(super) fn untrimmed_surface(
    s: &Handle<dyn GeomSurface>,
) -> Result<Handle<dyn GeomSurface>, Failure> {
    if live(s, "surface")?.is_instance_name("Geom_RectangularTrimmedSurface") {
        return Ok(live(
            &s.clone().cast::<GeomRectangularTrimmedSurface>(),
            "surface",
        )?
        .basis_surface());
    }
    Ok(s.clone())
}

/// The basis of a trimmed curve (`DynamicType() ==`), else the curve.
pub(super) fn untrimmed_curve(c: &Handle<dyn GeomCurve>) -> Result<Handle<dyn GeomCurve>, Failure> {
    if live(c, "curve")?.is_instance_name("Geom_TrimmedCurve") {
        return Ok(live(&c.clone().cast::<GeomTrimmedCurve>(), "curve")?.basis_curve());
    }
    Ok(c.clone())
}

/// The basis of a trimmed curve when the handle down casts to one (`DownCast`, a null
/// curve stays null), else the curve.
pub(super) fn down_cast_basis(c: &Handle<dyn GeomCurve>) -> Handle<dyn GeomCurve> {
    match c.clone().cast::<GeomTrimmedCurve>().get() {
        Some(t) => t.basis_curve(),
        None => c.clone(),
    }
}

/// `Draft_Modification`: the draft of faces of a shape about neutral planes, as a
/// modification of its geometry (topology kept).
///
/// ```
/// use rustycad_tkbrep::topods::Shape;
/// use rustycad_tkoffset::draft::{DraftErrorStatus, DraftModification};
/// let draft = DraftModification::new(&Shape::default())?;
/// assert!(!draft.is_done());
/// assert_eq!(draft.error(), DraftErrorStatus::NoError);
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
pub struct DraftModification {
    parent: StandardTransient,
    state: SharedState<DraftState>,
}

impl_transient!(
    DraftModification, "Draft_Modification", dyn BRepToolsModification,
    parent, interfaces: [dyn BRepToolsModification]
);

impl DraftModification {
    /// `Draft_Modification(S)`: the edge to face ancestors of the shape (the error
    /// status, uninitialized natively, is NoError); `DraftModification::new(&shape)?`.
    pub fn new(s: &Shape) -> Result<Self, Failure> {
        let mut state = DraftState {
            my_shape: s.clone(),
            ..DraftState::default()
        };
        if !s.is_null() {
            topexp::map_shapes_and_ancestors(
                s,
                ShapeEnum::Edge,
                ShapeEnum::Face,
                &mut state.my_ef_map,
            )?;
        }
        Ok(Self {
            parent: StandardTransient::default(),
            state: SharedState::new(state),
        })
    }

    /// `Clear()`: resets on the same shape (the ancestor map is cleared too, as natively);
    /// `draft.clear()`.
    pub fn clear(&self) {
        self.state.write().clear();
    }

    /// `Init(S)`: changes the basis shape and resets; `draft.init(&shape)?`.
    pub fn init(&self, s: &Shape) -> Result<(), Failure> {
        let mut state = self.state.write();
        state.my_shape = s.clone();
        state.clear();
        if !s.is_null() {
            let shape = state.my_shape.clone();
            topexp::map_shapes_and_ancestors(
                &shape,
                ShapeEnum::Edge,
                ShapeEnum::Face,
                &mut state.my_ef_map,
            )?;
        }
        Ok(())
    }

    /// `Add(F, Direction, Angle, NeutralPlane, Flag)`: adds the face and propagates the
    /// draft to its tangent neighbours; false with the problematic shape on failure;
    /// Standard_ConstructionError while a problematic shape is set;
    /// `draft.add(&face, dir, 0.1, &plane, true)?`.
    pub fn add(
        &self,
        f: &Face,
        direction: Direction3,
        angle: f64,
        neutral_plane: &Plane3,
        flag: bool,
    ) -> Result<bool, Failure> {
        let mut state = self.state.write();
        if !state.bad_shape.is_null() {
            return Err(Failure::with_message(
                "Standard_ConstructionError",
                "Draft_Modification::Add",
            ));
        }
        if state.my_comp {
            state.clear();
        }
        state.cur_face = f.clone();
        state.internal_add(f, direction, angle, neutral_plane, flag)
    }

    /// `Remove(F)`: removes the face and the faces and edges added with it;
    /// Standard_NoSuchObject for a face not added or after `Perform`;
    /// `draft.remove(&face)?`.
    pub fn remove(&self, f: &Face) -> Result<(), Failure> {
        self.state.write().remove(f)
    }

    /// `Perform()`: computes the new geometry; Standard_ConstructionError while a
    /// problematic shape is set; `draft.perform()?`.
    pub fn perform(&self) -> Result<(), Failure> {
        self.state.write().perform()
    }

    /// `IsDone()`; `draft.is_done()`.
    pub fn is_done(&self) -> bool {
        self.state.read().is_done()
    }

    /// `Error()`; `draft.error()`.
    pub fn error(&self) -> DraftErrorStatus {
        self.state.read().err_stat
    }

    /// `ProblematicShape()`; `draft.problematic_shape()`.
    pub fn problematic_shape(&self) -> Shape {
        self.state.read().bad_shape.clone()
    }

    /// `ConnectedFaces(F)`: the faces added together with the face;
    /// Standard_NoSuchObject for a face not added, StdFail_NotDone before a successful
    /// `Perform`; `draft.connected_faces(&face)?`.
    pub fn connected_faces(&self, f: &Face) -> Result<List<Shape>, Failure> {
        let mut state = self.state.write();
        if !state.my_f_map.contains(f) {
            return Err(Failure::with_message(
                "Standard_NoSuchObject",
                "Draft_Modification::ConnectedFaces",
            ));
        }
        if !state.is_done() {
            return Err(Failure::with_message(
                "StdFail_NotDone",
                "Draft_Modification::ConnectedFaces",
            ));
        }
        state.conne_f.clear();
        state.cur_face = state.my_f_map.find_from_key(f)?.root_face().clone();
        for i in 1..=state.my_f_map.extent() {
            let the_f = state.my_f_map.find_key(i)?.clone();
            if state
                .my_f_map
                .find_from_key(&the_f)?
                .root_face()
                .is_same(&state.cur_face)
            {
                state.conne_f.append(the_f);
            }
        }
        Ok(state.conne_f.clone())
    }

    /// `ModifiedFaces()`: the faces given a modification; StdFail_NotDone while a
    /// problematic shape is set; `draft.modified_faces()?`.
    pub fn modified_faces(&self) -> Result<List<Shape>, Failure> {
        let mut state = self.state.write();
        if !state.bad_shape.is_null() {
            return Err(Failure::with_message(
                "StdFail_NotDone",
                "Draft_Modification::ModifiedFaces",
            ));
        }
        state.conne_f.clear();
        for i in 1..=state.my_f_map.extent() {
            let the_f = state.my_f_map.find_key(i)?.clone();
            if !state.my_f_map.find_from_key(&the_f)?.root_face().is_null() {
                state.conne_f.append(the_f);
            }
        }
        Ok(state.conne_f.clone())
    }
}

impl DraftState {
    /// `Clear()`.
    fn clear(&mut self) {
        self.my_comp = false;
        self.my_f_map.clear(true);
        self.my_e_map.clear(true);
        self.my_v_map.clear(true);
        self.my_ef_map.clear(true);
        self.bad_shape.nullify();
        self.err_stat = DraftErrorStatus::NoError;
    }

    /// `IsDone()`.
    pub(super) fn is_done(&self) -> bool {
        self.my_comp && self.bad_shape.is_null()
    }

    /// `Remove(F)`.
    fn remove(&mut self, f: &Face) -> Result<(), Failure> {
        if !self.my_f_map.contains(f) || self.my_comp {
            return Err(Failure::with_message(
                "Standard_NoSuchObject",
                "Draft_Modification::Remove",
            ));
        }

        self.conne_f.clear();
        self.cur_face = self.my_f_map.find_from_key(f)?.root_face().clone();
        for i in 1..=self.my_f_map.extent() {
            let the_f = self.my_f_map.find_key(i)?.clone();
            if self
                .my_f_map
                .find_from_key(&the_f)?
                .root_face()
                .is_same(&self.cur_face)
            {
                if the_f.is_same(&self.bad_shape) {
                    self.bad_shape.nullify();
                }
                self.conne_f.append(the_f);
            }
        }
        for s in self.conne_f.iter() {
            self.my_f_map.remove_key(s);
        }

        self.conne_f.clear();
        for i in 1..=self.my_e_map.extent() {
            let the_e = self.my_e_map.find_key(i)?.clone();
            if self
                .my_e_map
                .find_from_key(&the_e)?
                .root_face()
                .is_same(&self.cur_face)
            {
                self.conne_f.append(the_e);
            }
        }
        for s in self.conne_f.iter() {
            self.my_e_map.remove_key(s);
        }
        Ok(())
    }

    /// `NewParameter(V, E, P, Tol)`: the new parameter and tolerance of the vertex on the
    /// edge, `None` for a vertex not modified.
    pub(super) fn new_parameter(
        &mut self,
        v: &Vertex,
        e: &Edge,
    ) -> Result<Option<(f64, f64)>, Failure> {
        if !self.is_done() {
            return Err(not_done_domain());
        }
        if !self.my_v_map.contains(v) {
            return Ok(None);
        }

        let mut p = self.my_v_map.change_from_key(v)?.parameter(e)?;
        // The native down cast of a trimmed curve to itself changes nothing.
        let gc = self.my_e_map.find_from_key(e)?.geometry().clone();
        let curve = live(&gc, "edge geometry")?;

        if curve.is_closed()? {
            let mut fv = topexp::first_vertex(e, false);
            let mut paramf = if self.my_v_map.contains(&fv) {
                self.my_v_map.change_from_key(&fv)?.parameter(e)?
            } else {
                BRepTool::parameter(&fv, e)?
            };

            // Patch
            let (first_par, last_par) = (curve.first_parameter(), curve.last_parameter());
            let pconf = precision::p_confusion();
            if (paramf - last_par).abs() <= pconf {
                paramf = first_par;
                fv.set_orientation(e.orientation());
                if v.is_equal(&fv) {
                    p = paramf;
                }
            }

            fv.set_orientation(e.orientation());
            if !v.is_equal(&fv) && p <= paramf {
                if curve.is_periodic() {
                    p += curve.period()?;
                } else {
                    p = curve.last_parameter();
                }
            }
        }

        let tol = BRepTool::vertex_tolerance(v)?.max(BRepTool::edge_tolerance(e)?);
        Ok(Some((p, tol)))
    }
}

impl BRepToolsModification for DraftModification {
    /// `NewSurface(F, S, L, Tol, RevWires, RevFace)`: the drafted surface of a face with
    /// a new geometry, at the identity; Standard_DomainError before a successful
    /// `Perform`.
    fn new_surface(&self, face: &Face, result: &mut NewSurface) -> Result<bool, Failure> {
        let state = self.state.read();
        if !state.is_done() {
            return Err(not_done_domain());
        }
        if !state.my_f_map.contains(face) || !state.my_f_map.find_from_key(face)?.new_geometry() {
            return Ok(false);
        }
        result.rev_wires = false;
        result.rev_face = false;
        result.tolerance = BRepTool::face_tolerance(face)?;
        let (surface, _) = BRepTool::surface_with_location(face)?;
        result.surface = surface;
        result.location = Default::default();
        result.surface = state.my_f_map.find_from_key(face)?.geometry().clone();
        Ok(true)
    }

    /// `NewCurve(E, C, L, Tol)`: the new curve of an edge with a new geometry.
    fn new_curve(&self, edge: &Edge, result: &mut NewCurve) -> Result<bool, Failure> {
        let state = self.state.read();
        if !state.is_done() {
            return Err(not_done_domain());
        }
        if !state.my_e_map.contains(edge) {
            return Ok(false);
        }
        let einf = state.my_e_map.find_from_key(edge)?;
        if !einf.new_geometry() {
            return Ok(false);
        }
        result.tolerance = einf.tolerance();
        result.tolerance = result.tolerance.max(BRepTool::edge_tolerance(edge)?);
        result.location = Default::default();
        result.curve = einf.geometry().clone();
        Ok(true)
    }

    /// `NewPoint(V, P, Tol)`: the new point of a modified vertex.
    fn new_point(
        &self,
        vertex: &Vertex,
        point: &mut Point3,
        tolerance: &mut f64,
    ) -> Result<bool, Failure> {
        let state = self.state.read();
        if !state.is_done() {
            return Err(not_done_domain());
        }
        if !state.my_v_map.contains(vertex) {
            return Ok(false);
        }
        *tolerance = BRepTool::vertex_tolerance(vertex)?;
        *point = state.my_v_map.find_from_key(vertex)?.geometry();
        Ok(true)
    }

    /// `NewCurve2d(E, F, NewE, NewF, C, Tol)`: the stored p-curve of the edge on the
    /// face, else the projection of its new curve trimmed to the range of the new edge
    /// (whose range is updated); on a periodic surface the p-curve is moved by a period
    /// towards the original one (in place: a stored p-curve is moved with it, as
    /// natively). Standard_NoSuchObject for a face that is not recorded.
    fn new_curve2d(
        &self,
        edge: &Edge,
        face: &Face,
        new_edge: &Edge,
        _new_face: &Face,
        curve: &mut Handle<dyn Geom2dCurve>,
        tolerance: &mut f64,
    ) -> Result<bool, Failure> {
        let state = self.state.read();
        if !state.is_done() {
            return Err(not_done_domain());
        }
        if !state.my_e_map.contains(edge) {
            return Ok(false);
        }

        let (mut fp, mut lp) = BRepTool::range(new_edge)?;

        let mut sb = state.my_f_map.find_from_key(face)?.geometry().clone();

        *tolerance = BRepTool::edge_tolerance(edge)?;

        let einf = state.my_e_map.find_from_key(edge)?;
        if einf.first_face().is_same(face) && !einf.first_pc().is_null() {
            *curve = einf.first_pc().clone();
        } else if einf.second_face().is_same(face) && !einf.second_pc().is_null() {
            *curve = einf.second_pc().clone();
        } else {
            if !einf.new_geometry() {
                let (fpi, lpi) = BRepTool::range(edge)?;
                if fpi <= fp && fp <= lpi && fpi <= lp && lp <= lpi {
                    return Ok(false);
                }
            }

            (fp, lp) = BRepTool::range(new_edge)?;
            let tc = GeomTrimmedCurve::new(einf.geometry(), fp, lp, true, true)?;
            fp = tc.first_parameter();
            lp = tc.last_parameter();
            let tc: Handle<dyn GeomCurve> = Handle::new(tc).cast();
            BRepBuilder::default().range(new_edge, fp, lp, false)?;
            (*curve, *tolerance) =
                geom_proj_lib::curve2d_range_tolerance(&tc, (fp, lp), &sb, *tolerance)?;
        }

        if live(&sb, "surface")?.is_instance_name("Geom_RectangularTrimmedSurface") {
            sb = live(
                &sb.clone().cast::<GeomRectangularTrimmedSurface>(),
                "surface",
            )?
            .basis_surface();
        }
        let surface = live(&sb, "surface")?;

        let mut je_recadre = false;
        if surface.is_instance_name("Geom_SurfaceOfLinearExtrusion") {
            let a_c = live(
                &sb.clone().cast::<GeomSurfaceOfLinearExtrusion>(),
                "surface",
            )?
            .basis_curve();
            if live(&a_c, "curve")?.is_instance_name("Geom_Circle") {
                je_recadre = true;
            }
        }

        je_recadre = je_recadre
            || surface.is_instance_name("Geom_CylindricalSurface")
            || surface.is_instance_name("Geom_SphericalSurface")
            || surface.is_instance_name("Geom_ConicalSurface");

        if je_recadre {
            let vectra = Vector2::new(2. * PI, 0.);
            let a_c2de = BRepTool::curve_on_face(edge, face)?;
            let pf: Point2 =
                live(&a_c2de.pcurve, "pcurve")?.value(0.5 * (a_c2de.first + a_c2de.last))?;
            let pcurve = live(curve, "pcurve")?;
            let new_pf: Point2 = pcurve.value(0.5 * (fp + lp))?;
            let a_d2 = new_pf.square_distance(pf);
            let mut a_v2dt = Vector2::default();
            let mut b_translate = false;
            if new_pf.translated(vectra).square_distance(pf) < a_d2 {
                a_v2dt = vectra;
                b_translate = !b_translate; // True
            } else if new_pf.translated(-vectra).square_distance(pf) < a_d2 {
                a_v2dt = -vectra;
                b_translate = !b_translate; // True
            }
            if b_translate {
                pcurve.translate(a_v2dt)?;
            }
        }

        let a_c3d = BRepTool::curve(new_edge)?;
        *tolerance =
            brep_tools::eval_and_update_tol(new_edge, &a_c3d.0, curve, &sb, a_c3d.1, a_c3d.2)?;
        Ok(true)
    }

    /// `NewParameter(V, E, P, Tol)`: the parameter of a modified vertex on the new curve
    /// of the edge, moved by a period (or to the last parameter) on a closed curve.
    fn new_parameter(
        &self,
        vertex: &Vertex,
        edge: &Edge,
        parameter: &mut f64,
        tolerance: &mut f64,
    ) -> Result<bool, Failure> {
        match self.state.write().new_parameter(vertex, edge)? {
            Some((p, tol)) => {
                *parameter = p;
                *tolerance = tol;
                Ok(true)
            }
            None => Ok(false),
        }
    }

    /// `Continuity(E, F1, F2, NewE, NewF1, NewF2)`: the continuity of the original edge.
    fn continuity(
        &self,
        edge: &Edge,
        face1: &Face,
        face2: &Face,
        _new_edge: &Edge,
        _new_face1: &Face,
        _new_face2: &Face,
    ) -> Result<Continuity, Failure> {
        BRepTool::continuity_on_faces(edge, face1, face2)
    }
}
