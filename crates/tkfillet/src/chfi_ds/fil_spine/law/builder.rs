//! `ChFiDS_FilSpine::ComputeLaw`: the composite radius law of one elementary spine.
use super::pieces::{Frame, LawHandle, Span, constant, make_laws, periodic_interpolated};
use crate::chfi_ds::ElSpine;
use crate::chfi_ds::errors::{domain_error, out_of_range};
use crate::chfi_ds::fil_spine::FilSpine;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::native::std_min;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkgeomalgo::law::LawComposite;
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::Xy;

/// The running state of `ComputeLaw`, named as the native locals.
struct LawState<'a> {
    spine: &'a FilSpine,
    tol3d: f64,
    fin: f64,
    curdeb: f64,
    curfin: f64,
    spinedeb: f64,
    spinefin: f64,
    ind: Vec<i32>,
    icur: i32,
    rdeb: f64,
    rfin: f64,
    lawencours: bool,
    laws: Vec<LawHandle>,
}

/// The whole `ComputeLaw(Els)`.
pub(super) fn compute(
    spine: &FilSpine,
    el_spine: &ElSpine,
) -> Result<Handle<LawComposite>, Failure> {
    let mut state = LawState::new(spine, el_spine)?;
    let composite = LawComposite::new();
    if el_spine.is_periodic()? {
        composite.set_periodic();
        if let Some(law) = state.periodic_el_spine_start()? {
            composite.change_laws(|laws| {
                laws.append(law);
            });
            return Ok(Handle::new(composite));
        }
    } else if spine.is_periodic() {
        state.periodic_spine_ends()?;
    } else {
        state.open_spine_ends()?;
    }
    state.walk_edges()?;
    // The native `lastloi` is never set, so nothing is appended after the walk.
    composite.change_laws(|laws| {
        for law in state.laws.drain(..) {
            laws.append(law);
        }
    });
    Ok(Handle::new(composite))
}

impl<'a> LawState<'a> {
    /// The locals before the branches: the bounds of the elementary spine, the indices
    /// of the edges it crosses (wrapped on a periodic guideline).
    fn new(spine: &'a FilSpine, el_spine: &ElSpine) -> Result<Self, Failure> {
        let deb = el_spine.first_parameter()?;
        let fin = el_spine.last_parameter()?;
        let ideb = spine.index_at(deb, true)?;
        let ifin = spine.index_at(fin, false)?;
        let len = spine.nb_edges();
        let mut bidfin = ifin;
        if spine.is_periodic() {
            // The native operand order: LastParameter(len) is read before `ideb > ifin`.
            if (deb < 0. && ideb > ifin) || (fin > spine.last_parameter_of(len)? && ideb > ifin) {
                bidfin += len;
            }
        }
        if len == 0 {
            // `(i - 1) % len` divides by zero natively (SIGFPE).
            return Err(Failure::with_message(
                "Standard_DivideByZero",
                "ChFiDS_FilSpine::ComputeLaw: the guideline has 0 edges; expected at least one",
            ));
        }
        let ind = (ideb..=bidfin).map(|i| ((i - 1) % len) + 1).collect();
        Ok(Self {
            spine,
            tol3d: precision::confusion(),
            fin,
            curdeb: deb,
            curfin: fin,
            spinedeb: spine.first_parameter(),
            spinefin: spine.last_parameter()?,
            ind,
            icur: 1,
            rdeb: 0.,
            rfin: 0.,
            lawencours: false,
            laws: Vec::new(),
        })
    }

    /// `ind(k)`: Standard_OutOfRange outside `[1, nbed]`.
    fn ind(&self, k: i32) -> Result<i32, Failure> {
        let count = self.ind.len() as i32;
        if k < 1 || k > count {
            return Err(out_of_range("ChFiDS_FilSpine::ComputeLaw ind", k, count));
        }
        Ok(self.ind[(k - 1) as usize])
    }

    fn nbed(&self) -> i32 {
        self.ind.len() as i32
    }

    /// The periodic elementary spine: restart the walk at the first constant edge, else
    /// return one periodic interpolation of all the radii.
    fn periodic_el_spine_start(&mut self) -> Result<Option<LawHandle>, Failure> {
        let len = self.spine.nb_edges();
        for k in 1..=len {
            if self.spine.is_constant_on_edge(k)? {
                self.restart_on_constant_edge(k, len)?;
                return Ok(None);
            }
        }
        self.periodic_interpolation().map(Some)
    }

