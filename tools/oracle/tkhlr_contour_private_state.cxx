// Observe all overloads and exception-sensitive state through the public native API.
void ct_state_report(Contap_Contour& c){
 ga_flag(c.IsDone());GA_TRY(ga_flag(c.IsEmpty()));GA_TRY(ga_int(c.NbLines()));GA_TRY(c.Line(0));
 GA_TRY(auto& f=c.SurfaceFunction();ga_int(f.FunctionType());ga_xyz(f.Direction().XYZ());ga_pnt(f.Eye()));
}
void ct_state_constructors(const occ::handle<Adaptor3d_Surface>& s,const occ::handle<Adaptor3d_TopolTool>& d,const gp_Vec& dir){
 gp_Pnt eye(3,4,5);Contap_Contour contours[]={Contap_Contour(dir),Contap_Contour(dir,.2),Contap_Contour(eye),Contap_Contour(s,d,dir),Contap_Contour(s,d,dir,.2),Contap_Contour(s,d,eye)};
 for(auto& c:contours){ct_state_report(c);GA_TRY(c.Perform(s,d));ct_state_report(c);}
}
void ct_state_mutate(Contap_Contour& c,const occ::handle<Adaptor3d_Surface>& s,const occ::handle<Adaptor3d_TopolTool>& d,const gp_Vec& dir){
 gp_Vec zero(0,0,0);gp_Pnt eye(3,4,5);
 GA_TRY(c.Perform(s,d,dir));ct_state_report(c);GA_TRY(c.Init(zero));ct_state_report(c);
 GA_TRY(c.Perform(s,d));ct_state_report(c);GA_TRY(c.Init(zero,.2));ct_state_report(c);
 GA_TRY(c.Perform(s,d,dir,.2));ct_state_report(c);GA_TRY(c.Perform(s,d,eye));ct_state_report(c);
 GA_TRY(c.SurfaceFunction().Set(gp_Pnt(0,0,0)));GA_TRY(c.Perform(s,d));ct_state_report(c);
}
void ct_private_state(std::istream& input){
 GaReader r{input};GA_TRY(auto surface=hl_contap_surface(r);occ::handle<Adaptor3d_TopolTool> domain=new Adaptor3d_TopolTool(surface);
 gp_Vec dir(r.dir());Contap_Contour contour;ct_state_report(contour);GA_TRY(contour.Perform(surface,domain));ct_state_report(contour);
 ct_state_constructors(surface,domain,dir);ct_state_mutate(contour,surface,domain,dir));ga_end();
}
