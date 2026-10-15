//! BRepOffsetAPI_MakePipe, MakeEvolved, MakeDraft, MakeFilling, NormalProjection and
//! FindContigousEdges probe operations, the Rust twin of
//! tools/oracle/tkoffset_probe_brepoffsetapi_wrappers.cxx (the records are documented
//! there).
use super::common::{Reader, attempt, end, flag, int, real};
use super::evolved_digest::evolved_digest;
use super::make_offset::{at, failed};
use super::make_pipe_shell::list;
use super::shape::shape;
use super::shapes::{TpShapes, tp_same_index, tp_shapes};
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Face, Shape};
use rustycad_tkbrep::toptools::IndexedMapOfShape;
use rustycad_tkernel::message::ProgressRange;
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{GeomPlane, GeomSurface};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkgeomalgo::geom_fill::GeomFillTrihedron;
use rustycad_tkmath::geom_abs::{JoinType, Shape as Continuity};
use rustycad_tkoffset::brep_offset_api::{
    BRepOffsetAPIFindContigousEdges, BRepOffsetAPIMakeDraft, BRepOffsetAPIMakeEvolved,
    BRepOffsetAPIMakeFilling, BRepOffsetAPIMakePipe, BRepOffsetAPINormalProjection, EvolvedOptions,
    FillingParameters,
};
use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakeSweep;
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakeShape, BRepBuilderAPITransitionMode};

/// The sub-shapes of a kind in explorer order.
fn explored(s: &Shape, kind: ShapeEnum) -> Vec<Shape> {
    Explorer::new(s, kind, ShapeEnum::Shape)
        .map(|e| e.collect())
        .unwrap_or_default()
}

/// The map of the shape (empty for a null shape).
fn map_of(s: &Shape) -> IndexedMapOfShape {
    let mut map = IndexedMapOfShape::default();
    if !s.is_null() {
        topexp::map_shapes(s, &mut map, true, true);
    }
    map
}

/// The index in the map of a shape, -1 for a null one.
fn index_in(map: &IndexedMapOfShape, s: &Shape) {
    int(if s.is_null() {
        -1
    } else {
        map.find_index(s) as i32
    });
}

/// The trihedron of a `GeomFill_Trihedron` value.
fn trihedron_of(value: i32) -> GeomFillTrihedron {
    use GeomFillTrihedron::*;
    [
        IsCorrectedFrenet,
        IsFixed,
        IsFrenet,
        IsConstantNormal,
        IsDarboux,
        IsGuideAC,
        IsGuidePlan,
        IsGuideACWithContact,
        IsGuidePlanWithContact,
        IsDiscreteTrihedron,
    ]
    .get(value as usize)
    .copied()
    .unwrap_or_default()
}

/// The continuity of its enumeration value (C0..CN).
fn continuity_of(value: i32) -> Continuity {
    match value {
        1 => Continuity::G1,
        2 => Continuity::C1,
        3 => Continuity::G2,
        4 => Continuity::C2,
        5 => Continuity::C3,
        6 => Continuity::CN,
        _ => Continuity::C0,
    }
}

/// The join of its enumeration value (0 arc, 1 tangent, 2 intersection).
fn join_of(value: i32) -> JoinType {
    match value {
        1 => JoinType::Tangent,
        2 => JoinType::Intersection,
        _ => JoinType::Arc,
    }
}

/// The pipe of the request.
fn pipe_of(
    r: &mut Reader<'_>,
    t: &TpShapes,
) -> Result<(BRepOffsetAPIMakePipe, Shape, Shape), Failure> {
    let spine = topods::wire(&at(t, r.integer())?)?;
    let profile = at(t, r.integer())?;
    let (mode, c1) = (r.integer(), r.flag());
    let pipe = if mode < 0 {
        BRepOffsetAPIMakePipe::new(&spine, &profile)?
    } else {
        BRepOffsetAPIMakePipe::with_mode(&spine, &profile, trihedron_of(mode), c1)?
    };
    Ok((pipe, spine.into_shape(), profile))
}