    /// A constant edge `k` of a periodic spine: the walk starts there, over one period.
    fn restart_on_constant_edge(&mut self, k: i32, len: i32) -> Result<(), Failure> {
        let deb = self.spine.first_parameter_of(k)?;
        (self.spinedeb, self.curdeb) = (deb, deb);
        self.fin = deb + self.spine.period()?;
        self.spinefin = self.fin;
        for l in 1..=len {
            let slot = self.ind.get_mut((l - 1) as usize);
            *slot.ok_or_else(|| out_of_range("ChFiDS_FilSpine::ComputeLaw ind", l, len))? =
                ((k + l - 2) % len) + 1;
        }
        self.rdeb = self.spine.radius_on_edge_index(k)?;
        self.rfin = self.rdeb;
        self.icur += 1;
        // With one edge, InPeriod would bring the end back to the start.
        self.curfin = if len == 1 {
            self.spine.last_parameter_of(k)?
        } else {
            elclib::in_period(
                self.spine.last_parameter_of(k)?,
                self.spinedeb,
                self.spinefin,
            )
        };
        self.laws
            .push(constant(self.rdeb, self.curdeb, self.curfin));
        self.curdeb = self.curfin;
        Ok(())
    }

    /// No constant edge on a periodic elementary spine: one periodic interpolation of the
    /// radii over one period from the first pair.
    fn periodic_interpolation(&mut self) -> Result<LawHandle, Failure> {
        let pairs = self.spine.par_and_rad();
        if pairs.is_empty() {
            return Err(domain_error("Radius not defined"));
        }
        let mut nbp = pairs.len();
        if nbp > 1 {
            self.fin = pairs[0].x() + self.spine.period()?;
            if pairs[nbp - 1].x() - self.fin < -self.tol3d {
                nbp += 1;
            }
        } else {
            nbp += 1;
        }
        let mut points: Vec<Xy> = pairs[..nbp - 1].to_vec();
        points.push(Xy::new(self.fin, points[0].y()));
        periodic_interpolated(&points)
    }

    /// A periodic guideline cut by a non-periodic elementary spine: the end radii come
    /// from the constant edges at or around both ends.
    fn periodic_spine_ends(&mut self) -> Result<(), Failure> {
        let len = self.spine.nb_edges();
        let first = self.ind(1)?;
        if self.spine.is_constant_on_edge(first)? {
            self.rdeb = self.spine.radius_on_edge_index(first)?;
            let end = self.spine.last_parameter_of(first)?;
            let end =
                elclib::in_period(end, self.spinedeb + self.tol3d, self.spinefin + self.tol3d);
            self.push_start_constant(std_min(self.fin, end));
        } else {
            let previous = if first - 1 == 0 { len } else { first - 1 };
            self.rdeb =
                self.constant_radius(previous, "AppendLaw : previous constant is missing!")?;
            self.lawencours = true;
        }
        let last = self.ind(self.nbed())?;
        if self.spine.is_constant_on_edge(last)? {
            self.rfin = self.spine.radius_on_edge_index(last)?;
            return Ok(());
        }
        let next = if last + 1 == len + 1 { 1 } else { last + 1 };
        self.rfin = self.constant_radius(next, "AppendLaw : next constant is missing!")?;
        Ok(())
    }

    /// An open guideline: the end radii come from the end edges, their neighbours, or
    /// the pairs (-1 when the end edge is the first or last of the guideline).
    fn open_spine_ends(&mut self) -> Result<(), Failure> {
        let len = self.spine.nb_edges();
        let first = self.ind(1)?;
        if self.spine.is_constant_on_edge(first)? {
            self.rdeb = self.spine.radius_on_edge_index(first)?;
            let end = std_min(self.fin, self.spine.last_parameter_of(first)?);
            self.push_start_constant(end);
        } else {
            self.rdeb = self.open_end_radius(
                first > 1,
                first - 1,
                "AppendLaw : previous constant is missing",
            )?;
            self.lawencours = true;
        }
        let last = self.ind(self.nbed())?;
        if self.spine.is_constant_on_edge(last)? {
            self.rfin = self.spine.radius_on_edge_index(last)?;
            return Ok(());
        }
        self.rfin =
            self.open_end_radius(last < len, last + 1, "AppendLaw : next constant is missing")?;
        Ok(())
    }

