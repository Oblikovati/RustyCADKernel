//! Direct translation of GeomHash_MeshHasher_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::geom_hash::{
    GeomHashPolygon2DHasher, GeomHashPolygon3DHasher, GeomHashPolygonOnTriHasher,
    GeomHashTriangulationHasher, PolygonOnTriHashKey,
};
use rustycad_tkmath::gp::{Point2, Point3};
use rustycad_tkmath::poly::{
    Polygon2D, Polygon3D, PolygonOnTriangulation, Triangle, Triangulation,
};

fn makePolygon2D(theDeflection: f64) -> Handle<Polygon2D> {
    let aPoly = Handle::new(Polygon2D::new(2).unwrap());
    aPoly.change_nodes(|nodes| {
        nodes.set_value(1, Point2::new(0.0, 0.0)).unwrap();
        nodes.set_value(2, Point2::new(1.0, 0.0)).unwrap();
    });
    aPoly.set_deflection(theDeflection);
    aPoly
}

fn makePolygon3D(theDeflection: f64) -> Handle<Polygon3D> {
    let aPoly = Handle::new(Polygon3D::new(2, false).unwrap());
    aPoly.change_nodes(|nodes| {
        nodes.set_value(1, Point3::new(0.0, 0.0, 0.0)).unwrap();
        nodes.set_value(2, Point3::new(1.0, 0.0, 0.0)).unwrap();
    });
    aPoly.set_deflection(theDeflection);
    aPoly
}

fn makePolygonOnTriangulation(theDeflection: f64) -> Handle<PolygonOnTriangulation> {
    let aPoly = Handle::new(PolygonOnTriangulation::new(2, false).unwrap());
    aPoly.set_node(1, 1).unwrap();
    aPoly.set_node(2, 2).unwrap();
    aPoly.set_deflection(theDeflection);
    aPoly
}

fn makeTriangulation(theDeflection: f64) -> Handle<Triangulation> {
    let aTri = Handle::new(Triangulation::with_sizes(3, 1, false, false).unwrap());
    aTri.set_node(1, Point3::new(0.0, 0.0, 0.0)).unwrap();
    aTri.set_node(2, Point3::new(1.0, 0.0, 0.0)).unwrap();
    aTri.set_node(3, Point3::new(0.0, 1.0, 0.0)).unwrap();
    aTri.set_triangle(1, Triangle::new(1, 2, 3)).unwrap();
    aTri.set_deflection(theDeflection);
    aTri
}

// `PolygonOnTriHashKey{Poly, TriRepId}`.
fn key(poly: Handle<PolygonOnTriangulation>, tri_rep_id: u32) -> PolygonOnTriHashKey {
    PolygonOnTriHashKey { poly, tri_rep_id }
}

#[test]
fn GeomHash_MeshHasherTest__Polygon2D_CloseDeflectionKeepsEqualHash() {
    let aHasher = GeomHashPolygon2DHasher::new(0.1, 0.01);
    let aPoly1 = makePolygon2D(0.004);
    let aPoly2 = makePolygon2D(0.0044);

    assert!(aHasher.equal(&aPoly1, &aPoly2));
    assert_eq!(aHasher.hash(&aPoly1), aHasher.hash(&aPoly2));
}

#[test]
fn GeomHash_MeshHasherTest__Polygon3D_CloseDeflectionKeepsEqualHash() {
    let aHasher = GeomHashPolygon3DHasher::new(0.1, 0.01);
    let aPoly1 = makePolygon3D(0.004);
    let aPoly2 = makePolygon3D(0.0044);

    assert!(aHasher.equal(&aPoly1, &aPoly2));
    assert_eq!(aHasher.hash(&aPoly1), aHasher.hash(&aPoly2));
}

#[test]
fn GeomHash_MeshHasherTest__PolygonOnTriangulation_CloseDeflectionKeepsEqualHash() {
    let aHasher = GeomHashPolygonOnTriHasher::new(0.1, 0.01);
    let aKey1 = key(makePolygonOnTriangulation(0.004), 7);
    let aKey2 = key(makePolygonOnTriangulation(0.0044), 7);

    assert!(aHasher.equal(&aKey1, &aKey2));
    assert_eq!(aHasher.hash(&aKey1), aHasher.hash(&aKey2));
}

#[test]
fn GeomHash_MeshHasherTest__Triangulation_CloseDeflectionKeepsEqualHash() {
    let aHasher = GeomHashTriangulationHasher::new(0.1, 0.01);
    let aTri1 = makeTriangulation(0.004);
    let aTri2 = makeTriangulation(0.0044);

    assert!(aHasher.equal(&aTri1, &aTri2));
    assert_eq!(aHasher.hash(&aTri1), aHasher.hash(&aTri2));
}

