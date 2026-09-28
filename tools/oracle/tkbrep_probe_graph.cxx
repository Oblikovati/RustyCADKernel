// The shape graphs of the TKBRep oracle: a request starts with a script that builds
// locations and shapes with TopoDS only, and the report prints shapes as descriptors
// that do not depend on addresses. The Rust twin is crates/tkbrep/examples/br_probe/graph.rs.
//
// Script: `nloc (kind params)... ncmd (code params)...` (see br_read_location and
// br_run_command). Location 0 is the identity; shapes are numbered from 0.
#include <TopoDS_TVertex.hxx>
#include <TopoDS_TEdge.hxx>
#include <TopoDS_TFace.hxx>
// Vertex and edge TShapes without geometry: TopoDS_TVertex is abstract and the
// TopoDS_TEdge constructor is protected, as BRep_TVertex and BRep_TEdge derive them.
class ProbeTVertex : public TopoDS_TVertex {
public:
 occ::handle<TopoDS_TShape> EmptyCopy() const override{return new ProbeTVertex();}
 DEFINE_STANDARD_RTTI_INLINE(ProbeTVertex,TopoDS_TVertex)
};
class ProbeTEdge : public TopoDS_TEdge {
public:
 DEFINE_STANDARD_RTTI_INLINE(ProbeTEdge,TopoDS_TEdge)
};
// A new shape of the given TopAbs_ShapeEnum value, FORWARD at the identity.
TopoDS_Shape br_make(int kind){
 TopoDS_Builder B;TopoDS_Shape S;
 switch(kind){
  case 0:{TopoDS_Compound C;B.MakeCompound(C);return C;}
  case 1:{TopoDS_CompSolid C;B.MakeCompSolid(C);return C;}
  case 2:{TopoDS_Solid C;B.MakeSolid(C);return C;}
  case 3:{TopoDS_Shell C;B.MakeShell(C);return C;}
  case 4:S.TShape(new TopoDS_TFace());break;
  case 5:{TopoDS_Wire C;B.MakeWire(C);return C;}
  case 6:S.TShape(new ProbeTEdge());break;
  default:S.TShape(new ProbeTVertex());break;}
 S.Location(TopLoc_Location());S.Orientation(TopAbs_FORWARD);return S;}
struct BrGraph{
 std::vector<TopLoc_Location> locs{TopLoc_Location()};
 std::vector<occ::handle<TopLoc_Datum3D>> datums;
 std::vector<TopoDS_Shape> shapes;
 // A registry shape; out-of-range indices name a null shape.
 TopoDS_Shape shape(int i)const{return i>=0&&i<int(shapes.size())?shapes[i]:TopoDS_Shape();}
 const TopLoc_Location& loc(int i)const{static TopLoc_Location none;return i>=0&&i<int(locs.size())?locs[i]:none;}
 int datum_id(const occ::handle<TopLoc_Datum3D>& d)const{
  for(size_t i=0;i<datums.size();++i)if(datums[i]==d)return int(i);return -1;}
 // The first registry shape with the same TShape, -1 when none.
 int tshape_id(const TopoDS_Shape& s)const{
  for(size_t i=0;i<shapes.size();++i)if(!shapes[i].IsNull()&&shapes[i].IsPartner(s))return int(i);return -1;}
};
void br_new_datum(BrGraph& g,const gp_Trsf& t){
 occ::handle<TopLoc_Datum3D> d=new TopLoc_Datum3D(t);g.datums.push_back(d);g.locs.push_back(TopLoc_Location(d));}
// Location kinds: 0 translation dx dy dz, 1 rotation about an axis through the origin
// (ax ay az angle), 2 product i j, 3 power i p, 4 scale s, 5 inverse i.
void br_read_location(BrGraph& g,BrReader& r){
 int kind=r.integer();gp_Trsf t;
 if(kind==0){double x=r.real(),y=r.real();t.SetTranslation(gp_Vec(x,y,r.real()));br_new_datum(g,t);return;}
 if(kind==1){double x=r.real(),y=r.real(),z=r.real();t.SetRotation(gp_Ax1(gp_Pnt(0,0,0),gp_Dir(x,y,z)),r.real());br_new_datum(g,t);return;}
 if(kind==4){t.SetScale(gp_Pnt(0,0,0),r.real());br_new_datum(g,t);return;}
 int i=r.integer();TopLoc_Location result;
 if(kind==2){int j=r.integer();BR_TRY(result=g.loc(i)*g.loc(j));}
 if(kind==3){int p=r.integer();BR_TRY(result=g.loc(i).Powered(p));}
 if(kind==5)BR_TRY(result=g.loc(i).Inverted());
 g.locs.push_back(result);}
