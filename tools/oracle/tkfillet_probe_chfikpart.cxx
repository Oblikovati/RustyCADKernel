// The ChFiKPart helpers that run without the TKBool data structure.
#include <ChFiKPart_ComputeData_CS.hxx>
#include <ChFiKPart_ComputeData_Fcts.hxx>
#include <gp_Circ.hxx>
#include <gp_Cylinder.hxx>

// `fl_kpart_fcts u f l eps UV1 UV2 pardeb parfin GUIDE gf gl SURFACE`: ChFiKPart_InPeriod,
// ChFiKPart_PCurve and ChFiKPart_ProjPC of the guide on [gf, gl] on the surface.
void fl_kpart_fcts_case(std::istream& in){
 GaReader r{in};double u=r.real(),f=r.real(),l=r.real(),eps=r.real();gp_Pnt2d uv1=r.pnt2d(),uv2=r.pnt2d();double pd=r.real(),pf=r.real();
 occ::handle<Geom_Curve> c=fl_geom_guide(r);double gf=r.real(),gl=r.real();occ::handle<Geom_Surface> s=fl_geom_surface(r);
 ga_real(ChFiKPart_InPeriod(u,f,l,eps));
 GA_TRY(fl_curve2d(ChFiKPart_PCurve(uv1,uv2,pd,pf)));
 GA_TRY(occ::handle<Geom2d_Curve> pc;ChFiKPart_ProjPC(GeomAdaptor_Curve(c,gf,gl),GeomAdaptor_Surface(s),pc);fl_curve2d(pc));
 ga_end();}

// `fl_corner_spine PLANE SURFACE P1S1 P1S2 P2S2 R`: ChFiKPart_CornerSpine.
void fl_corner_spine_case(std::istream& in){
 GaReader r{in};occ::handle<Adaptor3d_Surface> s1=fl_surface(r),s2=fl_surface(r);gp_Pnt2d p1s1=r.pnt2d(),p1s2=r.pnt2d(),p2s2=r.pnt2d();double rad=r.real();
 GA_TRY(gp_Cylinder cyl;gp_Circ circ;double first=-1,last=-1;ChFiKPart_CornerSpine(s1,s2,p1s1,p1s1,p1s2,p2s2,rad,cyl,circ,first,last);
  ga_pnt(cyl.Location());ga_dir(cyl.Position().Direction());ga_dir(cyl.Position().XDirection());ga_dir(cyl.Position().YDirection());ga_real(cyl.Radius());
  ga_pnt(circ.Location());ga_dir(circ.Axis().Direction());ga_dir(circ.XAxis().Direction());ga_real(circ.Radius());ga_real(first);ga_real(last));
 ga_end();}
