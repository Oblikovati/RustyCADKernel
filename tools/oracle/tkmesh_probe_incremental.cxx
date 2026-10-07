// BRepMesh_IncrementalMesh over the BRepMesh face algorithms at OCCT
// 3d097a0328e71b826377d4814ab05ec3c3d23871: one shape per surface type of
// BRepMesh_MeshAlgoFactory, each face reported by its counts and an FNV-1a hash of
// every node, UV node and triangle.
#include <BRepLib_MakeEdge.hxx>
#include <BRepLib_MakeFace.hxx>
#include <BRepMesh_IncrementalMesh.hxx>
#include <BRepPrimAPI_MakeBox.hxx>
#include <BRepPrimAPI_MakeCone.hxx>
#include <BRepPrimAPI_MakeCylinder.hxx>
#include <BRepPrimAPI_MakePrism.hxx>
#include <BRepPrimAPI_MakeRevol.hxx>
#include <BRepPrimAPI_MakeSphere.hxx>
#include <BRepPrimAPI_MakeTorus.hxx>
#include <BRep_Tool.hxx>
#include <Geom_BezierCurve.hxx>
#include <Geom_BezierSurface.hxx>
#include <IMeshTools_Parameters.hxx>
#include <NCollection_Array1.hxx>
#include <NCollection_Array2.hxx>
#include <Poly_Triangulation.hxx>
#include <TopExp_Explorer.hxx>
#include <TopoDS.hxx>
#include <gp_Ax1.hxx>
#include <cstring>

namespace
{
struct IncrementalHash
{
  uint64_t value = 1469598103934665603ULL;
  void add(uint64_t bits)
  {
    for (int i = 0; i < 8; ++i)
    {
      value ^= (bits >> (8 * i)) & 0xFF;
      value *= 1099511628211ULL;
    }
  }
  void real(double v)
  {
    uint64_t bits;
    std::memcpy(&bits, &v, sizeof(bits));
    add(bits);
  }
  void integer(int v) { add(static_cast<uint64_t>(static_cast<int64_t>(v))); }
};

TopoDS_Shape incremental_bezier_edge()
{
  NCollection_Array1<gp_Pnt> poles(1, 4);
  poles(1) = gp_Pnt(2., 0., 0.);
  poles(2) = gp_Pnt(4., 0., 1.);
  poles(3) = gp_Pnt(3., 0., 3.);
  poles(4) = gp_Pnt(5., 0., 4.);
  return BRepLib_MakeEdge(new Geom_BezierCurve(poles)).Edge();
}

TopoDS_Shape incremental_bezier_face()
{
  NCollection_Array2<gp_Pnt> poles(1, 3, 1, 3);
  for (int i = 1; i <= 3; ++i)
    for (int j = 1; j <= 3; ++j)
      poles(i, j) = gp_Pnt(i * 2., j * 2., ((i + j) % 3) * 0.75);
  return BRepLib_MakeFace(new Geom_BezierSurface(poles), 1.e-7).Face();
}

TopoDS_Shape incremental_shape(int kind)
{
  switch (kind)
  {
    case 0: return BRepPrimAPI_MakeBox(10., 10., 10.).Shape();
    case 1: return BRepPrimAPI_MakeCylinder(5., 10.).Shape();
    case 2: return BRepPrimAPI_MakeCone(10., 5., 10.).Shape();
    case 3: return BRepPrimAPI_MakeSphere(10.).Shape();
    case 4: return BRepPrimAPI_MakeTorus(10., 3.).Shape();
    case 5: return BRepPrimAPI_MakePrism(incremental_bezier_edge(), gp_Vec(0., 6., 0.)).Shape();
    case 6: return BRepPrimAPI_MakeRevol(incremental_bezier_edge(), gp_Ax1(gp::Origin(), gp::DZ()), 3.).Shape();
    default: return incremental_bezier_face();
  }
}

void incremental_face_report(const TopoDS_Face& face)
{
  TopLoc_Location location;
  const occ::handle<Poly_Triangulation> triangulation = BRep_Tool::Triangulation(face, location);
  ga_flag(!triangulation.IsNull());
  if (triangulation.IsNull())
    return;
  ga_int(triangulation->NbNodes());
  ga_int(triangulation->NbTriangles());
  IncrementalHash hash;
  for (int i = 1; i <= triangulation->NbNodes(); ++i)
  {
    const gp_Pnt point = triangulation->Node(i);
    hash.real(point.X()); hash.real(point.Y()); hash.real(point.Z());
    if (triangulation->HasUVNodes())
    {
      const gp_Pnt2d uv = triangulation->UVNode(i);
      hash.real(uv.X()); hash.real(uv.Y());
    }
  }
  for (int i = 1; i <= triangulation->NbTriangles(); ++i)
  {
    int a, b, c;
    triangulation->Triangle(i).Get(a, b, c);
    hash.integer(a); hash.integer(b); hash.integer(c);
  }
  std::cout << hash.value << " ";
}
} // namespace

