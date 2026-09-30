#include <Contap_TheSearchInside.hxx>
#include <Adaptor3d_TopolTool.hxx>
void ct_inside_report(const Contap_TheSearchInside& s){
 ga_flag(s.IsDone());
 GA_TRY(int n=s.NbPoints();ga_int(n);for(int i=1;i<=n;++i){const IntSurf_InteriorPoint& p=s.Value(i);
  ga_pnt(p.Value());ga_real(p.UParameter());ga_real(p.VParameter());ga_xyz(p.Direction().XYZ());ga_xy(p.Direction2d().XY());});
 GA_TRY(s.Value(0));GA_TRY(s.Value(s.NbPoints()+1));
}
void ct_inside_case(std::istream& input){
 GaReader r{input};GA_TRY(auto surface=hl_contap_surface(r);int mode=r.integer(),kind=r.integer();double u=r.real(),v=r.real();
 Contap_SurfFunction function;function.Set(surface);gp_Dir direction=r.dir();gp_Pnt eye=r.pnt();double angle=r.real();
 if(kind==0)function.Set(direction);else if(kind==1)function.Set(eye);else function.Set(direction,angle);
 function.Set(1e-7);occ::handle<Adaptor3d_TopolTool> domain=new Adaptor3d_TopolTool(surface);Contap_TheSearchInside search;
 if(mode==0){GA_TRY(search=Contap_TheSearchInside(function,surface,domain,1e-7));}
 if(mode==1){GA_TRY(search.Perform(function,surface,u,v));}
 if(mode==3){GA_TRY(search.Perform(function,surface,domain,1e-7));}
 ct_inside_report(search);
 if(mode!=2){GA_TRY(search.Perform(function,surface,u,v));ct_inside_report(search);});ga_end();
}