/// `oa_pipe program spine profile mode c1`.
pub fn pipe_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let t = match tp_shapes(&mut r) {
        Ok(t) => t,
        Err(error) => return failed(error),
    };
    let (mut pipe, spine, profile) = match pipe_of(&mut r, &t) {
        Ok(found) => found,
        Err(error) => return failed(error),
    };
    flag(pipe.is_done());
    let mut map = IndexedMapOfShape::default();
    if pipe.is_done() {
        attempt(|| pipe.shape().and_then(shape));
        map = map_of(&pipe.shape().cloned().unwrap_or_default());
    }
    attempt(|| pipe.first_shape().and_then(|s| shape(&s)));
    attempt(|| pipe.last_shape().and_then(|s| shape(&s)));
    real(pipe.error_on_surface());
    for s in &t.shapes {
        attempt(|| pipe.generated(s).map(|l| list(&map, l)));
    }
    let vertex_profile = profile.shape_type().ok() == Some(ShapeEnum::Vertex);
    for se in explored(&spine, ShapeEnum::Edge)
        .into_iter()
        .filter(|_| !vertex_profile)
    {
        for kind in [ShapeEnum::Edge, ShapeEnum::Vertex] {
            for p in explored(&profile, kind) {
                attempt(|| pipe.generated_by(&se, &p).map(|g| index_in(&map, &g)));
            }
        }
    }
    end();
}

/// The evolved shape of the request with its spine and profile.
fn evolved_of(
    r: &mut Reader<'_>,
    t: &TpShapes,
) -> Result<(BRepOffsetAPIMakeEvolved, Shape, Shape), Failure> {
    let spine = at(t, r.integer())?;
    let profile = topods::wire(&at(t, r.integer())?)?;
    let join_type = join_of(r.integer());
    let (is_axe_prof, is_solid, is_prof_on_spine) = (r.flag(), r.flag(), r.flag());
    let (tol, is_volume) = (r.real(), r.flag());
    let options = EvolvedOptions {
        join_type,
        is_axe_prof,
        is_solid,
        is_prof_on_spine,
        tol,
        is_volume,
        run_in_parallel: false,
    };
    let evo = BRepOffsetAPIMakeEvolved::with_spine(&spine, &profile, options)?;
    Ok((evo, spine, profile.into_shape()))
}

/// `oa_evolved program spine profile join axeprof solid profonspine tol volume`.
pub fn evolved_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let t = match tp_shapes(&mut r) {
        Ok(t) => t,
        Err(error) => return failed(error),
    };
    let (mut evo, spine, profile) = match evolved_of(&mut r, &t) {
        Ok(found) => found,
        Err(error) => return failed(error),
    };
    flag(evo.is_done());
    if evo.is_done() {
        let volume = evo.evolved().is_err();
        let result = evo.shape().cloned().unwrap_or_default();
        if volume {
            attempt(|| shape(&result));
        } else {
            attempt(|| evolved_digest(&result));
        }
    }
    attempt(|| evolved_digest(evo.top()));
    attempt(|| evolved_digest(evo.bottom()));
    let (spines, profiles) = (map_of(&spine), map_of(&profile));
    for i in 1..=spines.extent() {
        for j in 1..=profiles.extent() {
            let (Ok(s), Ok(p)) = (spines.find_key(i), profiles.find_key(j)) else {
                continue;
            };
            let generated = evo.generated_shapes(s, p);
            int(generated.len() as i32);
            for g in generated.iter() {
                int(g.shape_type().map_or(-1, |kind| kind as i32));
                int(g.orientation() as i32);
            }
        }
    }
    end();
}

/// The transition of a request (0 transformed, 1 right corner, 2 round corner).
fn transition_of(value: i32) -> BRepBuilderAPITransitionMode {
    match value {
        1 => BRepBuilderAPITransitionMode::RightCorner,
        2 => BRepBuilderAPITransitionMode::RoundCorner,
        _ => BRepBuilderAPITransitionMode::Transformed,
    }
}

