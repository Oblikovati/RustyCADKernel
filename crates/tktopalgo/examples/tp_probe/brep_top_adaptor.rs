//! BRepTopAdaptor probe operations over faces read by `tp_face`, the Rust twin of
//! tools/oracle/tktopalgo_probe_breptopadaptor.cxx. Edges are printed as their index
//! among the edges of the FORWARD face (`tp_edge_index`), states as integers.
use super::common::{Reader, attempt, end, flag, int, pnt, pnt2d, real, show};
use super::face::{tp_edge_index, tp_face};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorSurface;
use rustycad_tkbrep::topods::Vertex;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkg3d::adaptor3d::{Adaptor3dHVertex, Adaptor3dHVertexBase, Adaptor3dSurface};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorSurface;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::gp::Point2;
use rustycad_tktopalgo::brep_top_adaptor::{
    BRepTopAdaptorFClass2d, BRepTopAdaptorHVertex, BRepTopAdaptorTool, BRepTopAdaptorTopolTool,
};

/// `face tol ontol n (u v)...`: FClass2d states per point.
pub fn fclass2d_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let f = tp_face(&mut r)?;
        let (tol, ontol) = (r.real(), r.real());
        let c = BRepTopAdaptorFClass2d::new(&f.face, tol)?;
        show(c.perform_infinite_point().map(|value| value as i32), int);
        for _ in 0..r.integer() {
            let p = r.pnt2d();
            show(c.perform(p, true).map(|value| value as i32), int);
            show(c.perform(p, false).map(|value| value as i32), int);
            show(
                c.test_on_restriction(p, ontol, true)
                    .map(|value| value as i32),
                int,
            );
            show(
                c.test_on_restriction(p, ontol, false)
                    .map(|value| value as i32),
                int,
            );
        }
        Ok(())
    });
    end();
}

/// `face tol u v`: Perform and TestOnRestriction before and after Destroy.
pub fn destroy_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let f = tp_face(&mut r)?;
        let tol = r.real();
        let p = r.pnt2d();
        let mut c = BRepTopAdaptorFClass2d::new(&f.face, tol)?;
        int(c.perform(p, true)? as i32);
        int(c.test_on_restriction(p, tol, true)? as i32);
        c.destroy();
        int(c.perform(p, true)? as i32);
        int(c.test_on_restriction(p, tol, true)? as i32);
        int(c.perform_infinite_point()? as i32);
        Ok(())
    });
    end();
}

/// `tp_bta_vertices`: the vertices of `arc` through the tool.
fn vertices(
    t: &BRepTopAdaptorTopolTool,
    arc: &Handle<dyn Adaptor2dCurve2d>,
) -> Result<(), Failure> {
    t.initialize_curve(arc)?;
    let mut first: Handle<dyn Adaptor3dHVertex> = Handle::default();
    t.init_vertex_iterator()?;
    while t.more_vertex() {
        let v = t.vertex()?;
        if first.is_null() {
            first = v.clone();
        }
        int(t.vertex_orientation(&v)? as i32);
        show(t.vertex_tol_3d(&v), real);
        show(t.pnt(&v), pnt);
        // Through the base handle, as the native probe calls them: the vertex trait
        // dispatches to the BRepTopAdaptor_HVertex overrides.
        pnt2d(v.value());
        show(v.parameter(arc), real);
        show(v.resolution(arc), real);
        flag(v.is_same(&first)?);
        t.next_vertex()?;
    }
    attempt(|| t.vertex().map(|_| ()));
    Ok(())
}

/// The samples of a tool: counts and six sample points.
fn samples(t: &BRepTopAdaptorTopolTool) -> Result<(), Failure> {
    let (nu, nv) = (t.nb_samples_u()?, t.nb_samples_v()?);
    int(nu);
    int(nv);
    int(t.nb_samples()?);
    for i in [0, 1, nu, nu * nv / 2, nu * nv - 1, nu * nv] {
        attempt(|| {
            let (p2, p3) = t.sample_point(i)?;
            pnt2d(p2);
            pnt(p3);
            Ok(())
        });
    }
    Ok(())
}

/// `face restr`: the arcs of the tool with their vertices, the samples and the domain.
pub fn topol_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let f = tp_face(&mut r)?;
        let restr = r.flag();
        let s: Handle<dyn Adaptor3dSurface> =
            Handle::new(BRepAdaptorSurface::with_face(&f.face, restr)?).cast();
        let t = BRepTopAdaptorTopolTool::with_surface(&s)?;
        flag(t.has_3d());
        flag(t.more());
        t.init();
        while t.more() {
            let arc = t.value()?;
            let edge = t.edge()?;
            int(tp_edge_index(&f.face, &edge));
            int(t.curve_orientation(&arc)? as i32);
            show(t.curve_tol_3d(&arc), real);
            real(arc.first_parameter()?);
            real(arc.last_parameter()?);
            attempt(|| vertices(&t, &arc));
            t.next();
        }
        show(t.domain_is_infinite(), flag);
        attempt(|| samples(&t));
        Ok(())
    });
    end();
}

