// Native oracle for the whole BOPAlgo_Builder (General Fuse) on solid pairs: the result
// topology with its points and tolerances, the images and the history of every source shape.
#include <BOPAlgo_Builder.hxx>
#include <BOPAlgo_PaveFiller.hxx>
#include <BOPDS_DS.hxx>
#include <BRepPrimAPI_MakeBox.hxx>
#include <BRepPrimAPI_MakeCylinder.hxx>
#include <BRepPrimAPI_MakeSphere.hxx>
#include <BRep_Tool.hxx>
#include <TopExp.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Iterator.hxx>
#include <gp_Ax2.hxx>
#include <cstdint>
#include <cstdio>
#include <cstring>
#include <iostream>
#include <string>

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

typedef NCollection_IndexedMap<TopoDS_Shape, TopTools_ShapeMapHasher> ShapeIndex;

static void dumpResult(const TopoDS_Shape& result, ShapeIndex& all)
{
  ShapeIndex vertices, edges, faces, solids;
  TopExp::MapShapes(result, TopAbs_VERTEX, vertices);
  TopExp::MapShapes(result, TopAbs_EDGE, edges);
  TopExp::MapShapes(result, TopAbs_FACE, faces);
  TopExp::MapShapes(result, TopAbs_SOLID, solids);
  TopExp::MapShapes(result, all);
  std::cout << "result " << vertices.Extent() << ' ' << edges.Extent() << ' ' << faces.Extent()
            << ' ' << solids.Extent() << ' ' << all.Extent() << '\n';
  for (int i = 1; i <= vertices.Extent(); ++i)
  {
    const TopoDS_Vertex& v = TopoDS::Vertex(vertices(i));
    gp_Pnt p = BRep_Tool::Pnt(v);
    std::cout << "v " << h(p.X()) << ' ' << h(p.Y()) << ' ' << h(p.Z()) << ' '
              << h(BRep_Tool::Tolerance(v)) << '\n';
  }
  for (int i = 1; i <= edges.Extent(); ++i)
  {
    const TopoDS_Edge& e = TopoDS::Edge(edges(i));
    double f, l;
    BRep_Tool::Range(e, f, l);
    std::cout << "e " << h(BRep_Tool::Tolerance(e)) << ' ' << h(f) << ' ' << h(l) << ' '
              << BRep_Tool::Degenerated(e) << ':';
    for (TopoDS_Iterator it(e); it.More(); it.Next())
      std::cout << ' ' << vertices.FindIndex(it.Value()) << o(it.Value().Orientation());
    std::cout << '\n';
  }
  for (int i = 1; i <= faces.Extent(); ++i)
  {
    const TopoDS_Face& f = TopoDS::Face(faces(i));
    std::cout << "f " << h(BRep_Tool::Tolerance(f));
    for (TopoDS_Iterator w(f); w.More(); w.Next())
    {
      std::cout << " |" << o(w.Value().Orientation());
      for (TopoDS_Iterator it(w.Value()); it.More(); it.Next())
        std::cout << ' ' << edges.FindIndex(it.Value()) << o(it.Value().Orientation());
    }
    std::cout << '\n';
  }
  for (int i = 1; i <= solids.Extent(); ++i)
  {
    std::cout << "s";
    for (TopoDS_Iterator sh(solids(i)); sh.More(); sh.Next())
    {
      std::cout << " |" << o(sh.Value().Orientation());
      for (TopoDS_Iterator it(sh.Value()); it.More(); it.Next())
        std::cout << ' ' << faces.FindIndex(it.Value()) << o(it.Value().Orientation());
    }
    std::cout << '\n';
  }
}

static void run(const char* label, const TopoDS_Shape& a, const TopoDS_Shape& b)
{
  BOPAlgo_Builder builder;
  builder.AddArgument(a);
  builder.AddArgument(b);
  builder.Perform();
  std::cout << "case " << label << " errors " << builder.HasErrors() << " warnings "
            << builder.HasWarnings() << '\n';
  ShapeIndex all;
  dumpResult(builder.Shape(), all);
  const BOPDS_DS& ds = *builder.PDS();
  for (int i = 0; i < ds.NbSourceShapes(); ++i)
  {
    const TopoDS_Shape& s = ds.Shape(i);
    std::cout << "i " << i << ' ' << s.ShapeType();
    const NCollection_List<TopoDS_Shape>* images = builder.Images().Seek(s);
    if (images)
    {
      std::cout << " im";
      for (const TopoDS_Shape& im : *images)
        std::cout << ' ' << all.FindIndex(im) << o(im.Orientation());
    }
    std::cout << " h " << builder.Modified(s).Extent() << ' ' << builder.Generated(s).Extent()
              << ' ' << builder.IsDeleted(s) << '\n';
  }
}

static TopoDS_Shape box(const gp_Pnt& origin, double size)
{
  return BRepPrimAPI_MakeBox(origin, size, size, size).Shape();
}

int main()
{
  TopoDS_Shape unit = box(gp_Pnt(0, 0, 0), 10.);
  run("box-box", unit, box(gp_Pnt(5, 5, 5), 10.));
  run("box-box-face", unit, BRepPrimAPI_MakeBox(gp_Pnt(10, 2, 2), 5., 5., 5.).Shape());
  run("box-cylinder",
      unit,
      BRepPrimAPI_MakeCylinder(gp_Ax2(gp_Pnt(5, 5, -5), gp_Dir(0, 0, 1)), 2., 20.).Shape());
  run("box-sphere", unit, BRepPrimAPI_MakeSphere(gp_Pnt(10, 10, 10), 4.).Shape());
  run("box-inner", unit, box(gp_Pnt(3, 3, 3), 2.));
  return 0;
}
