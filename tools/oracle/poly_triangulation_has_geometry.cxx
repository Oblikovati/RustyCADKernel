#include <Poly_Triangulation.hxx>
#include <Bnd_Box.hxx>
#include <gp_Ax1.hxx>
#include <gp_Dir.hxx>
#include <gp_Pnt.hxx>
#include <gp_Trsf.hxx>
#include <iomanip>
#include <iostream>

class EdgeBackedTriangulation : public Poly_Triangulation
{
public:
  EdgeBackedTriangulation() : Poly_Triangulation(3, 0, false, false) {}
  void AddEdge() { ++myNbEdges; }
  bool HasGeometryBase() const { return Poly_Triangulation::HasGeometry(); }
  bool HasGeometry() const override
  {
    return NbNodes() > 0 && (NbTriangles() > 0 || myNbEdges > 0);
  }

private:
  int myNbEdges = 0;
};

void print_box(const Bnd_Box& theBox)
{
  double aXMin, aYMin, aZMin, aXMax, aYMax, aZMax;
  theBox.Get(aXMin, aYMin, aZMin, aXMax, aYMax, aZMax);
  std::cout << "rotated " << aXMin << ' ' << aYMin << ' ' << aZMin << ' ' << aXMax << ' '
            << aYMax << ' ' << aZMax << '\n';
}

int main()
{
  std::cout << std::setprecision(17);
  EdgeBackedTriangulation aMesh;
  for (int anIndex = 1; anIndex <= 3; ++anIndex)
  {
    aMesh.SetNode(anIndex, gp_Pnt(1.0, 2.0, 3.0));
  }
  std::cout << "base_nodes " << aMesh.HasGeometryBase() << '\n';
  std::cout << "virtual_nodes " << aMesh.HasGeometry() << '\n';
  aMesh.AddEdge();
  std::cout << "virtual_edge " << aMesh.HasGeometry() << '\n';
  Bnd_Box aCached;
  aCached.Add(gp_Pnt(-5.0, -5.0, -5.0));
  aCached.Add(gp_Pnt(5.0, 5.0, 5.0));
  aMesh.SetCachedMinMax(aCached);
  gp_Trsf aRotation;
  aRotation.SetRotation(gp_Ax1(gp_Pnt(0.0, 0.0, 0.0), gp_Dir(0.0, 0.0, 1.0)), 0.5);
  Bnd_Box aResult;
  aMesh.MinMax(aResult, aRotation, true);
  print_box(aResult);
}
