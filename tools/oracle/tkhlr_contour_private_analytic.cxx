#include "tkhlr_probe_contap.cxx"
#include "tkhlr_probe_contap_search_boundary.cxx"
void ct_analytic_report(const Contap_Line& line,const occ::handle<Adaptor3d_TopolTool>& domain){
 hl_contap_line_geometry(line);ct_private_report(line);if(line.TypeContour()==Contap_Restriction)ga_int(ct_sob_arc_rank(domain,line.Arc()));
 for(int index=1;index<=line.NbVertex();++index){const auto& p=line.Vertex(index);hl_contap_point_snapshot(p);if(p.IsOnArc())ga_int(ct_sob_arc_rank(domain,p.Arc()));}
}
void ct_private_analytic(std::istream& input){
 GaReader r{input};GA_TRY(auto surface=hl_contap_surface(r);occ::handle<Adaptor3d_TopolTool> domain=new Adaptor3d_TopolTool(surface);
 int mode=r.integer();gp_Dir direction=r.dir();Contap_Contour contour;
 if(mode==0)contour.Init(gp_Vec(direction));else if(mode==1)contour.Init(gp_Pnt(3,4,5));else contour.Init(gp_Vec(direction),.2);
 for(int run=0;run<2;++run){GA_TRY(contour.Perform(surface,domain));ga_flag(contour.IsDone());if(contour.IsDone()){
 ga_int(contour.NbLines());for(int i=1;i<=contour.NbLines();++i)ct_analytic_report(contour.Line(i),domain);}});ga_end();
}
