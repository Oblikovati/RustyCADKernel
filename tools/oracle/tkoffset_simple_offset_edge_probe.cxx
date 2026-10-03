// Oracle for BRepOffset_SimpleOffset::FillEdgeData's curve/location/tolerance outputs.
#include <BRepBuilderAPI_MakeFace.hxx>
#include <BRep_Builder.hxx>
#include <BRepOffset_SimpleOffset.hxx>
#include <BRep_Tool.hxx>
#include <Geom_Curve.hxx>
#include <TopLoc_Location.hxx>
#include <TopExp_Explorer.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Edge.hxx>
#include <TopoDS_Face.hxx>
#include <TopoDS_Compound.hxx>
#include <gp_Pln.hxx>
#include <gp_Pnt.hxx>
#include <gp_Trsf.hxx>
#include <gp_Vec.hxx>
#include <iostream>

void print_edge(const TopoDS_Edge& edge, BRepOffset_SimpleOffset& offset)
{
  occ::handle<Geom_Curve> curve;
  TopLoc_Location         location;
  double                  tolerance = 0.0;
  const bool              found     = offset.NewCurve(edge, curve, location, tolerance);
  double                  first = 0.0, last = 0.0;
  BRep_Tool::Range(edge, first, last);
  std::cout << found << ' ' << curve.IsNull() << ' ' << location.IsIdentity() << ' '
            << tolerance << ' ' << first << ' ' << last;
  if (!curve.IsNull())
  {
    const gp_Pnt start = curve->Value(first).Transformed(location.Transformation());
    const gp_Pnt end   = curve->Value(last).Transformed(location.Transformation());
    std::cout << ' ' << start.X() << ' ' << start.Y() << ' ' << start.Z() << ' '
              << end.X() << ' ' << end.Y() << ' ' << end.Z();
  }
  std::cout << '\n';
}

void run_shape(const TopoDS_Shape& shape)
{
  BRepOffset_SimpleOffset offset(shape, 10.0, 1.e-7);
  for (TopExp_Explorer explorer(shape, TopAbs_EDGE); explorer.More(); explorer.Next())
  {
    print_edge(TopoDS::Edge(explorer.Current()), offset);
  }
}

void print_basic_face_cases()
{
  const TopoDS_Face planeFace =
    BRepBuilderAPI_MakeFace(gp_Pln(gp::XOY()), -1., 1., -1., 1.).Face();
  std::cout << "plane\n";
  run_shape(planeFace);

  TopoDS_Face locatedFace = planeFace;
  gp_Trsf      translation;
  translation.SetTranslation(gp_Vec(0.0, 0.0, 3.0));
  locatedFace.Move(TopLoc_Location(translation));
  std::cout << "located\n";
  run_shape(locatedFace);

  TopoDS_Face reversedFace = planeFace;
  reversedFace.Reverse();
  std::cout << "reversed\n";
  run_shape(reversedFace);
}

TopoDS_Compound opposed_faces()
{
  TopoDS_Face firstFace =
    BRepBuilderAPI_MakeFace(gp_Pln(gp::XOY()), -1., 1., -1., 1.).Face();
  TopoDS_Face secondFace = firstFace;
  secondFace.Reverse();
  TopoDS_Compound adjacentFaces;
  BRep_Builder builder;
  builder.MakeCompound(adjacentFaces);
  builder.Add(adjacentFaces, firstFace);
  builder.Add(adjacentFaces, secondFace);
  return adjacentFaces;
}

int main()
{
  print_basic_face_cases();
  std::cout << "adjacent\n";
  run_shape(opposed_faces());
}
