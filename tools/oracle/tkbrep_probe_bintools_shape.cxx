// BinTools oracle, part 3: BRep scenes (the scripts of tkbrep_probe_brep.cxx, with mesh
// tweaks) written with BinTools::Write and BinTools_ShapeWriter, read back with
// BinTools::Read and BinTools_ShapeReader, described and written again; given byte
// strings read by every reader; and every truncation of a written shape set. The Rust
// twin is crates/tkbrep/examples/br_probe/bintools_shape.rs.
#include <BinTools_LocationSet.hxx>
#include <BinTools_ShapeReader.hxx>
#include <BinTools_ShapeSet.hxx>
#include <BinTools_ShapeWriter.hxx>
#include <NCollection_HArray1.hxx>
// Mesh tweaks between the geometry and the commands of a scene: 1 t normals, 2 t UV
// nodes from the node X and Y, 3 t deflection, 4 p d a 3D polygon with parameters
// i*d, 5 n d deflection and parameters i*d of a node array.
void bt_tweak(BrGeometry& g,BrReader& r){
 int code=r.integer(),i=r.integer();
 if(code==1){g.triangulations.at(i)->ComputeNormals();return;}
 if(code==2){const occ::handle<Poly_Triangulation>& t=g.triangulations.at(i);t->AddUVNodes();
  for(int k=1;k<=t->NbNodes();++k){gp_Pnt p=t->Node(k);t->SetUVNode(k,gp_Pnt2d(p.X(),p.Y()));}return;}
 double d=r.real();
 if(code==3){g.triangulations.at(i)->Deflection(d);return;}
 if(code==4){const occ::handle<Poly_Polygon3D>& old=g.polygons3d.at(i);NCollection_Array1<double> params(1,old->NbNodes());
  for(int k=1;k<=old->NbNodes();++k)params(k)=k*d;
  occ::handle<Poly_Polygon3D> p=new Poly_Polygon3D(old->Nodes(),params);p->Deflection(d);g.polygons3d.at(i)=p;return;}
 const occ::handle<Poly_PolygonOnTriangulation>& n=g.nodes.at(i);n->Deflection(d);
 occ::handle<NCollection_HArray1<double>> params=new NCollection_HArray1<double>(1,n->NbNodes());
 for(int k=1;k<=n->NbNodes();++k)params->SetValue(k,k*d);
 n->SetParameters(params);}
// A scene with tweaks: `nloc loc... ngeom geom... ntweak tweak... ncmd cmd...`.
BrScene bt_scene(BrReader& r){
 BrScene s;int nl=r.integer();for(int i=0;i<nl;++i)br_read_location(s.graph,r);
 int ng=r.integer();for(int i=0;i<ng;++i)br_read_geometry(s.geometry,r);
 int nt=r.integer();for(int i=0;i<nt;++i)BR_TRY(bt_tweak(s.geometry,r);br_word("-"));
 int nc=r.integer();for(int i=0;i<nc;++i)br_brep_command(s,r);return s;}
// A shape read back, without addresses: `N`, or type, orientation, TShape id (first
// seen order), the eight flags as bits, the location (datum count, then the matrix)
// and the children.
void bt_walk(const TopoDS_Shape& s,std::vector<const TopoDS_TShape*>& ids){
 if(s.IsNull()){br_word("N");return;}
 const TopoDS_TShape* t=s.TShape().get();auto found=std::find(ids.begin(),ids.end(),t);
 int id=int(found-ids.begin());if(found==ids.end())ids.push_back(t);
 br_int(s.ShapeType());br_int(s.Orientation());br_int(id);
 br_int(s.Free()|s.Modified()<<1|s.Checked()<<2|s.Orientable()<<3|s.Closed()<<4|s.Infinite()<<5|s.Convex()<<6|s.Locked()<<7);
 int n=0;for(TopLoc_Location l=s.Location();!l.IsIdentity();l=l.NextLocation())++n;br_int(n);
 if(n>0){gp_Trsf tr=s.Location().Transformation();for(int i=1;i<=3;++i)for(int j=1;j<=4;++j)br_real(tr.Value(i,j));}
 br_int(s.NbChildren());for(TopoDS_Iterator it(s,false,false);it.More();it.Next())bt_walk(it.Value(),ids);}
void bt_walk(const TopoDS_Shape& s){std::vector<const TopoDS_TShape*> ids;bt_walk(s,ids);}
std::string bt_write_set(const TopoDS_Shape& s,bool wt,bool wn,int version){
 std::ostringstream o;BT_TRY(BinTools::Write(s,o,wt,wn,BinTools_FormatVersion(version)));return o.str();}
// BinTools::Read of `bytes`: the console text, the failure, the stream state and the
// shape.
// Runs `read` with std::cout captured, then prints the console text and the failure.
template<class F> void bt_captured_read(F read){
 std::string type,message;
 std::string text=br_captured([&]{try{read();}catch(const Standard_Failure& e){type=e.ExceptionType();message=e.GetMessageString();}});
 br_text(text);if(!type.empty())bt_failure_parts(type,message);}
TopoDS_Shape bt_read_set(const std::string& bytes){
 std::istringstream is(bytes);TopoDS_Shape s;
 bt_captured_read([&]{BinTools::Read(s,is);});br_int(int(is.rdstate()));bt_walk(s);return s;}
std::string bt_write_refs(const std::vector<TopoDS_Shape>& shapes,bool wt,bool wn){
 BinTools_ShapeWriter w;w.SetWithTriangles(wt);w.SetWithNormals(wn);std::ostringstream o;
 for(const TopoDS_Shape& s:shapes)BT_TRY(w.Write(s,o));return o.str();}
