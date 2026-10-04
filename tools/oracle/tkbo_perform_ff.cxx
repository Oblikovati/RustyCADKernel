#include <BOPAlgo_PaveFiller.hxx>
#include <BOPDS_DS.hxx>
#include <BOPDS_Interf.hxx>
#include <BOPDS_Curve.hxx>
#include <BOPDS_Point.hxx>
#include <BRepBuilderAPI_MakeFace.hxx>
#include <Geom_CylindricalSurface.hxx>
#include <Geom_Plane.hxx>
#include <TopoDS_Face.hxx>
#include <NCollection_List.hxx>
#include <gp_Ax3.hxx>
#include <gp_Cylinder.hxx>
#include <gp_Pln.hxx>
#include <gp_Pnt.hxx>
#include <gp_Dir.hxx>
#include <iomanip>
#include <iostream>

int main()
{
  occ::handle<Geom_Plane> aPlane = new Geom_Plane(gp_Pln(gp::Origin(), gp::DZ()));
  TopoDS_Face aFace1 = BRepBuilderAPI_MakeFace(aPlane, -3., 3., -3., 3., 1.e-7);
  occ::handle<Geom_CylindricalSurface> aCylinder =
      new Geom_CylindricalSurface(gp_Ax3(), 1.);
  TopoDS_Face aFace2 = BRepBuilderAPI_MakeFace(aCylinder, 0., 2. * M_PI, -2., 2., 1.e-7);
  NCollection_List<TopoDS_Shape> aArguments;
  aArguments.Append(aFace1);
  aArguments.Append(aFace2);
  BOPAlgo_PaveFiller aFiller;
  aFiller.SetArguments(aArguments);
  aFiller.Perform();
  BOPDS_DS* aDS = aFiller.PDS();
  const auto& aFFs = aDS->InterfFF();
  std::cout << std::setprecision(17) << aFFs.Length();
  for (int i = 0; i < aFFs.Length(); ++i)
  {
    const BOPDS_InterfFF& aFF = aFFs(i);
    int nF1, nF2;
    aFF.Indices(nF1, nF2);
    std::cout << ' ' << nF1 << ' ' << nF2 << ' ' << aFF.Curves().Length() << ' '
              << aFF.Points().Length() << ' ' << aFF.TangentFaces();
    for (int j = 0; j < aFF.Curves().Length(); ++j)
    {
      const BOPDS_Curve& aStored = aFF.Curves()(j);
      const IntTools_Curve& aCurve = aStored.Curve();
      double xmin, ymin, zmin, xmax, ymax, zmax;
      aStored.Box().Get(xmin, ymin, zmin, xmax, ymax, zmax);
      std::cout << ' ' << aCurve.Curve()->FirstParameter() << ' '
                << aCurve.Curve()->LastParameter() << ' ' << aStored.Tolerance() << ' '
                << xmin << ' ' << ymin << ' ' << zmin << ' ' << xmax << ' ' << ymax << ' '
                << zmax;
    }
  }
  std::cout << ' ' << aDS->FaceInfo(0).VerticesOn().Extent() << ' '
            << aDS->FaceInfo(0).VerticesIn().Extent() << ' '
            << aDS->FaceInfo(10).VerticesOn().Extent() << ' '
            << aDS->FaceInfo(10).VerticesIn().Extent();
  BOPDS_DS aStageDS;
  aStageDS.SetArguments(aArguments);
  aStageDS.Init();
  NCollection_Map<int> aTouchedFaces;
  aTouchedFaces.Add(0);
  aTouchedFaces.Add(10);
  aStageDS.UpdateFaceInfoOn(aTouchedFaces);
  aStageDS.UpdateFaceInfoIn(aTouchedFaces);
  std::cout << ' ' << aStageDS.FaceInfo(0).VerticesOn().Extent() << ' '
            << aStageDS.FaceInfo(0).VerticesIn().Extent() << ' '
            << aStageDS.FaceInfo(10).VerticesOn().Extent() << ' '
            << aStageDS.FaceInfo(10).VerticesIn().Extent();
  std::cout << '\n';
  return aFiller.HasErrors() ? 1 : 0;
}
