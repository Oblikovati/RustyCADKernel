void ct_segment_seed(NCollection_Sequence<Contap_Line>& lines,const Contap_TheSearch& search){
 for(int index=1;index<=search.NbSegments();++index){const auto& segment=search.Segment(index);if(!segment.HasFirstPoint())continue;
 const auto& start=segment.FirstPoint();Contap_Point p(start.Value(),3,4);p.SetParameter(start.Parameter());
 lines(1).Add(p);lines(1).Add(p);lines(2).Add(p);}
}
void ct_private_segments(std::istream& input){
 GaReader r{input};GA_TRY(auto surface=hl_contap_surface(r);occ::handle<Adaptor3d_TopolTool> domain=new Adaptor3d_TopolTool(surface);
 int mode=r.integer();gp_Dir direction=r.dir();Contap_ArcFunction arc;arc.Set(surface);Contap_SurfFunction function;function.Set(surface);
 if(mode==0){arc.Set(direction);function.Set(direction);}else if(mode==1){gp_Pnt eye(3,4,5);arc.Set(eye);function.Set(eye);}
 else{arc.Set(direction,.2);function.Set(direction,.2);}function.Set(1e-7);
 bool seeded=r.flag();double tolerance=r.real();Contap_TheSearch search;search.Perform(arc,domain,1e-7,1e-7,false);
 ga_flag(search.IsDone());ga_int(search.NbSegments());auto lines=ct_attachment_candidates();if(seeded)ct_segment_seed(lines,search);
 GA_TRY(ProcessSegments(search,lines,tolerance,function,domain));for(const auto& line:lines)ct_private_report(line);ga_int(-1));ga_end();
}
