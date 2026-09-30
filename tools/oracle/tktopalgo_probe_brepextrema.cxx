// BRepExtrema probe operations, distance group (docs/topalgo-design.md section 13):
// DistShapeShape, DistanceSS, the Ext wrappers and Poly on shapes read by tp_shape
// (tktopalgo_probe_shape.cxx). A sub-shape is picked by `kind index` (tp_bxe_sub); a
// support is printed as its type, its index (by TShape) among the vertices, edges or faces
// of each record, and its orientation. The Rust twin is
// crates/tktopalgo/examples/tp_probe/brep_extrema.rs.
#include <BRepBuilderAPI_MakeVertex.hxx>
#include <BRepExtrema_DistShapeShape.hxx>
#include <BRepExtrema_DistanceSS.hxx>
#include <BRepExtrema_ExtCC.hxx>
#include <BRepExtrema_ExtCF.hxx>
#include <BRepExtrema_ExtFF.hxx>
#include <BRepExtrema_ExtPC.hxx>
#include <BRepExtrema_ExtPF.hxx>
#include <BRepExtrema_Poly.hxx>
#include <BRepBndLib.hxx>
// The index (from 0, by TShape) of `x` among the sub-shapes of its type in `s`, -1 when
// it is not one of them.
int tp_bxe_tindex(const TpShape& s,const TopoDS_Shape& x){
 auto find=[&](const auto& list){for(size_t i=0;i<list.size();++i)if(list[i].TShape()==x.TShape())return int(i);return -1;};
 if(x.ShapeType()==TopAbs_VERTEX)return find(s.vertices);
 if(x.ShapeType()==TopAbs_EDGE)return find(s.edges);
 if(x.ShapeType()==TopAbs_FACE)return find(s.faces);
 return -1;}
// A shape: -1 when null, else type, index in each record and orientation.
void tp_bxe_id(const TpShape& a,const TpShape& b,const TopoDS_Shape& x){
 if(x.IsNull()){ga_int(-1);return;}
 ga_int(int(x.ShapeType()));ga_int(tp_bxe_tindex(a,x));ga_int(tp_bxe_tindex(b,x));ga_int(int(x.Orientation()));}
// `kind index`: 0 the shape, 1 vertex, 2 edge, 3 face of the record lists (no location),
// 5 face, 6 edge, 7 vertex of an explorer of the shape (with its locations).
TopoDS_Shape tp_bxe_sub(GaReader& r,const TpShape& s){
 int kind=r.integer(),i=r.integer();
 if(kind==1)return s.vertices.at(i);
 if(kind==2)return s.edges.at(i);
 if(kind==3)return s.faces.at(i);
 if(kind>=5){TopAbs_ShapeEnum t=kind==5?TopAbs_FACE:kind==6?TopAbs_EDGE:TopAbs_VERTEX;int k=0;
  for(TopExp_Explorer e(s.shape,t);e.More();e.Next(),++k)if(k==i)return e.Current();return TopoDS_Shape();}
 return s.shape;}
// A vertex at a point with a tolerance (BRepBuilderAPI_MakeVertex when the tolerance is 0).
TopoDS_Vertex tp_bxe_vertex(GaReader& r){
 gp_Pnt p=r.pnt();double tol=r.real();
 if(tol<=0.)return BRepBuilderAPI_MakeVertex(p).Vertex();
 BRep_Builder B;TopoDS_Vertex v;B.MakeVertex(v,p,tol);return v;}
Extrema_ExtFlag tp_bxe_flag(int f){return f==0?Extrema_ExtFlag_MIN:f==1?Extrema_ExtFlag_MAX:Extrema_ExtFlag_MINMAX;}
Extrema_ExtAlgo tp_bxe_algo(int a){return a==0?Extrema_ExtAlgo_Grad:Extrema_ExtAlgo_Tree;}
// One solution `n` of a distance (every accessor, with its exception).
void tp_bxd_solution(const TpShape& a,const TpShape& b,const BRepExtrema_DistShapeShape& d,int n){
 GA_TRY(ga_int(int(d.SupportTypeShape1(n))));GA_TRY(ga_int(int(d.SupportTypeShape2(n))));
 GA_TRY(ga_pnt(d.PointOnShape1(n)));GA_TRY(ga_pnt(d.PointOnShape2(n)));
 GA_TRY(tp_bxe_id(a,b,d.SupportOnShape1(n)));GA_TRY(tp_bxe_id(a,b,d.SupportOnShape2(n)));
 GA_TRY(double t;d.ParOnEdgeS1(n,t);ga_real(t));GA_TRY(double t;d.ParOnEdgeS2(n,t);ga_real(t));
 GA_TRY(double u,v;d.ParOnFaceS1(n,u,v);ga_real(u);ga_real(v));
 GA_TRY(double u,v;d.ParOnFaceS2(n,u,v);ga_real(u);ga_real(v));}
