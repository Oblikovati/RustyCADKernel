// Native oracle for BOPAlgo_BuilderSolid (with BOPAlgo_ShellSplitter and
// BOPAlgo_Tools::ClassifyFaces) on face sets of primitive solids.
#include <BOPAlgo_BuilderSolid.hxx>
#include <BRepBuilderAPI_MakeFace.hxx>
#include <BRepPrimAPI_MakeBox.hxx>
#include <BRepPrimAPI_MakeCylinder.hxx>
#include <TopExp_Explorer.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Iterator.hxx>
#include <gp_Pln.hxx>
#include <cstdint>
#include <cstdio>
#include <cstring>
#include <iostream>
#include <string>
#include <vector>

static std::string h(double value)
{
  std::uint64_t bits;
  std::memcpy(&bits, &value, sizeof bits);
  char text[17];
  std::snprintf(text, sizeof text, "%016llx", static_cast<unsigned long long>(bits));
  return text;
}

static char o(TopAbs_Orientation orientation)
{
  return "FRIE"[orientation];
}

static int indexOf(const std::vector<TopoDS_Shape>& faces, const TopoDS_Shape& face)
{
  for (size_t i = 0; i < faces.size(); ++i)
    if (faces[i].IsSame(face))
      return static_cast<int>(i);
  return -1;
}

static void box(const BOPAlgo_BuilderSolid& builder, const TopoDS_Shape& area)
{
  const Bnd_Box* bounds = builder.GetBoxesMap().Seek(area);
  if (bounds == nullptr)
  {
    std::cout << " nobox";
    return;
  }
  if (bounds->IsVoid() || bounds->IsWhole())
  {
    std::cout << " box " << bounds->IsVoid() << bounds->IsWhole();
    return;
  }
  double x1, y1, z1, x2, y2, z2;
  bounds->Get(x1, y1, z1, x2, y2, z2);
  std::cout << " box " << h(x1) << ' ' << h(y1) << ' ' << h(z1) << ' ' << h(x2) << ' ' << h(y2)
            << ' ' << h(z2);
}

static void run(const char* label, const std::vector<TopoDS_Shape>& faces)
{
  BOPAlgo_BuilderSolid builder;
  NCollection_List<TopoDS_Shape> shapes;
  for (const TopoDS_Shape& f : faces)
    shapes.Append(f);
  builder.SetShapes(shapes);
  builder.Perform();
  std::cout << "case " << label << " errors " << builder.HasErrors() << " warnings "
            << builder.HasWarnings() << " loops " << builder.Loops().Extent() << " areas "
            << builder.Areas().Extent() << '\n';
  for (const TopoDS_Shape& area : builder.Areas())
  {
    std::cout << " a " << o(area.Orientation());
    box(builder, area);
    std::cout << '\n';
    for (TopoDS_Iterator shells(area); shells.More(); shells.Next())
    {
      const TopoDS_Shape& shell = shells.Value();
      std::cout << "  s " << o(shell.Orientation()) << ' ' << shell.Closed() << ':';
      for (TopoDS_Iterator it(shell); it.More(); it.Next())
        std::cout << ' ' << indexOf(faces, it.Value()) << o(it.Value().Orientation());
      std::cout << '\n';
    }
  }
}

static void addFaces(std::vector<TopoDS_Shape>& faces, const TopoDS_Shape& solid, bool reversed)
{
  for (TopExp_Explorer ex(solid, TopAbs_FACE); ex.More(); ex.Next())
    faces.push_back(reversed ? ex.Current().Reversed() : ex.Current());
}

int main()
{
  TopoDS_Shape big   = BRepPrimAPI_MakeBox(10., 10., 10.).Shape();
  TopoDS_Shape small = BRepPrimAPI_MakeBox(gp_Pnt(3, 3, 3), 2., 2., 2.).Shape();
  TopoDS_Shape far   = BRepPrimAPI_MakeBox(gp_Pnt(20, 0, 0), 2., 2., 2.).Shape();
  {
    std::vector<TopoDS_Shape> faces;
    addFaces(faces, big, false);
    run("box", faces);
  }
  {
    std::vector<TopoDS_Shape> faces;
    addFaces(faces, big, false);
    addFaces(faces, far, false);
    run("two", faces);
  }
  {
    std::vector<TopoDS_Shape> faces;
    addFaces(faces, big, false);
    addFaces(faces, small, true);
    run("cavity", faces);
  }
  {
    std::vector<TopoDS_Shape> faces;
    addFaces(faces, small, true);
    run("hole-only", faces);
  }
  {
    std::vector<TopoDS_Shape> faces;
    addFaces(faces, big, false);
    TopoDS_Face inner = BRepBuilderAPI_MakeFace(gp_Pln(gp_Pnt(0, 0, 5), gp_Dir(0, 0, 1)), 2., 4., 2., 4.).Face();
    faces.push_back(inner);
    faces.push_back(inner.Reversed());
    TopoDS_Face outer = BRepBuilderAPI_MakeFace(gp_Pln(gp_Pnt(0, 0, 5), gp_Dir(0, 0, 1)), 12., 14., 2., 4.).Face();
    faces.push_back(outer);
    faces.push_back(outer.Reversed());
    run("internal", faces);
  }
  {
    std::vector<TopoDS_Shape> faces;
    TopExp_Explorer ex(big, TopAbs_FACE);
    for (ex.Next(); ex.More(); ex.Next())
      faces.push_back(ex.Current());
    run("open", faces);
  }
  {
    std::vector<TopoDS_Shape> faces;
    addFaces(faces, BRepPrimAPI_MakeCylinder(2., 5.).Shape(), false);
    run("cylinder", faces);
  }
  {
    std::vector<TopoDS_Shape> faces;
    addFaces(faces, big, false);
    addFaces(faces, big, false);
    run("twice", faces);
  }
  return 0;
}
