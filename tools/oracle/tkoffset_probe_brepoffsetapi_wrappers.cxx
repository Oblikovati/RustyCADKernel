// BRepOffsetAPI_MakePipe, MakeEvolved, MakeDraft, MakeFilling, NormalProjection and
// FindContigousEdges probe operations; the Rust twin is
// crates/tkoffset/examples/of_probe/wrappers.rs.
//
// The shapes are shapes of a shape program (tktopalgo_probe_shapes.cxx); shapes print
// through pr_shape (tkprim_probe_shape.cxx) and lists through of_list
// (tkoffset_probe_brepoffsetapi_make_pipe_shell.cxx).
#include <BRepOffsetAPI_FindContigousEdges.hxx>
#include <BRepOffsetAPI_MakeDraft.hxx>
#include <BRepOffsetAPI_MakeEvolved.hxx>
#include <BRepOffsetAPI_MakeFilling.hxx>
#include <BRepOffsetAPI_MakePipe.hxx>
#include <BRepOffsetAPI_NormalProjection.hxx>
#include <Geom_Plane.hxx>
#include "tkbool_probe_evolved_digest.cxx"
// The sub-shapes of a kind in explorer order.
static std::vector<TopoDS_Shape> of_explored(const TopoDS_Shape& s,TopAbs_ShapeEnum kind){
 std::vector<TopoDS_Shape> out;for(TopExp_Explorer ex(s,kind);ex.More();ex.Next())out.push_back(ex.Current());return out;}
