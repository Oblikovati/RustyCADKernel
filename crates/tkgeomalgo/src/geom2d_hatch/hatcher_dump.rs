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
// Rust translation of Geom2dHatch_Hatcher::Dump at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The dump of a hatcher.
use super::Hatcher;
use crate::hatch_gen::IntersectionPointDump;
use crate::hatch_gen::text::stream_real;
use rustycad_tkg3d::top_abs::Orientation;
use std::fmt;

const RULE: &str = "========================================================";

// A title between two rules, then an empty line.
fn banner(out: &mut dyn fmt::Write, title: &str) -> fmt::Result {
    writeln!(out, "{RULE}\n{title}\n{RULE}\n")
}

impl Hatcher {
    /// Dump(): writes the native `std::cout` text of the hatcher (its settings, the
    /// points of every hatching and the orientation of every element) to `out`;
    /// `hatcher.dump(&mut text)?`.
    pub fn dump(&self, out: &mut dyn fmt::Write) -> fmt::Result {
        writeln!(out)?;
        banner(
            out,
            "=== Dump of the hatcher ================================",
        )?;
        self.dump_settings(out)?;
        writeln!(out)?;
        banner(
            out,
            "=== Hatchings ==========================================",
        )?;
        self.dump_hatchings(out)?;
        writeln!(out)?;
        banner(
            out,
            "=== Elements ===========================================",
        )?;
        self.dump_elements(out)?;
        writeln!(out)
    }

    fn dump_settings(&self, out: &mut dyn fmt::Write) -> fmt::Result {
        let keep = |flag: bool| if flag { "    " } else { "not " };
        let plural = |count: i32| if count == 1 { "" } else { "s" };
        let (points, segments) = (keep(self.my_keep_points), keep(self.my_keep_segments));
        writeln!(out, "The points   are {points}considered.")?;
        writeln!(out, "The segments are {segments}considered.")?;
        let (conf2d, conf3d) = (
            stream_real(self.my_confusion2d),
            stream_real(self.my_confusion3d),
        );
        writeln!(out, "2D Confusion tolerance : {conf2d}")?;
        writeln!(out, "3D Confusion tolerance : {conf3d}")?;
        let (nb_h, nb_e) = (self.my_nb_hatchings, self.my_nb_elements);
        writeln!(out, "{nb_h} hatching{}", plural(nb_h))?;
        writeln!(out, "{nb_e} element{}", plural(nb_e))
    }

    fn dump_hatchings(&self, out: &mut dyn fmt::Write) -> fmt::Result {
        for ind_h in 1..=self.my_nb_hatchings {
            write!(out, "Hatching # {ind_h}")?;
            let Some(hatching) = self.my_hatchings.seek(&ind_h) else {
                writeln!(out, " is not bound")?;
                continue;
            };
            writeln!(
                out,
                " contains {} restriction points :",
                hatching.nb_points()
            )?;
            for (slot, point) in hatching.points().iter().enumerate() {
                point.dump(slot as i32 + 1, out)?;
            }
            writeln!(out, "----------------------------------------------")?;
        }
        Ok(())
    }

    fn dump_elements(&self, out: &mut dyn fmt::Write) -> fmt::Result {
        for ind_e in 1..=self.my_nb_elements {
            write!(out, "Element # {ind_e}")?;
            let Ok(element) = self.my_elements.find(ind_e) else {
                writeln!(out, " is not bound")?;
                continue;
            };
            let orientation: &str = match element.orientation() {
                Orientation::Forward => "FORWARD",
                Orientation::Reversed => "REVERSED",
                Orientation::Internal => "INTERNAL",
                Orientation::External => "EXTERNAL",
            };
            writeln!(out, " is {orientation}")?;
        }
        Ok(())
    }
}
