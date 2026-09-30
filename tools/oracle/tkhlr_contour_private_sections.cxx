// Private LineConstructor on real surface domains; the original source is included by the wrapper.
void ct_private_walking(Contap_Line& line,bool duplicate,double offset){
 occ::handle<IntSurf_LineOn2S> points=new IntSurf_LineOn2S();
 for(int index=1;index<=7;++index){double x=duplicate?index/3:index;IntSurf_PntOn2S p;
 p.SetValue(gp_Pnt(x,0,0),17,19,offset+x*.1,offset);points->Add(p);}line.SetLineOn2S(points);
}
void ct_private_vertices(Contap_Line& line,int scenario){
 std::vector<double> parameters;
 switch(scenario){case 0:break;case 1:parameters={2};break;case 2:parameters={2,2};break;
 case 3:parameters={1,4,7};break;case 4:parameters={1,1.0000000005,4};break;default:parameters={-2,0,2};}
 for(double parameter:parameters){Contap_Point p(gp_Pnt(parameter,2,3),4,5);p.SetParameter(parameter);
 if(parameter==2){p.SetMultiple();p.SetInternal();}line.Add(p);}
}
void ct_private_report(const Contap_Line& line){
 ga_int(line.TypeContour());ga_int(line.TransitionOnS());ga_int(line.NbVertex());
 for(int index=1;index<=line.NbVertex();++index){const auto& p=line.Vertex(index);double u,v;
 ga_pnt(p.Value());ga_real(p.ParameterOnLine());p.Parameters(u,v);ga_real(u);ga_real(v);ga_flag(p.IsMultiple());ga_flag(p.IsInternal());}
 if(line.TypeContour()!=Contap_Walking)return;ga_int(line.NbPnts());
 for(int index=1;index<=line.NbPnts();++index){const auto& p=line.Point(index);double a,b,c,d;
 ga_pnt(p.Value());p.Parameters(a,b,c,d);ga_real(a);ga_real(b);ga_real(c);ga_real(d);}
}
void ct_private_sections(std::istream& input){
 GaReader r{input};GA_TRY(auto surface=hl_contap_surface(r);occ::handle<Adaptor3d_TopolTool> domain=new Adaptor3d_TopolTool(surface);
 int kind=r.integer(),scenario=r.integer();bool duplicate=r.flag();double offset=r.real();Contap_Line line;
 if(kind==0)ct_private_walking(line,duplicate,offset);else if(kind==1)line.SetValue(gp_Lin(gp_Pnt(2,0,0),gp_Dir(0,0,1)));
 else if(kind==2)line.SetValue(gp_Circ(gp_Ax2(),2));else{domain->Init();line.SetValue(domain->Value());}
 ct_private_vertices(line,scenario);line.SetTransitionOnS(IntSurf_Out);NCollection_Sequence<Contap_Line> output;
 GA_TRY(LineConstructor(output,domain,line,surface));ga_int(output.Length());
 for(int index=1;index<=output.Length();++index)ct_private_report(output(index));ct_private_report(line));ga_end();
}
