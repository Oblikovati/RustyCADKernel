// Rust translation of the Perform methods of
// ModelingAlgorithms/TKFeat/BRepFeat/BRepFeat_MakeCylindricalHole.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The placements of the hole: the entry (forward) and exit (reversed) points of the
//! axis on the shape found by walking its intersections, the cylinder between them, and
//! the parts of the cylinder kept after the cut.
use super::BRepFeatMakeCylindricalHole;
use super::statics::{box_parameters, create_cyl, cylinder};
use crate::brep_feat::Status;
use crate::errors::construction_error;
use crate::loc_ope::{CurveShapeIntersector, PntFace};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::Orientation;

/// The walk along the intersections of the axis: the orientation and the indices of the
/// last franc point found (`theOr`, `IndFrom`, `IndTo`).
struct Walk<'a> {
    asi: &'a CurveShapeIntersector,
    or: Orientation,
    from: i32,
    to: i32,
}

impl<'a> Walk<'a> {
    fn new(asi: &'a CurveShapeIntersector) -> Self {
        Self {
            asi,
            or: Orientation::Forward,
            from: 0,
            to: 0,
        }
    }

    /// `LocalizeAfter(Param, ...)`.
    fn after(&mut self, prm: f64) -> Result<bool, Failure> {
        self.asi
            .localize_after(prm, &mut self.or, &mut self.from, &mut self.to)
    }

    /// `LocalizeAfter(Index, ...)`.
    fn after_index(&mut self, ind: i32) -> Result<bool, Failure> {
        self.asi
            .localize_after_index(ind, &mut self.or, &mut self.from, &mut self.to)
    }

    /// `LocalizeBefore(Param, ...)`.
    fn before(&mut self, prm: f64) -> Result<bool, Failure> {
        self.asi
            .localize_before(prm, &mut self.or, &mut self.from, &mut self.to)
    }

    /// `LocalizeBefore(Index, ...)`.
    fn before_index(&mut self, ind: i32) -> Result<bool, Failure> {
        self.asi
            .localize_before_index(ind, &mut self.or, &mut self.from, &mut self.to)
    }

    fn point(&self, ind: i32) -> Result<PntFace, Failure> {
        Ok(self.asi.point(ind)?.clone())
    }

    /// The first franc point after the start, a reversed one walked back to its entry
    /// (`on reset`); whether found (the orientation then tells whether it is an entry).
    fn entry_after(&mut self, prm: f64) -> Result<bool, Failure> {
        let ok = self.after(prm)?;
        if ok && self.or == Orientation::Reversed {
            return self.before_index(self.from);
        }
        Ok(ok)
    }
}

impl BRepFeatMakeCylindricalHole {
    /// `Perform(Radius)`: the hole through all of the shape;
    /// `hole.perform(1.)?; hole.build()?`.
    pub fn perform(&mut self, radius: f64) -> Result<(), Failure> {
        let asi = self.start(false, None)?;
        if !asi.is_done() || asi.nb_points()? <= 0 {
            self.status = Status::InvalidPlacement;
            return Ok(());
        }
        // It is not possible to use infinite cylinder for topological operations.
        let a_object = self
            .builder
            .arguments()
            .first()
            .cloned()
            .unwrap_or_default();
        let (pmin, pmax) = box_parameters(&a_object, &self.axis)?;
        let heigth = 2. * (pmax - pmin);
        let orig =
            self.axis.location().xyz() + self.axis.direction().xyz() * ((3. * pmin - pmax) / 2.);
        let cyl = cylinder(orig, &self.axis, radius, heigth)?;
        self.validate = false;
        self.cut_by(cyl, true)
    }

    /// The cylinder between the entry and the exit cut from the shape; the parts of the
    /// cylinder left (None after InvalidPlacement).
    fn drill(
        &mut self,
        first: &PntFace,
        last: &PntFace,
        radius: f64,
    ) -> Result<Option<Vec<(Shape, f64)>>, Failure> {
        let cyl = create_cyl(first, last, radius, &self.axis)?;
        self.cut_by(cyl, true)?;
        self.parts()
    }

