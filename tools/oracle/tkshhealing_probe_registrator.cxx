// ShapeExtend_BasicMsgRegistrator and ShapeExtend_MsgRegistrator probe operations; the
// Rust twin is crates/tkshhealing/examples/sh_probe/registrator.rs.
//
// `sh_registrator program kind nobj nops (op)...`: kind 0 is a BasicMsgRegistrator, 1 a
// MsgRegistrator, used through a BasicMsgRegistrator handle; nobj new Standard_Transient
// objects; each op sends the message whose text is `m<k>` with gravity g:
//  0 object k g: Send(object, msg, g) (object -1: a null handle); 1 s k g: Send(shape, msg,
//  g) (shape -1: a null shape); 2 k g: Send(msg, g).
// Then the dynamic type name, IsKind of both classes and, for a MsgRegistrator, the
// messages kept for each object and for each shape of the table (IsBound, then the texts).
#include <ShapeExtend_MsgRegistrator.hxx>
#include <Message_Msg.hxx>
Message_Msg sh_rg_message(int k){Message_Msg msg;msg.Set((std::string("m")+std::to_string(k)).c_str());return msg;}
void sh_rg_list(const NCollection_List<Message_Msg>& list){
 ga_int(list.Extent());
 for(NCollection_List<Message_Msg>::Iterator it(list);it.More();it.Next())
  ga_text(TCollection_AsciiString(it.Value().Value()).ToCString());}
void sh_rg_maps(const TpShapes& t,const occ::handle<ShapeExtend_MsgRegistrator>& reg,
                const std::vector<occ::handle<Standard_Transient>>& objects){
 ga_int(reg->MapTransient().Extent());
 for(const occ::handle<Standard_Transient>& object:objects){
  ga_flag(reg->MapTransient().IsBound(object));
  if(reg->MapTransient().IsBound(object))sh_rg_list(reg->MapTransient().Find(object));}
 ga_int(reg->MapShape().Extent());
 for(const TopoDS_Shape& s:t.shapes){
  ga_flag(reg->MapShape().IsBound(s));if(reg->MapShape().IsBound(s))sh_rg_list(reg->MapShape().Find(s));}}
void sh_registrator_case(std::istream& input){
 GaReader r{input};TpShapes t=tp_shapes(r);int kind=r.integer();
 occ::handle<ShapeExtend_BasicMsgRegistrator> reg;
 if(kind==0)reg=new ShapeExtend_BasicMsgRegistrator();else reg=new ShapeExtend_MsgRegistrator();
 std::vector<occ::handle<Standard_Transient>> objects(r.integer());
 for(occ::handle<Standard_Transient>& object:objects)object=new Standard_Transient();
 int nops=r.integer();
 for(int k=0;k<nops;++k){int op=r.integer();
  if(op==0){int o=r.integer();int m=r.integer();Message_Gravity g=Message_Gravity(r.integer());
   reg->Send(o<0?occ::handle<Standard_Transient>():objects.at(o),sh_rg_message(m),g);}
  else if(op==1){int s=r.integer();int m=r.integer();reg->Send(sh_shape(t,s),sh_rg_message(m),Message_Gravity(r.integer()));}
  else{int m=r.integer();reg->Send(sh_rg_message(m),Message_Gravity(r.integer()));}}
 ga_text(reg->DynamicType()->Name());
 ga_flag(reg->IsKind(STANDARD_TYPE(ShapeExtend_BasicMsgRegistrator)));
 ga_flag(reg->IsKind(STANDARD_TYPE(ShapeExtend_MsgRegistrator)));
 occ::handle<ShapeExtend_MsgRegistrator> keeper=occ::down_cast<ShapeExtend_MsgRegistrator>(reg);
 if(!keeper.IsNull())sh_rg_maps(t,keeper,objects);
 ga_end();}
