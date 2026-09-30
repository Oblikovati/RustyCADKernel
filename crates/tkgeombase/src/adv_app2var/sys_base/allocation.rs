//! The allocator members of `AdvApp2Var_SysBase`: `mcrrqst_`, `mcrdelt_` and the
//! typed wrappers `macrai4_`, `macrar8_`, `macrdi4_`, `macrdr8_`.
//!
//! `reference` stands for the native address `t` of the caller's reference array:
//! with the unit, the size and the offset it identifies the allocation that
//! `mcrdelt_` releases, and its remainder modulo the unit pads the zone as the
//! address does natively.
use super::{AdvApp2VarSysBase, GUARD_WORD, HEAP_ALLOCATION, MAX_ALLOC_NB, Mitem};

/// `madbtbk_`: the debug mode of the Fortran runtime, always 0 (production), so the
/// guard checks `macrchk_` and the user-zone poisoning `maoverf_` never run.
const DEBUG_MODE: i32 = 0;

impl AdvApp2VarSysBase {
    /// Requests a zone of `isize` units of `iunit` bytes (`mcrrqst_`).
    ///
    /// `iercod`: 0 when done, 1 when `MAX_ALLOC_NB` zones are live, 2 for a unit
    /// other than 1, 2, 4 or 8, 3 when the system refuses the size (a negative
    /// byte count here). On success `iofset` receives the zone key; returns 0.
    /// `sys.mcrrqst_(8, 150, 0, &mut ofs, &mut ier)`.
    pub fn mcrrqst_(
        &mut self,
        iunit: i32,
        isize: i32,
        reference: usize,
        iofset: &mut isize,
        iercod: &mut i32,
    ) -> i32 {
        *iercod = self.request_code(iunit, isize, reference);
        if *iercod == 0 {
            *iofset = self.record_zone(iunit, isize, reference);
        }
        // L9900: the protection level only applies to one request.
        self.lprot = 0;
        0
    }

    /// Checks the request as the native `goto L9001/L9002/L9003` chain does and
    /// returns its error code.
    fn request_code(&self, iunit: i32, isize: i32, reference: usize) -> i32 {
        if self.icore.len() >= MAX_ALLOC_NB {
            return 1;
        }
        if ![1, 2, 4, 8].contains(&iunit) {
            return 2;
        }
        match zone_bytes(iunit, isize, reference) {
            Some(_) => 0,
            None => 3,
        }
    }

    /// Records a granted request and updates the statistics; returns its key.
    fn record_zone(&mut self, iunit: i32, isize: i32, reference: usize) -> isize {
        let izu: i32 = zone_bytes(iunit, isize, reference).unwrap_or(0);
        self.next_offset += 1;
        let item = Mitem {
            prot: self.lprot,
            unit: iunit as u8,
            reqsize: isize,
            loc: reference,
            offset: self.next_offset,
            alloctype: HEAP_ALLOCATION as u8,
            size: izu + 24,
            rank: self.icore.len() as i32 + 1,
            guards: [GUARD_WORD; 2],
            zone: vec![0; izu as usize],
        };
        let requested: i32 = iunit * item.reqsize;
        self.icore.push(item);
        self.nrqst[HEAP_ALLOCATION] += 1;
        self.nbyte[HEAP_ALLOCATION] += requested;
        self.mbyte[HEAP_ALLOCATION] = self.mbyte[HEAP_ALLOCATION].max(self.nbyte[HEAP_ALLOCATION]);
        self.next_offset
    }

    /// Releases the zone identified by the four request values (`mcrdelt_`).
    ///
    /// `iercod`: 0 when done, 3 when no live zone matches (the codes 1 and 2 of a
    /// static zone or a refused `free` cannot occur). On success `iofset` becomes
    /// the largest positive offset, as natively; returns 0.
    /// `sys.mcrdelt_(8, 150, 0, &mut ofs, &mut ier)`.
    pub fn mcrdelt_(
        &mut self,
        iunit: i32,
        isize: i32,
        reference: usize,
        iofset: &mut isize,
        iercod: &mut i32,
    ) -> i32 {
        *iercod = 0;
        let found: Option<usize> = self.icore.iter().rposition(|item| {
            i32::from(item.unit) == iunit
                && item.reqsize == isize
                && item.loc == reference
                && item.offset == *iofset
        });
        let Some(n) = found else {
            *iercod = 3;
            return 0;
        };
        if DEBUG_MODE == 1 {
            self.macrchk_();
        }
        let item: Mitem = self.icore.remove(n);
        let kind: usize = usize::from(item.alloctype);
        self.ndelt[kind] += 1;
        self.nbyte[kind] -= i32::from(item.unit) * item.reqsize;
        *iofset = isize::MAX;
        0
    }

    /// Checks the guard words of every live zone (`macrchk_`); a damaged guard is
    /// reported (`macrerr_`, a no-op) and disarmed. Unreachable in production mode.
    fn macrchk_(&mut self) {
        for item in &mut self.icore {
            for guard in &mut item.guards {
                // A NaN guard is a disarmed one (the native address set to -1).
                if !guard.is_nan() && *guard != GUARD_WORD {
                    *guard = f64::NAN;
                }
            }
        }
    }

    /// Requests `nbelem` integers beyond the caller's `maxelm` (`macrai4_`): no zone
    /// and `iofset = 0` when they fit; otherwise `mcrrqst_` with the size of `int`.
    /// `sys.macrai4_(200, 100, 0, &mut ofs, &mut ier)`.
    pub fn macrai4_(
        &mut self,
        nbelem: i32,
        maxelm: i32,
        reference: usize,
        iofset: &mut isize,
        iercod: &mut i32,
    ) -> i32 {
        self.request_beyond(4, nbelem, maxelm, reference, iofset, iercod)
    }