    /// The radius of the neighbour `neighbour` when `has_neighbour`, else -1 (or
    /// Standard_DomainError without any radius).
    fn open_end_radius(
        &self,
        has_neighbour: bool,
        neighbour: i32,
        missing: &str,
    ) -> Result<f64, Failure> {
        if has_neighbour {
            return self.constant_radius(neighbour, missing);
        }
        if self.spine.par_and_rad().is_empty() {
            return Err(domain_error("AppendLaw : no radius on vertex"));
        }
        Ok(-1.)
    }

    /// The radius of the constant edge `index`, else Standard_DomainError `missing`.
    fn constant_radius(&self, index: i32, missing: &str) -> Result<f64, Failure> {
        if self.spine.is_constant_on_edge(index)? {
            return self.spine.radius_on_edge_index(index);
        }
        Err(domain_error(missing))
    }

    /// The constant law of the start edge, up to `end`.
    fn push_start_constant(&mut self, end: f64) {
        self.curfin = end;
        self.laws
            .push(constant(self.rdeb, self.curdeb, self.curfin));
        self.curdeb = self.curfin;
        self.icur += 1;
    }

    /// The loop over the remaining edges: a constant edge closes the pending law and adds
    /// its constant piece; a variable edge opens (or extends) a pending law, made at the
    /// last edge.
    fn walk_edges(&mut self) -> Result<(), Failure> {
        while self.icur <= self.nbed() {
            let edge = self.ind(self.icur)?;
            if self.spine.is_constant_on_edge(edge)? {
                self.constant_edge(edge)?;
            } else {
                self.variable_edge(edge)?;
            }
            self.icur += 1;
        }
        Ok(())
    }

    fn periodic_bound(&self, w: f64) -> f64 {
        elclib::in_period(w, self.spinedeb + self.tol3d, self.spinefin + self.tol3d)
    }

    /// A constant edge of the walk.
    fn constant_edge(&mut self, edge: i32) -> Result<(), Failure> {
        let rcur = self.spine.radius_on_edge_index(edge)?;
        if self.lawencours {
            self.make_pending(rcur)?;
            self.lawencours = false;
            self.curdeb = self.curfin;
        }
        self.curfin = self.spine.last_parameter_of(edge)?;
        if self.spine.is_periodic() {
            self.curfin = self.periodic_bound(self.curfin);
            if edge == self.ind(self.nbed())? {
                self.curfin = self.last_edge_end(edge)?;
            } else {
                self.curfin = std_min(self.fin, self.curfin);
            }
        }
        if (self.curfin - self.curdeb) > self.tol3d {
            self.rdeb = rcur;
            self.laws
                .push(constant(self.rdeb, self.curdeb, self.curfin));
            self.curdeb = self.curfin;
        }
        Ok(())
    }

    /// The end of the last edge of a periodic guideline: the end of the elementary spine
    /// when the edge passes over the periodic origin, else the edge end capped by it.
    fn last_edge_end(&self, edge: i32) -> Result<f64, Failure> {
        let start = self.periodic_bound(self.spine.first_parameter_of(edge)?);
        if start >= self.curfin {
            return Ok(self.fin);
        }
        Ok(std_min(self.fin, self.curfin))
    }

    /// A variable edge of the walk.
    fn variable_edge(&mut self, edge: i32) -> Result<(), Failure> {
        self.curfin = self.spine.last_parameter_of(edge)?;
        if self.spine.is_periodic() {
            self.curfin = self.periodic_bound(self.curfin);
        }
        self.curfin = std_min(self.fin, self.curfin);
        self.lawencours = true;
        if edge != self.ind(self.nbed())? {
            return Ok(());
        }
        if self.spine.is_periodic() {
            self.curfin = self.periodic_bound(self.spine.last_parameter_of(edge)?);
            self.curfin = self.last_edge_end(edge)?;
        } else if edge == self.spine.nb_edges() {
            // The end of the guideline, possibly extended.
            self.curfin = self.fin;
        }
        self.make_pending(self.rfin)
    }

    /// `mklaw` over `[curdeb, curfin]` from `rdeb` to `end_radius`.
    fn make_pending(&mut self, end_radius: f64) -> Result<(), Failure> {
        let span = Span {
            start: self.curdeb,
            end: self.curfin,
            start_radius: self.rdeb,
            end_radius,
        };
        let frame = Frame {
            recadre: self.spine.is_periodic(),
            deb: self.spinedeb,
            fin: self.spinefin,
            tol3d: self.tol3d,
        };
        make_laws(&mut self.laws, self.spine.par_and_rad(), span, frame)
    }
}
