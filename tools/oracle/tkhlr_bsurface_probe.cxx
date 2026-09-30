// HLRBRep_BSurfaceTool oracle, pinned OCCT 3d097a03; all library code unchanged.
#include <iostream>
#include <sstream>
#include <cstring>
#include <cstdint>
#include <HLRBRep_BSurfaceTool.hxx>
#include <BRep_Builder.hxx>
#include <TopoDS_Face.hxx>
#include <Geom_Plane.hxx>
#include <Geom_CylindricalSurface.hxx>
#include <Geom_ConicalSurface.hxx>
#include <Geom_SphericalSurface.hxx>
#include <Geom_ToroidalSurface.hxx>
#include <Geom_BezierSurface.hxx>
#include <Geom_BSplineSurface.hxx>
#include <Geom_Parabola.hxx>
#include <Geom_OffsetSurface.hxx>
#include <Geom_SurfaceOfLinearExtrusion.hxx>
#include <Geom_SurfaceOfRevolution.hxx>
#include "tkgeomalgo_probe_common.cxx"
using BST=HLRBRep_BSurfaceTool;
Handle(Geom_Surface) bst_spline(int kind){
 NCollection_Array2<gp_Pnt> p(1,9,1,9);NCollection_Array2<double>w(1,9,1,9);
 for(int i=1;i<=9;++i)for(int j=1;j<=9;++j){p(i,j)=gp_Pnt(i-1,j-1,i*j%3);w(i,j)=i==2&&j==3?2.:1.;}
 if(kind==5)return new Geom_BezierSurface(p);if(kind==6)return new Geom_BezierSurface(p,w);
 NCollection_Array1<double>k(1,7);NCollection_Array1<int>m(1,7);
 for(int i=1;i<=7;++i){k(i)=i-1;m(i)=i==1||i==7?4:1;}
 if(kind==7)return new Geom_BSplineSurface(p,k,k,m,m,3,3,false,false);
 return new Geom_BSplineSurface(p,w,k,k,m,m,3,3,false,false);
}
Handle(Geom_Surface) bst_geometry(int kind){
 gp_Ax3 a;
 if(kind==11)return new Geom_OffsetSurface(bst_spline(5),.5,false);
 switch(kind){case 0:return new Geom_Plane(a);case 1:return new Geom_CylindricalSurface(a,2.);
 case 2:return new Geom_ConicalSurface(a,.4,2.);case 3:return new Geom_SphericalSurface(a,2.);
 case 4:return new Geom_ToroidalSurface(a,3.,1.);case 5:case 6:case 7:case 8:return bst_spline(kind);}
 Handle(Geom_Curve)c=new Geom_Parabola(gp_Ax2(),1.);
 if(kind==9)return new Geom_SurfaceOfLinearExtrusion(c,gp_Dir(0,0,1));
 return new Geom_SurfaceOfRevolution(c,gp_Ax1(gp_Pnt(0,-2,0),gp_Dir(1,0,0)));
}
BRepAdaptor_Surface bst_surface(int kind,double shift){
 TopoDS_Face f;BRep_Builder b;b.MakeFace(f,bst_geometry(kind),1e-7);
 gp_Trsf tr;tr.SetTranslation(gp_Vec(shift,2*shift,-shift));f.Location(TopLoc_Location(tr));
 return BRepAdaptor_Surface(f,false);
}
void bst_metadata_bounds(const BRepAdaptor_Surface& s){
GA_TRY(ga_real(BST::FirstUParameter(s)));
GA_TRY(ga_real(BST::FirstVParameter(s)));
GA_TRY(ga_real(BST::LastUParameter(s)));
GA_TRY(ga_real(BST::LastVParameter(s)));
GA_TRY(ga_real(BST::UPeriod(s)));
GA_TRY(ga_real(BST::VPeriod(s)));
GA_TRY(ga_flag(BST::IsUClosed(s)));
GA_TRY(ga_flag(BST::IsVClosed(s)));
GA_TRY(ga_flag(BST::IsUPeriodic(s)));
GA_TRY(ga_flag(BST::IsVPeriodic(s)));
}
void bst_metadata_spline(const BRepAdaptor_Surface& s){
GA_TRY(ga_int(BST::UDegree(s)));
GA_TRY(ga_int(BST::VDegree(s)));
GA_TRY(ga_int(BST::NbUPoles(s)));
GA_TRY(ga_int(BST::NbVPoles(s)));
GA_TRY(ga_int(BST::NbUKnots(s)));
GA_TRY(ga_int(BST::NbVKnots(s)));
GA_TRY(ga_flag(BST::IsURational(s)));
GA_TRY(ga_flag(BST::IsVRational(s)));
}
void bst_metadata_kind(const BRepAdaptor_Surface& s){
GA_TRY(ga_int(BST::GetType(s)));
GA_TRY(ga_int(BST::UContinuity(s)));
GA_TRY(ga_int(BST::VContinuity(s)));
}
void bst_evaluation(const BRepAdaptor_Surface&s,double u,double v){
GA_TRY(ga_pnt(BST::Value(s,u,v)));GA_TRY(gp_Pnt p;BST::D0(s,u,v,p);ga_pnt(p));
GA_TRY(gp_Pnt p;gp_Vec d1u,d1v;BST::D1(s,u,v,p,d1u,d1v);ga_pnt(p);ga_xyz(d1u.XYZ());ga_xyz(d1v.XYZ()););
GA_TRY(gp_Pnt p;gp_Vec d1u,d1v,d2u,d2v,d2uv;BST::D2(s,u,v,p,d1u,d1v,d2u,d2v,d2uv);ga_pnt(p);ga_xyz(d1u.XYZ());ga_xyz(d1v.XYZ());ga_xyz(d2u.XYZ());ga_xyz(d2v.XYZ());ga_xyz(d2uv.XYZ()););
GA_TRY(gp_Pnt p;gp_Vec d1u,d1v,d2u,d2v,d2uv,d3u,d3v,d3uuv,d3uvv;BST::D3(s,u,v,p,d1u,d1v,d2u,d2v,d2uv,d3u,d3v,d3uuv,d3uvv);ga_pnt(p);ga_xyz(d1u.XYZ());ga_xyz(d1v.XYZ());ga_xyz(d2u.XYZ());ga_xyz(d2v.XYZ());ga_xyz(d2uv.XYZ());ga_xyz(d3u.XYZ());ga_xyz(d3v.XYZ());ga_xyz(d3uuv.XYZ());ga_xyz(d3uvv.XYZ()););
for(auto orders:{std::pair<int,int>{0,0},{1,0},{0,1},{2,1},{1,3}}){GA_TRY(ga_xyz(BST::DN(s,u,v,orders.first,orders.second).XYZ()));}
for(double t:{-1.,0.,1e-7,2.}){GA_TRY(ga_real(BST::UResolution(s,t)));GA_TRY(ga_real(BST::VResolution(s,t)));}}
void bst_geometry_queries(const BRepAdaptor_Surface&s){
GA_TRY(auto v=BST::Plane(s);ga_pnt(v.Location()));
GA_TRY(auto v=BST::Cylinder(s);ga_real(v.Radius()));
GA_TRY(auto v=BST::Cone(s);ga_real(v.SemiAngle()));
GA_TRY(auto v=BST::Sphere(s);ga_real(v.Radius()));
GA_TRY(auto v=BST::Torus(s);ga_real(v.MajorRadius()));
GA_TRY(auto v=BST::AxeOfRevolution(s);ga_pnt(v.Location());ga_xyz(v.Direction().XYZ()));
GA_TRY(ga_xyz(BST::Direction(s).XYZ()));GA_TRY(ga_pnt(BST::BasisCurve(s)->Value(.5)));GA_TRY(ga_pnt(BST::Bezier(s)->Value(.5,.5)));GA_TRY(ga_pnt(BST::BSpline(s)->Value(.5,.5)));}
void bst_intervals(const BRepAdaptor_Surface&s){for(auto shape:{GeomAbs_C0,GeomAbs_C1,GeomAbs_C2,GeomAbs_C3,GeomAbs_CN}){
GA_TRY(int n=BST::NbUIntervals(s,shape);ga_int(n);NCollection_Array1<double>b(1,n+1);BST::UIntervals(s,b,shape);for(double v:b)ga_real(v));
GA_TRY(int n=BST::NbVIntervals(s,shape);ga_int(n);NCollection_Array1<double>b(1,n+1);BST::VIntervals(s,b,shape);for(double v:b)ga_real(v));}}
void bst_trims(const BRepAdaptor_Surface&s,double first,double last){
for(auto trim:{BST::UTrim,BST::VTrim}){GA_TRY(auto t=trim(s,first,last,1e-7);ga_real(t->FirstUParameter());ga_real(t->LastUParameter());ga_real(t->FirstVParameter());ga_real(t->LastVParameter());ga_pnt(t->Value(.5,.5)));}}
void bst_case(std::istream& input){GaReader r{input};GA_TRY(int kind=r.integer();double shift=r.real(),u=r.real(),v=r.real(),first=r.real(),last=r.real();auto s=bst_surface(kind,shift);
bst_metadata_bounds(s);bst_metadata_spline(s);bst_metadata_kind(s);bst_evaluation(s,u,v);bst_geometry_queries(s);bst_intervals(s);bst_trims(s,first,last);
GA_TRY(ga_int(BST::NbSamplesU(s)));GA_TRY(ga_int(BST::NbSamplesV(s)));GA_TRY(ga_int(BST::NbSamplesU(s,first,last)));GA_TRY(ga_int(BST::NbSamplesV(s,first,last))));ga_end();}
int main(){std::string line;while(std::getline(std::cin,line)){std::istringstream input(line);std::string operation;input>>operation;
if(operation.empty())continue;if(operation!="bsurface"){std::cerr<<"Unknown probe operation: "<<operation<<std::endl;return 2;}bst_case(input);}}
