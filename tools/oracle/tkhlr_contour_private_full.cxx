// Full public Perform exercises the general branch and its native console diagnostics.
void ct_private_full(std::istream& input){
 GaReader r{input};GA_TRY(auto surface=hl_contap_surface(r);occ::handle<Adaptor3d_TopolTool> domain=new Adaptor3d_TopolTool(surface);
 int mode=r.integer();gp_Dir direction=r.dir();Contap_Contour contour;
 if(mode==0)contour.Init(gp_Vec(direction));else if(mode==1)contour.Init(gp_Pnt(3,4,5));else contour.Init(gp_Vec(direction),.2);
 for(int run=0;run<2;++run){ga_captured([&]{contour.Perform(surface,domain);});ga_flag(contour.IsDone());if(contour.IsDone()){
 ga_int(contour.NbLines());for(int i=1;i<=contour.NbLines();++i)ct_analytic_report(contour.Line(i),domain);}});ga_end();
}