// The state of a distance, its solutions (at most 40) and the accessors at 0 and N+1.
void tp_bxd_state(const TpShape& a,const TpShape& b,const BRepExtrema_DistShapeShape& d){
 ga_flag(d.IsDone());ga_int(d.NbSolution());GA_TRY(ga_real(d.Value()));ga_flag(d.InnerSolution());ga_flag(d.IsMultiThread());
 int n=std::min(d.NbSolution(),40);
 for(int i=1;i<=n;++i)tp_bxd_solution(a,b,d,i);
 tp_bxd_solution(a,b,d,0);tp_bxd_solution(a,b,d,d.NbSolution()+1);}
// `shape1 shape2 deflection mode multithread flag algo again`: mode 0 the constructor with
// the deflection (flag and algo), 1 the default one then SetMultiThread, SetFlag, SetAlgo,
// SetDeflection, LoadS1, LoadS2 and Perform, 2 the constructor without deflection; `again`
// performs once more, then with the shapes swapped.
void tp_bxd_dist_case(std::istream& input){
 GaReader r{input};
 GA_TRY(TpShape a=tp_shape(r);TpShape b=tp_shape(r);double defl=r.real();int mode=r.integer();bool mt=r.flag();
  Extrema_ExtFlag f=tp_bxe_flag(r.integer());Extrema_ExtAlgo al=tp_bxe_algo(r.integer());bool again=r.flag();
  std::unique_ptr<BRepExtrema_DistShapeShape> d;
  if(mode==0){GA_TRY(d.reset(new BRepExtrema_DistShapeShape(a.shape,b.shape,defl,f,al)));}
  else if(mode==2){GA_TRY(d.reset(new BRepExtrema_DistShapeShape(a.shape,b.shape,f,al)));}
  else{d.reset(new BRepExtrema_DistShapeShape());d->SetMultiThread(mt);d->SetFlag(f);d->SetAlgo(al);d->SetDeflection(defl);
   GA_TRY(d->LoadS1(a.shape));GA_TRY(d->LoadS2(b.shape));GA_TRY(ga_flag(d->Perform()));}
  if(d){tp_bxd_state(a,b,*d);
   if(again){GA_TRY(ga_flag(d->Perform()));tp_bxd_state(a,b,*d);
    GA_TRY(d->LoadS1(b.shape));GA_TRY(d->LoadS2(a.shape));GA_TRY(ga_flag(d->Perform()));tp_bxd_state(b,a,*d);}});
 ga_end();}
// `shape1 shape2 deflection`: Dump of a performed distance, then of an empty one.
void tp_bxd_dump_case(std::istream& input){
 GaReader r{input};
 GA_TRY(TpShape a=tp_shape(r);TpShape b=tp_shape(r);double defl=r.real();
  BRepExtrema_DistShapeShape d(a.shape,b.shape,defl);ga_captured([&]{d.Dump(std::cout);});
  BRepExtrema_DistShapeShape empty;ga_captured([&]{empty.Dump(std::cout);}));
 ga_end();}
// One solution of DistanceSS.
void tp_bxd_ss_solution(const TpShape& a,const TpShape& b,const BRepExtrema_SolutionElem& s){
 ga_real(s.Dist());ga_pnt(s.Point());ga_int(int(s.SupportKind()));
 tp_bxe_id(a,b,s.Vertex());tp_bxe_id(a,b,s.Edge());tp_bxe_id(a,b,s.Face());
 double t;s.EdgeParameter(t);ga_real(t);double u,v;s.FaceParameter(u,v);ga_real(u);ga_real(v);}
// `shape1 sub1 shape2 sub2 boxes dstref deflection flag algo`: DistanceSS between two
// sub-shapes, with void boxes (boxes 0) or their BRepBndLib boxes (1).
void tp_bxd_ss_case(std::istream& input){
 GaReader r{input};
 GA_TRY(TpShape a=tp_shape(r);TopoDS_Shape s1=tp_bxe_sub(r,a);TpShape b=tp_shape(r);TopoDS_Shape s2=tp_bxe_sub(r,b);
  bool boxes=r.flag();double dst=r.real(),defl=r.real();Extrema_ExtFlag f=tp_bxe_flag(r.integer());
  Extrema_ExtAlgo al=tp_bxe_algo(r.integer());Bnd_Box b1,b2;
  if(boxes){BRepBndLib::Add(s1,b1);BRepBndLib::Add(s2,b2);}
  BRepExtrema_DistanceSS ss(s1,s2,b1,b2,dst,defl,f,al);ga_flag(ss.IsDone());ga_real(ss.DistValue());
  ga_int(ss.Seq1Value().Length());ga_int(ss.Seq2Value().Length());
  for(const BRepExtrema_SolutionElem& s:ss.Seq1Value())tp_bxd_ss_solution(a,b,s);
  for(const BRepExtrema_SolutionElem& s:ss.Seq2Value())tp_bxd_ss_solution(a,b,s););
 ga_end();}
// `shape1 shape2`: BRepExtrema_Poly::Distance.
void tp_bxd_poly_case(std::istream& input){
 GaReader r{input};
 GA_TRY(TpShape a=tp_shape(r);TpShape b=tp_shape(r);gp_Pnt p1,p2;double dist=0.;
  bool found=BRepExtrema_Poly::Distance(a.shape,b.shape,p1,p2,dist);ga_flag(found);ga_pnt(p1);ga_pnt(p2);ga_real(dist));
 ga_end();}
