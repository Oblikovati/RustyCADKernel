// TopoDS oracle: flags, iteration, identity, hashing, casts, dumps, HShape and the
// shape alerts of every shape of a scripted graph. The Rust twin is
// crates/tkbrep/examples/br_probe/topods.rs.
#include <Message_AlertExtended.hxx>
#include <Message_Messenger.hxx>
#include <Message_PrinterToReport.hxx>
#include <Message_Report.hxx>
#include <TopoDS_AlertAttribute.hxx>
#include <TopoDS_AlertWithShape.hxx>
#include <TopoDS_HShape.hxx>
// The TopLoc hash recurrence on explicit (datum address, power) pairs, head first
// (the formula of TopLoc_Location::HashCode, compared by the TKMath oracle).
size_t br_location_hash(const std::vector<std::pair<size_t,int>>& items){
 if(items.empty())return 0;size_t h=opencascade::MurmurHash::optimalSeed<size_t>();
 for(const auto& item:items){size_t c[3]={item.first,opencascade::hash(item.second),h};h=opencascade::hashBytes(c,sizeof(c));}
 return h;}
// The std::hash<TopoDS_Shape> formula on explicit parts.
size_t br_shape_hash(size_t address,size_t location_hash){
 return location_hash==0?opencascade::hash(address):opencascade::MurmurHash::hash_combine(address,sizeof(void*),location_hash);}
std::vector<std::pair<size_t,int>> br_items(const BrGraph& g,const TopLoc_Location& l,bool real){
 std::vector<std::pair<size_t,int>> items;
 for(TopLoc_Location c=l;!c.IsIdentity();c=c.NextLocation()){
  size_t address=real?size_t(c.FirstDatum().get()):size_t(0x100*(g.datum_id(c.FirstDatum())+2));
  items.push_back({address,c.FirstPower()});}
 return items;}
// The hash: whether the real value is the formula on the real address, then the formula
// on synthetic addresses (0x1000 per TShape id, 0x100 per datum id).
void br_hash(const BrGraph& g,const TopoDS_Shape& s){
 size_t real=br_shape_hash(size_t(s.TShape().get()),br_location_hash(br_items(g,s.Location(),true)));
 br_flag(std::hash<TopoDS_Shape>{}(s)==real&&TopTools_ShapeMapHasher{}(s)==real);
 br_flag(s.Location().HashCode()==br_location_hash(br_items(g,s.Location(),true)));
 size_t address=s.IsNull()?0:size_t(0x1000*(g.tshape_id(s)+1));
 std::cout<<br_shape_hash(address,br_location_hash(br_items(g,s.Location(),false)))<<' ';}
// The TShape members of a null shape dereference a null handle natively: only
// NbChildren, which checks, is printed for it.
void br_flags(const TopoDS_Shape& s){
 if(s.IsNull()){br_word("N");br_int(s.NbChildren());return;}
 br_int(s.ShapeType());br_flag(s.Free());br_flag(s.Modified());br_flag(s.Checked());br_flag(s.Orientable());
 br_flag(s.Closed());br_flag(s.Infinite());br_flag(s.Convex());br_flag(s.Locked());
 br_int(s.NbChildren());br_int(s.TShape()->NbChildren());
 br_word(s.TShape()->DynamicType()->Name());br_flag(s.TShape()->IsKind(STANDARD_TYPE(TopoDS_TShape)));
 occ::handle<TopoDS_TShape> copy=s.TShape()->EmptyCopy();
 br_word(copy->DynamicType()->Name());br_int(copy->NbChildren());br_int(copy->ShapeType());}
void br_iterate(const BrGraph& g,const TopoDS_Shape& s,bool ori,bool loc){
 TopoDS_Iterator it(s,ori,loc);int n=0;
 for(;it.More()&&n<1000;it.Next(),++n)br_desc(g,it.Value());
 br_word("|");BR_TRY(it.Value());it.Next();br_flag(it.More());}
void br_casts(const TopoDS_Shape& s){
 BR_TRY(TopoDS::Vertex(s);br_word("-"));BR_TRY(TopoDS::Edge(s);br_word("-"));
 BR_TRY(TopoDS::Wire(s);br_word("-"));BR_TRY(TopoDS::Face(s);br_word("-"));
 BR_TRY(TopoDS::Shell(s);br_word("-"));BR_TRY(TopoDS::Solid(s);br_word("-"));
 BR_TRY(TopoDS::CompSolid(s);br_word("-"));BR_TRY(TopoDS::Compound(s);br_word("-"));}
// `br_topods script`: per shape, the flags, the four iterations, the hash and the
// casts, then the IsPartner/IsSame/IsEqual matrix.
void br_topods_case(std::istream& in){
 BrReader r{in};BrGraph g=br_graph(r);
 for(const TopoDS_Shape& s:g.shapes){
  br_desc(g,s);br_flags(s);
  for(int c=0;c<4;++c)br_iterate(g,s,c&1,c&2);
  br_hash(g,s);br_casts(s);}
 for(const TopoDS_Shape& a:g.shapes)for(const TopoDS_Shape& b:g.shapes)
  br_int(int(a.IsPartner(b))+2*int(a.IsSame(b))+4*int(a.IsEqual(b))+8*int(a!=b));
 br_end();}
std::string br_dumped(const TopoDS_Shape& s,int depth){std::ostringstream o;s.DumpJson(o,depth);return br_masked(o.str());}
// `br_dump script`: the JSON dumps of every shape and TShape, HShape, the shape alerts.
void br_dump_case(std::istream& in){
 BrReader r{in};BrGraph g=br_graph(r);
 for(const TopoDS_Shape& s:g.shapes){
  for(int depth=-1;depth<=2;++depth)br_text(br_dumped(s,depth));
  if(!s.IsNull()){std::ostringstream o;s.TShape()->DumpJson(o);br_text(br_masked(o.str()));}
  occ::handle<TopoDS_HShape> h=new TopoDS_HShape(s);br_flag(h->Shape().IsEqual(s));
  h->ChangeShape().Reverse();br_int(h->Shape().Orientation());h->Shape(TopoDS_Shape());br_flag(h->Shape().IsNull());
  occ::handle<TopoDS_AlertWithShape> a=new TopoDS_AlertWithShape(s);
  br_flag(a->SupportsMerge());br_flag(a->Merge(a));br_flag(a->GetShape().IsEqual(s));
  a->SetShape(TopoDS_Shape());br_flag(a->GetShape().IsNull());br_word(a->DynamicType()->Name());
  occ::handle<TopoDS_AlertAttribute> t=new TopoDS_AlertAttribute(s,"probe");
  br_flag(t->GetShape().IsEqual(s));br_text(t->GetName().ToCString());br_text(br_masked(t->Stream().str()));
  std::ostringstream o;t->DumpJson(o);br_text(br_masked(o.str()));br_word(t->DynamicType()->Name());}
 occ::handle<Message_Report> report=new Message_Report();
 occ::handle<Message_PrinterToReport> printer=new Message_PrinterToReport();printer->SetReport(report);
 occ::handle<Message_Messenger> messenger=new Message_Messenger(printer);
 for(const TopoDS_Shape& s:g.shapes)messenger<<s;
 br_int(report->GetAlerts(Message_Info).Size());
 br_end();}
