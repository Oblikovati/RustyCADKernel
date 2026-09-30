#include <Contap_TheSearch.hxx>
int ct_sob_arc_rank(const occ::handle<Adaptor3d_TopolTool>& domain,const occ::handle<Adaptor2d_Curve2d>& arc){
 int rank=0;for(domain->Init();domain->More();domain->Next()){++rank;if(domain->Value()==arc)return rank;}return 0;}
void ct_sob_path_point(const occ::handle<Adaptor3d_TopolTool>& d,const Contap_ThePathPointOfTheSearch& p){
 ga_pnt(p.Value());ga_real(p.Tolerance());ga_flag(p.IsNew());ga_real(p.Parameter());ga_int(ct_sob_arc_rank(d,p.Arc()));
 GA_TRY(const occ::handle<Adaptor3d_HVertex>& v=p.Vertex();ga_pnt2d(v->Value());ga_real(v->Resolution(p.Arc())));
}
template<class S> void ct_sob_sob_report(const S& s,const occ::handle<Adaptor3d_TopolTool>& d){
 ga_flag(s.IsDone());GA_TRY(ga_flag(s.AllArcSolution()));
 GA_TRY(int n=s.NbPoints();ga_int(n);for(int i=1;i<=n;++i)ct_sob_path_point(d,s.Point(i)));
 GA_TRY(int n=s.NbSegments();ga_int(n);for(int i=1;i<=n;++i){const Contap_TheSegmentOfTheSearch& g=s.Segment(i);
  ga_int(ct_sob_arc_rank(d,g.Curve()));ga_flag(g.HasFirstPoint());GA_TRY(ct_sob_path_point(d,g.FirstPoint()));
  ga_flag(g.HasLastPoint());GA_TRY(ct_sob_path_point(d,g.LastPoint()));});
 GA_TRY(s.Point(0));GA_TRY(s.Point(s.NbPoints()+1));GA_TRY(s.Segment(0));GA_TRY(s.Segment(s.NbSegments()+1));
}
void ct_boundary_case(std::istream& input){
 GaReader r{input};GA_TRY(auto surface=hl_contap_surface(r);int mode=r.integer(),kind=r.integer();bool recheck=r.flag();
 gp_Dir direction=r.dir();gp_Pnt eye=r.pnt();double angle=r.real();Contap_ArcFunction function;function.Set(surface);
 if(kind==0)function.Set(direction);else if(kind==1)function.Set(eye);else function.Set(direction,angle);
 occ::handle<Adaptor3d_TopolTool> domain=new Adaptor3d_TopolTool(surface);Contap_TheSearch search;ct_sob_sob_report(search,domain);
 for(int i=0;i<mode;++i){GA_TRY(search.Perform(function,domain,1e-7,1e-7,recheck));ct_sob_sob_report(search,domain);});ga_end();
}
