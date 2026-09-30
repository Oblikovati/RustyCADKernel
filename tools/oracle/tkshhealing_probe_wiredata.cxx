// ShapeExtend_WireData probe operations; the Rust twin is
// crates/tkshhealing/examples/sh_probe/wire_data.rs.
//
// `sh_wiredata program ctor nops (op)...`: the shapes come from a TKTopAlgo shape program
// (tktopalgo_probe_shapes.cxx); ctor 0 is ShapeExtend_WireData(), ctor 1 `w chained
// manifold` the constructor from the wire of index w. Each op runs in GA_TRY:
//  0 w chained manifold: Init(wire), printing its result; 1 e atnum: Add(edge);
//  2 w atnum: Add(wire); 3 s atnum: Add(shape); 4 kind s mode: AddOriented of the edge
//  (kind 0), wire (1) or shape (2); 5 num: Remove(num); 6 e num: Set(edge, num);
//  7: Reverse(); 8 f: Reverse(face) (f = -1: a null face); 9 num: SetLast(num);
//  10: SetDegeneratedLast(); 11 enforce: ComputeSeams(enforce); 12: Clear();
//  13 mode: ManifoldMode() = mode; 14 w chained manifold: Init(other), other built from
//  the wire; 15 w chained manifold atnum: Add(other, atnum); 16: Init(itself);
//  17: NonmanifoldEdges(), then Clear(), then the length of the kept handle.
// A shape index -1 is a null shape. The digest follows: the manifold mode, each edge
// (IsSame index in the table, orientation), each non-manifold edge, IsSeam and Index of
// each rank, Edge(-1), Edge(n + 1), NonmanifoldEdge(-1), then Wire() and WireAPIMake()
// (unless a null edge is among the non-manifold edges in the manifold mode).
#include <ShapeExtend_WireData.hxx>
#include <NCollection_HSequence.hxx>
TopoDS_Shape sh_shape(const TpShapes& t,int i){return i<0?TopoDS_Shape():t.shapes.at(i);}
TopoDS_Edge sh_edge(const TpShapes& t,int i){return i<0?TopoDS_Edge():TopoDS::Edge(t.shapes.at(i));}
TopoDS_Wire sh_wire(const TpShapes& t,int i){return i<0?TopoDS_Wire():TopoDS::Wire(t.shapes.at(i));}
TopoDS_Face sh_face(const TpShapes& t,int i){return i<0?TopoDS_Face():TopoDS::Face(t.shapes.at(i));}
// An edge of the wire data: its IsSame index in the table and its orientation.
void sh_wd_edge(const TpShapes& t,const TopoDS_Edge& e){ga_int(tp_same_index(t,e));ga_int(e.Orientation());}
void sh_wd_add_oriented(const TpShapes& t,const occ::handle<ShapeExtend_WireData>& wd,int kind,int s,int mode){
 if(kind==0)wd->AddOriented(sh_edge(t,s),mode);
 else if(kind==1)wd->AddOriented(sh_wire(t,s),mode);
 else wd->AddOriented(sh_shape(t,s),mode);}
occ::handle<ShapeExtend_WireData> sh_wd_other(const TpShapes& t,const int* a){
 return new ShapeExtend_WireData(sh_wire(t,a[0]),a[1]!=0,a[2]!=0);}
void sh_wd_op(const TpShapes& t,const occ::handle<ShapeExtend_WireData>& wd,int op,const int* a){
 if(op==0)ga_flag(wd->Init(sh_wire(t,a[0]),a[1]!=0,a[2]!=0));
 else if(op==1)wd->Add(sh_edge(t,a[0]),a[1]);
 else if(op==2)wd->Add(sh_wire(t,a[0]),a[1]);
 else if(op==3)wd->Add(sh_shape(t,a[0]),a[1]);
 else if(op==4)sh_wd_add_oriented(t,wd,a[0],a[1],a[2]);
 else if(op==5)wd->Remove(a[0]);
 else if(op==6)wd->Set(sh_edge(t,a[0]),a[1]);
 else if(op==7)wd->Reverse();
 else if(op==8)wd->Reverse(sh_face(t,a[0]));
 else if(op==9)wd->SetLast(a[0]);
 else if(op==10)wd->SetDegeneratedLast();
 else if(op==11)wd->ComputeSeams(a[0]!=0);
 else if(op==12)wd->Clear();
 else if(op==13)wd->ManifoldMode()=a[0]!=0;
 else if(op==14)wd->Init(sh_wd_other(t,a));
 else if(op==15)wd->Add(sh_wd_other(t,a),a[3]);
 else if(op==16)wd->Init(wd);
 else{occ::handle<NCollection_HSequence<TopoDS_Shape>> kept=wd->NonmanifoldEdges();wd->Clear();ga_int(kept->Length());}}
// The number of integer operands of each op, read before it runs so that a raised op
// leaves the reader in step.
int sh_wd_operand_count(int op){
 static const int counts[]={3,2,2,2,3,1,2,0,1,1,0,1,0,1,3,4,0,0};return counts[op];}
void sh_wd_digest(const TpShapes& t,const occ::handle<ShapeExtend_WireData>& wd){
 ga_flag(wd->ManifoldMode());int n=wd->NbEdges();ga_int(n);
 for(int i=1;i<=n;++i)sh_wd_edge(t,wd->Edge(i));
 int nm=wd->NbNonManifoldEdges();ga_int(nm);bool null_edge=false;
 for(int i=1;i<=nm;++i){sh_wd_edge(t,wd->NonmanifoldEdge(i));null_edge|=wd->NonmanifoldEdge(i).IsNull();}
 for(int i=1;i<=n;++i){ga_flag(wd->IsSeam(i));ga_int(wd->Index(wd->Edge(i)));}
 if(n>0)sh_wd_edge(t,wd->Edge(-1));
 GA_TRY(sh_wd_edge(t,wd->Edge(n+1)))
 ga_flag(wd->NonmanifoldEdge(-1).IsNull());
 // A null edge Add put among the non-manifold edges crashes Wire() natively (BRep_Builder
 // adds its null TShape): the wires are left out then.
 if(null_edge&&wd->ManifoldMode())return;
 GA_TRY(tp_shape(wd->Wire()))
 GA_TRY(tp_shape(wd->WireAPIMake()))}
void sh_wiredata_case(std::istream& input){
 GaReader r{input};TpShapes t=tp_shapes(r);occ::handle<ShapeExtend_WireData> wd;
 if(r.integer()==0)wd=new ShapeExtend_WireData();
 else{int w=r.integer();bool chained=r.flag();bool manifold=r.flag();
  GA_TRY(wd=new ShapeExtend_WireData(sh_wire(t,w),chained,manifold))}
 int nops=r.integer();
 for(int k=0;k<nops;++k){int op=r.integer();int a[4]={0,0,0,0};
  for(int i=0;i<sh_wd_operand_count(op);++i)a[i]=r.integer();
  if(!wd.IsNull())GA_TRY(sh_wd_op(t,wd,op,a))}
 if(!wd.IsNull())sh_wd_digest(t,wd);
 ga_end();}
