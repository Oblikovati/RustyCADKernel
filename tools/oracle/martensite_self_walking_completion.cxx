// SPDX-License-Identifier: GPL-2.0-only
// Finite inner walking witness, not a CheckerSI self-face completion certificate.
#include <GeomAdaptor_Surface.hxx>
#include <Geom_Plane.hxx>
#include <Geom_SphericalSurface.hxx>
#include <IntSurf_LineOn2S.hxx>
#include <IntWalk_PWalking.hxx>
#include <Standard_Failure.hxx>
#include <iostream>

static gp_Ax3 frame(double z) {
  return gp_Ax3(gp_Pnt(0, 0, z), gp_Dir(0, 0, 1));
}

static void tangent_witness() {
  Handle(Geom_Surface) sphere = new Geom_SphericalSurface(frame(0.3), 1.0);
  Handle(Geom_Surface) plane = new Geom_Plane(frame(1.3));
  Handle(Adaptor3d_Surface) first = new GeomAdaptor_Surface(sphere, 0, 2 * M_PI, -M_PI_2, M_PI_2);
  Handle(Adaptor3d_Surface) second = new GeomAdaptor_Surface(plane, -2, 2, -2, 2);
  NCollection_Array1<double> start(1, 4);
  start(1) = 0; start(2) = M_PI_2; start(3) = 0; start(4) = 0;
  IntWalk_PWalking walking(first, second, 1e-6, 1e-8, 0.01, 0.1);
  walking.Perform(start);
  std::cout << "tangent " << walking.IsDone();
  try { std::cout << ' ' << walking.NbPoints(); }
  catch (const Standard_Failure& failure) { std::cout << ' ' << failure.ExceptionType(); }
  std::cout << '\n';
}

int main() {
  tangent_witness();
  return 0;
}
