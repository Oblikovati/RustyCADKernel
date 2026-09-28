// TopTools oracle: the location set and the shape set of a scripted graph, their dump,
// write and read texts, and reads of given (also malformed) texts. The Rust twin is
// crates/tkbrep/examples/br_probe/toptools.rs.
#include <TopTools_LocationSet.hxx>
#include <TopTools_ShapeSet.hxx>
// A shape set whose hooks make TopoDS-only shapes, so that written sets read back, and
// that leave a trace of each call in the text and in counters.
class ProbeShapeSet : public TopTools_ShapeSet {
public:
 int added=0,checked=0;
 void Clear() override{TopTools_ShapeSet::Clear();added=0;checked=0;}
 void AddGeometry(const TopoDS_Shape& S) override{added+=1+S.NbChildren();}
 void DumpGeometry(Standard_OStream& OS) const override{OS<<"geometry "<<added<<"\n";}
 void WriteGeometry(Standard_OStream& OS,const Message_ProgressRange&) override{OS<<"Probe "<<added<<"\n";}
 void ReadGeometry(Standard_IStream& IS,const Message_ProgressRange&) override{std::string w;IS>>w>>added;}
 void DumpGeometry(const TopoDS_Shape& S,Standard_OStream& OS) const override{OS<<"  children "<<S.NbChildren()<<"\n";}
 void WriteGeometry(const TopoDS_Shape& S,Standard_OStream& OS) const override{OS<<"g"<<S.NbChildren();}
 void ReadGeometry(const TopAbs_ShapeEnum T,Standard_IStream& IS,TopoDS_Shape& S) override{std::string w;IS>>w;S=br_make(int(T));}
 void AddShapes(TopoDS_Shape& S1,const TopoDS_Shape& S2) override{TopoDS_Builder().Add(S1,S2);}
 void Check(const TopAbs_ShapeEnum,TopoDS_Shape&) override{checked+=1;}
};
// A shape of a set read back: `N`, or its index in the set, orientation and location.
void br_set_desc(const TopTools_ShapeSet& set,const TopoDS_Shape& s){
 if(s.IsNull()){br_word("N");return;}
 br_int(set.Index(s.Located(TopLoc_Location())));br_int(s.Orientation());
 const TopTools_LocationSet& locs=set.Locations();br_int(locs.Index(s.Location()));}
// Every shape of a set: type, flags and sub-shapes, all by set indices.
void br_set_shapes(const TopTools_ShapeSet& set){
 br_int(set.NbShapes());
 for(int i=1;i<=set.NbShapes();++i){
  const TopoDS_Shape& s=set.Shape(i);br_int(s.ShapeType());
  br_flag(s.Free());br_flag(s.Modified());br_flag(s.Checked());br_flag(s.Orientable());
  br_flag(s.Closed());br_flag(s.Infinite());br_flag(s.Convex());br_flag(s.Locked());
  for(TopoDS_Iterator it(s,false,false);it.More();it.Next())br_set_desc(set,it.Value());
  br_word("|");}}
std::string br_set_written(ProbeShapeSet& set,int precision){
 std::ostringstream o;o.precision(precision);BR_TRY(set.Write(o));br_int(o.precision());return o.str();}
// `br_shapeset format script root`: Add, Index, Shape, the locations, DumpExtent,
// Dump, the shape records, Write, and the text read back into a new set.
void br_shapeset_case(std::istream& in){
 BrReader r{in};int format=r.integer();BrGraph g=br_graph(r);TopoDS_Shape root=g.shape(r.integer());
 ProbeShapeSet set;set.SetFormatNb(format);br_int(set.FormatNb());
 br_int(set.Add(root));for(const TopoDS_Shape& s:g.shapes)br_int(set.Add(s));
 br_int(set.NbShapes());br_int(set.added);
 for(const TopoDS_Shape& s:g.shapes){br_int(set.Index(s));br_int(set.Locations().Index(s.Location()));}
 for(int i=0;i<=set.NbShapes()+1;++i)BR_TRY(br_desc(g,set.Shape(i)));
 for(int i=-1;i<=int(g.locs.size())+2;++i)br_chain(g,set.Locations().Location(i));
 {std::ostringstream o;set.DumpExtent(o);br_text(o.str());TCollection_AsciiString t("x");set.DumpExtent(t);br_text(t.ToCString());}
 {std::ostringstream o;BR_TRY(set.Dump(o));br_text(br_masked(o.str()));}
 for(const TopoDS_Shape& s:g.shapes){
  std::ostringstream d,w;set.Dump(s,d);set.Write(s,w);br_text(d.str());br_text(w.str());
  std::istringstream back(w.str());TopoDS_Shape read;BR_TRY(set.Read(read,back));br_desc(g,read);}
 std::string text=br_set_written(set,6);br_text(text);
 ProbeShapeSet back;std::istringstream input(text);
 br_text(br_captured([&]{BR_TRY(back.Read(input));}));br_int(int(input.rdstate()));
 br_int(back.FormatNb());br_int(back.added);br_int(back.checked);br_set_shapes(back);
 br_text(br_set_written(back,3));
 back.Clear();br_int(back.NbShapes());br_int(back.added);
 br_end();}
