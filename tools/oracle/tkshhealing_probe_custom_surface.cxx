// ShapeCustom_Surface probe operations; the Rust twin is
// crates/tkshhealing/examples/sh_probe/custom_surface.rs.
//
// `sc_surface (surface) nops (op)...`: a ShapeCustom_Surface on the surface record of
// tkshhealing_probe_construct_package.cxx, then the ops in order on the same tool:
//  0 tol substitute: ConvertToAnalytical, then the result digest and Gap();
//  1 substitute preci: ConvertToPeriodic, then the result digest with its B-spline data.
// After the ops the digest of the input surface shows the native in-place SetUPeriodic.
#include <ShapeCustom_Surface.hxx>
void sc_surface_case(std::istream& input){
 GaReader r{input};occ::handle<Geom_Surface> s=scp_surface(r);ShapeCustom_Surface tool(s);
 int nops=r.integer();
 for(int k=0;k<nops;++k){int op=r.integer();
  if(op==0){double tol=r.real();bool sub=r.flag();
   GA_TRY(occ::handle<Geom_Surface> a=tool.ConvertToAnalytical(tol,sub);shc_surface(a))ga_real(tool.Gap());}
  else{bool sub=r.flag();double preci=r.real();
   GA_TRY(occ::handle<Geom_Surface> p=tool.ConvertToPeriodic(sub,preci);shc_surface(p);
    scp_bspline_surface(occ::down_cast<Geom_BSplineSurface>(p)))}}
 shc_surface(s);scp_bspline_surface(occ::down_cast<Geom_BSplineSurface>(s));ga_end();}