/// `face restr tol n (u v)...`: Classify and IsThePointOn of a tool.
pub fn classify_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let f = tp_face(&mut r)?;
        let (restr, tol) = (r.flag(), r.real());
        let s: Handle<dyn Adaptor3dSurface> =
            Handle::new(BRepAdaptorSurface::with_face(&f.face, restr)?).cast();
        let t = BRepTopAdaptorTopolTool::with_surface(&s)?;
        for _ in 0..r.integer() {
            let p = r.pnt2d();
            show(t.classify(p, tol, true).map(|value| value as i32), int);
            show(t.classify(p, tol, false).map(|value| value as i32), int);
            show(t.is_the_point_on(p, tol, true), flag);
            show(t.is_the_point_on(p, tol, false), flag);
        }
        t.destroy();
        show(
            t.classify(Point2::new(0.0, 0.0), tol, true)
                .map(|value| value as i32),
            int,
        );
        Ok(())
    });
    end();
}

/// `face`: BRepTopAdaptor_Tool on the face and on its adaptor.
pub fn tool_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let f = tp_face(&mut r)?;
        let mut a = BRepTopAdaptorTool::new();
        flag(a.get_surface().is_null());
        flag(a.get_topol_tool().is_null());
        let b = BRepTopAdaptorTool::with_face(&f.face, 1e-9)?;
        int(b.get_surface().get_type()? as i32);
        real(b.get_surface().first_u_parameter()?);
        real(b.get_surface().last_v_parameter()?);
        show(b.get_topol_tool().nb_samples(), int);
        let s: Handle<dyn Adaptor3dSurface> =
            Handle::new(BRepAdaptorSurface::with_face(&f.face, false)?).cast();
        let c = BRepTopAdaptorTool::with_surface(&s, 1e-9)?;
        flag(c.get_surface().object_address() == s.object_address());
        show(c.get_topol_tool().domain_is_infinite(), flag);
        a.init_face(&f.face, 1e-9)?;
        int(a.get_surface().get_type()? as i32);
        a.init_surface(&s, 0.0)?;
        flag(a.get_surface().object_address() == s.object_address());
        a.set_topol_tool(&c.get_topol_tool());
        flag(a.get_topol_tool().object_address() == c.get_topol_tool().object_address());
        a.destroy();
        Ok(())
    });
    end();
}

/// `face`: the errors of the tool.
pub fn errors_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let f = tp_face(&mut r)?;
        let t = BRepTopAdaptorTopolTool::new();
        int(t.classify(Point2::new(0.0, 0.0), 1e-7, true)? as i32);
        attempt(|| t.initialize());
        let gs: Handle<dyn Adaptor3dSurface> = Handle::new(GeomAdaptorSurface::with_surface(
            &BRepTool::surface(&f.face)?,
        )?)
        .cast();
        attempt(|| t.initialize_surface(&gs));
        attempt(|| BRepTopAdaptorTopolTool::with_surface(&gs).map(|_| ()));
        let gc: Handle<dyn Adaptor2dCurve2d> = Handle::new(Geom2dAdaptorCurve::default()).cast();
        attempt(|| t.initialize_curve(&gc));
        show(t.curve_tol_3d(&gc), real);
        let hv: Handle<dyn Adaptor3dHVertex> = Handle::new(Adaptor3dHVertexBase::new(
            Point2::new(1.0, 2.0),
            Orientation::Reversed,
            0.5,
        ))
        .cast();
        int(t.vertex_orientation(&hv)? as i32);
        show(t.vertex_tol_3d(&hv), real);
        show(t.pnt(&hv), pnt);
        let nv = Handle::new(BRepTopAdaptorHVertex::new(
            &Vertex::default(),
            &Handle::default(),
        ));
        let nvb: Handle<dyn Adaptor3dHVertex> = nv.cast();
        show(t.vertex_tol_3d(&nvb), real);
        show(t.pnt(&nvb), pnt);
        pnt2d(nv.value());
        int(nv.orientation() as i32);
        let s: Handle<dyn Adaptor3dSurface> =
            Handle::new(BRepAdaptorSurface::with_face(&f.face, true)?).cast();
        t.initialize_surface(&s)?;
        t.init();
        if t.more() {
            t.initialize_curve(&t.value()?)?;
            t.init_vertex_iterator()?;
            while t.more_vertex() {
                t.next_vertex()?;
            }
            attempt(|| t.vertex().map(|_| ()));
        }
        Ok(())
    });
    end();
}