// `br_setread base format bytes`: Read of a given text into a ProbeShapeSet (base 0) or
// a TopTools_ShapeSet (base 1: only texts without shapes, since its hooks make none).
void br_setread_case(std::istream& in){
 BrReader r{in};bool base=r.flag();int format=r.integer();std::istringstream text(r.bytes());
 ProbeShapeSet probe;TopTools_ShapeSet plain;TopTools_ShapeSet& set=base?plain:static_cast<TopTools_ShapeSet&>(probe);
 set.SetFormatNb(format);
 br_text(br_captured([&]{BR_TRY(set.Read(text));}));br_int(int(text.rdstate()));
 br_int(set.FormatNb());br_int(probe.checked);br_set_shapes(set);
 {std::ostringstream o;set.DumpExtent(o);br_text(o.str());}
 std::string rest;char c;text.clear();while(text.get(c))rest+=c;br_text(rest);
 br_end();}
void br_locations(const TopTools_LocationSet& set,int count){
 for(int i=-1;i<=count+1;++i){
  const TopLoc_Location& l=set.Location(i);
  br_int(set.Index(l));const gp_Trsf& t=l.Transformation();
  for(int row=1;row<=3;++row)for(int col=1;col<=4;++col)br_real(t.Value(row,col));
  std::vector<std::pair<int,int>> items;
  for(TopLoc_Location c=l;!c.IsIdentity();c=c.NextLocation())items.push_back({set.Index(TopLoc_Location(c.FirstDatum())),c.FirstPower()});
  br_int(items.size());for(auto& item:items){br_int(item.first);br_int(item.second);}}}
std::string br_locations_written(const TopTools_LocationSet& set,int precision){
 std::ostringstream o;o.precision(precision);set.Write(o);br_int(o.precision());return o.str();}
// `br_locset script`: Add and Index of every location of the script, Location, Dump,
// Write, and the text read back.
void br_locset_case(std::istream& in){
 BrReader r{in};BrGraph g=br_graph(r);TopTools_LocationSet set;
 for(const TopLoc_Location& l:g.locs)br_int(set.Add(l));
 for(const TopLoc_Location& l:g.locs)br_int(set.Index(l));
 int count=0;while(!set.Location(count+1).IsIdentity())++count;br_int(count);
 br_locations(set,count);
 {std::ostringstream o;set.Dump(o);br_text(o.str());}
 std::string text=br_locations_written(set,6);br_text(text);
 TopTools_LocationSet back;std::istringstream input(text);BR_TRY(back.Read(input));br_int(int(input.rdstate()));
 br_locations(back,count);br_text(br_locations_written(back,4));
 back.Clear();br_flag(back.Location(1).IsIdentity());
 br_end();}
// `br_locread bytes`: Read of a given (possibly malformed) Locations text.
void br_locread_case(std::istream& in){
 BrReader r{in};std::istringstream text(r.bytes());TopTools_LocationSet set;
 br_text(br_captured([&]{BR_TRY(set.Read(text));}));br_int(int(text.rdstate()));
 int count=0;while(!set.Location(count+1).IsIdentity())++count;br_int(count);br_locations(set,count);
 std::ostringstream o;set.Dump(o);br_text(o.str());
 br_end();}
