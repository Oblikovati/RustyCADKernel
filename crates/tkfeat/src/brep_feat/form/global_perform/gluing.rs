// Rust translation of the gluing part (operation 1) of BRepFeat_Form::GlobalPerform of
// ModelingAlgorithms/TKFeat/BRepFeat/BRepFeat_Form.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The gluing of the tool on the basis: the tool cut by the solids of the shapes From and
//! Until, its glued faces bound to the basis faces they lie in (with their confused
//! edges), the limiting faces of the basis bound too; the gluing is abandoned for the
//! local Boolean operation when a face is not inside its basis face or the gluer does not
//! make the expected operation or a valid shape.
use super::{Form, Run};
use crate::brep_feat::{is_inside, tool};
use crate::loc_ope::{FindEdges, Gluer, Operation};
use rustycad_tkbo::brep_algo_api::BRepAlgoAPICut;
use rustycad_tkbool::brep_algo;
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Compound, Face, Shape};
use rustycad_tkbrep::toptools::ShapeMapHasher;
use rustycad_tkernel::ncollection::{DataMap, List};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;

/// The descendants of the faces of the shapes From or Until kept by the cut of the tool
/// (`locmap`).
type LocalMap = DataMap<Shape, List<Shape>, ShapeMapHasher>;

/// The compound of the solids made of the shapes From and Until (cut by FFrom && FUntil).
pub(super) fn limit_tools(
    form: &Form,
    run: &Run,
    from: bool,
    until: bool,
) -> Result<Shape, Failure> {
    let b = BRepBuilder::default();
    let mut comp = Compound::default();
    b.make_compound(&mut comp);
    let mut comp = comp.into_shape();
    let limits = &run.limits;
    for (s, use_it, face, ori) in [
        (&form.s_from, from, &limits.ffrom, limits.oriffrom),
        (&form.s_until, until, &limits.funtil, limits.orifuntil),
    ] {
        if !s.is_null() && use_it {
            let solid = tool(s, face, ori)?;
            if !solid.is_null() {
                b.add(&mut comp, &solid)?;
            }
        }
    }
    Ok(comp)
}

/// The solids of the result of the cut, in a compound.
pub(super) fn cut_solids(tr_p: &BRepAlgoAPICut) -> Result<Shape, Failure> {
    let b = BRepBuilder::default();
    let mut g = Compound::default();
    b.make_compound(&mut g);
    let mut g = g.into_shape();
    for s in Explorer::new(tr_p.shape(), ShapeEnum::Solid, ShapeEnum::Shape)? {
        b.add(&mut g, &s)?;
    }
    Ok(g)
}

/// What the cut made of a face: none when deleted, its modifications, or itself.
pub(super) fn cut_descendants(tr_p: &BRepAlgoAPICut, fac: &Shape) -> Result<List<Shape>, Failure> {
    let mut l = List::default();
    if tr_p.is_deleted(fac)? {
        return Ok(l);
    }
    for m in tr_p.modified(fac)? {
        l.append(m);
    }
    if l.is_empty() {
        l.append(fac.clone());
    }
    Ok(l)
}

/// The faces of the shape From or Until mapped to what the cut made of them: in the
/// history when the shape is not in the basis, else in `locmap`.
fn map_limit_faces(
    form: &mut Form,
    tr_p: &BRepAlgoAPICut,
    s: &Shape,
    in_shape: bool,
    locmap: &mut LocalMap,
) -> Result<(), Failure> {
    if s.is_null() {
        return Ok(());
    }
    for fac in Explorer::new(s, ShapeEnum::Face, ShapeEnum::Shape)? {
        let desc = cut_descendants(tr_p, &fac)?;
        if in_shape {
            locmap.bind(fac, desc);
        } else {
            form.map.bind(fac, desc);
        }
    }
    Ok(())
}

/// Binds the face `fnew` of the tool to the basis face `fbase`, with their confused
/// edges.
fn bind_with_edges(the_glue: &mut Gluer, fnew: &Face, fbase: &Face) -> Result<(), Failure> {
    the_glue.bind_faces(fnew, fbase)?;
    let mut the_fe = FindEdges::with_shapes(fnew, fbase)?;
    the_fe.init_iterator();
    while the_fe.more() {
        the_glue.bind_edges(&the_fe.edge_from()?, &the_fe.edge_to()?)?;
        the_fe.next();
    }
    Ok(())
}

/// The gluing of the cut tool: each piece of a glued face bound to its basis face when
/// inside it (a piece outside switches to the Boolean operation); the Collage flag of the
/// last piece examined is returned.
fn glue_cut_tool(
    form: &mut Form,
    run: &mut Run,
    tr_p: &BRepAlgoAPICut,
    the_glue: &mut Gluer,
) -> Result<bool, Failure> {
    let mut collage = true;
    let glued: Vec<(Shape, Shape)> = form
        .glued_f
        .items()
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();
    for (gl, glface) in glued {
        let ldsc = cut_descendants(tr_p, &gl)?;
        let glface = topods::face(&glface)?;
        for d in ldsc.iter() {
            let fac = topods::face(d)?;
            collage = is_inside(&fac, &glface)?;
            if !collage {
                run.the_ope = 2;
                run.change_ope = true;
                break;
            }
            bind_with_edges(the_glue, &fac, &glface)?;
        }
    }
    Ok(collage)
}

