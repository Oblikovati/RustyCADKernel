#pragma once
#include <Geom2d_Curve.hxx>
#include <BOPTools_AlgoTools2D.hxx>
#include <BRepBuilderAPI_MakeFace.hxx>
#include <Geom_Plane.hxx>
#include <Geom_CylindricalSurface.hxx>
#include <Geom_ToroidalSurface.hxx>
#include <Geom_Line.hxx>
#include <Geom_Circle.hxx>
#include <Geom_TrimmedCurve.hxx>
#include <Geom_SurfaceOfRevolution.hxx>
#include <Geom_BezierSurface.hxx>
#include <NCollection_Array2.hxx>
#include <IntTools_Context.hxx>
#include <Standard_Failure.hxx>
#include <iostream>
#include <iomanip>

struct PCurveFixture{TopoDS_Face face;Handle(Geom_Curve) curve;double first,last;};
PCurveFixture pcurve_fixture(int kind){
  Handle(Geom_Surface) surface;Handle(Geom_Curve) curve;double first,last,umin,umax,vmin,vmax;
  if(kind==0){surface=new Geom_Plane(gp_Ax3());curve=new Geom_Line(gp_Pnt(-1,.5,0),gp_Dir(1,0,0));first=-2;last=3;umin=-2;umax=2;vmin=-2;vmax=2;}
  else if(kind==1){surface=new Geom_CylindricalSurface(gp_Ax3(),2.);curve=new Geom_Line(gp_Pnt(2,0,-1),gp_Dir(0,0,1));first=-1;last=3;umin=0;umax=2*M_PI;vmin=-2;vmax=2;}
  else if(kind==2){surface=new Geom_ToroidalSurface(gp_Ax3(),5.,2.);curve=new Geom_Circle(gp_Ax2(),7.);first=-.5;last=7.;umin=0;umax=2*M_PI;vmin=0;vmax=2*M_PI;}
  else if(kind==3){curve=new Geom_Line(gp_Pnt(2,0,-1),gp_Dir(0,0,1));surface=new Geom_SurfaceOfRevolution(curve,gp_Ax1(gp_Pnt(),gp_Dir(0,0,1)));first=-1;last=3;umin=0;umax=2*M_PI;vmin=-2;vmax=2;}
  else{NCollection_Array2<gp_Pnt> poles(1,2,1,2);for(int u=1;u<=2;u++)for(int v=1;v<=2;v++)poles(u,v)=gp_Pnt(u-1,v-1,0);surface=new Geom_BezierSurface(poles);curve=new Geom_Line(gp_Pnt(0,.5,0),gp_Dir(1,0,0));first=0;last=1;umin=0;umax=1;vmin=0;vmax=1;}
  auto face=BRepBuilderAPI_MakeFace(surface,umin,umax,vmin,vmax,1e-7).Face();
return {face,curve,first,last};
}