// `program spine profile mode c1`: a BRepOffsetAPI_MakePipe (the two-argument constructor
// for mode -1); prints IsDone, the shape, the first and last shapes, ErrorOnSurface,
// Generated of every program shape and, unless the profile is a vertex (its tables are
// null natively, the queries crash), Generated(spine edge, profile edge or vertex).
void of_oa_pipe_case(std::istream& input){
 GaReader r{input};TpShapes t;
 try{t=tp_shapes(r);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';ga_end();return;}
 TopoDS_Wire spine=TopoDS::Wire(t.shapes.at(r.integer()));TopoDS_Shape profile=t.shapes.at(r.integer());
 int mode=r.integer();bool c1=r.flag();
 std::unique_ptr<BRepOffsetAPI_MakePipe> pipe;
 try{pipe.reset(mode<0?new BRepOffsetAPI_MakePipe(spine,profile):new BRepOffsetAPI_MakePipe(spine,profile,(GeomFill_Trihedron)mode,c1));}
 catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';ga_end();return;}
 ga_flag(pipe->IsDone());
 NCollection_IndexedMap<TopoDS_Shape,TopTools_ShapeMapHasher> map;
 if(pipe->IsDone()){GA_TRY(pr_shape(pipe->Shape()))TopExp::MapShapes(pipe->Shape(),map);}
 GA_TRY(pr_shape(pipe->FirstShape()))GA_TRY(pr_shape(pipe->LastShape()))
 ga_real(pipe->ErrorOnSurface());
 for(const TopoDS_Shape& s:t.shapes)GA_TRY(of_list(map,pipe->Generated(s)))
 if(profile.ShapeType()!=TopAbs_VERTEX)for(const TopoDS_Shape& se:of_explored(spine,TopAbs_EDGE)){
  for(const TopoDS_Shape& pe:of_explored(profile,TopAbs_EDGE))GA_TRY(const TopoDS_Shape g=pipe->Generated(se,pe);ga_int(g.IsNull()?-1:map.FindIndex(g)))
  for(const TopoDS_Shape& pv:of_explored(profile,TopAbs_VERTEX))GA_TRY(const TopoDS_Shape g=pipe->Generated(se,pv);ga_int(g.IsNull()?-1:map.FindIndex(g)))}
 ga_end();}
// `program spine profile join axeprof solid profonspine tol volume`: a
// BRepOffsetAPI_MakeEvolved; prints the exception or IsDone, the shape (the volume through
// pr_shape, the classic result, the top and the bottom through the address-independent
// bl_evolved_digest of tkbool_probe_evolved_digest.cxx, as their sub-shape order follows
// shape addresses natively), then for every spine and profile sub-shape pair the size,
// types and orientations of GeneratedShapes.
void of_oa_evolved_case(std::istream& input){
 GaReader r{input};TpShapes t;
 try{t=tp_shapes(r);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';ga_end();return;}
 TopoDS_Shape spine=t.shapes.at(r.integer());TopoDS_Wire profile=TopoDS::Wire(t.shapes.at(r.integer()));
 GeomAbs_JoinType join=(GeomAbs_JoinType)r.integer();bool axe=r.flag(),solid=r.flag(),onspine=r.flag();
 double tol=r.real();bool volume=r.flag();
 std::unique_ptr<BRepOffsetAPI_MakeEvolved> evo;
 try{evo.reset(new BRepOffsetAPI_MakeEvolved(spine,profile,join,axe,solid,onspine,tol,volume));}
 catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';ga_end();return;}
 ga_flag(evo->IsDone());
 if(evo->IsDone()){if(volume){GA_TRY(pr_shape(evo->Shape()))}else{GA_TRY(bl_evolved_digest(evo->Shape()))}}
 GA_TRY(bl_evolved_digest(evo->Top()))GA_TRY(bl_evolved_digest(evo->Bottom()))
 NCollection_IndexedMap<TopoDS_Shape,TopTools_ShapeMapHasher> spines,profiles;
 TopExp::MapShapes(spine,spines);TopExp::MapShapes(profile,profiles);
 for(int i=1;i<=spines.Extent();++i)for(int j=1;j<=profiles.Extent();++j){
  const NCollection_List<TopoDS_Shape>& g=evo->GeneratedShapes(spines(i),profiles(j));
  ga_int(g.Size());for(const TopoDS_Shape& s:g){ga_int(s.ShapeType());ga_int(s.Orientation());}}
 ga_end();}
// `program shape dx dy dz angle style amin amax internal kind...`: a
// BRepOffsetAPI_MakeDraft with SetOptions unless style is -1 and SetDraft, performed up
// to a length (kind 0 `length`), a plane (kind 1 `origin normal keep`) or a stop shape
// (kind 2 `shape keep`); prints the exception or IsDone, the shape, the shell and
// Generated of every program shape.
void of_oa_draft_case(std::istream& input){
 GaReader r{input};TpShapes t;
 try{t=tp_shapes(r);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';ga_end();return;}
 TopoDS_Shape shape=t.shapes.at(r.integer());gp_Dir dir=r.dir();double angle=r.real();
 int style=r.integer();double amin=r.real(),amax=r.real();bool internal=r.flag();
 try{
  BRepOffsetAPI_MakeDraft draft(shape,dir,angle);
  if(style>=0)draft.SetOptions((BRepBuilderAPI_TransitionMode)style,amin,amax);
  draft.SetDraft(internal);
  int kind=r.integer();
  if(kind==0)draft.Perform(r.real());
  else if(kind==1){gp_Pnt o=r.pnt();gp_Dir n=r.dir();bool keep=r.flag();draft.Perform(new Geom_Plane(o,n),keep);}
  else{const TopoDS_Shape& stop=t.shapes.at(r.integer());draft.Perform(stop,r.flag());}
  ga_flag(draft.IsDone());
  NCollection_IndexedMap<TopoDS_Shape,TopTools_ShapeMapHasher> map;
  if(draft.IsDone()){GA_TRY(pr_shape(draft.Shape()))TopExp::MapShapes(draft.Shape(),map);}
  GA_TRY(pr_shape(draft.Shell()))
  for(const TopoDS_Shape& s:t.shapes)GA_TRY(of_list(map,draft.Generated(s)))}
 catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
 ga_end();}
// The record of bf_filling (tkbool_probe_brepfill_filling.cxx) through
// BRepOffsetAPI_MakeFilling: prints each Add, IsDone after Build, the shape through
// Shape() (StdFail_NotDone when not done), the errors and Generated when done.
void of_oa_filling_case(std::istream& input){
 GaReader r{input};TpShapes t;
 try{t=tp_shapes(r);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';ga_end();return;}
 int degree=r.integer(),nbpts=r.integer(),nbiter=r.integer();bool aniso=r.flag();
 double tol2d=r.real(),tol3d=r.real(),tolang=r.real(),tolcurv=r.real();
 int maxdeg=r.integer(),maxseg=r.integer();
 BRepOffsetAPI_MakeFilling f(degree,nbpts,nbiter,aniso,tol2d,tol3d,tolang,tolcurv,maxdeg,maxseg);
 try{
  int n=r.integer();
  for(int i=0;i<n;i++){const TopoDS_Edge e=TopoDS::Edge(t.shapes.at(r.integer()));int face=r.integer();
   GeomAbs_Shape order=(GeomAbs_Shape)r.integer();bool bound=r.flag();
   if(face<0)ga_int(f.Add(e,order,bound));else ga_int(f.Add(e,TopoDS::Face(t.shapes.at(face)),order,bound));}
  int m=r.integer();
  for(int i=0;i<m;i++){const TopoDS_Face face=TopoDS::Face(t.shapes.at(r.integer()));ga_int(f.Add(face,(GeomAbs_Shape)r.integer()));}
  int k=r.integer();
  for(int i=0;i<k;i++){
   if(r.integer()==0){ga_int(f.Add(r.pnt()));continue;}
   double u=r.real(),v=r.real();const TopoDS_Face face=TopoDS::Face(t.shapes.at(r.integer()));
   ga_int(f.Add(u,v,face,(GeomAbs_Shape)r.integer()));}
  int init=r.integer();if(init>=0)f.LoadInitSurface(TopoDS::Face(t.shapes.at(init)));
  f.Build();}
 catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';ga_end();return;}
 ga_flag(f.IsDone());
 GA_TRY(pr_shape(f.Shape()))
 if(!f.IsDone()){ga_end();return;}
 GA_TRY(ga_real(f.G0Error());ga_real(f.G1Error());ga_real(f.G2Error()))
 NCollection_IndexedMap<TopoDS_Shape,TopTools_ShapeMapHasher> map;
 TopExp::MapShapes(f.Shape(),map);
 for(const TopoDS_Shape& g:t.shapes)GA_TRY(of_list(map,f.Generated(g)))
 ga_end();}
// `program target n edge... maxdist with3d limit`: a BRepOffsetAPI_NormalProjection of the
// edges on the target; prints the exception or IsDone, the projection, Generated of
// every edge and, when done, BuildWire and its wires.
void of_oa_normal_projection_case(std::istream& input){
 GaReader r{input};TpShapes t;
 try{t=tp_shapes(r);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';ga_end();return;}
 BRepOffsetAPI_NormalProjection proj(t.shapes.at(r.integer()));
 int n=r.integer();std::vector<TopoDS_Shape> edges;
 for(int i=0;i<n;i++)edges.push_back(t.shapes.at(r.integer()));
 double maxdist=r.real();bool with3d=r.flag(),limit=r.flag();
 try{for(const TopoDS_Shape& e:edges)proj.Add(e);
  if(maxdist>=0)proj.SetMaxDistance(maxdist);proj.Compute3d(with3d);proj.SetLimit(limit);proj.Build();}
 catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';ga_end();return;}
 ga_flag(proj.IsDone());
 GA_TRY(pr_shape(proj.Projection()))
 NCollection_IndexedMap<TopoDS_Shape,TopTools_ShapeMapHasher> map;
 TopExp::MapShapes(proj.Projection(),map);
 for(const TopoDS_Shape& e:edges)GA_TRY(of_list(map,proj.Generated(e)))
 if(proj.IsDone()){NCollection_List<TopoDS_Shape> wires;GA_TRY(ga_flag(proj.BuildWire(wires));ga_int(wires.Extent()))}
 ga_end();}
// `program n shape... tol option`: a BRepOffsetAPI_FindContigousEdges of the shapes;
// prints NbContigousEdges, for each the program index (IsSame, -2 for none) of the edge
// and the size and indices of its couple, NbDegeneratedShapes and IsModified of every
// program shape.
void of_oa_contigous_case(std::istream& input){
 GaReader r{input};TpShapes t;
 try{t=tp_shapes(r);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';ga_end();return;}
 int n=r.integer();std::vector<TopoDS_Shape> shapes;
 for(int i=0;i<n;i++)shapes.push_back(t.shapes.at(r.integer()));
 double tol=r.real();bool option=r.flag();
 BRepOffsetAPI_FindContigousEdges finder(tol,option);
 try{for(const TopoDS_Shape& s:shapes)finder.Add(s);finder.Perform();}
 catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';ga_end();return;}
 int nb=finder.NbContigousEdges();ga_int(nb);
 for(int i=1;i<=nb;i++){
  GA_TRY(ga_int(tp_same_index(t,finder.ContigousEdge(i))))
  GA_TRY(const NCollection_List<TopoDS_Shape>& c=finder.ContigousEdgeCouple(i);ga_int(c.Extent());for(const TopoDS_Shape& s:c)ga_int(tp_same_index(t,s)))}
 GA_TRY(finder.ContigousEdge(nb+1))
 ga_int(finder.NbDegeneratedShapes());
 for(const TopoDS_Shape& s:t.shapes)ga_flag(finder.IsModified(s));
 ga_end();}
