// Contap records at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#include <Adaptor2d_Curve2d.hxx>
#include <Contap_Point.hxx>
#include <Contap_Line.hxx>
#include <Geom2dAdaptor_Curve.hxx>
#include <Adaptor3d_HVertex.hxx>
void hl_contap_point_snapshot(const Contap_Point& p) {
 ga_pnt(p.Value()); double u,v;p.Parameters(u,v);ga_real(u);ga_real(v);ga_real(p.ParameterOnLine());
 ga_flag(p.IsMultiple());ga_flag(p.IsInternal());ga_flag(p.IsOnArc());ga_flag(p.IsVertex());
 GA_TRY(ga_flag(p.Arc().IsNull())); GA_TRY(ga_real(p.ParameterOnArc()));
 GA_TRY(ga_int(p.TransitionOnLine().TransitionType())); GA_TRY(ga_int(p.TransitionOnArc().TransitionType()));
 GA_TRY(ga_flag(p.Vertex().IsNull()));
}
void hl_contap_point_flags(Contap_Point& p, GaReader& r) {
 int mask=r.integer();bool null=r.flag();
 IntSurf_Transition t1(true,IntSurf_TypeTrans(r.integer())),t2(false,IntSurf_TypeTrans(r.integer()));
 occ::handle<Adaptor2d_Curve2d> arc;occ::handle<Adaptor3d_HVertex> vertex;
 if(!null){arc=new Geom2dAdaptor_Curve();vertex=new Adaptor3d_HVertex(gp_Pnt2d(1,2),TopAbs_FORWARD,1e-7);}
 double param=r.real();if(mask&1)p.SetArc(arc,param,t1,t2);if(mask&2)p.SetVertex(vertex);
 if(mask&4)p.SetMultiple();if(mask&8)p.SetInternal();
}
void hl_contap_point_case(std::istream& in) {
 GaReader r{in};int mode=r.integer();gp_Pnt position=r.pnt();double u=r.real(),v=r.real();
 Contap_Point p=mode==0?Contap_Point():Contap_Point(position,u,v);hl_contap_point_snapshot(p);
 p.SetParameter(r.real());hl_contap_point_flags(p,r);hl_contap_point_snapshot(p);
 Contap_Point copy=p;p.SetValue(position,u,v);hl_contap_point_snapshot(p);hl_contap_point_snapshot(copy);ga_end();
}
void hl_contap_line_geometry(const Contap_Line& l) {
 ga_int(l.TypeContour());ga_int(l.TransitionOnS());
 GA_TRY(gp_Lin a=l.Line();ga_pnt(a.Location());ga_dir(a.Direction()));
 GA_TRY(gp_Circ a=l.Circle();ga_pnt(a.Location());ga_dir(a.Position().Direction());ga_dir(a.Position().XDirection());ga_real(a.Radius()));
 GA_TRY(ga_flag(l.Arc().IsNull()));GA_TRY(ga_int(l.NbPnts()));GA_TRY(ga_pnt(l.Point(1).Value()));
}
void hl_contap_line_vertices(const Contap_Line& l) {
 ga_int(l.NbVertex());for(int i=1;i<=l.NbVertex();++i){const auto& v=l.Vertex(i);ga_real(v.ParameterOnLine());ga_pnt(v.Value());ga_flag(v.IsMultiple());}
 for(int i:{-1,0,l.NbVertex()+1}){GA_TRY(l.Vertex(i));}
}
void hl_contap_configure_line(Contap_Line& l,int mode) {
 if(mode==0)l.SetValue(gp_Lin(gp_Pnt(1,2,3),gp_Dir(1,0,0)));
 else if(mode==1)l.SetValue(gp_Circ(gp_Ax2(),5));
 else if(mode==2)l.SetValue(occ::handle<Adaptor2d_Curve2d>(new Geom2dAdaptor_Curve()));
 else l.SetLineOn2S(new IntSurf_LineOn2S());
}
void hl_contap_line_case(std::istream& in) {
 GaReader r{in};Contap_Line l;hl_contap_configure_line(l,r.integer());l.SetTransitionOnS(IntSurf_TypeTrans(r.integer()));
 if(l.TypeContour()==Contap_Walking)l.Add(IntSurf_PntOn2S());int count=r.integer();
 for(int i=0;i<count;++i){Contap_Point p(gp_Pnt(i,0,0),0,0);p.SetParameter(r.real());l.Add(p);}
 hl_contap_line_geometry(l);hl_contap_line_vertices(l);Contap_Line copy=l;
 if(copy.NbVertex()>0)copy.Vertex(1).SetMultiple();hl_contap_line_vertices(l);
 copy.ResetSeqOfVertex();ga_int(copy.NbVertex());hl_contap_line_vertices(l);
 Contap_Line shared=l;l.Clear();ga_int(l.NbVertex());hl_contap_line_vertices(shared);
 if(!shared.LineOn2S().IsNull())ga_int(shared.LineOn2S()->NbPoints());ga_end();
}
