// HLRAlgo_PolyInternalData, the Rust twin of crates/tkhlr/examples/hl_probe/hlr_algo_internal.rs:
// UpdateLinks, AddNode, the split UpdateLinks, the Inc/Dec members and Dump. Every entry a
// request prints was written first: the native arrays start uninitialized.
#include <HLRAlgo_PolyInternalData.hxx>
#include <HLRAlgo_TriangleData.hxx>
void hl_print_internal(const occ::handle<HLRAlgo_PolyInternalData>& pid){
 ga_int(pid->NbPINod());ga_int(pid->NbPISeg());ga_int(pid->NbTData());ga_flag(pid->Planar());ga_flag(pid->IntOutL());
 ga_int(pid->TData().Upper());ga_int(pid->PISeg().Upper());ga_int(pid->PINod().Upper());
 for(int i=1;i<=pid->NbPISeg();++i){const HLRAlgo_PolyInternalSegment& s=pid->PISeg()(i);
  for(int v:{s.LstSg1,s.LstSg2,s.NxtSg1,s.NxtSg2,s.Conex1,s.Conex2})ga_int(v);}
 for(int i=1;i<=pid->NbTData();++i){const HLRAlgo_TriangleData& t=pid->TData()(i);for(int v:{t.Node1,t.Node2,t.Node3,t.Flags})ga_int(v);}
 for(int i=1;i<=pid->NbPINod();++i){const occ::handle<HLRAlgo_PolyInternalNode>& nd=pid->PINod()(i);
  HLRAlgo_PolyInternalNode::NodeIndices& ni=nd->Indices();for(int v:{ni.NdSg,ni.Flag,ni.Edg1,ni.Edg2})ga_int(v);
  HLRAlgo_PolyInternalNode::NodeData& d=nd->Data();ga_xyz(d.Point);ga_xyz(d.Normal);ga_xy(d.UV);ga_real(d.PCu1);ga_real(d.PCu2);ga_real(d.Scal);}
 ga_captured([&]{pid->Dump();});}
// `nbNod nbTri (x y z nx ny nz u v scal)*nbNod (n1 n2 n3 flags)*nbTri n ops`: a PolyInternalData
// with its nodes and triangles, UpdateLinks over its own arrays, then ops: 0 AddNode(i, j,
// coef, x, y, z), 1 the split UpdateLinks(ip1, ip2, ip3), 2 Planar(B), 3 IntOutL(B), 4
// DecPISeg then IncPISeg; the whole data after UpdateLinks and after each op. The array
// pointers are the object's own arrays, the same pointer twice.
void hl_ha_internal_case(std::istream& input){
 GaReader r{input};int nn=r.integer(),nt=r.integer();
 GA_TRY(occ::handle<HLRAlgo_PolyInternalData> pid=new HLRAlgo_PolyInternalData(nn,nt);
  for(int i=1;i<=nn;++i){HLRAlgo_PolyInternalNode::NodeData& d=pid->PINod()(i)->Data();d.Point=r.xyz();d.Normal=r.xyz();d.UV=r.pnt2d().XY();d.Scal=r.real();}
  for(int i=1;i<=nt;++i){HLRAlgo_TriangleData& t=pid->TData()(i);t.Node1=r.integer();t.Node2=r.integer();t.Node3=r.integer();t.Flags=r.integer();}
  pid->UpdateLinks(pid->TData(),pid->PISeg(),pid->PINod());hl_print_internal(pid);
  for(int n=r.integer();n>0;--n){int op=r.integer();
   NCollection_Array1<HLRAlgo_TriangleData>* t1=&pid->TData();NCollection_Array1<HLRAlgo_TriangleData>* t2=t1;
   NCollection_Array1<HLRAlgo_PolyInternalSegment>* s1=&pid->PISeg();NCollection_Array1<HLRAlgo_PolyInternalSegment>* s2=s1;
   NCollection_Array1<occ::handle<HLRAlgo_PolyInternalNode>>* n1=&pid->PINod();NCollection_Array1<occ::handle<HLRAlgo_PolyInternalNode>>* n2=n1;
   if(op==0){int i=r.integer(),j=r.integer();double c=r.real(),x=r.real(),y=r.real(),z=r.real();
    HLRAlgo_PolyInternalNode::NodeData a=pid->PINod()(i)->Data(),b=pid->PINod()(j)->Data();
    ga_int(pid->AddNode(a,b,n1,n2,c,x,y,z));}
   else if(op==1){int a=r.integer(),b=r.integer(),c=r.integer();pid->UpdateLinks(a,b,c,t1,t2,s1,s2,n1,n2);}
   else if(op==2)pid->Planar(r.flag());else if(op==3)pid->IntOutL(r.flag());
   else{pid->DecPISeg();pid->IncPISeg(s1,s2);}
   hl_print_internal(pid);})
 ga_end();}
// `nbNod nbTri n (which same)*n`: the Inc members (which 0 TData, 1 PISeg, 2 PINod) with the
// same or two different array pointers (the second one names a copy), then the Dec members;
// the counts, the array bounds, and which pointers now name the object's arrays.
void hl_ha_grow_case(std::istream& input){
 GaReader r{input};int nn=r.integer(),nt=r.integer();
 GA_TRY(occ::handle<HLRAlgo_PolyInternalData> pid=new HLRAlgo_PolyInternalData(nn,nt);
  ga_int(pid->TData().Upper());ga_int(pid->PISeg().Upper());ga_int(pid->PINod().Upper());
  NCollection_Array1<HLRAlgo_TriangleData> tcopy(0,0);NCollection_Array1<HLRAlgo_PolyInternalSegment> scopy(0,0);
  NCollection_Array1<occ::handle<HLRAlgo_PolyInternalNode>> ncopy(0,0);
  for(int n=r.integer();n>0;--n){int which=r.integer();bool same=r.flag();
   if(which==0){NCollection_Array1<HLRAlgo_TriangleData>* a=&pid->TData();NCollection_Array1<HLRAlgo_TriangleData>* b=same?a:&tcopy;
    pid->IncTData(a,b);ga_flag(a==&pid->TData());ga_flag(b==&pid->TData());}
   else if(which==1){NCollection_Array1<HLRAlgo_PolyInternalSegment>* a=&pid->PISeg();NCollection_Array1<HLRAlgo_PolyInternalSegment>* b=same?a:&scopy;
    pid->IncPISeg(a,b);ga_flag(a==&pid->PISeg());ga_flag(b==&pid->PISeg());}
   else{NCollection_Array1<occ::handle<HLRAlgo_PolyInternalNode>>* a=&pid->PINod();NCollection_Array1<occ::handle<HLRAlgo_PolyInternalNode>>* b=same?a:&ncopy;
    pid->IncPINod(a,b);ga_flag(a==&pid->PINod());ga_flag(b==&pid->PINod());}
   ga_int(pid->NbTData());ga_int(pid->NbPISeg());ga_int(pid->NbPINod());
   ga_int(pid->TData().Upper());ga_int(pid->PISeg().Upper());ga_int(pid->PINod().Upper());}
  pid->DecTData();pid->DecPISeg();pid->DecPINod();ga_int(pid->NbTData());ga_int(pid->NbPISeg());ga_int(pid->NbPINod());)
 ga_end();}
