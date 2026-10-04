// Protected hooks at OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
#include <Bnd_OBB.hxx>
#include <Poly_Triangulation.hxx>
#include <iostream>

struct ExposedObb : Bnd_OBB { using Bnd_OBB::ProcessOnePoint; };
struct ExposedMesh : Poly_Triangulation { using Poly_Triangulation::unsetCachedMinMax; };

static void emitObb(const ExposedObb& box)
{
  std::cout << "obb " << box.IsVoid() << ' ' << box.IsAABox();
  for (const gp_XYZ& xyz : {box.Center(), box.XDirection(), box.YDirection(), box.ZDirection()})
    std::cout << ' ' << xyz.X() << ' ' << xyz.Y() << ' ' << xyz.Z();
  std::cout << ' ' << box.XHSize() << ' ' << box.YHSize() << ' ' << box.ZHSize() << '\n';
}

static void obbCases()
{
  ExposedObb box;
  box.ProcessOnePoint(gp_Pnt(3, -4, 5));
  emitObb(box);
  box.SetCenter(gp_Pnt(17, 18, 19));
  box.SetXComponent(gp_Dir(0, 1, 0), 6);
  box.SetYComponent(gp_Dir(-1, 0, 0), 7);
  box.SetZComponent(gp_Dir(0, 0, 1), 8);
  box.SetAABox(false);
  box.ProcessOnePoint(gp_Pnt(-8, 9, -10));
  emitObb(box);
  box.ProcessOnePoint(gp_Pnt(-8, 9, -10));
  emitObb(box);
}

static void emitMesh(const ExposedMesh& mesh)
{
  Bnd_Box box;
  bool found = mesh.MinMax(box, gp_Trsf(), false);
  std::cout << "mesh " << mesh.HasCachedMinMax() << ' ' << mesh.CachedMinMax().IsVoid()
            << ' ' << mesh.NbNodes() << ' ' << mesh.NbTriangles() << ' ' << found;
  if (found)
  {
    double x1,y1,z1,x2,y2,z2;
    box.Get(x1,y1,z1,x2,y2,z2);
    std::cout << ' ' << x1 << ' ' << y1 << ' ' << z1 << ' ' << x2 << ' ' << y2 << ' ' << z2;
  }
  std::cout << '\n';
}

static void meshCases()
{
  ExposedMesh mesh;
  mesh.unsetCachedMinMax();
  emitMesh(mesh);
  Bnd_Box cached;
  cached.Update(-10,-20,-30,10,20,30);
  mesh.SetCachedMinMax(cached);
  emitMesh(mesh);
  mesh.unsetCachedMinMax();
  emitMesh(mesh);
  mesh.ResizeNodes(2, false);
  mesh.SetNode(1, gp_Pnt(1,2,3));
  mesh.SetNode(2, gp_Pnt(4,5,6));
  mesh.SetCachedMinMax(cached);
  mesh.unsetCachedMinMax();
  emitMesh(mesh);
  mesh.unsetCachedMinMax();
  emitMesh(mesh);
}

int main() { obbCases(); meshCases(); }
