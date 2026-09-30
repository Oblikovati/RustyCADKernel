// Rust translation of the Insert members of ModelingAlgorithms/TKGeomAlgo/Intf/Intf_Interference.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Intf_Interference::Insert: merging a tangent zone into the known zones, and a
//! section segment into the section lines.
use super::interference::Interference;
use super::section_line::SectionLine;
use super::section_point::SectionPoint;
use super::sequence_access::{item, item_mut};
use super::tangent_zone::TangentZone;
use rustycad_tkernel::standard::Failure;

/// Where a new zone meets a known one: the zone, the stop points in it (`lunp`,
/// `lotp`), the points of the new zone (`lunl`, `lotl`) and the search direction.
#[derive(Clone, Copy, Debug, Default)]
struct ZoneJunction {
    zone: i32,
    lunp: i32,
    lotp: i32,
    lunl: i32,
    lotl: i32,
    same: bool,
}

/// The one-based index `((i - 1) % n) + 1` that walks the new zone cyclically.
fn cyclic(i: i32, n: i32) -> i32 {
    ((i - 1) % n) + 1
}

/// Where a section segment touches a section line: the line, whether the free point
/// goes at its beginning, and the free point.
struct LineJunction {
    line: i32,
    begin: bool,
    free_point: SectionPoint,
}

/// How a segment touches a line: whether the free point goes at the beginning, whether
/// the line closes, and whether the free point is `pfin`.
struct Touch {
    begin: bool,
    closes: bool,
    free_is_end: bool,
}

/// The native branches over the IsEnd results `nd` (of pdeb) and `nf` (of pfin).
fn classify_touch(nd: i32, nf: i32) -> Option<Touch> {
    let touch = |begin: bool, closes: bool, free_is_end: bool| Touch {
        begin,
        closes,
        free_is_end,
    };
    if nd == 1 {
        return Some(touch(true, nf > 1, true));
    }
    if nd > 1 {
        return Some(touch(false, nf == 1, true));
    }
    if nf == 1 {
        return Some(touch(true, false, false));
    }
    (nf > 1).then(|| touch(false, false, false))
}

impl Interference {
    /// Insert(TangentZone): merges `zone` into the first known zone that shares one of
    /// its points (an edge in either direction, else a single point), then re-inserts
    /// the grown zone recursively (appending it when that fails). Returns false, and
    /// changes nothing, when there are no zones or none shares a point;
    /// `interference.insert_zone(&zone)?`.
    pub fn insert_zone(&mut self, zone: &TangentZone) -> Result<bool, Failure> {
        if self.tangent_zones.is_empty() {
            return Ok(false);
        }
        let junction: ZoneJunction = self.find_junction(zone)?;
        if junction.lotp != 0 {
            self.splice_edge(zone, junction)?;
        } else if junction.lunp > 0 {
            self.splice_point(zone, junction)?;
        } else {
            return Ok(false);
        }
        let grown: TangentZone = item(&self.tangent_zones, junction.zone)?.clone();
        self.tangent_zones.remove(junction.zone as usize)?;
        if !self.insert_zone(&grown)? {
            self.tangent_zones.append(grown);
        }
        Ok(true)
    }

    /// The native search loops: stops at the first shared edge; a shared point alone
    /// is remembered (the last one found wins) and the search goes on.
    fn find_junction(&self, zone: &TangentZone) -> Result<ZoneJunction, Failure> {
        let mut junction = ZoneJunction::default();
        for iz in 1..=self.tangent_zones.length() {
            let known: &TangentZone = item(&self.tangent_zones, iz)?;
            for ipz1 in 1..=known.number_of_points() {
                self.match_edge(known, zone, iz, ipz1, &mut junction)?;
                if junction.lotp != 0 {
                    return Ok(junction);
                }
            }
        }
        Ok(junction)
    }

    /// The inner loop over the new zone's points for the known point `ipz1`.
    fn match_edge(
        &self,
        known: &TangentZone,
        zone: &TangentZone,
        iz: i32,
        ipz1: i32,
        junction: &mut ZoneJunction,
    ) -> Result<(), Failure> {
        let npcz: i32 = known.number_of_points();
        let ipz0: i32 = if ipz1 - 1 <= 0 { npcz } else { ipz1 - 1 };
        let ipz2: i32 = (ipz1 % npcz) + 1;
        let nplz: i32 = zone.number_of_points();
        for ilz1 in 1..=nplz {
            let ilz2: i32 = (ilz1 % nplz) + 1;
            if !known.get_point(ipz1)?.is_equal(zone.get_point(ilz1)?) {
                continue;
            }
            let (lunp, lotp, same) = if known.get_point(ipz0)?.is_equal(zone.get_point(ilz2)?) {
                (ipz0, ipz1, false)
            } else if known.get_point(ipz2)?.is_equal(zone.get_point(ilz2)?) {
                (ipz1, ipz2, true)
            } else {
                (junction.zone, junction.lunp, junction.lunl) = (iz, ipz1, ilz1);
                continue;
            };
            *junction = ZoneJunction {
                zone: iz,
                lunp,
                lotp,
                lunl: ilz1,
                lotl: ilz2,
                same,
            };
            return Ok(());
        }
        Ok(())
    }

