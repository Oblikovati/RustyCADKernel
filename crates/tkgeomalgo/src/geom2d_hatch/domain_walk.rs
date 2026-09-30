// Rust translation of the point loop of Geom2dHatch_Hatcher::ComputeDomains(IndH) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The walk along the intersection points of one hatching that opens and closes
//! its domains.
use super::Hatching;
use crate::hatch_gen::{Domain, ErrorStatus, PointOnHatching};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::State;

/// What the walk does after a point: go on, go to the next point without the
/// domain and last-point steps (the native `continue`), or stop (`return`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Flow {
    Next,
    Skip,
    Stop,
}

/// The variables the native loop keeps across the points: the keep flags, the
/// index of the saved point (`ISav`, 0 for none), whether a point is saved
/// (`SavPnt`) and the number of opened segments.
pub(super) struct DomainWalk {
    points: bool,
    segments: bool,
    isav: i32,
    sav_pnt: bool,
    nb_opened_segments: i32,
}

// The data of the current point: its index, whether it is the first or last one,
// whether no domain was made before it, its states and its segment flags.
struct Current {
    ipnt: i32,
    first: bool,
    last: bool,
    no_domain: bool,
    point: PointOnHatching,
    before: State,
    after: State,
    begin: bool,
    end: bool,
}

// The end of an incoherent parity: the status when no domain was made, else the
// hatching is done with its domains.
fn incoherent(hatching: &mut Hatching, no_domain: bool) {
    if no_domain {
        hatching.set_status(ErrorStatus::IncoherentParity);
    } else {
        hatching.set_is_done(true);
    }
}

impl DomainWalk {
    pub(super) fn new(points: bool, segments: bool) -> Self {
        Self {
            points,
            segments,
            isav: 0,
            sav_pnt: false,
            nb_opened_segments: 0,
        }
    }

    /// One point of the loop: the first- and last-point initializations, the general
    /// case, the domain it makes and the domain closed at the last point.
    pub(super) fn step(
        &mut self,
        hatching: &mut Hatching,
        ipnt: i32,
        nb_pnt: i32,
    ) -> Result<Flow, Failure> {
        let mut cur = current(hatching, ipnt, nb_pnt)?;
        if cur.first {
            self.init_first(&mut cur);
        }
        if cur.last {
            complete_last_states(&mut cur);
        }
        let mut domain = Domain::new();
        let (flow, to_append) = self.general_case(hatching, &mut cur, nb_pnt, &mut domain)?;
        if flow != Flow::Next {
            return Ok(flow);
        }
        if to_append {
            hatching.add_domain(domain.clone());
        }
        if cur.last {
            return self.close_last(hatching, &cur, &mut domain);
        }
        Ok(Flow::Next)
    }

    // The initializations due to the first point.
    fn init_first(&mut self, cur: &mut Current) {
        self.sav_pnt = false;
        self.isav = 0;
        self.nb_opened_segments = 0;
        if cur.end {
            if cur.after == State::Unknown {
                cur.after = State::In;
            }
            if cur.begin && cur.before == State::Unknown {
                cur.before = State::In;
            }
            if self.segments {
                self.save(0);
            }
            return;
        }
        if cur.begin && cur.before == State::Unknown {
            cur.before = State::In;
        }
        if cur.before == State::In {
            self.save(0);
        }
    }

    fn save(&mut self, isav: i32) {
        self.sav_pnt = true;
        self.isav = isav;
    }

    fn forget(&mut self) {
        self.sav_pnt = false;
        self.isav = 0;
    }

    // A domain from the saved point (infinite when none) to the current point.
    fn close_at(
        &self,
        hatching: &Hatching,
        cur: &Current,
        domain: &mut Domain,
    ) -> Result<(), Failure> {
        self.open_from_saved(hatching, domain)?;
        domain.set_second_point(&cur.point);
        Ok(())
    }

    fn open_from_saved(&self, hatching: &Hatching, domain: &mut Domain) -> Result<(), Failure> {
        if self.isav != 0 {
            domain.set_first_point(hatching.point(self.isav)?);
        }
        Ok(())
    }
}

