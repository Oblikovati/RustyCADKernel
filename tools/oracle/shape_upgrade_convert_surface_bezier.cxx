#include <Geom_Plane.hxx>
#include <Geom_BezierSurface.hxx>
#include <Geom_BSplineSurface.hxx>
#include <Geom_OffsetSurface.hxx>
#include <Geom_Circle.hxx>
#include <Geom_Line.hxx>
#include <Geom_SurfaceOfLinearExtrusion.hxx>
#include <Geom_SurfaceOfRevolution.hxx>
#include <Geom_RectangularTrimmedSurface.hxx>
#include <NCollection_Array2.hxx>
#include <NCollection_Array1.hxx>
#include <NCollection_HSequence.hxx>
#include <ShapeExtend_CompositeSurface.hxx>
#include <ShapeUpgrade_ConvertSurfaceToBezierBasis.hxx>
#include <gp_Ax3.hxx>
#include <gp_Ax1.hxx>
#include <gp_Ax2.hxx>
#include <gp_Dir.hxx>
#include <gp_Lin.hxx>
#include <gp_Pnt.hxx>
#include <iostream>

static void dump(const char* name, const occ::handle<Geom_Surface>& surface,
                 double u1, double u2, double v1, double v2, bool split = false)
{
  occ::handle<ShapeUpgrade_ConvertSurfaceToBezierBasis> converter =
    new ShapeUpgrade_ConvertSurfaceToBezierBasis;
  converter->Init(surface, u1, u2, v1, v2);
  if (split)
  {
    occ::handle<NCollection_HSequence<double>> uValues = new NCollection_HSequence<double>;
    uValues->Append(0.25); uValues->Append(0.75);
    converter->SetUSplitValues(uValues);
  }
  converter->Compute(true);
  converter->Build(false);
  occ::handle<ShapeExtend_CompositeSurface> segments = converter->Segments();
  occ::handle<ShapeExtend_CompositeSurface> result = converter->ResSurfaces();
  double pu1, pu2, pv1, pv2;
  segments->Patch(1, 1)->Bounds(pu1, pu2, pv1, pv2);
  std::cout << name << " status=" << converter->Status(ShapeExtend_DONE1) << ","
            << converter->Status(ShapeExtend_DONE2) << " segments=" << segments->NbUPatches()
            << "x" << segments->NbVPatches() << " patch=" << segments->Patch(1, 1)->DynamicType()->Name()
            << " patch_bounds=" << pu1 << "," << pu2 << "," << pv1 << "," << pv2
            << " result=" << result->NbUPatches() << "x" << result->NbVPatches()
            << " result_patch=" << result->Patch(1, 1)->DynamicType()->Name() << "\n";
}

int main()
{
  occ::handle<Geom_Surface> plane = new Geom_Plane(gp_Ax3());
  dump("plane", plane, -2., 3., -1., 4.);
  NCollection_Array2<gp_Pnt> poles(1, 2, 1, 2);
  poles(1,1)=gp_Pnt(0.,0.,0.); poles(1,2)=gp_Pnt(0.,1.,0.);
  poles(2,1)=gp_Pnt(1.,0.,0.); poles(2,2)=gp_Pnt(1.,1.,1.);
  occ::handle<Geom_Surface> bezier = new Geom_BezierSurface(poles);
  dump("bezier", bezier, .2, .8, .1, .9);
  occ::handle<Geom_Surface> trimmed = new Geom_RectangularTrimmedSurface(plane,-1.,2.,-3.,1.);
  dump("trimmed", trimmed, -1., 2., -3., 1.);
  occ::handle<Geom_Surface> offset = new Geom_OffsetSurface(plane, 2.);
  dump("offset", offset, -1., 2., -3., 1.);
  NCollection_Array2<gp_Pnt> poles3(1,3,1,2);
  for (int i=1;i<=3;i++) for (int j=1;j<=2;j++) poles3(i,j)=gp_Pnt(i-1.,j-1.,i*j*.1);
  NCollection_Array1<double> uk(1,3), vk(1,2);
  NCollection_Array1<int> um(1,3), vm(1,2);
  uk(1)=0.; uk(2)=.5; uk(3)=1.; um(1)=2; um(2)=1; um(3)=2;
  vk(1)=0.; vk(2)=1.; vm(1)=2; vm(2)=2;
  occ::handle<Geom_Surface> spline = new Geom_BSplineSurface(poles3,uk,vk,um,vm,1,1);
  dump("bspline", spline, 0., 1., 0., 1., true);
  occ::handle<Geom_Circle> circle = new Geom_Circle(gp_Ax2(gp_Pnt(2.,0.,0.),gp_Dir(0.,1.,0.)),1.);
  occ::handle<Geom_Surface> revolution = new Geom_SurfaceOfRevolution(circle,gp_Ax1(gp_Pnt(),gp_Dir(0.,0.,1.)));
  double ru1,ru2,rv1,rv2; revolution->Bounds(ru1,ru2,rv1,rv2);
  dump("revolution", revolution, ru1, ru2, rv1, rv2);
  occ::handle<Geom_Line> line = new Geom_Line(gp_Lin(gp_Pnt(),gp_Dir(1.,0.,0.)));
  occ::handle<Geom_Surface> extrusion = new Geom_SurfaceOfLinearExtrusion(line,gp_Dir(0.,0.,1.));
  dump("extrusion", extrusion, -2., 3., -1., 2.);
  return 0;
}