// mesh_incremental_shape <kind 0..7> <deflection 0..1> <mode 0..3>
void mesh_incremental_shape(std::istream& input)
{
  int kind, deflection, mode;
  input >> kind >> deflection >> mode;
  const TopoDS_Shape shape = incremental_shape(kind);
  IMeshTools_Parameters parameters;
  parameters.Deflection = deflection == 0 ? 0.5 : 0.1;
  parameters.InternalVerticesMode = mode == 1;
  parameters.EnableControlSurfaceDeflectionAllSurfaces = mode == 2;
  parameters.Relative = mode == 3;
  BRepMesh_IncrementalMesh mesher(shape, parameters);
  std::cout << "OK ";
  ga_flag(mesher.IsDone());
  ga_int(mesher.GetStatusFlags());
  int faces = 0;
  for (TopExp_Explorer explorer(shape, TopAbs_FACE); explorer.More(); explorer.Next(), ++faces)
    incremental_face_report(TopoDS::Face(explorer.Current()));
  ga_int(faces);
  ga_end();
}

#include <BRepMesh_Triangulator.hxx>
#include <NCollection_DynamicArray.hxx>

// mesh_triangulator <kind 0..5>: planar polygons (kind 4 has a hole) through
// BRepMesh_Triangulator; every output triangle is printed.
void mesh_triangulator(std::istream& input)
{
  int kind;
  input >> kind;
  const double xy[][2] = {{0, 0}, {4, 0}, {4, 3}, {0, 3}, {2, 5}, {1, 1}, {3, 1}, {3, 2}, {1, 2}, {5, 1.5}};
  NCollection_DynamicArray<gp_XYZ> nodes;
  for (const auto& p : xy)
    nodes.Append(gp_XYZ(p[0], p[1], 0.25 * p[0]));
  NCollection_List<NCollection_Sequence<int>> wires;
  const std::vector<std::vector<int>> outlines[] = {
    {{0, 1, 2}}, {{0, 1, 2, 3}}, {{0, 6, 2, 3}}, {{0, 1, 9, 2, 4, 3}},
    {{0, 1, 2, 4, 3}, {5, 8, 7, 6}}, {{0, 5, 1, 9, 2, 7, 4, 8, 3}}};
  for (const auto& outline : outlines[kind])
  {
    NCollection_Sequence<int> wire;
    for (int index : outline)
      wire.Append(index);
    wires.Append(wire);
  }
  BRepMesh_Triangulator triangulator(nodes, wires, gp_Dir(-0.25, 0., 1.));
  NCollection_List<Poly_Triangle> triangles;
  std::cout << "OK ";
  ga_flag(triangulator.Perform(triangles));
  ga_int(triangles.Extent());
  for (NCollection_List<Poly_Triangle>::Iterator it(triangles); it.More(); it.Next())
  {
    int a, b, c;
    it.Value().Get(a, b, c);
    ga_int(a); ga_int(b); ga_int(c);
  }
  ga_end();
}