    /// A shared edge: the other points of the new zone go before `lotp`, walking from
    /// `lotl + 1` back to `lunl`.
    fn splice_edge(&mut self, zone: &TangentZone, junction: ZoneJunction) -> Result<(), Failure> {
        let nplz: i32 = zone.number_of_points();
        let mut lotp: i32 = junction.lotp;
        let mut ilc: i32 = junction.lotl + 1;
        while cyclic(ilc, nplz) != junction.lunl {
            let point: SectionPoint = *zone.get_point(cyclic(ilc, nplz))?;
            item_mut(&mut self.tangent_zones, junction.zone)?.insert_before(lotp, point)?;
            if !junction.same {
                lotp += 1;
            }
            ilc += 1;
        }
        Ok(())
    }

    /// A shared point: every point of the new zone, from `lunl` around to `lunl`
    /// again (so that point twice), goes before `lunp`.
    fn splice_point(&mut self, zone: &TangentZone, junction: ZoneJunction) -> Result<(), Failure> {
        let nplz: i32 = zone.number_of_points();
        let mut lunp: i32 = junction.lunp;
        let mut ilc: i32 = junction.lunl;
        let mut looped = false;
        loop {
            let point: SectionPoint = *zone.get_point(cyclic(ilc, nplz))?;
            item_mut(&mut self.tangent_zones, junction.zone)?.insert_before(lunp, point)?;
            lunp += 1;
            if looped && cyclic(ilc, nplz) == junction.lunl {
                return Ok(());
            }
            looped = true;
            ilc += 1;
        }
    }

    /// Insert(pdeb, pfin): adds the section segment `pdeb pfin` to the section lines.
    /// A segment touching the end of a line extends it (closing it when both ends
    /// touch), and joins it with a second line that its free point touches; otherwise
    /// it starts a new line. Raises Standard_NoSuchObject for an empty section line;
    /// `interference.insert_points(&pdeb, &pfin)?`.
    pub fn insert_points(
        &mut self,
        pdeb: &SectionPoint,
        pfin: &SectionPoint,
    ) -> Result<(), Failure> {
        let Some(junction) = self.touched_line(pdeb, pfin)? else {
            let mut line = SectionLine::new();
            line.append(*pdeb);
            line.append(*pfin);
            self.section_lines.append(line);
            return Ok(());
        };
        if self.join_lines(&junction)? {
            return self.section_lines.remove(junction.line as usize);
        }
        let line: &mut SectionLine = item_mut(&mut self.section_lines, junction.line)?;
        if junction.begin {
            line.prepend(junction.free_point);
        } else {
            line.append(junction.free_point);
        }
        Ok(())
    }

    /// The first line whose end is `pdeb` or `pfin`; closes it when the segment
    /// touches both of its ends through `pdeb`, as natively.
    fn touched_line(
        &mut self,
        pdeb: &SectionPoint,
        pfin: &SectionPoint,
    ) -> Result<Option<LineJunction>, Failure> {
        for ils in 1..=self.section_lines.length() {
            let line: &mut SectionLine = item_mut(&mut self.section_lines, ils)?;
            let (nd, nf) = (line.is_end(pdeb)?, line.is_end(pfin)?);
            let Some(touch) = classify_touch(nd, nf) else {
                continue;
            };
            if touch.closes {
                line.close();
            }
            let free_point: SectionPoint = if touch.free_is_end { *pfin } else { *pdeb };
            let begin: bool = touch.begin;
            return Ok(Some(LineJunction {
                line: ils,
                begin,
                free_point,
            }));
        }
        Ok(None)
    }

    /// Joins the touched line to another line that the free point ends, reversing it
    /// as needed; returns whether it did (the touched line is then empty).
    fn join_lines(&mut self, junction: &LineJunction) -> Result<bool, Failure> {
        for ils in 1..=self.section_lines.length() {
            if ils == junction.line {
                continue;
            }
            let nd: i32 = item(&self.section_lines, ils)?.is_end(&junction.free_point)?;
            if nd == 0 {
                continue;
            }
            let mut moved: SectionLine =
                std::mem::take(item_mut(&mut self.section_lines, junction.line)?);
            // An end at the beginning takes the line before it, reversed when the free
            // point was at the line's beginning; an end at the end takes it after.
            if junction.begin == (nd == 1) {
                moved.reverse();
            }
            let other: &mut SectionLine = item_mut(&mut self.section_lines, ils)?;
            if nd == 1 {
                other.prepend_line(&mut moved);
            } else {
                other.append_line(&mut moved);
            }
            return Ok(true);
        }
        Ok(false)
    }
}