/// The gluing when the tool is cut by the solids of the limiting shapes.
fn glue_after_cut(
    form: &mut Form,
    run: &mut Run,
    comp: &Shape,
    locmap: &mut LocalMap,
    the_glue: &mut Gluer,
) -> Result<bool, Failure> {
    let tr_p = BRepAlgoAPICut::from_shapes(&form.g_shape, comp)?;
    if Explorer::new(tr_p.shape(), ShapeEnum::Solid, ShapeEnum::Shape)?
        .current()
        .is_null()
    {
        run.the_ope = 2;
        run.change_ope = true;
        return Ok(false);
    }
    // Only solids are preserved
    let the_g_shape = cut_solids(&tr_p)?;
    if !brep_algo::is_valid(&the_g_shape)? {
        run.the_ope = 2;
        run.change_ope = true;
        return Ok(false);
    }
    let (s_from, s_until) = (form.s_from.clone(), form.s_until.clone());
    map_limit_faces(form, &tr_p, &s_from, run.from_in_shape, locmap)?;
    map_limit_faces(form, &tr_p, &s_until, run.until_in_shape, locmap)?;
    form.update_descendants_bop(&tr_p, &the_g_shape, true)?; // skip faces
    the_glue.init(&form.sbase, &the_g_shape);
    glue_cut_tool(form, run, &tr_p, the_glue)
}

/// The gluing of the whole tool: each glued face of the tool bound to its basis face
/// when inside it.
fn glue_whole_tool(form: &mut Form, run: &mut Run, the_glue: &mut Gluer) -> Result<bool, Failure> {
    the_glue.init(&form.sbase, &form.g_shape);
    let glued: Vec<(Shape, Shape)> = form
        .glued_f
        .items()
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();
    for (glface, fac) in glued {
        if !Explorer::new(&form.g_shape, ShapeEnum::Face, ShapeEnum::Shape)?
            .any(|x| x.is_same(&glface))
        {
            continue;
        }
        let (glface, fac) = (topods::face(&glface)?, topods::face(&fac)?);
        if !is_inside(&glface, &fac)? {
            run.the_ope = 2;
            run.change_ope = true;
            return Ok(false);
        }
        bind_with_edges(the_glue, &glface, &fac)?;
    }
    Ok(true)
}

/// Add gluing on start and end face if necessary: the faces of the limiting shape `s`
/// lying in the basis bound to their pieces (FindEdges first for the start face, the
/// face binding first for the end one, as natively).
fn glue_limit_faces(
    s: &Shape,
    locmap: &LocalMap,
    the_glue: &mut Gluer,
    edges_first: bool,
) -> Result<(), Failure> {
    for f2 in Explorer::new(s, ShapeEnum::Face, ShapeEnum::Shape)? {
        let fac2 = topods::face(&f2)?;
        let pieces = locmap.find(&f2)?;
        for p in pieces.iter() {
            let fac1 = topods::face(p)?;
            if edges_first {
                let mut the_fe = FindEdges::with_shapes(&fac1, &fac2)?;
                the_glue.bind_faces(&fac1, &fac2)?;
                the_fe.init_iterator();
                while the_fe.more() {
                    the_glue.bind_edges(&the_fe.edge_from()?, &the_fe.edge_to()?)?;
                    the_fe.next();
                }
            } else {
                bind_with_edges(the_glue, &fac1, &fac2)?;
            }
        }
    }
    Ok(())
}

/// The gluer of the tool on the basis, the operation switching to 2 when the gluing is
/// not possible.
pub(super) fn prepare_glue(form: &mut Form, run: &mut Run) -> Result<Gluer, Failure> {
    let mut the_glue = Gluer::new();
    // cut by FFrom && FUntil
    let comp = limit_tools(form, run, true, true)?;
    let mut locmap: LocalMap = DataMap::default();
    let has_solid = Explorer::new(&comp, ShapeEnum::Solid, ShapeEnum::Shape)?.more();
    let collage = if has_solid && !comp.is_null() && !form.g_shape.is_null() {
        glue_after_cut(form, run, &comp, &mut locmap, &mut the_glue)?
    } else {
        glue_whole_tool(form, run, &mut the_glue)?
    };
    if run.from_in_shape && collage {
        glue_limit_faces(&form.s_from, &locmap, &mut the_glue, true)?;
    }
    if run.until_in_shape && collage {
        glue_limit_faces(&form.s_until, &locmap, &mut the_glue, false)?;
    }
    let ope = the_glue.ope_type();
    let expected = if form.fuse {
        Operation::Fuse
    } else {
        Operation::Cut
    };
    if ope == Operation::Invalid || ope != expected || !collage {
        run.the_ope = 2;
        run.change_ope = true;
    }
    Ok(the_glue)
}

/// The gluing itself: on a valid result the history follows the gluer, else the
/// operation switches to 2.
pub(super) fn perform_glue(
    form: &mut Form,
    run: &mut Run,
    the_glue: &mut Gluer,
) -> Result<(), Failure> {
    the_glue.perform()?;
    if !the_glue.is_done() || !brep_algo::is_valid(the_glue.resulting_shape()?)? {
        run.the_ope = 2;
        run.change_ope = true;
        return Ok(());
    }
    form.update_descendants_glue(the_glue)?;
    form.new_edges = the_glue.edges().clone();
    form.tgt_edges = the_glue.tgt_edges().clone();
    form.base.set_done();
    form.base.my_shape = the_glue.resulting_shape()?.clone();
    Ok(())
}
