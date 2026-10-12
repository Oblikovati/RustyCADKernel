// Actual BRep/Poly faces isolate native textured triangulation without renderer substitutions.
#include <BRep_Builder.hxx>
#include <Geom_Plane.hxx>
#include <Geom_RectangularTrimmedSurface.hxx>
#include <Poly_Triangulation.hxx>
#include <TopoDS_Face.hxx>
#include <TopoDS.hxx>
#include <gp_Ax1.hxx>
#include <gp_Pln.hxx>
#include <gp_Trsf.hxx>
static occ::handle<Poly_Triangulation> native_textured_face_mesh(int scenario) {
  const auto mesh = occ::handle<Poly_Triangulation>(new Poly_Triangulation(4, 3, true));
  const gp_Pnt points[] = {gp_Pnt(-2,-3,0),gp_Pnt(4,-3,0),gp_Pnt(4,5,0),gp_Pnt(-2,5,0)};
  for (int node = 1; node <= 4; ++node) {
    mesh->SetNode(node, points[node-1]);
    mesh->SetUVNode(node, gp_Pnt2d(points[node-1].X(), points[node-1].Y()));
  }
  mesh->SetTriangle(1, Poly_Triangle(1,2,3)); mesh->SetTriangle(2, Poly_Triangle(1,3,4));
  mesh->SetTriangle(3, Poly_Triangle(1,2,2));
  if (scenario & 1) {
    mesh->AddNormals();
    for (int node = 1; node <= 4; ++node) mesh->SetNormal(node, gp_Dir(0,0,1));
  }
  return mesh;
}
static TopoDS_Face native_textured_face(int scenario) {
  BRep_Builder builder; TopoDS_Face face;
  const auto mesh = native_textured_face_mesh(scenario);
  builder.MakeFace(face, mesh);
  if (!(scenario & 4)) {
    const occ::handle<Geom_Surface> plane = new Geom_Plane(gp_Pln(gp_Pnt(0,0,0),gp_Dir(0,0,1)));
    const occ::handle<Geom_Surface> finite = new Geom_RectangularTrimmedSurface(plane,-2.,4.,-3.,5.,true,true);
    builder.UpdateFace(face,finite,TopLoc_Location(),1e-7);
  }
  if (scenario & 2) face.Reverse();
  if (scenario & 8) {
    gp_Trsf transform; transform.SetMirror(gp_Ax2(gp_Pnt(1,2,3),gp_Dir(0,0,1)));
    face = TopoDS::Face(face.Located(TopLoc_Location(transform),false));
  }
  return face;
}