    /// Requests `nbelem` doubles beyond the caller's `maxelm` (`macrar8_`);
    /// `sys.macrar8_(200, 100, 0, &mut ofs, &mut ier)`.
    pub fn macrar8_(
        &mut self,
        nbelem: i32,
        maxelm: i32,
        reference: usize,
        iofset: &mut isize,
        iercod: &mut i32,
    ) -> i32 {
        self.request_beyond(8, nbelem, maxelm, reference, iofset, iercod)
    }

    fn request_beyond(
        &mut self,
        iunit: i32,
        nbelem: i32,
        maxelm: i32,
        reference: usize,
        iofset: &mut isize,
        iercod: &mut i32,
    ) -> i32 {
        if nbelem > maxelm {
            return self.mcrrqst_(iunit, nbelem, reference, iofset, iercod);
        }
        *iercod = 0;
        *iofset = 0;
        0
    }

    /// Releases an integer zone of `macrai4_` (`macrdi4_`); nothing when `iofset` is 0.
    /// `sys.macrdi4_(200, 100, 0, &mut ofs, &mut ier)`.
    pub fn macrdi4_(
        &mut self,
        nbelem: i32,
        _maxelm: i32,
        reference: usize,
        iofset: &mut isize,
        iercod: &mut i32,
    ) -> i32 {
        self.release_beyond(4, nbelem, reference, iofset, iercod)
    }

    /// Releases a double zone of `macrar8_` (`macrdr8_`); nothing when `iofset` is 0.
    /// `sys.macrdr8_(200, 100, 0, &mut ofs, &mut ier)`.
    pub fn macrdr8_(
        &mut self,
        nbelem: i32,
        _maxelm: i32,
        reference: usize,
        iofset: &mut isize,
        iercod: &mut i32,
    ) -> i32 {
        self.release_beyond(8, nbelem, reference, iofset, iercod)
    }

    fn release_beyond(
        &mut self,
        iunit: i32,
        nbelem: i32,
        reference: usize,
        iofset: &mut isize,
        iercod: &mut i32,
    ) -> i32 {
        if *iofset != 0 {
            return self.mcrdelt_(iunit, nbelem, reference, iofset, iercod);
        }
        *iercod = 0;
        0
    }
}

/// The user-zone size of `mcrrqst_`: the request plus the misalignment of the
/// reference, rounded up to 8 bytes; `None` when the system would refuse it.
fn zone_bytes(iunit: i32, isize: i32, reference: usize) -> Option<i32> {
    let misalignment: i32 = (reference % iunit as usize) as i32;
    let mut izu: i32 = iunit.checked_mul(isize)?.checked_add(misalignment)?;
    let irest: i32 = izu % 8;
    if irest != 0 {
        izu = izu + 8 - irest;
    }
    (izu >= 0).then_some(izu)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_request_is_recorded_until_released() {
        let mut sys = AdvApp2VarSysBase::new();
        let (mut ofs, mut ier) = (0isize, -1);
        sys.mcrrqst_(8, 3, 5, &mut ofs, &mut ier);
        assert_eq!((ier, sys.ncore(), sys.live_bytes()), (0, 1, 24));
        // The reference misaligned by 5 bytes pads the 24 requested bytes to 32.
        assert_eq!(sys.zone_mut(ofs).map(|zone| zone.len()), Some(32));
        sys.mcrdelt_(8, 3, 5, &mut ofs, &mut ier);
        assert_eq!((ier, sys.ncore(), ofs), (0, 0, isize::MAX));
    }

    #[test]
    fn invalid_units_full_tables_and_unknown_zones_are_refused() {
        let mut sys = AdvApp2VarSysBase::new();
        let (mut ofs, mut ier) = (0isize, 0);
        sys.mcrrqst_(3, 1, 0, &mut ofs, &mut ier);
        assert_eq!(ier, 2);
        for _ in 0..MAX_ALLOC_NB {
            sys.mcrrqst_(1, 1, 0, &mut ofs, &mut ier);
        }
        sys.mcrrqst_(1, 1, 0, &mut ofs, &mut ier);
        assert_eq!((ier, sys.ncore()), (1, MAX_ALLOC_NB));
        let mut unknown: isize = -7;
        sys.mcrdelt_(1, 1, 0, &mut unknown, &mut ier);
        assert_eq!(ier, 3);
        sys.mcrrqst_(8, -5, 0, &mut ofs, &mut ier);
        assert_eq!(ier, 1);
    }

    #[test]
    fn typed_wrappers_allocate_only_beyond_the_local_capacity() {
        let mut sys = AdvApp2VarSysBase::new();
        let (mut ofs, mut ier) = (9isize, 9);
        sys.macrar8_(50, 100, 0, &mut ofs, &mut ier);
        assert_eq!((ofs, ier, sys.ncore()), (0, 0, 0));
        sys.macrai4_(150, 100, 0, &mut ofs, &mut ier);
        assert_eq!((ier, sys.ncore()), (0, 1));
        sys.macrdi4_(150, 100, 0, &mut ofs, &mut ier);
        assert_eq!((ier, sys.ncore()), (0, 0));
        let mut none: isize = 0;
        sys.macrdr8_(150, 100, 0, &mut none, &mut ier);
        assert_eq!(ier, 0);
    }
}