// The location of shape s, as `count (datum power)...`, head first.
void br_chain(const BrGraph& g,const TopLoc_Location& l){
 std::vector<std::pair<int,int>> items;
 for(TopLoc_Location c=l;!c.IsIdentity();c=c.NextLocation())items.push_back({g.datum_id(c.FirstDatum()),c.FirstPower()});
 br_int(items.size());for(auto& i:items){br_int(i.first);br_int(i.second);}}
// A shape descriptor: `N` for a null shape, else the TShape id, the orientation and the location chain.
void br_desc(const BrGraph& g,const TopoDS_Shape& s){
 if(s.IsNull()){br_word("N");return;}
 br_int(g.tshape_id(s));br_int(s.Orientation());br_chain(g,s.Location());}
// The child used by the add and remove commands: shape c, oriented o, located at loc l.
TopoDS_Shape br_child(const BrGraph& g,BrReader& r){
 int c=r.integer(),o=r.integer(),l=r.integer();
 return g.shape(c).Oriented(TopAbs_Orientation(o)).Located(g.loc(l));}
void br_set_flag(TopoDS_Shape& s,int bit,bool v){
 switch(bit){case 0:s.Free(v);break;case 1:s.Modified(v);break;case 2:s.Checked(v);break;
  case 3:s.Orientable(v);break;case 4:s.Closed(v);break;case 5:s.Infinite(v);break;
  case 6:s.Convex(v);break;default:s.Locked(v);break;}}
// Commands, each printing `-` or the exception it raised: 1 make kind, 2 add p c o l,
// 3 remove p c o l, 4 set s o l (orientation and location of shape s), 5 moved copy
// s o l, 6 flag s bit v, 7 empty copy s, 8 Location s l raise, 9 Move s l raise,
// 10 reversed copy s, 11 complemented copy s, 12 composed copy s o, 13 nullify s,
// 14 null shape.
void br_run_command(BrGraph& g,BrReader& r){
 int code=r.integer();
 try{
  switch(code){
   case 1:g.shapes.push_back(br_make(r.integer()));break;
   case 2:{int p=r.integer();TopoDS_Shape c=br_child(g,r);TopoDS_Builder().Add(g.shapes.at(p),c);break;}
   case 3:{int p=r.integer();TopoDS_Shape c=br_child(g,r);TopoDS_Builder().Remove(g.shapes.at(p),c);break;}
   case 4:{int s=r.integer(),o=r.integer(),l=r.integer();g.shapes.at(s).Orientation(TopAbs_Orientation(o));g.shapes.at(s).Location(g.loc(l));break;}
   case 5:{int s=r.integer(),o=r.integer(),l=r.integer();g.shapes.push_back(g.shape(s).Oriented(TopAbs_Orientation(o)).Moved(g.loc(l)));break;}
   case 6:{int s=r.integer(),bit=r.integer();br_set_flag(g.shapes.at(s),bit,r.flag());break;}
   case 7:g.shapes.push_back(g.shape(r.integer()).EmptyCopied());break;
   case 8:{int s=r.integer(),l=r.integer();g.shapes.at(s).Location(g.loc(l),r.flag());break;}
   case 9:{int s=r.integer(),l=r.integer();g.shapes.at(s).Move(g.loc(l),r.flag());break;}
   case 10:g.shapes.push_back(g.shape(r.integer()).Reversed());break;
   case 11:g.shapes.push_back(g.shape(r.integer()).Complemented());break;
   case 12:{int s=r.integer();g.shapes.push_back(g.shape(s).Composed(TopAbs_Orientation(r.integer())));break;}
   case 13:g.shapes.at(r.integer()).Nullify();break;
   default:g.shapes.push_back(TopoDS_Shape());break;}
  br_word("-");
 }catch(const Standard_Failure& e){br_word(e.ExceptionType());}}
BrGraph br_graph(BrReader& r){
 BrGraph g;int nl=r.integer();for(int i=0;i<nl;++i)br_read_location(g,r);
 int nc=r.integer();for(int i=0;i<nc;++i)br_run_command(g,r);return g;}