fn current(hatching: &Hatching, ipnt: i32, nb_pnt: i32) -> Result<Current, Failure> {
    let point: PointOnHatching = hatching.point(ipnt)?.clone();
    Ok(Current {
        ipnt,
        first: ipnt == 1,
        last: ipnt == nb_pnt,
        no_domain: hatching.nb_domains() == 0,
        before: point.state_before(),
        after: point.state_after(),
        begin: point.segment_beginning(),
        end: point.segment_end(),
        point,
    })
}

// The initializations due to the last point: an UNKNOWN state on a segment side is
// IN.
fn complete_last_states(cur: &mut Current) {
    if cur.end && cur.after == State::Unknown {
        cur.after = State::In;
    }
    if cur.begin && cur.before == State::Unknown {
        cur.before = State::In;
    }
}

impl DomainWalk {
    // The general case: by the segment flags of the point.
    fn general_case(
        &mut self,
        hatching: &mut Hatching,
        cur: &mut Current,
        nb_pnt: i32,
        domain: &mut Domain,
    ) -> Result<(Flow, bool), Failure> {
        match (cur.end, cur.begin) {
            (true, true) => self.segment_both(hatching, cur, domain),
            (true, false) => self.segment_end(hatching, cur, domain),
            (false, true) => self.segment_begin(hatching, cur, domain),
            (false, false) => self.plain_point(hatching, cur, nb_pnt, domain),
        }
    }

    // A point that both ends and begins a segment.
    fn segment_both(
        &mut self,
        hatching: &mut Hatching,
        cur: &Current,
        domain: &mut Domain,
    ) -> Result<(Flow, bool), Failure> {
        if cur.before != State::In && cur.after != State::In {
            hatching.set_status(ErrorStatus::IncompatibleStates);
            return Ok((Flow::Stop, false));
        }
        if !self.points {
            return Ok((Flow::Next, false));
        }
        if self.segments {
            return self.close_and_save(hatching, cur, domain);
        }
        let in_in: bool = cur.before == State::In && cur.after == State::In;
        if self.sav_pnt && !in_in {
            incoherent(hatching, cur.no_domain);
            return Ok((Flow::Stop, false));
        }
        domain.set_points(&cur.point, &cur.point);
        self.forget();
        Ok((Flow::Next, true))
    }

    // Closes a domain at the current point, which becomes the saved one; stops on
    // an incoherent parity when no point is saved.
    fn close_and_save(
        &mut self,
        hatching: &mut Hatching,
        cur: &Current,
        domain: &mut Domain,
    ) -> Result<(Flow, bool), Failure> {
        if !self.sav_pnt {
            incoherent(hatching, cur.no_domain);
            return Ok((Flow::Stop, false));
        }
        self.close_at(hatching, cur, domain)?;
        self.save(cur.ipnt);
        Ok((Flow::Next, true))
    }

    // A point that ends a segment.
    fn segment_end(
        &mut self,
        hatching: &mut Hatching,
        cur: &Current,
        domain: &mut Domain,
    ) -> Result<(Flow, bool), Failure> {
        let mut to_append: bool = false;
        if self.segments && cur.after == State::Out {
            if !self.sav_pnt {
                incoherent(hatching, cur.no_domain);
                return Ok((Flow::Stop, false));
            }
            self.close_at(hatching, cur, domain)?;
            to_append = true;
        } else if self.segments && self.points {
            self.close_at(hatching, cur, domain)?;
            to_append = true;
            self.save(cur.ipnt);
        } else if !self.segments && cur.after == State::In {
            self.save(cur.ipnt);
        }
        self.nb_opened_segments -= 1;
        Ok((Flow::Next, to_append))
    }

    // A point that begins a segment.
    fn segment_begin(
        &mut self,
        hatching: &mut Hatching,
        cur: &Current,
        domain: &mut Domain,
    ) -> Result<(Flow, bool), Failure> {
        let mut outcome = (Flow::Next, false);
        if self.segments && cur.before == State::Out {
            self.save(cur.ipnt);
        } else if (self.segments && self.points) || (!self.segments && cur.before == State::In) {
            // Modified by Sergey KHROMOV - Fri Jan  5 2001: the point is saved
            // (natively `SavPnt = false; ISav = 0` before).
            outcome = self.close_and_save(hatching, cur, domain)?;
            if outcome.0 == Flow::Stop {
                return Ok(outcome);
            }
        }
        self.nb_opened_segments += 1;
        Ok(outcome)
    }