std::vector<TopoDS_Shape> bt_read_refs(const std::string& bytes,int count,int skip=0){
 BinTools_ShapeReader rd;std::istringstream is(bytes);is.seekg(skip);std::vector<TopoDS_Shape> shapes;
 for(int i=0;i<count;++i){TopoDS_Shape s;BT_TRY(rd.Read(is,s));br_int(int(is.rdstate()));bt_walk(s);shapes.push_back(s);}
 return shapes;}
// `br_bin_shape wt wn version scene root`: the shape set, read back and written again;
// the shape written twice with a shape writer, read back and written again; the set
// indices.
void br_bin_shape_case(std::istream& in){
 BrReader r{in};bool wt=r.flag(),wn=r.flag();int version=r.integer();
 BrScene sc=bt_scene(r);TopoDS_Shape root=sc.graph.shape(r.integer());
 std::string bytes=bt_write_set(root,wt,wn,version);bt_bytes(bytes);
 TopoDS_Shape back=bt_read_set(bytes);bt_bytes(bt_write_set(back,wt,wn,version));
 std::string refs=bt_write_refs({root,root},wt,wn);bt_bytes(refs);
 bt_bytes(bt_write_refs(bt_read_refs(refs,2),wt,wn));
 BinTools_ShapeSet set;set.SetWithTriangles(wt);set.SetWithNormals(wn);set.SetFormatNb(version);br_int(set.FormatNb());
 BT_TRY(br_int(set.Add(root)));br_int(set.NbShapes());br_int(set.Locations().NbLocations());
 for(const TopoDS_Shape& s:sc.graph.shapes)br_int(set.Index(s));
 br_end();}
// `br_bin_read reader zeros bytes`: 0 BinTools::Read, 1 BinTools_ShapeReader (then `skip
// count`: the start position and the number of shapes read), 2 curve
// set, 3 2D curve set, 4 surface set, 5 location set, 6 BinTools_ShapeSet::Read; each
// prints what it read and writes it again.
template<class Set,class Get> void bt_read_geometry_set(const std::string& bytes,Get get){
 Set set;std::istringstream is(bytes);BT_TRY(set.Read(is));br_int(int(is.rdstate()));
 for(int i=1;i<=64;++i){try{bt_eval(get(set,i));}catch(const Standard_Failure& e){br_word(e.ExceptionType());break;}}
 bt_bytes(bt_set_bytes(set));}
void br_bin_read_case(std::istream& in){
 BrReader r{in};int reader=r.integer();std::string bytes=bt_read_bytes(r);
 switch(reader){
  case 0:{TopoDS_Shape s=bt_read_set(bytes);bt_bytes(bt_write_set(s,true,false,BinTools_FormatVersion_CURRENT));break;}
  case 1:{int skip=r.integer(),count=r.integer();bt_bytes(bt_write_refs(bt_read_refs(bytes,count,skip),true,false));break;}
  case 2:bt_read_geometry_set<BinTools_CurveSet>(bytes,[](const BinTools_CurveSet& s,int i){return s.Curve(i);});break;
  case 3:bt_read_geometry_set<BinTools_Curve2dSet>(bytes,[](const BinTools_Curve2dSet& s,int i){return s.Curve2d(i);});break;
  case 4:bt_read_geometry_set<BinTools_SurfaceSet>(bytes,[](const BinTools_SurfaceSet& s,int i){return s.Surface(i);});break;
  case 5:{BinTools_LocationSet set;std::istringstream is(bytes);BT_TRY(set.Read(is));br_int(int(is.rdstate()));
   br_int(set.NbLocations());std::ostringstream o;BT_TRY(set.Write(o));bt_bytes(o.str());break;}
  default:{BinTools_ShapeSet set;std::istringstream is(bytes);
   bt_captured_read([&]{set.Read(is);});br_int(int(is.rdstate()));br_int(set.FormatNb());br_int(set.NbShapes());
   std::ostringstream o;BT_TRY(set.Write(o));bt_bytes(o.str());break;}}
 br_end();}
// The cut positions a truncation test skips: a stream that ends after the tag of a
// geometry table and before the first digit of its count leaves the count
// uninitialised natively (the extraction cannot start).
bool bt_undefined_cut(const std::string& bytes,size_t cut){
 for(const char* tag:{"Locations ","Curve2ds ","Curves ","Surfaces "}){
  size_t at=bytes.find(tag);if(at==std::string::npos)continue;
  size_t word_end=at+std::strlen(tag)-1;if(cut>=word_end&&cut<=word_end+1)return true;}
 return false;}
// `br_bin_truncate wt wn version step scene root`: BinTools::Read of every prefix of the
// written shape set whose length is a multiple of `step`: the failure (or the shape
// type), the console text and the stream state.
void br_bin_truncate_case(std::istream& in){
 BrReader r{in};bool wt=r.flag(),wn=r.flag();int version=r.integer(),step=r.integer();
 BrScene sc=bt_scene(r);TopoDS_Shape root=sc.graph.shape(r.integer());
 std::string bytes=bt_write_set(root,wt,wn,version);br_int((long long)bytes.size());
 for(size_t cut=0;cut<bytes.size();cut+=size_t(step)){
  if(bt_undefined_cut(bytes,cut))continue;
  std::istringstream is(bytes.substr(0,cut));TopoDS_Shape s;std::string type,message;
  std::string text=br_captured([&]{try{BinTools::Read(s,is);}catch(const Standard_Failure& e){type=e.ExceptionType();message=e.GetMessageString();}});
  if(!type.empty())bt_failure_parts(type,message);else br_int(s.IsNull()?-1:int(s.ShapeType()));
  br_text(text);br_int(int(is.rdstate()));}
 br_end();}
