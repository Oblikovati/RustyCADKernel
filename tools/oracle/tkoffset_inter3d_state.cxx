#include <BRepAlgo_AsDes.hxx>
#include <BRepAlgo_Image.hxx>
#include <BRep_Tool.hxx>
#include <BRepBuilderAPI_MakeFace.hxx>
#include <BRepBuilderAPI_MakePolygon.hxx>
#include <BRepCheck_Analyzer.hxx>
#include <BRepClass_FaceClassifier.hxx>
#include <BRepOffset_Inter3d.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Edge.hxx>
#include <TopoDS_Face.hxx>
#include <TopoDS_Wire.hxx>
#include <gp_Pln.hxx>
#include <iostream>

static TopoDS_Face makeFace(const gp_Pln& plane)
{
  return BRepBuilderAPI_MakeFace(plane).Face();
}

static TopoDS_Face makeRectangularFace(const gp_Pln& plane,
                                      double umin,
                                      double umax,
                                      double vmin,
                                      double vmax)
{
  return BRepBuilderAPI_MakeFace(plane, umin, umax, vmin, vmax).Face();
}

static TopoDS_Face makeBoundedFace(const gp_Pln& plane)
{
  return makeRectangularFace(plane, -1.0, 1.0, -1.0, 1.0);
}

static TopoDS_Face makeTriangleFace(double z)
{
  gp_Pln plane(gp_Pnt(0, 0, z), gp_Dir(0, 0, 1));
  TopoDS_Wire wire = BRepBuilderAPI_MakePolygon(gp_Pnt(-2, -1, z),
                                                gp_Pnt(2, -1, z),
                                                gp_Pnt(0, 1, z),
                                                true)
                       .Wire();
  return BRepBuilderAPI_MakeFace(plane, wire, true).Face();
}

static TopoDS_Face makeFaceWithHole(double z)
{
  gp_Pln plane(gp_Pnt(0, 0, z), gp_Dir(0, 0, 1));
  TopoDS_Wire outer = BRepBuilderAPI_MakePolygon(gp_Pnt(-3, -2, z),
                                                 gp_Pnt(3, -2, z),
                                                 gp_Pnt(3, 2, z),
                                                 gp_Pnt(-3, 2, z),
                                                 true)
                        .Wire();
  BRepBuilderAPI_MakeFace face(plane, outer, true);
  TopoDS_Wire hole = BRepBuilderAPI_MakePolygon(gp_Pnt(-1, -1, z),
                                                gp_Pnt(1, -1, z),
                                                gp_Pnt(1, 1, z),
                                                gp_Pnt(-1, 1, z),
                                                true)
                       .Wire();
  face.Add(TopoDS::Wire(hole.Reversed()));
  return face.Face();
}