/// The draft of the request, performed.
fn draft_of(r: &mut Reader<'_>, t: &TpShapes) -> Result<BRepOffsetAPIMakeDraft, Failure> {
    let draft_shape = at(t, r.integer())?;
    let (dir, angle) = (r.dir(), r.real());
    let (style, amin, amax, internal) = (r.integer(), r.real(), r.real(), r.flag());
    let mut draft = BRepOffsetAPIMakeDraft::new(&draft_shape, dir, angle)?;
    if style >= 0 {
        draft.set_options(transition_of(style), amin, amax);
    }
    draft.set_draft(internal);
    match r.integer() {
        0 => draft.perform_length(r.real())?,
        1 => {
            let (o, n, keep) = (r.pnt(), r.dir(), r.flag());
            let plane: Handle<dyn GeomSurface> = Handle::new(GeomPlane::from_point(o, n)?).cast();
            draft.perform_surface(&plane, keep)?;
        }
        _ => {
            let stop = at(t, r.integer())?;
            draft.perform_shape(&stop, r.flag())?;
        }
    }
    Ok(draft)
}

/// `oa_draft program shape dx dy dz angle style amin amax internal kind...`.
pub fn draft_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let t = match tp_shapes(&mut r) {
        Ok(t) => t,
        Err(error) => return failed(error),
    };
    let mut draft = match draft_of(&mut r, &t) {
        Ok(draft) => draft,
        Err(error) => return failed(error),
    };
    flag(draft.is_done());
    let mut map = IndexedMapOfShape::default();
    if draft.is_done() {
        attempt(|| draft.shape().and_then(shape));
        map = map_of(&draft.shape().cloned().unwrap_or_default());
    }
    attempt(|| shape(draft.shell().as_ref()));
    for s in &t.shapes {
        attempt(|| draft.generated(s).map(|l| list(&map, l)));
    }
    end();
}

/// The face of a program index.
fn face_at(t: &TpShapes, index: i32) -> Result<Face, Failure> {
    topods::face(&at(t, index)?)
}

/// The constraints of the filling request, each printing its index.
fn filling_adds(
    r: &mut Reader<'_>,
    t: &TpShapes,
    f: &mut BRepOffsetAPIMakeFilling,
) -> Result<(), Failure> {
    for _ in 0..r.integer() {
        let e = topods::edge(&at(t, r.integer())?)?;
        let face = r.integer();
        let (order, bound) = (continuity_of(r.integer()), r.flag());
        if face < 0 {
            int(f.add_edge(&e, order, bound));
        } else {
            int(f.add_edge_on_face(&e, &face_at(t, face)?, order, bound));
        }
    }
    for _ in 0..r.integer() {
        let face = face_at(t, r.integer())?;
        int(f.add_face(&face, continuity_of(r.integer())));
    }
    for _ in 0..r.integer() {
        if r.integer() == 0 {
            int(f.add_point(r.pnt())?);
            continue;
        }
        let (u, v) = (r.real(), r.real());
        let face = face_at(t, r.integer())?;
        int(f.add_point_on_face(u, v, &face, continuity_of(r.integer()))?);
    }
    let init = r.integer();
    if init >= 0 {
        f.load_init_surface(&face_at(t, init)?);
    }
    f.build(&ProgressRange::default())
}

/// The filling of the request's parameters.
fn filling_of(r: &mut Reader<'_>) -> BRepOffsetAPIMakeFilling {
    let (degree, nb_pts_on_cur, nb_iter, anisotropie) =
        (r.integer(), r.integer(), r.integer(), r.flag());
    let (tol2d, tol3d, tol_ang, tol_curv) = (r.real(), r.real(), r.real(), r.real());
    let (max_deg, max_segments) = (r.integer(), r.integer());
    BRepOffsetAPIMakeFilling::new(FillingParameters {
        degree,
        nb_pts_on_cur,
        nb_iter,
        anisotropie,
        tol2d,
        tol3d,
        tol_ang,
        tol_curv,
        max_deg,
        max_segments,
    })
}

