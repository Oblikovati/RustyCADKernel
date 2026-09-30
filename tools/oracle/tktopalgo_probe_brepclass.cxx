// BRepClass probe operations over faces read by tp_face (tktopalgo_probe_face.cxx); the
// Rust twin is crates/tktopalgo/examples/tp_probe/brep_class.rs. Edges are printed as
// their index among the edges of the FORWARD face (tp_edge_index).
#include <BRep_Tool.hxx>
#include <BRepClass_Edge.hxx>
#include <BRepClass_FClass2dOfFClassifier.hxx>
#include <BRepClass_FClassifier.hxx>
#include <BRepClass_FaceClassifier.hxx>
#include <BRepClass_FaceExplorer.hxx>
#include <BRepClass_FacePassiveClassifier.hxx>
#include <BRepClass_Intersector.hxx>
#include <Precision.hxx>
#include <gp_Lin2d.hxx>
// The state of a BRepClass_Edge: edge, face, next edge, pcurve range, box and switches.
void tp_edge_data(const TpFace& f,const BRepClass_Edge& e){
 ga_int(tp_edge_index(f.face,e.Edge()));ga_flag(e.Face().IsNull());
 if(!e.Face().IsNull()){ga_flag(e.Face().IsSame(f.face));ga_int(e.Face().Orientation());}
 ga_int(tp_edge_index(f.face,e.NextEdge()));ga_flag(e.Curve().IsNull());
 ga_real(e.FirstParameter());ga_real(e.LastParameter());ga_int(int(e.BoundingBoxState()));
 ga_flag(e.BoundingBox().IsVoid());
 if(!e.BoundingBox().IsVoid()){double a,b,c,d;e.BoundingBox().Get(a,b,c,d);ga_real(a);ga_real(b);ga_real(c);ga_real(d);}
 ga_real(e.MaxTolerance());ga_flag(e.UseBndBox());}
// One BRepClass_Edge command: 0 SetGeometry (the pcurve of edge k), 1 SetBoundingBox x0 y0
// x1 y1, 2 SetBoundingBox (void), 3 SetBoundingBoxUnavailable, 4 SetEdge j o, 5 SetFace o,
// 6 SetNextEdge j, 7 SetMaxTolerance t, 8 SetUseBndBox b.
void tp_edge_command(GaReader& r,const TpFace& f,BRepClass_Edge& e,int k){
 int code=r.integer();
 if(code==0){double a,b;occ::handle<Geom2d_Curve> c=BRep_Tool::CurveOnSurface(f.edges.at(k),f.face,a,b);e.SetGeometry(c,a,b);}
 else if(code==1){double x0=r.real(),y0=r.real(),x1=r.real(),y1=r.real();Bnd_Box2d box;box.Update(x0,y0,x1,y1);e.SetBoundingBox(box);}
 else if(code==2)e.SetBoundingBox(Bnd_Box2d());
 else if(code==3)e.SetBoundingBoxUnavailable();
 else if(code==4){int j=r.integer();e.SetEdge(TopoDS::Edge(f.edges.at(j).Oriented(TopAbs_Orientation(r.integer()))));}
 else if(code==5)e.SetFace(TopoDS::Face(f.face.Oriented(TopAbs_Orientation(r.integer()))));
 else if(code==6)e.SetNextEdge(f.edges.at(r.integer()));
 else if(code==7)e.SetMaxTolerance(r.real());
 else e.SetUseBndBox(r.flag());}
// `face k mode n (command)...`: a BRepClass_Edge (0 default, 1 edge k of the face), its
// state, then its state after each command.
void tp_brc_edge_case(std::istream& input){
 GaReader r{input};
 GA_TRY(TpFace f=tp_face(r);int k=r.integer();BRepClass_Edge e;
  if(r.flag())e=BRepClass_Edge(f.edges.at(k),f.face);
  tp_edge_data(f,e);int n=r.integer();
  for(int i=0;i<n;++i){tp_edge_command(r,f,e,k);tp_edge_data(f,e);})
 ga_end();}
// The edges of an explorer in wire order: orientation, then the edge state.
void tp_explorer_edges(const TpFace& f,BRepClass_FaceExplorer& x){
 for(x.InitWires();x.MoreWires();x.NextWire()){
  ga_flag(x.RejectWire(gp_Lin2d(),1.));
  for(x.InitEdges();x.MoreEdges();x.NextEdge()){
   BRepClass_Edge e;TopAbs_Orientation o=TopAbs_EXTERNAL;x.CurrentEdge(e,o);
   ga_int(o);ga_flag(x.RejectEdge(gp_Lin2d(),1.));tp_edge_data(f,e);}}}