int main()
{
  TopoDS_Face f1 = makeFace(gp_Pln(gp_Pnt(0, 0, 0), gp_Dir(0, 0, 1)));
  TopoDS_Face f2 = makeFace(gp_Pln(gp_Pnt(0, 0, 0), gp_Dir(0, 1, 0)));
  TopoDS_Face source1 = makeFace(gp_Pln(gp_Pnt(0, 0, 2), gp_Dir(0, 0, 1)));
  TopoDS_Face source2 = makeFace(gp_Pln(gp_Pnt(0, 2, 0), gp_Dir(0, 1, 0)));
  occ::handle<BRepAlgo_AsDes> asDes = new BRepAlgo_AsDes();
  BRepOffset_Inter3d intersections(asDes, TopAbs_IN, 1.0e-7);

  std::cout << "before=" << intersections.IsDone(f1, f2) << '\n';
  BRepAlgo_Image images;
  images.Bind(source1, f1);
  images.Bind(source2, f2);
  intersections.FaceInter(f1, f2, images);
  std::cout << "after=" << intersections.IsDone(f1, f2) << ','
            << intersections.IsDone(f2, f1) << '\n';
  std::cout << "touched=" << intersections.TouchedFaces().Extent()
            << " new_edges=" << intersections.NewEdges().Extent()
            << " descendants=" << asDes->Descendant(f1).Extent() << ','
            << asDes->Descendant(f2).Extent() << '\n';
  const TopoDS_Edge& section = TopoDS::Edge(intersections.NewEdges().FindKey(1));
  TopLoc_Location location;
  double first, last;
  occ::handle<Geom_Curve> curve = BRep_Tool::Curve(section, location, first, last);
  gp_Pnt start = curve->Value(first), finish = curve->Value(last);
  std::cout << "section=" << first << ',' << last << ' '
            << start.X() << ',' << start.Y() << ',' << start.Z() << ' '
            << finish.X() << ',' << finish.Y() << ',' << finish.Z() << ' '
            << static_cast<int>(section.Orientation()) << ','
            << static_cast<int>(TopoDS::Edge(asDes->Descendant(f2).First()).Orientation())
            << '\n';

  TopoDS_Face f3 = makeFace(gp_Pln(gp_Pnt(0, 0, 4), gp_Dir(0, 0, 1)));
  intersections.SetDone(f1, f3);
  intersections.SetDone(f1, f3);
  std::cout << "explicit=" << intersections.IsDone(f1, f3) << ','
            << intersections.IsDone(f3, f1) << '\n';

  TopoDS_Face bf1 = makeBoundedFace(gp_Pln(gp_Pnt(0, 0, 0), gp_Dir(0, 0, 1)));
  TopoDS_Face bf2 = makeBoundedFace(gp_Pln(gp_Pnt(0, 0, 0), gp_Dir(0, 1, 0)));
  TopoDS_Face bs1 = makeBoundedFace(gp_Pln(gp_Pnt(0, 0, 2), gp_Dir(0, 0, 1)));
  TopoDS_Face bs2 = makeBoundedFace(gp_Pln(gp_Pnt(0, 2, 0), gp_Dir(0, 1, 0)));
  occ::handle<BRepAlgo_AsDes> boundedAsDes = new BRepAlgo_AsDes();
  BRepOffset_Inter3d boundedIntersections(boundedAsDes, TopAbs_IN, 1.0e-7);
  BRepAlgo_Image boundedImages;
  boundedImages.Bind(bs1, bf1);
  boundedImages.Bind(bs2, bf2);
  boundedIntersections.FaceInter(bf1, bf2, boundedImages);
  std::cout << "bounded=" << boundedIntersections.IsDone(bf1, bf2) << ','
            << boundedIntersections.TouchedFaces().Extent() << ','
            << boundedIntersections.NewEdges().Extent() << ','
            << boundedAsDes->Descendant(bf1).Extent() << ','
            << boundedAsDes->Descendant(bf2).Extent() << '\n';
  if (!boundedIntersections.NewEdges().IsEmpty())
  {
    const TopoDS_Edge& boundedEdge = TopoDS::Edge(boundedIntersections.NewEdges().FindKey(1));
    TopLoc_Location boundedLocation;
    double boundedFirst, boundedLast;
    occ::handle<Geom_Curve> boundedCurve =
      BRep_Tool::Curve(boundedEdge, boundedLocation, boundedFirst, boundedLast);
    const gp_Pnt boundedStart = boundedCurve->Value(boundedFirst);
    const gp_Pnt boundedFinish = boundedCurve->Value(boundedLast);
    std::cout << "bounded_section=" << boundedFirst << ',' << boundedLast << ' '
              << boundedStart.X() << ',' << boundedStart.Y() << ',' << boundedStart.Z() << ' '
              << boundedFinish.X() << ',' << boundedFinish.Y() << ',' << boundedFinish.Z() << ' '
              << static_cast<int>(boundedEdge.Orientation()) << ','
              << static_cast<int>(TopoDS::Edge(boundedAsDes->Descendant(bf2).First()).Orientation())
              << '\n';
  }

  TopoDS_Face tf1 = makeTriangleFace(0.0);
  TopoDS_Face tf2 = makeBoundedFace(gp_Pln(gp_Pnt(0, 0, 0), gp_Dir(0, 1, 0)));
  TopoDS_Face ts1 = makeTriangleFace(2.0);
  TopoDS_Face ts2 = makeBoundedFace(gp_Pln(gp_Pnt(0, 2, 0), gp_Dir(0, 1, 0)));
  occ::handle<BRepAlgo_AsDes> triangleAsDes = new BRepAlgo_AsDes();
  BRepOffset_Inter3d triangleIntersections(triangleAsDes, TopAbs_IN, 1.0e-7);
  BRepAlgo_Image triangleImages;
  triangleImages.Bind(ts1, tf1);
  triangleImages.Bind(ts2, tf2);
  triangleIntersections.FaceInter(tf1, tf2, triangleImages);
  std::cout << "triangle=" << triangleIntersections.IsDone(tf1, tf2) << ','
            << triangleIntersections.NewEdges().Extent() << '\n';
  if (!triangleIntersections.NewEdges().IsEmpty())
  {
    const TopoDS_Edge& triangleEdge = TopoDS::Edge(triangleIntersections.NewEdges().FindKey(1));
    TopLoc_Location triangleLocation;
    double triangleFirst, triangleLast;
    occ::handle<Geom_Curve> triangleCurve =
      BRep_Tool::Curve(triangleEdge, triangleLocation, triangleFirst, triangleLast);
    const gp_Pnt triangleStart = triangleCurve->Value(triangleFirst);
    const gp_Pnt triangleFinish = triangleCurve->Value(triangleLast);
    std::cout << "triangle_section=" << triangleFirst << ',' << triangleLast << ' '
              << triangleStart.X() << ',' << triangleStart.Y() << ',' << triangleStart.Z() << ' '
              << triangleFinish.X() << ',' << triangleFinish.Y() << ',' << triangleFinish.Z() << '\n';
  }

  TopoDS_Face hf1 = makeFaceWithHole(0.0);
  TopoDS_Face hf2 = makeRectangularFace(gp_Pln(gp_Pnt(0, 0, 0), gp_Dir(0, 1, 0)),
                                        -1.0,
                                        1.0,
                                        -3.0,
                                        3.0);
  TopoDS_Face hs1 = makeFaceWithHole(2.0);
  TopoDS_Face hs2 = makeRectangularFace(gp_Pln(gp_Pnt(0, 2, 0), gp_Dir(0, 1, 0)),
                                        -1.0,
                                        1.0,
                                        -3.0,
                                        3.0);
  occ::handle<BRepAlgo_AsDes> holeAsDes = new BRepAlgo_AsDes();
  BRepOffset_Inter3d holeIntersections(holeAsDes, TopAbs_IN, 1.0e-7);
  BRepAlgo_Image holeImages;
  holeImages.Bind(hs1, hf1);
  holeImages.Bind(hs2, hf2);
  holeIntersections.FaceInter(hf1, hf2, holeImages);
  std::cout << "hole=" << holeIntersections.IsDone(hf1, hf2) << ','
            << holeIntersections.NewEdges().Extent() << ','
            << holeAsDes->Descendant(hf1).Extent() << ','
            << holeAsDes->Descendant(hf2).Extent() << ','
            << BRepCheck_Analyzer(hf1).IsValid() << ','
            << BRepCheck_Analyzer(hf2).IsValid() << ','
            << static_cast<int>(BRepClass_FaceClassifier(hf1, gp_Pnt2d(0, 0), 1e-7).State()) << ','
            << static_cast<int>(BRepClass_FaceClassifier(hf1, gp_Pnt2d(2, 0), 1e-7).State()) << '\n';
  for (int index = 1; index <= holeIntersections.NewEdges().Extent(); ++index)
  {
    const TopoDS_Edge& holeEdge = TopoDS::Edge(holeIntersections.NewEdges().FindKey(index));
    TopLoc_Location holeLocation;
    double holeFirst, holeLast;
    occ::handle<Geom_Curve> holeCurve =
      BRep_Tool::Curve(holeEdge, holeLocation, holeFirst, holeLast);
    const gp_Pnt holeStart = holeCurve->Value(holeFirst);
    const gp_Pnt holeFinish = holeCurve->Value(holeLast);
    std::cout << "hole_section=" << holeFirst << ',' << holeLast << ' '
              << holeStart.X() << ',' << holeStart.Y() << ',' << holeStart.Z() << ' '
              << holeFinish.X() << ',' << holeFinish.Y() << ',' << holeFinish.Z() << '\n';
  }
}
