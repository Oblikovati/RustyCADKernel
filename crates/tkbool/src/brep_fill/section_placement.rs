// Rust translation of ModelingAlgorithms/TKBool/BRepFill/BRepFill_SectionPlacement.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepFill_SectionPlacement`: where a section stands on the path of a location law
//! and the transformation that brings it there.
use super::location_law::BRepFillLocationLaw;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorCompCurve;
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Edge, Shape, Vertex};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::geom::{
    GeomBoundedCurve, GeomCartesianPoint, GeomCurve, GeomGeometry, GeomTrimmedCurve,
};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkgeomalgo::geom_fill::GeomFillSectionPlacement;
use rustycad_tkgeombase::geom_convert::CompCurveToBSplineCurve;
use rustycad_tkmath::convert::ParameterisationType;
use rustycad_tkmath::gp::Transform3;

/// The placement of a section on a location law (`BRepFill_SectionPlacement`);
/// `let place = BRepFillSectionPlacement::new(&law, &profile, false, false)?;`.
pub struct BRepFillSectionPlacement {
    my_law: Handle<dyn BRepFillLocationLaw>,
    my_section: Shape,
    my_trsf: Transform3,
    my_param: f64,
    my_index: i32,
}

/// The law of a live handle.
fn live_law(law: &Handle<dyn BRepFillLocationLaw>) -> Result<&dyn BRepFillLocationLaw, Failure> {
    law.get().ok_or_else(|| {
        Failure::with_type(
            "Standard_NullObject",
            "BRepFill_SectionPlacement: null location law; expected a law",
            "BRepFill_SectionPlacement",
        )
    })
}

/// The 3d curve of a non-degenerated edge with a curve, trimmed to its range.
fn edge_curve(edge: &Edge) -> Result<Option<Handle<dyn GeomCurve>>, Failure> {
    if edge.is_null() || BRepTool::degenerated(edge)? {
        return Ok(None);
    }
    let (curve, first, last) = BRepTool::curve(edge)?;
    if curve.is_null() {
        return Ok(None);
    }
    Ok(Some(
        Handle::new(GeomTrimmedCurve::new(&curve, first, last, true, true)?).cast(),
    ))
}

/// The geometry of the section: its edges concatenated into one curve (a junction is
/// added within min(Confusion, vertex tolerance), then max of them), or the point of
/// its first vertex when it has no edge with a curve.
fn section_geometry(section: &Shape) -> Result<Handle<dyn GeomGeometry>, Failure> {
    let mut explorer = Explorer::new(section, ShapeEnum::Edge, ShapeEnum::Shape)?;
    let mut first: Option<Handle<dyn GeomCurve>> = None;
    while explorer.more() && first.is_none() {
        first = edge_curve(&topods::edge(explorer.current())?)?;
        explorer.next()?;
    }
    let Some(first) = first else {
        // No edge found: the section is a vertex.
        let vertices = Explorer::new(section, ShapeEnum::Vertex, ShapeEnum::Shape)?;
        let p = BRepTool::pnt(&topods::vertex(vertices.current())?)?;
        return Ok(Handle::new(GeomCartesianPoint::new(p.x(), p.y(), p.z())).cast());
    };
    if !explorer.more() {
        return Ok(first.cast());
    }
    let mut converter = CompCurveToBSplineCurve::with_curve(
        &first.cast::<dyn GeomBoundedCurve>(),
        ParameterisationType::TgtThetaOver2,
    )?;
    while explorer.more() {
        let edge: Edge = topods::edge(explorer.current())?;
        explorer.next()?;
        let Some(trimmed) = edge_curve(&edge)? else {
            continue;
        };
        let (v1, v2) = topexp::vertices(&edge, false);
        let vertex_tol: f64 =
            BRepTool::vertex_tolerance(&v1)?.max(BRepTool::vertex_tolerance(&v2)?);
        let bounded = trimmed.cast::<dyn GeomBoundedCurve>();
        if !converter.add(
            &bounded,
            precision::confusion().min(vertex_tol),
            false,
            true,
            0,
        )? {
            converter.add(
                &bounded,
                precision::confusion().max(vertex_tol),
                false,
                true,
                0,
            )?;
        }
    }
    Ok(converter.bspline_curve().cast())
}

/// `SearchParam(Law, Ind, TheV)`: the parameter of the vertex on the law curve of edge
/// `ind` (mapped onto the reversed law curve for a REVERSED edge).
fn search_param(law: &dyn BRepFillLocationLaw, ind: i32, the_v: &Vertex) -> Result<f64, Failure> {
    let e: Edge = law.edge(ind)?;
    let mut t: f64 = BRepTool::parameter(the_v, &e)?;
    if e.orientation() == rustycad_tkg3d::top_abs::Orientation::Reversed {
        let (_, f, l) = BRepTool::curve(&e)?;
        let curve = law
            .law(ind)?
            .get()
            .map(|g| g.get_curve())
            .unwrap_or_default();
        let c = curve.get().ok_or_else(|| {
            Failure::with_type(
                "Standard_NullObject",
                "BRepFill_SectionPlacement: law without curve; expected a curve",
                "BRepFill_SectionPlacement",
            )
        })?;
        let (lf, ll) = (c.first_parameter()?, c.last_parameter()?);
        t = ll - (t - f) * (ll - lf) / (l - f);
    }
    Ok(t)
}

