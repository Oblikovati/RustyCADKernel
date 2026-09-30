// Search outputs are real Contap instances; these helpers are from the included native source.
NCollection_Sequence<Contap_Line> ct_attachment_candidates(){
 NCollection_Sequence<Contap_Line> lines;
 for(gp_Pnt location:{gp_Pnt(0,2,0),gp_Pnt(0,-2,0),gp_Pnt(2,0,0)}){Contap_Line line;line.SetValue(gp_Lin(location,gp_Dir(0,0,1)));lines.Append(line);}
 for(gp_Dir direction:{gp_Dir(0,0,1),gp_Dir(1,0,0)}){Contap_Line line;line.SetValue(gp_Circ(gp_Ax2(gp_Pnt(),direction),2));lines.Append(line);}return lines;
}
void ct_attachment_transition(const IntSurf_Transition& value){
 GA_TRY(ga_flag(value.IsTangent()));ga_int(value.TransitionType());GA_TRY(ga_int(value.Situation()));GA_TRY(ga_flag(value.IsOpposite()));
}
void ct_attachment_report(const NCollection_Sequence<Contap_Line>& lines){
 ga_int(lines.Length());for(const auto& line:lines){ct_private_report(line);
 for(int index=1;index<=line.NbVertex();++index){const auto& p=line.Vertex(index);ga_flag(p.IsOnArc());ga_flag(p.IsVertex());ga_real(p.ParameterOnArc());
 ct_attachment_transition(p.TransitionOnLine());ct_attachment_transition(p.TransitionOnArc());if(p.IsVertex())ga_pnt2d(p.Vertex()->Value());}}
}
void ct_private_attachment(std::istream& input){
 GaReader r{input};GA_TRY(auto surface=hl_contap_surface(r);occ::handle<Adaptor3d_TopolTool> domain=new Adaptor3d_TopolTool(surface);
 int mode=r.integer();gp_Dir direction=r.dir();Contap_ArcFunction arc;arc.Set(surface);Contap_SurfFunction function;function.Set(surface);
 if(mode==0){arc.Set(direction);function.Set(direction);}else if(mode==1){gp_Pnt eye(3,4,5);arc.Set(eye);function.Set(eye);}
 else{arc.Set(direction,.2);function.Set(direction,.2);}function.Set(1e-7);
 Contap_TheSearch boundary;boundary.Perform(arc,domain,1e-7,1e-7,false);ga_flag(boundary.IsDone());ga_int(boundary.NbPoints());ga_int(boundary.NbSegments());
 auto lines=ct_attachment_candidates();GA_TRY(PutPointsOnLine(boundary,surface,lines));ct_attachment_report(lines);
 Contap_TheSearchInside inside(function,surface,domain,1e-7);ga_flag(inside.IsDone());ga_int(inside.NbPoints());
 NCollection_Sequence<IntSurf_InteriorPoint> output;GA_TRY(KeepInsidePoints(inside,boundary,function,output));ga_int(output.Length());
 for(const auto& point:output){double u,v;ga_pnt(point.Value());point.Parameters(u,v);ga_real(u);ga_real(v);ga_xyz(point.Direction().XYZ());ga_xy(point.Direction2d().XY());});ga_end();
}
