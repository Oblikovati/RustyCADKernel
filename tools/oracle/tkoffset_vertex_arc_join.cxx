#include <BRepOffset_Offset.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRepBuilderAPI_MakeVertex.hxx>
#include <BRepCheck_Analyzer.hxx>
#include <BRepGProp.hxx>
#include <BRep_Tool.hxx>
#include <Geom_SphericalSurface.hxx>
#include <GProp_GProps.hxx>
#include <gp_Sphere.hxx>
#include <TopExp_Explorer.hxx>
#include <iostream>
#include <string>

int main(int argc, char** argv) {
  const double offset = argc > 1 ? std::stod(argv[1]) : 2.0;
  TopoDS_Vertex vertex = BRepBuilderAPI_MakeVertex(gp_Pnt(0, 0, 0));
  NCollection_List<TopoDS_Shape> edges;
  for (const gp_Pnt& point : {gp_Pnt(3, 0, 0), gp_Pnt(0, 3, 0), gp_Pnt(0, 0, 3)}) {
    edges.Append(BRepBuilderAPI_MakeEdge(gp_Pnt(0, 0, 0), point));
  }
  BRepOffset_Offset join(vertex, edges, offset);
  const TopoDS_Face& face = join.Face();
  TopLoc_Location location;
  const auto surface = Handle(Geom_SphericalSurface)::DownCast(BRep_Tool::Surface(face, location));
  const gp_Sphere sphere = surface->Sphere();
  const gp_Ax3 frame = sphere.Position();
  int edgeCount = 0;
  int vertexCount = 0;
  for (TopExp_Explorer it(face, TopAbs_EDGE); it.More(); it.Next()) ++edgeCount;
  for (TopExp_Explorer it(face, TopAbs_VERTEX); it.More(); it.Next()) ++vertexCount;
  GProp_GProps props;
  BRepGProp::SurfaceProperties(face, props);
  std::cout << surface->DynamicType()->Name() << ' ' << join.Status() << ' '
            << edgeCount << ' ' << vertexCount << ' ' << sphere.Radius() << ' '
            << int(face.Orientation()) << ' '
            << frame.Location().X() << ' ' << frame.Location().Y() << ' ' << frame.Location().Z() << ' '
            << frame.Direction().X() << ' ' << frame.Direction().Y() << ' ' << frame.Direction().Z() << ' '
            << frame.XDirection().X() << ' ' << frame.XDirection().Y() << ' ' << frame.XDirection().Z() << ' '
            << BRepCheck_Analyzer(face, true).IsValid() << ' ' << props.Mass() << '\n';
}