    /// `PerformThruNext(Radius, Cont)`: the hole from the first entry after the axis
    /// origin to the next exit (validated on request); `hole.perform_thru_next(1., true)?`.
    pub fn perform_thru_next(&mut self, radius: f64, cont: bool) -> Result<(), Failure> {
        let asi = self.start(false, Some(cont))?;
        if !asi.is_done() {
            self.status = Status::InvalidPlacement;
            return Ok(());
        }
        let Some((first, last)) = thru_next_points(&asi)? else {
            self.status = Status::InvalidPlacement;
            return Ok(());
        };
        let Some(parts) = self.drill(&first, &last, radius)? else {
            return Ok(());
        };
        if parts.is_empty() {
            return Ok(());
        }
        // preserve the smallest as parameter along the axis
        let tokeep = smallest_within(&parts, first.parameter(), last.parameter())
            .unwrap_or_else(|| closest_outside(&parts, first.parameter(), last.parameter()));
        self.keep_one(&parts, &tokeep)
    }

    /// `PerformUntilEnd(Radius, Cont)`: the hole from the first entry after the axis
    /// origin to the last exit of the shape; `hole.perform_until_end(1., true)?`.
    pub fn perform_until_end(&mut self, radius: f64, cont: bool) -> Result<(), Failure> {
        let asi = self.start(false, Some(cont))?;
        if !asi.is_done() {
            self.status = Status::InvalidPlacement;
            return Ok(());
        }
        let mut w = Walk::new(&asi);
        let mut ok = w.entry_after(0.)?;
        // natively a found point not forward keeps `ok` with default points
        let (mut first, mut last) = (PntFace::default(), PntFace::default());
        if ok && w.or == Orientation::Forward {
            first = w.point(w.from)?;
            ok = w.before_index(asi.nb_points()? + 1)? && w.or == Orientation::Reversed;
            if ok {
                last = w.point(w.to)?;
            }
        }
        if !ok {
            self.status = Status::InvalidPlacement;
            return Ok(());
        }
        let Some(parts) = self.drill(&first, &last, radius)? else {
            return Ok(());
        };
        // preserve everything above the First
        for (p, parbar) in &parts {
            if *parbar > first.parameter() {
                self.builder.keep_part(p)?;
            }
        }
        Ok(())
    }

    /// `Perform(Radius, PFrom, PTo, Cont)`: the hole between the parameters `pfrom` and
    /// `pto` along the axis; `hole.perform_between(1., 0., 5., true)?`.
    pub fn perform_between(
        &mut self,
        radius: f64,
        pfrom: f64,
        pto: f64,
        cont: bool,
    ) -> Result<(), Failure> {
        let asi = self.start(false, Some(cont))?;
        if !asi.is_done() {
            self.status = Status::InvalidPlacement;
            return Ok(());
        }
        let (the_pfrom, the_pto) = if pfrom < pto {
            (pfrom, pto)
        } else {
            (pto, pfrom)
        };
        let mut w = Walk::new(&asi);
        let mut ok = w.entry_after(the_pfrom)?;
        // natively a found point not forward keeps `ok` with default points
        let (mut first, mut last) = (PntFace::default(), PntFace::default());
        if ok && w.or == Orientation::Forward {
            first = w.point(w.from)?;
            ok = w.before(the_pto)?;
            if ok {
                if w.or == Orientation::Forward {
                    ok = w.after_index(w.to)?;
                }
                if ok && w.or == Orientation::Reversed {
                    last = w.point(w.to)?;
                }
            }
        }
        if !ok {
            self.status = Status::InvalidPlacement;
            return Ok(());
        }
        let Some(parts) = self.drill(&first, &last, radius)? else {
            return Ok(());
        };
        // preserve parts between First and Last
        for (p, parbar) in &parts {
            if *parbar >= first.parameter() && *parbar <= last.parameter() {
                self.builder.keep_part(p)?;
            }
        }
        Ok(())
    }