impl BRepFillSectionPlacement {
    /// `BRepFill_SectionPlacement(Law, Section, WithContact, WithCorrection)`.
    pub fn new(
        law: &Handle<dyn BRepFillLocationLaw>,
        section: &Shape,
        with_contact: bool,
        with_correction: bool,
    ) -> Result<Self, Failure> {
        Self::with_vertex(
            law,
            section,
            &Shape::default(),
            with_contact,
            with_correction,
        )
    }

    /// `BRepFill_SectionPlacement(Law, Section, Vertex, WithContact, WithCorrection)`:
    /// the section placed near the path vertex `vertex` when it is one.
    pub fn with_vertex(
        law: &Handle<dyn BRepFillLocationLaw>,
        section: &Shape,
        vertex: &Shape,
        with_contact: bool,
        with_correction: bool,
    ) -> Result<Self, Failure> {
        let mut placement = Self {
            my_law: law.clone(),
            my_section: section.clone(),
            my_trsf: Transform3::default(),
            my_param: 0.,
            my_index: 0,
        };
        placement.perform(with_contact, with_correction, vertex)?;
        Ok(placement)
    }

    /// `Transformation()`; `placement.transformation()`.
    pub fn transformation(&self) -> &Transform3 {
        &self.my_trsf
    }

    /// `AbscissaOnPath()`: the abscissa of the placement on the path;
    /// `placement.abscissa_on_path()?`.
    pub fn abscissa_on_path(&self) -> Result<f64, Failure> {
        live_law(&self.my_law)?.abscissa(self.my_index, self.my_param)
    }

    /// `Perform(WithContact, WithCorrection, Vertex)`.
    fn perform(
        &mut self,
        with_contact: bool,
        with_correction: bool,
        vertex: &Shape,
    ) -> Result<(), Failure> {
        let law = live_law(&self.my_law)?;
        let a_section = section_geometry(&self.my_section)?;
        let mut placement = GeomFillSectionPlacement::new(&law.law(1)?, &a_section)?;
        let wire_adaptor: Handle<dyn Adaptor3dCurve> =
            Handle::new(BRepAdaptorCompCurve::with_wire(law.wire(), false)?).cast();
        placement.perform_on_path(&wire_adaptor, precision::confusion())?;
        let (index1, mut index2) = law_interval(law, placement.parameter_on_path())?;
        let a_vertex: Vertex = topods::vertex(vertex)?;
        let (index1, on_law) = vertex_on_law(law, &a_vertex, index1)?;
        if on_law {
            index2 = 0;
        }
        self.place_on(&mut placement, law, index1, on_law.then_some(&a_vertex))?;
        self.my_trsf = placement.transformation(with_contact, with_correction)?;
        self.my_index = index1;
        self.my_param = placement.parameter_on_path();
        let angle: f64 = placement.angle();
        if index2 == 0 {
            return Ok(());
        }
        let mut reversed = a_vertex.clone();
        if on_law && index1 == index2 {
            reversed.reverse();
        }
        self.place_on(&mut placement, law, index2, on_law.then_some(&reversed))?;
        if placement.angle() > angle {
            self.my_trsf = placement.transformation(with_contact, with_correction)?;
            self.my_index = index2;
            self.my_param = placement.parameter_on_path();
        }
        Ok(())
    }

    /// The placement on the law of edge `index`, at the parameter of the vertex when
    /// the section is located by a path vertex.
    fn place_on(
        &self,
        placement: &mut GeomFillSectionPlacement,
        law: &dyn BRepFillLocationLaw,
        index: i32,
        vertex: Option<&Vertex>,
    ) -> Result<(), Failure> {
        placement.set_location(&law.law(index)?);
        match vertex {
            None => placement.perform(precision::confusion()),
            Some(v) => placement.perform_at(search_param(law, index, v)?, precision::confusion()),
        }
    }
}

/// The law interval `[i - 1, i]` holding the section parameter on the concatenated
/// path, and the neighbouring law when the parameter is at a knot; Standard_
/// ConstructionError when no interval holds it.
fn law_interval(law: &dyn BRepFillLocationLaw, section_param: f64) -> Result<(i32, i32), Failure> {
    let conf: f64 = precision::p_confusion();
    let nb: i32 = law.nb_law()?;
    for idx in 1..=nb {
        let (curr, next) = (f64::from(idx - 1), f64::from(idx));
        if !(curr - conf <= section_param && next + conf >= section_param) {
            continue;
        }
        let mut idx2: i32 = 0;
        if (section_param - curr).abs() < conf && idx > 1 {
            idx2 = idx - 1;
        } else if (section_param - next).abs() < conf && idx < nb {
            idx2 = idx + 1;
        }
        return Ok((idx, idx2));
    }
    Err(Failure::with_type(
        "Standard_ConstructionError",
        &format!("Interval is not found: section parameter {section_param}; expected 0..={nb}"),
        "BRepFill_SectionPlacement",
    ))
}

/// The first law whose edge ends at the vertex, if any (the given index otherwise).
fn vertex_on_law(
    law: &dyn BRepFillLocationLaw,
    vertex: &Vertex,
    index1: i32,
) -> Result<(i32, bool), Failure> {
    if vertex.is_null() {
        return Ok((index1, false));
    }
    for idx in 1..=law.nb_law()? {
        let (v1, v2) = topexp::vertices(&law.edge(idx)?, false);
        if v1.is_same(vertex) || v2.is_same(vertex) {
            return Ok((idx, true));
        }
    }
    Ok((index1, false))
}
