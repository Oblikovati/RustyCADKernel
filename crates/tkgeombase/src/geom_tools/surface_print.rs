// Created on: 1993-07-19
// Created by: Remi LEQUETTE
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
// Rust translation of GeomTools_SurfaceSet::PrintSurface and its static `Print`
// overloads for the elementary, swept, trimmed and offset surfaces
// (GeomTools_SurfaceSet.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The text of a `Geom_Surface`: the dump layout and the compact layout of the BRep
//! ASCII `Surfaces` section, with the type codes 1 (plane) to 11 (offset surface).
use super::curve_print::print_curve;
use super::curve_text::print_conic_head;
use super::surface_print_spline::{print_bezier_surface, print_bspline_surface};
use super::text_out::{TextOut, direction3, point3};
use super::type_codes::surface as code;
use super::{null_geometry, undefined_type_handler};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkernel::storage::IoStream;
use rustycad_tkg2d::geom2d_hash::class_dispatch::exact_class;
use rustycad_tkg3d::geom::{
    GeomBSplineSurface, GeomBezierSurface, GeomConicalSurface, GeomCylindricalSurface,
    GeomOffsetSurface, GeomPlane, GeomRectangularTrimmedSurface, GeomSphericalSurface, GeomSurface,
    GeomSurfaceOfLinearExtrusion, GeomSurfaceOfRevolution, GeomSweptSurface, GeomToroidalSurface,
};
use rustycad_tkmath::gp::{Axis1, Point3};

/// `GeomTools_SurfaceSet::PrintSurface`: dispatches on the exact class; any other
/// class goes to the active undefined-type handler. A null handle is
/// Standard_NullObject.
pub(crate) fn print_surface(
    surface: &Handle<dyn GeomSurface>,
    stream: &mut IoStream,
    compact: bool,
) -> Result<(), Failure> {
    if surface.is_null() {
        return Err(null_geometry(
            "GeomTools_SurfaceSet::PrintSurface",
            "Geom_Surface",
        ));
    }
    let handled = {
        let mut out = TextOut::new(stream, compact);
        print_elementary(surface, &mut out)?
            || print_swept(surface, &mut out)?
            || print_derived(surface, &mut out)?
    };
    if handled {
        return Ok(());
    }
    undefined_type_handler().print_surface(surface, stream, compact)
}

/// The plane, cylinder and cone, then the sphere and torus; false for another class.
fn print_elementary(
    surface: &Handle<dyn GeomSurface>,
    out: &mut TextOut<'_>,
) -> Result<bool, Failure> {
    if let Some(plane) = exact_class::<GeomPlane, _>(surface) {
        let pln = plane.pln();
        print_frame(out, &PLANE_HEAD, pln.location(), pln.axes());
    } else if let Some(cylinder) = exact_class::<GeomCylindricalSurface, _>(surface) {
        let cyl = cylinder.cylinder()?;
        print_frame(out, &CYLINDER_HEAD, cyl.location(), cyl.axes());
        out.label(b"\n  Radius :");
        out.real(cyl.radius());
    } else if let Some(cone) = exact_class::<GeomConicalSurface, _>(surface) {
        let cone = cone.cone()?;
        print_frame(out, &CONE_HEAD, cone.location(), cone.axes());
        out.label(b"\n  Radius :");
        out.real(cone.ref_radius());
        out.text(b"\n");
        out.label(b"\n  Angle :");
        out.real(cone.semi_angle());
    } else {
        return print_round(surface, out);
    }
    out.end();
    Ok(true)
}

/// The sphere and the torus; false for another class.
fn print_round(surface: &Handle<dyn GeomSurface>, out: &mut TextOut<'_>) -> Result<bool, Failure> {
    if let Some(sphere) = exact_class::<GeomSphericalSurface, _>(surface) {
        let sphere = sphere.sphere()?;
        print_frame(out, &SPHERE_HEAD, sphere.location(), sphere.axes());
        out.label(b"\n  Radius :");
        out.real(sphere.radius());
    } else if let Some(torus) = exact_class::<GeomToroidalSurface, _>(surface) {
        let torus = torus.torus()?;
        print_frame(out, &TORUS_HEAD, torus.location(), torus.axes());
        // The dump separates the torus radii by a space only, without a comma.
        out.label(b"\n  Radii  :");
        out.real(torus.major_radius());
        out.text(b" ");
        out.real(torus.minor_radius());
    } else {
        return Ok(false);
    }
    out.end();
    Ok(true)
}

/// The fixed part of an elementary surface record: type code, dump name and the dump
/// label of the location (`Center` for the sphere, `Origin` for the others).
struct RecordHead {
    code: i32,
    name: &'static [u8],
    location: &'static [u8],
}