#[test]
fn GeomHash_MeshHasherTest__Polygon2D_DifferentSameCountGeometryChangesHash() {
    let aHasher = GeomHashPolygon2DHasher::new(0.1, 0.01);
    let aPoly1 = makePolygon2D(0.004);
    let aPoly2 = makePolygon2D(0.004);
    aPoly2.change_nodes(|nodes| nodes.set_value(2, Point2::new(2.0, 0.0)).unwrap());

    assert!(!aHasher.equal(&aPoly1, &aPoly2));
    assert_ne!(aHasher.hash(&aPoly1), aHasher.hash(&aPoly2));
}

#[test]
fn GeomHash_MeshHasherTest__Polygon3D_DifferentSameCountGeometryChangesHash() {
    let aHasher = GeomHashPolygon3DHasher::new(0.1, 0.01);
    let aPoly1 = makePolygon3D(0.004);
    let aPoly2 = makePolygon3D(0.004);
    aPoly2.change_nodes(|nodes| nodes.set_value(2, Point3::new(2.0, 0.0, 0.0)).unwrap());

    assert!(!aHasher.equal(&aPoly1, &aPoly2));
    assert_ne!(aHasher.hash(&aPoly1), aHasher.hash(&aPoly2));
}

#[test]
fn GeomHash_MeshHasherTest__PolygonOnTriangulation_DifferentSameCountIndicesChangesHash() {
    let aHasher = GeomHashPolygonOnTriHasher::new(0.1, 0.01);
    let aKey1 = key(makePolygonOnTriangulation(0.004), 7);
    let aKey2 = key(makePolygonOnTriangulation(0.004), 7);
    aKey2.poly.set_node(2, 3).unwrap();

    assert!(!aHasher.equal(&aKey1, &aKey2));
    assert_ne!(aHasher.hash(&aKey1), aHasher.hash(&aKey2));
}

#[test]
fn GeomHash_MeshHasherTest__Triangulation_DifferentSameCountGeometryChangesHash() {
    let aHasher = GeomHashTriangulationHasher::new(0.1, 0.01);
    let aTri1 = makeTriangulation(0.004);
    let aTri2 = makeTriangulation(0.004);
    aTri2.set_node(3, Point3::new(0.0, 2.0, 0.0)).unwrap();

    assert!(!aHasher.equal(&aTri1, &aTri2));
    assert_ne!(aHasher.hash(&aTri1), aHasher.hash(&aTri2));
}

#[test]
fn GeomHash_MeshHasherTest__NullHandlesDoNotCrash() {
    let aPoly2dHasher = GeomHashPolygon2DHasher::default();
    let aPoly3dHasher = GeomHashPolygon3DHasher::default();
    let aPolyOnTriHasher = GeomHashPolygonOnTriHasher::default();
    let aTriHasher = GeomHashTriangulationHasher::default();
    let aNullPoly2d: Handle<Polygon2D> = Handle::default();
    let aNullPoly3d: Handle<Polygon3D> = Handle::default();
    let aNullPolyOnTri: Handle<PolygonOnTriangulation> = Handle::default();
    let aNullTri: Handle<Triangulation> = Handle::default();

    assert_eq!(0usize, aPoly2dHasher.hash(&aNullPoly2d));
    assert!(aPoly2dHasher.equal(&aNullPoly2d, &aNullPoly2d));
    assert!(!aPoly2dHasher.equal(&aNullPoly2d, &makePolygon2D(0.004)));

    assert_eq!(0usize, aPoly3dHasher.hash(&aNullPoly3d));
    assert!(aPoly3dHasher.equal(&aNullPoly3d, &aNullPoly3d));
    assert!(!aPoly3dHasher.equal(&aNullPoly3d, &makePolygon3D(0.004)));

    assert_eq!(0usize, aTriHasher.hash(&aNullTri));
    assert!(aTriHasher.equal(&aNullTri, &aNullTri));
    assert!(!aTriHasher.equal(&aNullTri, &makeTriangulation(0.004)));

    let aNullKey1 = key(aNullPolyOnTri.clone(), 7);
    let aNullKey2 = key(aNullPolyOnTri.clone(), 7);
    let aNullKey3 = key(aNullPolyOnTri, 8);
    assert!(aPolyOnTriHasher.equal(&aNullKey1, &aNullKey2));
    assert!(!aPolyOnTriHasher.equal(&aNullKey1, &aNullKey3));
    assert!(!aPolyOnTriHasher.equal(&aNullKey1, &key(makePolygonOnTriangulation(0.004), 7)));
}

#[test]
fn GeomHash_MeshHasherTest__HashToleranceAffectsNumericFields() {
    let aPoly = makePolygon2D(0.06);

    let aFineHasher = GeomHashPolygon2DHasher::new(0.1, 0.01);
    let aCoarseHasher = GeomHashPolygon2DHasher::new(0.1, 0.1);

    assert_ne!(aFineHasher.hash(&aPoly), aCoarseHasher.hash(&aPoly));
}
