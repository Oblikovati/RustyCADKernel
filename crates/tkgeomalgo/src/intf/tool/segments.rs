// Created on: 1993-06-23
// Created by: Didier PIFFAULT
// Copyright (c) 1993-1999 Matra Datavision
// Copyright (c) 1999-2014 OPEN CASCADE SAS
//
// This file is part of Open CASCADE Technology software library.
//
// This library is free software; you can redistribute it and/or modify it under
// the terms of the GNU Lesser General Public License version 2.1 as published
// by the Free Software Foundation, with special exception defined in the file
// OCCT_LGPL_EXCEPTION.txt. Consult the file LICENSE_LGPL_21.txt included in OCCT
// distribution for complete text of the license and disclaimer of any warranty.
//
// Alternatively, this file may be used under the terms of Open CASCADE
// commercial license or contractual agreement.
// Rust translation of the segment loops of Intf_Tool::Hypr2dBox, Parab2dBox, HyprBox
// and ParabBox (ModelingAlgorithms/TKGeomAlgo/Intf/Intf_Tool.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The loops that turn the sorted border crossings of a conic into parameter
//! segments, sampling the curve into the result box on each segment.
use super::Tool;
use crate::intf::native_int_cast;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;

/// The capacity of the native beginOnCurve/endOnCurve arrays.
const MAX_SEGMENTS: i32 = 6;

impl Tool {
    /// The loop shared by Hypr2dBox, Parab2dBox and ParabBox: an entering crossing
    /// (positive sine) opens a segment, a leaving one closes it (opening one at
    /// -`Precision::Infinite` first when none is open) and samples the curve at tenths
    /// of the parameter; at most six segments, as the fixed arrays hold. `sines` holds
    /// the crossing sine of each sorted point, None keeping the previous value
    /// (natively a border code outside the switch keeps it).
    pub(super) fn sweep_segments(&mut self, sines: &[Option<f64>], mut sample: impl FnMut(f64)) {
        let (mut sinan, mut out) = (0.0_f64, true);
        for (npi, sine) in sines.iter().enumerate() {
            sinan = sine.unwrap_or(sinan);
            // The native positive test, so that a NaN sine is skipped.
            let significant: bool = sinan.abs() > precision::angular();
            if !significant {
                continue;
            }
            if sinan > 0.0 {
                if self.nb_seg < MAX_SEGMENTS {
                    out = false;
                    self.begin_on_curve[self.nb_seg as usize] = self.parint[npi];
                    self.nb_seg += 1;
                }
                continue;
            }
            self.close_segment(npi, out);
            out = true;
            self.sample_last_segment(&mut sample);
        }
        if !out && self.nb_seg > 0 {
            self.end_on_curve[(self.nb_seg - 1) as usize] = precision::infinite();
        }
    }

    fn close_segment(&mut self, npi: usize, out: bool) {
        if out && self.nb_seg < MAX_SEGMENTS {
            self.begin_on_curve[self.nb_seg as usize] = -precision::infinite();
            self.nb_seg += 1;
        }
        if self.nb_seg > 0 {
            self.end_on_curve[(self.nb_seg - 1) as usize] = self.parint[npi];
        }
    }

    /// Samples the last segment, clipped to [-10, 10], at the tenths strictly inside
    /// its integer truncation, stepping by 1 inside [-10, 10] and by 10 beyond it.
    fn sample_last_segment(&self, sample: &mut impl FnMut(f64)) {
        let last: usize = (self.nb_seg - 1) as usize;
        let begin: f64 = self.begin_on_curve[last];
        let end: f64 = self.end_on_curve[last];
        let ipmin: i32 = if begin < -10.0 {
            -10
        } else {
            native_int_cast(begin)
        };
        let ipmax: i32 = if end > 10.0 { 10 } else { native_int_cast(end) };
        let (mut ip, ipmax) = (
            ipmin.wrapping_mul(10).wrapping_add(1),
            ipmax.wrapping_mul(10).wrapping_sub(1),
        );
        while ip <= ipmax {
            sample(f64::from(ip) / 10.0);
            ip += if ip.abs() <= 10 { 1 } else { 10 };
        }
    }

    /// The HyprBox loop (NIZNHY-PKV, 11 Jul 2008; jgv, 10 Nov 2009): an entering
    /// crossing opens a segment ending at 10, a leaving one closes it (opening one at
    /// -10 first) and samples the curve at the tenths of the segment clipped to
    /// [-10, 10], in doubles. Natively the loop never bounds the segment count and
    /// writes past the six-entry arrays; the port raises Standard_OutOfRange instead.
    pub(super) fn sweep_hyperbola_segments(
        &mut self,
        sines: &[Option<f64>],
        mut sample: impl FnMut(f64),
    ) -> Result<(), Failure> {
        let (mut sinan, mut out) = (0.0_f64, true);
        for (npi, sine) in sines.iter().enumerate() {
            sinan = sine.unwrap_or(sinan);
            // The native positive test, so that a NaN sine is skipped.
            let significant: bool = sinan.abs() > precision::angular();
            if !significant {
                continue;
            }
            if sinan > 0.0 {
                out = false;
                let slot: usize = self.next_segment_slot()?;
                self.begin_on_curve[slot] = self.parint[npi];
                self.end_on_curve[slot] = 10.0;
                continue;
            }
            if out {
                let slot: usize = self.next_segment_slot()?;
                self.begin_on_curve[slot] = -10.0;
            }
            let last: usize = (self.nb_seg - 1) as usize;
            self.end_on_curve[last] = self.parint[npi];
            out = true;
            self.sample_hyperbola_segment(last, &mut sample);
        }
        Ok(())
    }

    fn next_segment_slot(&mut self) -> Result<usize, Failure> {
        if self.nb_seg >= MAX_SEGMENTS {
            return Err(Failure::out_of_range(&format!(
                "Intf_Tool::HyprBox: segment {}; expected at most {MAX_SEGMENTS}",
                self.nb_seg + 1
            )));
        }
        self.nb_seg += 1;
        Ok((self.nb_seg - 1) as usize)
    }

    fn sample_hyperbola_segment(&self, last: usize, sample: &mut impl FnMut(f64)) {
        let mut ipmin: f64 = -10.0;
        if self.begin_on_curve[last] > ipmin {
            ipmin = self.begin_on_curve[last];
        }
        let mut ipmax: f64 = 10.0;
        if self.end_on_curve[last] < ipmax {
            ipmax = self.end_on_curve[last];
        }
        let (mut ip, ipmax) = (ipmin * 10.0 + 1.0, ipmax * 10.0 - 1.0);
        while ip <= ipmax {
            sample(ip / 10.0);
            ip += if ip.abs() <= 10.0 { 1.0 } else { 10.0 };
        }
    }
}
