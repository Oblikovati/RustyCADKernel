// ShapeExtend_Explorer probe operations; the Rust twin is
// crates/tkshhealing/examples/sh_probe/explorer.rs.
//
// `sh_explorer program op ...` over the shapes of a TKTopAlgo shape program; a list of
// shapes is `n (index)...` (n = -1: a null sequence handle, index -1: a null shape):
//  0 list: CompoundFromSeq; 1 s expcomp: SeqFromCompound; 2 list pre clear: ListFromSeq
//  into a list holding the shape `pre` first (pre = -2: an empty list); 3 list:
//  SeqFromList; 4 s compound: ShapeType; 5 s type explore compound: SortedCompound;
//  6 list mask: DispatchList, output k pre-created holding the shape of index 0 when bit
//  k of mask is set, null otherwise.
// A shape is printed as its IsSame index in the table, its orientation and, for a new
// shape, its children the same way (one level).
#include <ShapeExtend_Explorer.hxx>
void sh_ex_item(const TpShapes& t,const TopoDS_Shape& s){
 ga_int(tp_same_index(t,s));ga_int(s.IsNull()?-1:int(s.Orientation()));}
void sh_ex_shape(const TpShapes& t,const TopoDS_Shape& s){
 sh_ex_item(t,s);if(s.IsNull())return;ga_int(s.ShapeType());ga_int(s.NbChildren());
 for(TopoDS_Iterator it(s);it.More();it.Next())sh_ex_item(t,it.Value());}
occ::handle<NCollection_HSequence<TopoDS_Shape>> sh_ex_list(GaReader& r,const TpShapes& t){
 int n=r.integer();if(n<0)return nullptr;
 occ::handle<NCollection_HSequence<TopoDS_Shape>> seq=new NCollection_HSequence<TopoDS_Shape>();
 for(int i=0;i<n;++i)seq->Append(sh_shape(t,r.integer()));
 return seq;}
void sh_ex_seq(const TpShapes& t,const occ::handle<NCollection_HSequence<TopoDS_Shape>>& seq){
 if(seq.IsNull()){ga_int(-1);return;}
 ga_int(seq->Length());for(int i=1;i<=seq->Length();++i)sh_ex_item(t,seq->Value(i));}
void sh_ex_list_from_seq(GaReader& r,const TpShapes& t,const ShapeExtend_Explorer& ex){
 occ::handle<NCollection_HSequence<TopoDS_Shape>> seq=sh_ex_list(r,t);int pre=r.integer();bool clear=r.flag();
 NCollection_List<TopoDS_Shape> list;if(pre>=-1)list.Append(sh_shape(t,pre));
 ex.ListFromSeq(seq,list,clear);ga_int(list.Extent());
 for(NCollection_List<TopoDS_Shape>::Iterator it(list);it.More();it.Next())sh_ex_item(t,it.Value());}
void sh_ex_dispatch(GaReader& r,const TpShapes& t,const ShapeExtend_Explorer& ex){
 occ::handle<NCollection_HSequence<TopoDS_Shape>> list=sh_ex_list(r,t);int mask=r.integer();
 occ::handle<NCollection_HSequence<TopoDS_Shape>> out[8];
 for(int k=0;k<8;++k)if(mask>>k&1){out[k]=new NCollection_HSequence<TopoDS_Shape>();out[k]->Append(t.shapes.at(0));}
 ex.DispatchList(list,out[0],out[1],out[2],out[3],out[4],out[5],out[6],out[7]);
 for(int k=0;k<8;++k)sh_ex_seq(t,out[k]);}
void sh_explorer_case(std::istream& input){
 GaReader r{input};TpShapes t=tp_shapes(r);ShapeExtend_Explorer ex;int op=r.integer();
 if(op==0){occ::handle<NCollection_HSequence<TopoDS_Shape>> seq=sh_ex_list(r,t);GA_TRY(TopoDS_Shape c=ex.CompoundFromSeq(seq);sh_ex_shape(t,c);tp_shape(c))}
 else if(op==1){int s=r.integer();bool exp=r.flag();sh_ex_seq(t,ex.SeqFromCompound(sh_shape(t,s),exp));}
 else if(op==2)sh_ex_list_from_seq(r,t,ex);
 else if(op==3){occ::handle<NCollection_HSequence<TopoDS_Shape>> seq=sh_ex_list(r,t);
  NCollection_List<TopoDS_Shape> list;for(int i=1;i<=seq->Length();++i)list.Append(seq->Value(i));
  sh_ex_seq(t,ex.SeqFromList(list));}
 else if(op==4){int s=r.integer();ga_int(ex.ShapeType(sh_shape(t,s),r.flag()));}
 else if(op==5){int s=r.integer();TopAbs_ShapeEnum type=TopAbs_ShapeEnum(r.integer());bool explore=r.flag();
  TopoDS_Shape res=ex.SortedCompound(sh_shape(t,s),type,explore,r.flag());sh_ex_shape(t,res);tp_shape(res);}
 else sh_ex_dispatch(r,t,ex);
 ga_end();}
