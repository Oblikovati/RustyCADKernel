Contap_Line ct_walking_fixture(const occ::handle<Adaptor3d_Surface>& surface,int count,int path,bool seed){
 occ::handle<IntSurf_LineOn2S> points=new IntSurf_LineOn2S();
 for(int index=0;index<count;++index){double t=count==1?0.:-1.5+3.*index/(count-1);double u=path==1?M_PI/2:t,v=path==0?0.:path==1?t:t*.25;
 IntSurf_PntOn2S p;p.SetValue(surface->Value(u,v),17,19,u,v);points->Add(p);}
 Contap_Line line;line.SetLineOn2S(points);if(seed&&count>0){for(int index:{1,(count+1)/2,count}){
 const auto& source=line.Point(index);double u,v;source.ParametersOnS2(u,v);Contap_Point p(source.Value(),u,v);p.SetParameter(index);line.Add(p);}}return line;
}
void ct_private_walking_internal(std::istream& input){
 GaReader r{input};GA_TRY(auto surface=hl_contap_surface(r);int mode=r.integer();gp_Dir direction=r.dir();
 Contap_ArcFunction arc;arc.Set(surface);Contap_SurfFunction function;function.Set(surface);
 if(mode==0){arc.Set(direction);function.Set(direction);}else if(mode==1){gp_Pnt eye(3,4,5);arc.Set(eye);function.Set(eye);}
 else{arc.Set(direction,.2);function.Set(direction,.2);}function.Set(1e-7);
 int count=r.integer(),path=r.integer();bool seed=r.flag();Contap_Line line=ct_walking_fixture(surface,count,path,seed);
 ga_captured([&]{ComputeInternalPoints(line,function,1e-7,1e-7);});ct_private_report(line);ga_int(-1));ga_end();
}
