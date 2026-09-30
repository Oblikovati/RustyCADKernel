void ct_internal_seed(Contap_Line& line,double first,double last,int scenario){
 std::vector<double> parameters;
 if(scenario==1)parameters={(first+last)/2};else if(scenario==2)parameters={first,last};
 else if(scenario==3)parameters={first,(first+last)/2,(first+last)/2,last};
 for(double parameter:parameters){Contap_Point point(gp_Pnt(1,2,3),4,5);point.SetParameter(parameter);line.Add(point);}
}
void ct_private_internal(std::istream& input){
 GaReader r{input};GA_TRY(auto surface=hl_contap_surface(r);occ::handle<Adaptor3d_TopolTool> domain=new Adaptor3d_TopolTool(surface);
 int mode=r.integer();gp_Dir direction=r.dir();Contap_ArcFunction arcfunc;arcfunc.Set(surface);Contap_SurfFunction function;function.Set(surface);
 if(mode==0){arcfunc.Set(direction);function.Set(direction);}else if(mode==1){gp_Pnt eye(3,4,5);arcfunc.Set(eye);function.Set(eye);}
 else{arcfunc.Set(direction,.2);function.Set(direction,.2);}function.Set(1e-7);
 int scenario=r.integer();domain->Init();while(domain->More()){
 auto arc=domain->Value();double first=Contap_HCurve2dTool::FirstParameter(arc),last=Contap_HCurve2dTool::LastParameter(arc);
 Contap_Line line;line.SetValue(arc);ct_internal_seed(line,first,last,scenario);
 GA_TRY(ComputeInternalPointsOnRstr(line,first,last,function));ct_private_report(line);domain->Next();}ga_int(-1));ga_end();
}