// `face gaptol usebox`: the explorer's switches, ShouldUseBndBox, then its edges, and
// its edges again after boxes are switched off and on.
void tp_brc_explorer_case(std::istream& input){
 GaReader r{input};
 GA_TRY(TpFace f=tp_face(r);BRepClass_FaceExplorer x(f.face);double gap=r.real();bool box=r.flag();
  ga_real(x.MaxTolerance());ga_flag(x.UseBndBox());ga_flag(x.Reject(gp_Pnt2d()));
  x.SetMaxTolerance(gap);ga_flag(x.ShouldUseBndBox());x.SetUseBndBox(box);ga_flag(x.UseBndBox());
  x.SetUseBndBox(box);tp_explorer_edges(f,x);
  x.SetUseBndBox(false);ga_flag(x.UseBndBox());x.SetUseBndBox(true);tp_explorer_edges(f,x))
 ga_end();}
void tp_line(bool found,const gp_Lin2d& l,double par){
 ga_flag(found);ga_pnt2d(l.Location());ga_xy(l.Direction().XY());ga_real(par);}
// `face u v n`: Segment, then n-1 OtherSegment from (u, v).
void tp_brc_segment_case(std::istream& input){
 GaReader r{input};
 GA_TRY(TpFace f=tp_face(r);BRepClass_FaceExplorer x(f.face);gp_Pnt2d p=r.pnt2d();int n=r.integer();
  gp_Lin2d l;double par=-1.;bool ok=x.Segment(p,l,par);tp_line(ok,l,par);
  for(int i=1;i<n;++i){ok=x.OtherSegment(p,l,par);tp_line(ok,l,par);})
 ga_end();}
// `face u v`: CheckPoint until it accepts the point (at most 4 calls), with the point.
void tp_brc_checkpoint_case(std::istream& input){
 GaReader r{input};
 GA_TRY(TpFace f=tp_face(r);BRepClass_FaceExplorer x(f.face);gp_Pnt2d p=r.pnt2d();
  for(int i=0;i<4;++i){bool ok=x.CheckPoint(p);ga_flag(ok);ga_pnt2d(p);if(ok)break;})
 ga_end();}
// The explorer edge of rank k in wire order, with the explorer's switches.
BRepClass_Edge tp_explorer_edge(BRepClass_FaceExplorer& x,int k){
 int i=0;
 for(x.InitWires();x.MoreWires();x.NextWire())
  for(x.InitEdges();x.MoreEdges();x.NextEdge(),++i)
   if(i==k){BRepClass_Edge e;TopAbs_Orientation o;x.CurrentEdge(e,o);return e;}
 return BRepClass_Edge();}
// The tangent, normal and curvature of LocalGeometry at u.
void tp_local(const BRepClass_Intersector& inter,const BRepClass_Edge& e,double u){
 gp_Dir2d t,n;double c=-1.;
 GA_TRY(inter.LocalGeometry(e,u,t,n,c);ga_xy(t.XY());ga_xy(n.XY());ga_real(c))}
// Box modes before the Perform: 0 as the explorer gives it, 1 unavailable, 2 geometry set
// again (NotBuilt), 3 edge set again (no pcurve cached).
void tp_box_mode(BRepClass_Edge& e,int mode){
 if(mode==1)e.SetBoundingBoxUnavailable();
 else if(mode==2){occ::handle<Geom2d_Curve> c=e.Curve();e.SetGeometry(c,e.FirstParameter(),e.LastParameter());}
 else if(mode==3)e.SetEdge(e.Edge());}
// `face k P D par tol gaptol usebox mode`: Perform of the line (P, D) on the explorer edge
// k, the results, the edge's box state, then LocalGeometry at the ends and middle.
void tp_brc_intersect_case(std::istream& input){
 GaReader r{input};
 GA_TRY(TpFace f=tp_face(r);BRepClass_FaceExplorer x(f.face);int k=r.integer();
  gp_Pnt2d p=r.pnt2d();gp_Dir2d d=r.dir2d();double par=r.real(),tol=r.real();
  x.SetMaxTolerance(r.real());x.SetUseBndBox(r.flag());BRepClass_Edge e=tp_explorer_edge(x,k);
  tp_box_mode(e,r.integer());BRepClass_Intersector inter;
  GA_TRY(inter.Perform(gp_Lin2d(p,d),par,tol,e);ga_inter(inter))
  ga_int(int(e.BoundingBoxState()));
  // Without a pcurve LocalGeometry dereferences a null curve natively (a crash).
  double a,b;bool has_pcurve=!BRep_Tool::CurveOnSurface(e.Edge(),e.Face(),a,b).IsNull();
  ga_flag(has_pcurve);a=e.FirstParameter();b=e.LastParameter();
  if(has_pcurve){tp_local(inter,e,a);tp_local(inter,e,0.5*(a+b));tp_local(inter,e,b);})
 ga_end();}
