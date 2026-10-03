// Repository-owned fixture for OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
#include <ShapeUpgrade_SplitSurfaceAngle.hxx>
#include <Geom_CylindricalSurface.hxx>
#include <Geom_ConicalSurface.hxx>
#include <Geom_SphericalSurface.hxx>
#include <Geom_ToroidalSurface.hxx>
#include <Geom_SurfaceOfRevolution.hxx>
#include <Geom_Line.hxx>
#include <Geom_Plane.hxx>
#include <Geom_OffsetSurface.hxx>
#include <Geom_RectangularTrimmedSurface.hxx>
#include <gp.hxx>
#include <iostream>
#include <iomanip>

class SeededAngle : public ShapeUpgrade_SplitSurfaceAngle
{
public:
  SeededAngle(double angle) : ShapeUpgrade_SplitSurfaceAngle(angle) {}
  void Seed(int value) { myStatus = value; }
  int Bits() const { return myStatus; }
};
static void report(const char* name, const occ::handle<Geom_Surface>& surface,
                   double first, double last, double maximum, int seed = 0)
{
  SeededAngle splitter(maximum);
  splitter.Init(surface,first,last,0,1);
  splitter.Seed(seed);
  auto shared = splitter.USplitValues();
  splitter.Compute(false);
  std::cout << name << ' ' << splitter.Bits() << ' ' << (shared == splitter.USplitValues());
  for (int i=1;i<=shared->Length();++i) std::cout << ' ' << shared->Value(i);
  std::cout << '\n';
}
int main()
{
  std::cout << std::setprecision(17);
  occ::handle<Geom_Surface> cylinder = new Geom_CylindricalSurface(gp::XOY(),1);
  report("cylinder-two",cylinder,0,2*M_PI,M_PI);
  report("cylinder-one",cylinder,0,1,2);
  report("cylinder-equal",cylinder,0,2,2);
  report("cylinder-three",cylinder,0,5,2);
  report("cone",new Geom_ConicalSurface(gp::XOY(),.5,1),0,5,2);
  report("sphere",new Geom_SphericalSurface(gp::XOY(),1),0,5,2);
  report("torus",new Geom_ToroidalSurface(gp::XOY(),3,1),0,5,2);
  report("revolution",new Geom_SurfaceOfRevolution(new Geom_Line(gp_Ax1(gp_Pnt(1,0,0),gp::DZ())),gp::OZ()),0,5,2);
  report("plane-retains",new Geom_Plane(gp::XOY()),0,5,2,8);
  occ::handle<Geom_Surface> trim = new Geom_RectangularTrimmedSurface(cylinder,0.,1.,0.,1.,true,true);
  report("trim-small-retains",trim,0,1,2,8);
  report("trim-equal",trim,0,1,1,8);
  report("trim-wide",new Geom_RectangularTrimmedSurface(cylinder,0.,2.,0.,1.,true,true),0,1,2,8);
  report("offset",new Geom_OffsetSurface(cylinder,.2),0,5,2);
  report("trim-offset",new Geom_RectangularTrimmedSurface(new Geom_OffsetSurface(cylinder,.2),0.,5.,0.,1.,true,true),0,5,2,8);
  report("negative-count",cylinder,0,5,-2);
}
