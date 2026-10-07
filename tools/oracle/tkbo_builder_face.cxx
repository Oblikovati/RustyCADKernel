// Native oracle for BOPAlgo_BuilderFace (with BOPAlgo_WireSplitter) on planar and
// cylindrical faces split by extra edges given in both orientations.
#include <BOPAlgo_BuilderFace.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRepBuilderAPI_MakeFace.hxx>
#include <BRepBuilderAPI_MakeVertex.hxx>
#include <BRepPrimAPI_MakeCylinder.hxx>
#include <BRepTools.hxx>
#include <BRep_Tool.hxx>
#include <Geom2d_Line.hxx>
#include <Geom_Surface.hxx>
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

static int indexOf(const std::vector<TopoDS_Shape>& edges, const TopoDS_Shape& edge)
{
  for (size_t i = 0; i < edges.size(); ++i)
    if (edges[i].IsSame(edge))
      return static_cast<int>(i);
  return -1;
}

static void run(const char* label, const TopoDS_Face& face, const std::vector<TopoDS_Shape>& edges)
{
  BOPAlgo_BuilderFace builder;
  NCollection_List<TopoDS_Shape> shapes;
  for (const TopoDS_Shape& e : edges)
    shapes.Append(e);
  builder.SetFace(face);
  builder.SetShapes(shapes);
  builder.Perform();
  std::cout << "case " << label << " errors " << builder.HasErrors() << " warnings "
            << builder.HasWarnings() << " loops " << builder.Loops().Extent() << " areas "
            << builder.Areas().Extent() << '\n';
  for (const TopoDS_Shape& area : builder.Areas())
  {
    double u1, u2, v1, v2;
    BRepTools::UVBounds(TopoDS::Face(area), u1, u2, v1, v2);
    std::cout << " a " << o(area.Orientation()) << ' ' << h(u1) << ' ' << h(u2) << ' ' << h(v1)
              << ' ' << h(v2) << '\n';
    for (TopoDS_Iterator wires(area); wires.More(); wires.Next())
    {
      const TopoDS_Shape& wire = wires.Value();
      std::cout << "  w " << o(wire.Orientation()) << ' ' << wire.Closed() << ':';
      for (TopoDS_Iterator it(wire); it.More(); it.Next())
        std::cout << ' ' << indexOf(edges, it.Value()) << o(it.Value().Orientation());
      std::cout << '\n';
    }
  }
}

static TopoDS_Vertex vertexAt(const TopoDS_Shape& shape, const gp_Pnt& point)
{
  for (TopExp_Explorer ex(shape, TopAbs_VERTEX); ex.More(); ex.Next())
    if (BRep_Tool::Pnt(TopoDS::Vertex(ex.Current())).Distance(point) < 1e-9)
      return TopoDS::Vertex(ex.Current());
  return BRepBuilderAPI_MakeVertex(point).Vertex();
}

static void both(std::vector<TopoDS_Shape>& edges, const TopoDS_Edge& edge)
{
  edges.push_back(edge.Oriented(TopAbs_FORWARD));
  edges.push_back(edge.Oriented(TopAbs_REVERSED));
}

static std::vector<TopoDS_Shape> faceEdges(const TopoDS_Face& face)
{
  std::vector<TopoDS_Shape> edges;
  for (TopExp_Explorer ex(face, TopAbs_EDGE); ex.More(); ex.Next())
    edges.push_back(ex.Current());
  return edges;
}

static void innerSquare(std::vector<TopoDS_Shape>& edges, const TopoDS_Vertex (&v)[4])
{
  for (int i = 0; i < 4; ++i)
    both(edges, BRepBuilderAPI_MakeEdge(v[i], v[(i + 1) % 4]).Edge());
}

int main()
{
  TopoDS_Face square = BRepBuilderAPI_MakeFace(gp_Pln(), 0., 10., 0., 10.).Face();
  TopoDS_Vertex c00 = vertexAt(square, gp_Pnt(0, 0, 0));
  TopoDS_Vertex c11 = vertexAt(square, gp_Pnt(10, 10, 0));
  {
    std::vector<TopoDS_Shape> edges = faceEdges(square);
    both(edges, BRepBuilderAPI_MakeEdge(c00, c11).Edge());
    run("diagonal", square, edges);
  }
  TopoDS_Vertex inner[4] = {BRepBuilderAPI_MakeVertex(gp_Pnt(2, 2, 0)).Vertex(),
                            BRepBuilderAPI_MakeVertex(gp_Pnt(4, 2, 0)).Vertex(),
                            BRepBuilderAPI_MakeVertex(gp_Pnt(4, 4, 0)).Vertex(),
                            BRepBuilderAPI_MakeVertex(gp_Pnt(2, 4, 0)).Vertex()};
  {
    std::vector<TopoDS_Shape> edges = faceEdges(square);
    innerSquare(edges, inner);
    run("hole", square, edges);
  }
  {
    std::vector<TopoDS_Shape> edges = faceEdges(square);
    innerSquare(edges, inner);
    both(edges, BRepBuilderAPI_MakeEdge(c00, inner[0]).Edge());
    run("bridge", square, edges);
  }
  {
    std::vector<TopoDS_Shape> edges = faceEdges(square);
    both(edges, BRepBuilderAPI_MakeEdge(c00, BRepBuilderAPI_MakeVertex(gp_Pnt(5, 5, 0)).Vertex()).Edge());
    run("dangling", square, edges);
  }
  {
    std::vector<TopoDS_Shape> edges = faceEdges(square);
    both(edges, BRepBuilderAPI_MakeEdge(gp_Pnt(3, 3, 0), gp_Pnt(6, 3, 0)).Edge());
    run("floating", square, edges);
  }
  {
    TopoDS_Shape cylinder = BRepPrimAPI_MakeCylinder(2., 5.).Shape();
    TopExp_Explorer ex(cylinder, TopAbs_FACE);
    TopoDS_Face lateral = TopoDS::Face(ex.Current());
    std::vector<TopoDS_Shape> edges = faceEdges(lateral);
    occ::handle<Geom_Surface> surface = BRep_Tool::Surface(lateral);
    occ::handle<Geom2d_Line> line = new Geom2d_Line(gp_Pnt2d(3., 1.), gp_Dir2d(0., 1.));
    both(edges, BRepBuilderAPI_MakeEdge(line, surface, 0., 2.).Edge());
    run("cylinder", lateral, edges);
  }
  return 0;
}
