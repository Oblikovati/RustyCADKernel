// Shared dump of a Boolean result for the TKBO oracles: the vertices with their points and
// tolerances, the edges with their tolerance, range and vertices, the faces with their wires
// and the solids with their shells, all by index in the result maps.
#ifndef TKBO_SHAPE_DUMP_HXX
#define TKBO_SHAPE_DUMP_HXX
#include <BRep_Tool.hxx>
#include <TopExp.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Iterator.hxx>
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

#endif
