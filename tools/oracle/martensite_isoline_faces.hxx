// Reusable real face fixtures for Martensite StdPrs/WF native oracles; OCCT3d097a0.
#include <BRepBuilderAPI_MakeFace.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRepBuilderAPI_MakeWire.hxx>
#include <Geom_Plane.hxx>
#include <Geom_CylindricalSurface.hxx>
#include <Geom_ConicalSurface.hxx>
#include <Geom_SphericalSurface.hxx>
#include <Geom_ToroidalSurface.hxx>
#include <Geom_BezierSurface.hxx>
#include <Geom_BSplineSurface.hxx>
#include <TopoDS_Wire.hxx>
#include <gp_Circ.hxx>
#include <Geom_RectangularTrimmedSurface.hxx>
#include <GeomConvert.hxx>
#include <NCollection_Array2.hxx>
#include <TopoDS_Face.hxx>
#include <gp_Trsf.hxx>
static occ::handle<Geom_Surface> isoline_bezier_surface() {
  NCollection_Array2<gp_Pnt> poles(1,3,1,3);
  for(int i=1;i<=3;++i)for(int j=1;j<=3;++j)poles(i,j)=gp_Pnt((i-1)*2,(j-1)*2,(i==2?2:0)+(j==2?1:0));
  return new Geom_BezierSurface(poles);
}
static occ::handle<Geom_Surface> isoline_fixture_surface(int kind) {
  const gp_Ax3 axes;
  if(kind==1||kind==5)return new Geom_CylindricalSurface(axes,3);
  if(kind==2)return new Geom_SphericalSurface(axes,3);
  if(kind==3)return new Geom_ToroidalSurface(axes,5,2);
  if(kind==4)return new Geom_ConicalSurface(axes,.4,3);
  if(kind==6)return isoline_bezier_surface();
  if(kind==7)return GeomConvert::SurfaceToBSplineSurface(isoline_bezier_surface());
  return new Geom_Plane(axes);
}
static TopoDS_Face isoline_annulus_face() {
  const auto outer=BRepBuilderAPI_MakeWire(BRepBuilderAPI_MakeEdge(gp_Circ(gp_Ax2(),4))).Wire();
  auto inner=BRepBuilderAPI_MakeWire(BRepBuilderAPI_MakeEdge(gp_Circ(gp_Ax2(),1))).Wire();inner.Reverse();
  BRepBuilderAPI_MakeFace builder(outer,true);builder.Add(inner);
  return builder.Face();
}
static TopoDS_Face isoline_fixture_face(int kind) {
  if(kind==9)return isoline_annulus_face();
  auto surface=isoline_fixture_surface(kind);
  double u0=-2,u1=3,v0=-1,v1=4;
  if(kind>=1&&kind<=5){u0=0;u1=1.6;v0=-2;v1=2;}
  if(kind==2||kind==3){v0=-1;v1=1;}
  if(kind==6||kind==7){u0=v0=0;u1=v1=1;}
  if(kind==8)surface=new Geom_RectangularTrimmedSurface(surface,u0,u1,v0,v1);
  auto face=BRepBuilderAPI_MakeFace(surface,u0,u1,v0,v1,1e-7).Face();
  if(kind==5){gp_Trsf transform;transform.SetTranslation(gp_Vec(7,-3,2));face.Location(TopLoc_Location(transform));}
  return face;
}
