// Geom2dHatch probe operations (tkgeomalgo_probe.cxx); the Rust twin is
// crates/tkgeomalgo/examples/ga_probe/geom2d_hatch.rs (hatcher: geom2d_hatcher.rs). Curves are
// read by ga_curve2d (tkgeomalgo_probe_geom2dint.cxx) followed by `u0 u1`: the adaptor is
// bounded to [u0, u1] when u0 < u1, else on the whole curve. The computations run under
// GaMute: the hatcher writes " Intersector -> Done = False " on std::cout, which the port
// does not.
#include <Geom2dHatch_Classifier.hxx>
#include <Geom2dHatch_Element.hxx>
#include <Geom2dHatch_Elements.hxx>
#include <Geom2dHatch_FClass2dOfClassifier.hxx>
#include <Geom2dHatch_Hatcher.hxx>
#include <Geom2dHatch_Hatching.hxx>
#include <Geom2dHatch_Intersector.hxx>
#include <HatchGen_Domain.hxx>
Geom2dAdaptor_Curve ga_hatch_adaptor(GaReader& r){
 occ::handle<Geom2d_Curve> c=ga_curve2d(r);double u0=r.real(),u1=r.real();
 if(u0<u1)return Geom2dAdaptor_Curve(c,u0,u1);return Geom2dAdaptor_Curve(c);
}
TopAbs_Orientation ga_orient(GaReader& r){return TopAbs_Orientation(r.integer());}
void ga_adaptor_out(const Geom2dAdaptor_Curve& a){
 ga_flag(!a.Curve().IsNull());if(a.Curve().IsNull())return;ga_real(a.FirstParameter());ga_real(a.LastParameter());
}
// `curve u0 u1 u`: LocalGeometry at `u`.
void ga_g2hatch_local_case(std::istream& input){
 GaReader r{input};
 GA_TRY(Geom2dAdaptor_Curve a=ga_hatch_adaptor(r);double u=r.real();Geom2dHatch_Intersector i(1e-7,1e-7);
  gp_Dir2d t,n;double c=-1.;i.LocalGeometry(a,u,t,n,c);ga_dir2d(t);ga_dir2d(n);ga_real(c));
 ga_end();
}
// `line p tol curve u0 u1`: Perform of the segment [0, p] of the line (p may be RealLast),
// then a copy (neither done nor reversed) and an assigned intersector.
void ga_g2hatch_perform_case(std::istream& input){
 GaReader r{input};
 GA_TRY(gp_Lin2d l=ga_read_lin2d(r);double p=r.real(),tol=r.real();Geom2dAdaptor_Curve a=ga_hatch_adaptor(r);
  Geom2dHatch_Intersector i;ga_real(i.ConfusionTolerance());ga_real(i.TangencyTolerance());i.SetReversedParameters(true);
  GA_TRY({GaMute m;i.Perform(l,p,tol,a);}ga_inter(i));
  Geom2dHatch_Intersector c(i);ga_inter(c);Geom2dHatch_Intersector s(1.,2.);s=i;ga_inter(s);
  ga_real(s.ConfusionTolerance());ga_real(s.TangencyTolerance()));
 ga_end();
}
// `curve1 u0 u1 curve2 v0 v1 conf tang reversed`: Intersect(C1, C2) with the tolerances set
// by the constructor, then again after the setters with ten times larger ones.
void ga_g2hatch_intersect_case(std::istream& input){
 GaReader r{input};
 GA_TRY(Geom2dAdaptor_Curve a1=ga_hatch_adaptor(r),a2=ga_hatch_adaptor(r);double conf=r.real(),tang=r.real();bool rev=r.flag();
  Geom2dHatch_Intersector i(conf,tang);i.SetReversedParameters(rev);GA_TRY({GaMute m;i.Intersect(a1,a2);}ga_inter(i));
  i.SetConfusionTolerance(conf*10.);i.SetTangencyTolerance(tang*10.);ga_real(i.ConfusionTolerance());ga_real(i.TangencyTolerance());
  GA_TRY({GaMute m;i.Intersect(a1,a2);}ga_inter(i)));
 ga_end();
}
// `n (key orientation curve u0 u1)*n`: the elements, bound in order.
void ga_read_elements(GaReader& r,Geom2dHatch_Elements& e,int& maxkey){
 int n=r.integer();maxkey=0;
 for(int k=0;k<n;k++){int key=r.integer();TopAbs_Orientation o=ga_orient(r);Geom2dAdaptor_Curve a=ga_hatch_adaptor(r);
  ga_flag(e.Bind(key,Geom2dHatch_Element(a,o)));if(key>maxkey)maxkey=key;}
}
void ga_segment_out(bool ok,const gp_Lin2d& l,double par){ga_flag(ok);ga_lin2d(l);ga_real(par);}
// The edge iteration: the current edge before InitEdges, every edge, the one past the end.
void ga_elements_edges(Geom2dHatch_Elements& e){
 Geom2dAdaptor_Curve c;TopAbs_Orientation o=TopAbs_FORWARD;
 e.InitWires();ga_flag(e.MoreWires());e.NextWire();ga_flag(e.MoreWires());
 e.InitEdges();for(;e.MoreEdges();e.NextEdge()){e.CurrentEdge(c,o);ga_adaptor_out(c);ga_int(o);}
 GA_TRY(e.CurrentEdge(c,o));e.NextEdge();ga_flag(e.MoreEdges());
}
// `elements px py nseg`: the explorer members, Segment and up to nseg OtherSegment, the
// map members by key, the copy constructor.
void ga_g2hatch_elements_case(std::istream& input){
 GaReader r{input};
 GA_TRY(Geom2dHatch_Elements e;int maxkey;
  {Geom2dAdaptor_Curve c;TopAbs_Orientation o=TopAbs_FORWARD;GA_TRY(e.CurrentEdge(c,o));}ga_flag(e.MoreEdges());
  ga_read_elements(r,e,maxkey);gp_Pnt2d p=r.pnt2d();int nseg=r.integer();ga_elements_edges(e);
  gp_Pnt2d q=p;ga_flag(e.CheckPoint(q));ga_pnt2d(q);ga_flag(e.Reject(p));
  gp_Lin2d l;double par=-1.;bool ok=e.Segment(p,l,par);ga_segment_out(ok,l,par);ga_flag(e.RejectWire(l,par));ga_flag(e.RejectEdge(l,par));
  for(int k=0;k<nseg&&ok;k++){ok=e.OtherSegment(p,l,par);ga_segment_out(ok,l,par);}
  for(int key=0;key<=maxkey+1;key++){ga_flag(e.IsBound(key));GA_TRY(ga_int(e.Find(key).Orientation());ga_adaptor_out(e(key).ChangeCurve()));}
  GA_TRY(e.ChangeFind(maxkey).Orientation(TopAbs_REVERSED);ga_int(e.Find(maxkey).Orientation()));
  Geom2dHatch_Elements c(e);ga_flag(c.IsBound(maxkey));ok=c.OtherSegment(p,l,par);ga_segment_out(ok,l,par);
  Geom2dHatch_Element d;ga_adaptor_out(d.Curve());GA_TRY(Geom2dHatch_Element f(e.Find(maxkey).Curve());ga_int(f.Orientation());
   f.Orientation(TopAbs_INTERNAL);ga_int(f.Orientation());ga_adaptor_out(f.ChangeCurve()));
  ga_flag(e.UnBind(maxkey));ga_flag(e.UnBind(maxkey));ga_elements_edges(e);e.Clear();ga_flag(e.IsBound(1));ga_elements_edges(e));
 ga_end();
}
// A unit square (FORWARD, counterclockwise) around (0.5, 0.5).
void ga_square_elements(Geom2dHatch_Elements& e){
 const double x[5]={0.,1.,1.,0.,0.},y[5]={0.,0.,1.,1.,0.};
 for(int k=0;k<4;k++){gp_Pnt2d a(x[k],y[k]),b(x[k+1],y[k+1]);occ::handle<Geom2d_Line> l=new Geom2d_Line(a,gp_Dir2d(gp_Vec2d(a,b)));
  e.Bind(k+1,Geom2dHatch_Element(Geom2dAdaptor_Curve(l,0.,1.),TopAbs_FORWARD));}
}
void ga_classifier_out(const Geom2dHatch_Classifier& c){
 ga_int(c.State());ga_flag(c.Rejected());ga_flag(c.NoWires());GA_TRY(ga_adaptor_out(c.Edge()));GA_TRY(ga_real(c.EdgeParameter()));
 ga_int(c.Position());
}
// `elements px py tol`: a classifier primed on the unit square around its centre (so that the
// edge position is defined natively), then the classification of the point; the default
// classifier; the constructor.
void ga_g2hatch_classify_case(std::istream& input){
 GaReader r{input};
 GA_TRY(Geom2dHatch_Elements e;int maxkey;ga_read_elements(r,e,maxkey);gp_Pnt2d p=r.pnt2d();double tol=r.real();
  Geom2dHatch_Classifier d;ga_int(d.State());ga_flag(d.Rejected());ga_flag(d.NoWires());GA_TRY(ga_real(d.EdgeParameter()));
  Geom2dHatch_Classifier c;Geom2dHatch_Elements square;ga_square_elements(square);
  {GaMute m;c.Perform(square,gp_Pnt2d(0.5,0.5),1e-7);}ga_classifier_out(c);
  GA_TRY({GaMute m;c.Perform(e,p,tol);}ga_classifier_out(c));
  GA_TRY(Geom2dHatch_Elements e2;ga_square_elements(e2);std::unique_ptr<Geom2dHatch_Classifier> k;
   {GaMute m;k.reset(new Geom2dHatch_Classifier(e2,p,tol));}ga_int(k->State());ga_flag(k->NoWires())));
 ga_end();
}
// `line par tol n (orientation curve u0 u1)*n`: FClass2d Reset then Compare with each edge.
void ga_g2hatch_fclass_case(std::istream& input){
 GaReader r{input};
 GA_TRY(gp_Lin2d l=ga_read_lin2d(r);double par=r.real(),tol=r.real();int n=r.integer();
  Geom2dHatch_FClass2dOfClassifier f;ga_real(f.Parameter());ga_int(f.ClosestIntersection());ga_int(f.State());ga_flag(f.IsHeadOrEnd());
  f.Reset(l,par,tol);ga_real(f.Parameter());ga_int(f.State());
  for(int k=0;k<n;k++){TopAbs_Orientation o=ga_orient(r);Geom2dAdaptor_Curve a=ga_hatch_adaptor(r);
   GA_TRY({GaMute m;f.Compare(a,o);}ga_int(f.ClosestIntersection());ga_int(f.State());ga_flag(f.IsHeadOrEnd());
    ga_real(f.Parameter());ga_flag(f.Intersector().IsDone()));});
 ga_end();
}
