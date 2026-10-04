#include <BRepAlgo_AsDes.hxx>
#include <BRepAlgo_Image.hxx>
#include <BRep_Tool.hxx>
#include <BRepBuilderAPI_MakeFace.hxx>
#include <BRepOffset_Inter3d.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Edge.hxx>
#include <TopoDS_Face.hxx>
#include <gp_Pln.hxx>
#include <iostream>

static TopoDS_Face makeFace(const gp_Pln& plane)
{
  return BRepBuilderAPI_MakeFace(plane).Face();
}

static TopoDS_Face makeBoundedFace(const gp_Pln& plane)
{
  return BRepBuilderAPI_MakeFace(plane, -1.0, 1.0, -1.0, 1.0).Face();
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
}