const ORIGIN: &[u8] = b"\n  Origin :";
const PLANE_HEAD: RecordHead = RecordHead {
    code: code::PLANE,
    name: b"Plane",
    location: ORIGIN,
};
const CYLINDER_HEAD: RecordHead = RecordHead {
    code: code::CYLINDER,
    name: b"CylindricalSurface",
    location: ORIGIN,
};
const CONE_HEAD: RecordHead = RecordHead {
    code: code::CONE,
    name: b"ConicalSurface",
    location: ORIGIN,
};
const SPHERE_HEAD: RecordHead = RecordHead {
    code: code::SPHERE,
    name: b"SphericalSurface",
    location: b"\n  Center :",
};
const TORUS_HEAD: RecordHead = RecordHead {
    code: code::TORUS,
    name: b"ToroidalSurface",
    location: ORIGIN,
};

/// The main axis and the X and Y axes of a gp elementary surface. The sphere's main
/// axis is `Position().Axis()` natively, the same axis.
trait PlacementAxes {
    fn axes(&self) -> [Axis1; 3];
}

macro_rules! placement_axes {
    ($($surface:ty),+) => {
        $(impl PlacementAxes for $surface {
            fn axes(&self) -> [Axis1; 3] {
                [self.axis(), self.x_axis(), self.y_axis()]
            }
        })+
    };
}
placement_axes!(
    rustycad_tkmath::gp::Plane3,
    rustycad_tkmath::gp::Cylinder3,
    rustycad_tkmath::gp::Cone3,
    rustycad_tkmath::gp::Torus3
);

impl PlacementAxes for rustycad_tkmath::gp::Sphere3 {
    fn axes(&self) -> [Axis1; 3] {
        [self.position().axis(), self.x_axis(), self.y_axis()]
    }
}

/// The head of an elementary surface record: the location, then the three axis
/// directions.
fn print_frame(out: &mut TextOut<'_>, head: &RecordHead, location: Point3, axes: [Axis1; 3]) {
    let [main, x, y] = axes.map(|axis| direction3(axis.direction()));
    let fields: [(&[u8], &[f64]); 4] = [
        (head.location, &point3(location)),
        (b"\n  Axis   :", &main),
        (b"\n  XAxis  :", &x),
        (b"\n  YAxis  :", &y),
    ];
    print_conic_head(out, head.code, head.name, &fields);
}

/// The surfaces of linear extrusion and of revolution; false for another class.
fn print_swept(surface: &Handle<dyn GeomSurface>, out: &mut TextOut<'_>) -> Result<bool, Failure> {
    let basis = if let Some(extrusion) = exact_class::<GeomSurfaceOfLinearExtrusion, _>(surface) {
        out.tag(code::LINEAR_EXTRUSION, b"SurfaceOfLinearExtrusion");
        out.field(b"\n  Direction :", &direction3(extrusion.direction()));
        extrusion.basis_curve()
    } else if let Some(revolution) = exact_class::<GeomSurfaceOfRevolution, _>(surface) {
        out.tag(code::REVOLUTION, b"SurfaceOfRevolution");
        out.field(b"\n  Origin    :", &point3(revolution.location()));
        out.field(b"\n  Direction :", &direction3(revolution.direction()));
        revolution.basis_curve()
    } else {
        return Ok(false);
    };
    out.label(b"\n  Basis curve : ");
    out.text(b"\n");
    let compact = out.compact();
    print_curve(&basis, out.stream(), compact)?;
    Ok(true)
}

/// The Bezier and B-spline surfaces and the trimmed and offset surfaces; false for
/// another class.
fn print_derived(
    surface: &Handle<dyn GeomSurface>,
    out: &mut TextOut<'_>,
) -> Result<bool, Failure> {
    if let Some(bezier) = exact_class::<GeomBezierSurface, _>(surface) {
        print_bezier_surface(&bezier, out)?;
        return Ok(true);
    }
    if let Some(bspline) = exact_class::<GeomBSplineSurface, _>(surface) {
        print_bspline_surface(&bspline, out)?;
        return Ok(true);
    }
    let basis = if let Some(trimmed) = exact_class::<GeomRectangularTrimmedSurface, _>(surface) {
        print_trimmed_head(&trimmed, out);
        trimmed.basis_surface()
    } else if let Some(offset) = exact_class::<GeomOffsetSurface, _>(surface) {
        out.tag(code::OFFSET, b"OffsetSurface");
        out.label(b"\nOffset : ");
        out.real(offset.offset());
        out.text(b"\n");
        offset.basis_surface()
    } else {
        return Ok(false);
    };
    out.label(b"BasisSurface :\n");
    let compact = out.compact();
    print_surface(&basis, out.stream(), compact)?;
    Ok(true)
}

/// The tag and the `U1 U2 V1 V2` bounds of a rectangular trimmed surface.
fn print_trimmed_head(trimmed: &Handle<GeomRectangularTrimmedSurface>, out: &mut TextOut<'_>) {
    let (u1, u2, v1, v2) = trimmed.bounds();
    out.tag(code::RECTANGULAR, b"RectangularTrimmedSurface");
    out.label(b"\nParameters : ");
    for (index, value) in [u1, u2, v1, v2].into_iter().enumerate() {
        if index > 0 {
            out.text(b" ");
        }
        out.real(value);
    }
    out.text(b"\n");
}