    /// `PerformBlind(Radius, Length, Cont)`: the hole of depth `length` from the first
    /// entry after the axis origin (HoleTooLong when the next exit is nearer);
    /// `hole.perform_blind(1., 4., true)?`.
    pub fn perform_blind(&mut self, radius: f64, length: f64, cont: bool) -> Result<(), Failure> {
        if length <= 0. {
            return Err(construction_error(
                "BRepFeat_MakeCylindricalHole::PerformBlind",
                "non-positive length",
            ));
        }
        let asi = self.start(true, Some(cont))?;
        if !asi.is_done() {
            self.status = Status::InvalidPlacement;
            return Ok(());
        }
        let mut w = Walk::new(&asi);
        if !(w.entry_after(0.)? && w.or == Orientation::Forward) {
            self.status = Status::InvalidPlacement;
            return Ok(());
        }
        // check a priori the length of the hole
        let (ind_from, ind_to) = (w.from, w.to);
        if !w.after_index(ind_to)? {
            self.status = Status::InvalidPlacement;
            return Ok(());
        }
        if asi.point(w.from)?.parameter() <= length {
            self.status = Status::HoleTooLong;
            return Ok(());
        }
        let first = asi.point(ind_from)?.parameter();
        self.blind_tool(radius, length, first)
    }

    /// The tool of a blind hole: a cylinder from the box start along the axis, its top
    /// face at the depth; the part of the cylinder nearest after the entry kept.
    fn blind_tool(&mut self, radius: f64, length: f64, first: f64) -> Result<(), Failure> {
        // It is not possible to use infinite cylinder for topological operations.
        let a_object = self
            .builder
            .arguments()
            .first()
            .cloned()
            .unwrap_or_default();
        let (pmin, _) = box_parameters(&a_object, &self.axis)?;
        if pmin > length {
            self.status = Status::InvalidPlacement;
            return Ok(());
        }
        let heigth = 3. * (length - pmin) / 2.;
        let orig =
            self.axis.location().xyz() + self.axis.direction().xyz() * ((3. * pmin - length) / 2.);
        let cyl = cylinder(orig, &self.axis, radius, heigth)?;
        self.cut_by(cyl, false)?;
        let Some(parts) = self.parts()? else {
            return Ok(());
        };
        if parts.is_empty() {
            return Ok(());
        }
        // preserve the smallest as parameter along the axis
        let tokeep =
            smallest_within(&parts, first, f64::MAX).unwrap_or_else(|| closest_to(&parts, first));
        self.keep_one(&parts, &tokeep)
    }
}

/// The entry and exit of `PerformThruNext`: from the first franc point after the origin,
/// forward to the next exit, or back from an exit to its entry.
fn thru_next_points(asi: &CurveShapeIntersector) -> Result<Option<(PntFace, PntFace)>, Failure> {
    let mut w = Walk::new(asi);
    if !w.after(0.)? {
        return Ok(None);
    }
    if w.or == Orientation::Forward {
        let first = w.point(w.from)?;
        if !w.after_index(w.to)? || w.or != Orientation::Reversed {
            return Ok(None);
        }
        return Ok(Some((first, w.point(w.to)?)));
    }
    // TopAbs_REVERSED
    let last = w.point(w.to)?;
    if !w.before_index(w.from)? || w.or != Orientation::Forward {
        return Ok(None);
    }
    Ok(Some((w.point(w.from)?, last)))
}

/// The part of smallest barycentre parameter within `[first, last]` (the last one of
/// equal parameters), None without one.
fn smallest_within(parts: &[(Shape, f64)], first: f64, last: f64) -> Option<Shape> {
    let (mut parmin, mut tokeep) = (last, None);
    for (p, parbar) in parts {
        if *parbar >= first && *parbar <= last && *parbar <= parmin {
            parmin = *parbar;
            tokeep = Some(p.clone());
        }
    }
    tokeep
}

/// preserve the closest interval: the part before `first` nearest to it (natively the
/// parts after `last` are only examined in the branch of a part before, so never).
fn closest_outside(parts: &[(Shape, f64)], first: f64, last: f64) -> Shape {
    let (mut dmin, mut tokeep) = (f64::MAX, Shape::default());
    for (p, parbar) in parts {
        if *parbar < first {
            if first - parbar < dmin {
                dmin = first - parbar;
                tokeep = p.clone();
            } else if parbar - last < dmin {
                // parbar > Last
                dmin = parbar - last;
                tokeep = p.clone();
            }
        }
    }
    tokeep
}

/// preserve the closest interval: the part whose barycentre is nearest to `first`.
fn closest_to(parts: &[(Shape, f64)], first: f64) -> Shape {
    let (mut dmin, mut tokeep) = (f64::MAX, Shape::default());
    for (p, parbar) in parts {
        if (first - parbar).abs() < dmin {
            dmin = (first - parbar).abs();
            tokeep = p.clone();
        }
    }
    tokeep
}
