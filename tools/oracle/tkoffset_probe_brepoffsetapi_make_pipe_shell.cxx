// BRepOffsetAPI_MakePipeShell probe operations; the Rust twin is
// crates/tkoffset/examples/of_probe/make_pipe_shell.rs.
//
// The spine, the profiles and the supports are shapes of a shape program
// (tktopalgo_probe_shapes.cxx); shapes print through pr_shape (tkprim_probe_shape.cxx).
#include <memory>
#include <BRepOffsetAPI_MakePipeShell.hxx>
// The indices (in the map of the result, -1 for none) of a list of shapes.
static void of_list(const NCollection_IndexedMap<TopoDS_Shape,TopTools_ShapeMapHasher>& m,
                    const NCollection_List<TopoDS_Shape>& l){
 ga_int(l.Extent());for(const TopoDS_Shape& s:l)ga_int(s.IsNull()?-1:m.FindIndex(s));}
// The location mode of a request: 0 none (the corrected Frenet default), 1 SetMode(true),
// 2 SetMode(false), 3 SetDiscreteMode, 4 `origin dir xdir` SetMode(gp_Ax2), 5 `dir`
// SetMode(BiNormal), 6 `support` SetMode(SpineSupport) (prints its result), 7 `guide
// curvilinear contact` SetMode(AuxiliarySpine, CurvilinearEquivalence, KeepContact).
static void of_pipe_mode(GaReader& r,const TpShapes& t,BRepOffsetAPI_MakePipeShell& pipe){
 int mode=r.integer();
 if(mode==1)pipe.SetMode(true);
 else if(mode==2)pipe.SetMode(false);
 else if(mode==3)pipe.SetDiscreteMode();
 else if(mode==4){gp_Pnt o=r.pnt();gp_Dir d=r.dir();gp_Dir x=r.dir();pipe.SetMode(gp_Ax2(o,d,x));}
 else if(mode==5)pipe.SetMode(r.dir());
 else if(mode==6)ga_flag(pipe.SetMode(t.shapes.at(r.integer())));
 else if(mode==7){TopoDS_Wire g=TopoDS::Wire(t.shapes.at(r.integer()));bool c=r.flag();
  pipe.SetMode(g,c,(BRepFill_TypeOfContact)r.integer());}}
// `program spine mode... transition n (profile vertex contact correction)... solid`: a
// pipe shell maker with the location mode, SetTransitionMode unless `transition` is -1 (the
// constructor default otherwise) and the profiles (Add, at the spine vertex `vertex` unless
// -1); prints MakeSolid before Build (StdFail_NotDone), IsReady, IsDone after Build,
// GetStatus, when done ErrorOnSurface and the shape, the first and last shapes, with
// `solid` MakeSolid and the shape, Generated of every shape of the program and the
// number of the sections of Simulate(3).
void of_oa_pipe_shell_case(std::istream& input){
 GaReader r{input};TpShapes t;
 try{t=tp_shapes(r);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';ga_end();return;}
 TopoDS_Wire spine=TopoDS::Wire(t.shapes.at(r.integer()));
 std::unique_ptr<BRepOffsetAPI_MakePipeShell> pipe;
 try{pipe.reset(new BRepOffsetAPI_MakePipeShell(spine));of_pipe_mode(r,t,*pipe);}
 catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';ga_end();return;}
 int transition=r.integer();
 if(transition>=0)pipe->SetTransitionMode((BRepBuilderAPI_TransitionMode)transition);
 int n=r.integer();
 for(int k=0;k<n;++k){const TopoDS_Shape& p=t.shapes.at(r.integer());int v=r.integer();bool c=r.flag(),w=r.flag();
  if(v<0){GA_TRY(pipe->Add(p,c,w))}else{GA_TRY(pipe->Add(p,TopoDS::Vertex(t.shapes.at(v)),c,w))}}
 bool solid=r.flag();
 GA_TRY(ga_flag(pipe->MakeSolid()))
 ga_flag(pipe->IsReady());
 try{pipe->Build();}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';ga_end();return;}
 bool done=pipe->IsDone();
 ga_flag(done);ga_int(pipe->GetStatus());
 NCollection_IndexedMap<TopoDS_Shape,TopTools_ShapeMapHasher> map;
 if(done){ga_real(pipe->ErrorOnSurface());GA_TRY(pr_shape(pipe->Shape()))}
 GA_TRY(pr_shape(pipe->FirstShape()))
 GA_TRY(pr_shape(pipe->LastShape()))
 if(solid&&done)GA_TRY(ga_flag(pipe->MakeSolid());pr_shape(pipe->Shape()))
 if(done)TopExp::MapShapes(pipe->Shape(),map);
 for(const TopoDS_Shape& s:t.shapes)GA_TRY(of_list(map,pipe->Generated(s)))
 if(done){NCollection_List<TopoDS_Shape> l;GA_TRY(pipe->Simulate(3,l);ga_int(l.Extent()))}
 ga_end();}