/// `oa_filling` (the record of bf_filling).
pub fn filling_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let t = match tp_shapes(&mut r) {
        Ok(t) => t,
        Err(error) => return failed(error),
    };
    let mut f = filling_of(&mut r);
    if let Err(error) = filling_adds(&mut r, &t, &mut f) {
        return failed(error);
    }
    flag(f.is_done());
    attempt(|| f.shape().and_then(shape));
    if !f.is_done() {
        return end();
    }
    attempt(|| {
        real(f.g0_error()?);
        real(f.g1_error()?);
        real(f.g2_error()?);
        Ok(())
    });
    let map = map_of(&f.shape().cloned().unwrap_or_default());
    for g in &t.shapes {
        attempt(|| f.generated(g).map(|l| list(&map, l)));
    }
    end();
}

/// The projection of the request and its edges, built.
fn projection_of(
    r: &mut Reader<'_>,
    t: &TpShapes,
) -> Result<(BRepOffsetAPINormalProjection, Vec<Shape>), Failure> {
    let mut proj = BRepOffsetAPINormalProjection::with_shape(&at(t, r.integer())?);
    let mut edges = Vec::new();
    for _ in 0..r.integer() {
        edges.push(at(t, r.integer())?);
    }
    let (maxdist, with3d, limit) = (r.real(), r.flag(), r.flag());
    for e in &edges {
        proj.add(e)?;
    }
    if maxdist >= 0. {
        proj.set_max_distance(maxdist);
    }
    proj.compute3d(with3d);
    proj.set_limit(limit);
    proj.build(&ProgressRange::default())?;
    Ok((proj, edges))
}

/// `oa_normal_projection program target n edge... maxdist with3d limit`.
pub fn normal_projection_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let t = match tp_shapes(&mut r) {
        Ok(t) => t,
        Err(error) => return failed(error),
    };
    let (mut proj, edges) = match projection_of(&mut r, &t) {
        Ok(found) => found,
        Err(error) => return failed(error),
    };
    flag(proj.is_done());
    attempt(|| shape(proj.projection()));
    let map = map_of(&proj.projection().clone());
    for e in &edges {
        attempt(|| proj.generated(e).map(|l| list(&map, l)));
    }
    if proj.is_done() {
        attempt(|| {
            let mut wires = List::default();
            flag(proj.build_wire(&mut wires)?);
            int(wires.len() as i32);
            Ok(())
        });
    }
    end();
}

/// The finder of the request, performed.
fn finder_of(r: &mut Reader<'_>, t: &TpShapes) -> Result<BRepOffsetAPIFindContigousEdges, Failure> {
    let mut shapes = Vec::new();
    for _ in 0..r.integer() {
        shapes.push(at(t, r.integer())?);
    }
    let (tol, option) = (r.real(), r.flag());
    let mut finder = BRepOffsetAPIFindContigousEdges::new(tol, option);
    for s in &shapes {
        finder.add(s)?;
    }
    finder.perform()?;
    Ok(finder)
}

/// `oa_contigous program n shape... tol option`.
pub fn contigous_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let t = match tp_shapes(&mut r) {
        Ok(t) => t,
        Err(error) => return failed(error),
    };
    let finder = match finder_of(&mut r, &t) {
        Ok(finder) => finder,
        Err(error) => return failed(error),
    };
    let nb = finder.nb_contigous_edges();
    int(nb);
    for i in 1..=nb {
        attempt(|| {
            finder
                .contigous_edge(i)
                .map(|e| int(tp_same_index(&t, e.as_ref())))
        });
        attempt(|| {
            let couple = finder.contigous_edge_couple(i)?;
            int(couple.len() as i32);
            for s in couple.iter() {
                int(tp_same_index(&t, s));
            }
            Ok(())
        });
    }
    attempt(|| finder.contigous_edge(nb + 1).map(|_| ()));
    int(finder.nb_degenerated_shapes());
    for s in &t.shapes {
        flag(finder.is_modified(s));
    }
    end();
}