// `face k1 k2 P D par tol`: one intersector on the explorer edges k1 then k2; the
// results after each Perform.
void tp_brc_intersect2_case(std::istream& input){
 GaReader r{input};
 GA_TRY(TpFace f=tp_face(r);BRepClass_FaceExplorer x(f.face);int k1=r.integer(),k2=r.integer();
  gp_Pnt2d p=r.pnt2d();gp_Dir2d d=r.dir2d();double par=r.real(),tol=r.real();
  BRepClass_Edge e1=tp_explorer_edge(x,k1),e2=tp_explorer_edge(x,k2);BRepClass_Intersector inter;
  inter.Perform(gp_Lin2d(p,d),par,tol,e1);ga_inter(inter);
  inter.Perform(gp_Lin2d(p,d),par,tol,e2);ga_inter(inter))
 ga_end();}
// A face classification: state, rejected, no wires, then (not rejected) the edge, its
// parameter and, once an edge is set, the position (uninitialized before natively).
template<class C> void tp_classification(const TpFace& f,const C& c){
 ga_int(c.State());ga_flag(c.Rejected());ga_flag(c.NoWires());
 int edge=-3;GA_TRY(edge=tp_edge_index(f.face,c.Edge().Edge());ga_int(edge))
 GA_TRY(ga_real(c.EdgeParameter()))
 if(edge>=0||edge==-2)ga_int(c.Position());}
// `face u v tol usebox gaptol`: BRepClass_FClassifier over an explorer, then
// BRepClass_FaceClassifier over the face and over an explorer.
void tp_brc_classify_case(std::istream& input){
 GaReader r{input};
 GA_TRY(TpFace f=tp_face(r);gp_Pnt2d p=r.pnt2d();double tol=r.real();bool box=r.flag();double gap=r.real();
  GA_TRY(BRepClass_FaceExplorer x(f.face);x.SetMaxTolerance(gap);x.SetUseBndBox(box);
   BRepClass_FClassifier c(x,p,tol);tp_classification(f,c))
  GA_TRY(BRepClass_FaceClassifier c(f.face,p,tol,box,gap);tp_classification(f,c))
  GA_TRY(BRepClass_FaceExplorer x(f.face);x.SetMaxTolerance(gap);x.SetUseBndBox(box);
   BRepClass_FaceClassifier c(x,p,tol);tp_classification(f,c)))
 ga_end();}
// `face x y z tol usebox gaptol`: BRepClass_FaceClassifier of a 3d point.
void tp_brc_classify3d_case(std::istream& input){
 GaReader r{input};
 GA_TRY(TpFace f=tp_face(r);gp_Pnt p=r.pnt();double tol=r.real();bool box=r.flag();double gap=r.real();
  BRepClass_FaceClassifier c(f.face,p,tol,box,gap);tp_classification(f,c))
 ga_end();}
template<class C> void tp_passive_state(const C& c){
 ga_int(c.ClosestIntersection());ga_int(c.State());ga_real(c.Parameter());ga_flag(c.IsHeadOrEnd());}
// The Reset and Compare of a 2d classifier over the explorer edges.
template<class C> void tp_passive(const TpFace& f,GaReader& r){
 (void)f;C c;tp_passive_state(c);ga_flag(c.Intersector().IsDone());
 BRepClass_FaceExplorer x(f.face);gp_Pnt2d p=r.pnt2d();gp_Dir2d d=r.dir2d();double par=r.real();
 c.Reset(gp_Lin2d(p,d),par,r.real());tp_passive_state(c);
 for(x.InitWires();x.MoreWires();x.NextWire())
  for(x.InitEdges();x.MoreEdges();x.NextEdge()){
   BRepClass_Edge e;TopAbs_Orientation o;x.CurrentEdge(e,o);
   GA_TRY(c.Compare(e,o);tp_passive_state(c))}}
// `face mode P D par tol`: FacePassiveClassifier (mode 0) or FClass2dOfFClassifier (1).
void tp_brc_passive_case(std::istream& input){
 GaReader r{input};
 GA_TRY(TpFace f=tp_face(r);
  if(r.flag())tp_passive<BRepClass_FClass2dOfFClassifier>(f,r);
  else tp_passive<BRepClass_FacePassiveClassifier>(f,r))
 ga_end();}
// No fields: the default classifiers and intersector.
void tp_brc_defaults_case(std::istream&){
 BRepClass_FaceClassifier a;ga_int(a.State());ga_flag(a.Rejected());ga_flag(a.NoWires());
 GA_TRY(ga_flag(a.Edge().Edge().IsNull()))GA_TRY(ga_real(a.EdgeParameter()))
 BRepClass_FClassifier b;ga_int(b.State());ga_flag(b.Rejected());ga_flag(b.NoWires());
 GA_TRY(ga_real(b.Edge().MaxTolerance()))GA_TRY(ga_real(b.EdgeParameter()))
 BRepClass_Intersector c;ga_flag(c.IsDone());
 BRepClass_Edge e;ga_real(e.MaxTolerance());ga_flag(e.UseBndBox());ga_int(int(e.BoundingBoxState()));
 ga_end();}