    // A point inside no segment. Provisional solution (lbr, 11 Aug 97): with two
    // points, OUT-OUT becomes OUT-IN at the first and IN-OUT at the second.
    fn plain_point(
        &mut self,
        hatching: &mut Hatching,
        cur: &mut Current,
        nb_pnt: i32,
        domain: &mut Domain,
    ) -> Result<(Flow, bool), Failure> {
        if cur.before == State::Out && cur.after == State::Out && nb_pnt == 2 {
            if cur.first {
                cur.after = State::In;
            } else {
                cur.before = State::In;
            }
        }
        match (cur.before, cur.after) {
            (State::Out, State::Out) => self.out_out(hatching, cur, domain),
            (State::Out, State::In) => {
                self.save(cur.ipnt);
                Ok((Flow::Next, false))
            }
            (State::In, State::Out) => self.in_out(hatching, cur, domain),
            (State::In, State::In) => self.in_in(hatching, cur, domain),
            _ => {
                hatching.set_status(ErrorStatus::IncompatibleStates);
                Ok((Flow::Stop, false))
            }
        }
    }

    fn out_out(
        &mut self,
        hatching: &mut Hatching,
        cur: &Current,
        domain: &mut Domain,
    ) -> Result<(Flow, bool), Failure> {
        if self.sav_pnt {
            incoherent(hatching, cur.no_domain);
            return Ok((Flow::Stop, false));
        }
        if !self.points {
            return Ok((Flow::Next, false));
        }
        domain.set_points(&cur.point, &cur.point);
        self.save(cur.ipnt);
        Ok((Flow::Next, true))
    }

    fn in_out(
        &mut self,
        hatching: &mut Hatching,
        cur: &Current,
        domain: &mut Domain,
    ) -> Result<(Flow, bool), Failure> {
        if !self.sav_pnt {
            incoherent(hatching, cur.no_domain);
            return Ok((Flow::Stop, false));
        }
        self.close_at(hatching, cur, domain)?;
        self.forget();
        Ok((Flow::Next, true))
    }

    // IN-IN: a domain end only when points are kept. Without opened segment and
    // without saved point the native code goes on to the next point (`continue`,
    // its `return` commented out).
    fn in_in(
        &mut self,
        hatching: &mut Hatching,
        cur: &Current,
        domain: &mut Domain,
    ) -> Result<(Flow, bool), Failure> {
        if !self.points {
            return Ok((Flow::Next, false));
        }
        if self.nb_opened_segments == 0 {
            if !self.sav_pnt {
                incoherent(hatching, cur.no_domain);
                return Ok((Flow::Skip, false));
            }
            return self.close_and_save(hatching, cur, domain);
        }
        if self.segments {
            return self.close_and_save(hatching, cur, domain);
        }
        if self.sav_pnt {
            incoherent(hatching, cur.no_domain);
            return Ok((Flow::Stop, false));
        }
        domain.set_points(&cur.point, &cur.point);
        self.forget();
        Ok((Flow::Next, true))
    }
}

impl DomainWalk {
    // The processing of the last point: the domain from the saved point to the
    // infinite when the hatching goes on inside after it.
    fn close_last(
        &self,
        hatching: &mut Hatching,
        cur: &Current,
        domain: &mut Domain,
    ) -> Result<Flow, Failure> {
        domain.set_infinite_points();
        let open_after: bool = match (cur.end, cur.begin) {
            (true, true) | (false, true) => self.segments,
            (true, false) | (false, false) => cur.after == State::In,
        };
        if !open_after {
            return Ok(Flow::Next);
        }
        if !self.sav_pnt {
            incoherent(hatching, cur.no_domain);
            return Ok(Flow::Stop);
        }
        self.open_from_saved(hatching, domain)?;
        hatching.add_domain(domain.clone());
        Ok(Flow::Next)
    }
}
