// TopOpeBRepDS_DataStructure, TopOpeBRepDS_HDataStructure and TopOpeBRepDS_BuildTool probe
// operations; the Rust twin is crates/tkbool/examples/bl_probe/top_ope_brep_ds_build.rs.
//
// The faces are shapes of a shape program (tktopalgo_probe_shapes.cxx); curves print
// through bl_curve3d and bl_curve2d (tkbool_probe_brepfill_multi_line.cxx).
#include <TopOpeBRepDS_BuildTool.hxx>
#include <TopOpeBRepDS_CurvePointInterference.hxx>
#include <TopOpeBRepDS_HDataStructure.hxx>
#include <TopOpeBRepDS_PointIterator.hxx>
#include <TopOpeBRepDS_SurfaceCurveInterference.hxx>
// The edge: its tolerance, range, 3d curve, p-curves on both faces and vertex parameters.
static void bl_ds_edge(const TopoDS_Edge& e,const TopoDS_Face& f1,const TopoDS_Face& f2){
 GA_TRY(ga_real(BRep_Tool::Tolerance(e)))
 GA_TRY(double a,b;BRep_Tool::Range(e,a,b);ga_real(a);ga_real(b))
 GA_TRY(double a,b;bl_curve3d(BRep_Tool::Curve(e,a,b)))
 for(const TopoDS_Face& f:{f1,f2}){GA_TRY(double a,b;bl_curve2d(BRep_Tool::CurveOnSurface(e,f,a,b));ga_real(a);ga_real(b))}
 for(TopExp_Explorer ex(e,TopAbs_VERTEX);ex.More();ex.Next()){
  GA_TRY(const TopoDS_Vertex& v=TopoDS::Vertex(ex.Current());ga_int((int)v.Orientation());ga_real(BRep_Tool::Parameter(v,e));ga_real(BRep_Tool::Tolerance(v)))}}
// A curve of the data structure: its curve, tolerance, range, p-curves and mother.
static void bl_ds_curve(const TopOpeBRepDS_Curve& c){
 bl_curve3d(c.Curve());ga_real(c.Tolerance());
 double a=-1,b=-1;ga_flag(c.Range(a,b));ga_real(a);ga_real(b);
 GA_TRY(bl_curve2d(c.Curve1()))GA_TRY(bl_curve2d(c.Curve2()))ga_int(c.Mother());ga_int(c.DSIndex());}
// `program f1 f2 walk type c3d pc1 pc2 p1 p2 n (x y z u1 v1 u2 v2)...`: a curve of the
// data structure between the faces (the B-splines of degree 1 through the points for a
// walking line, else the unit circle at z = 1 on the Z axis) with its surface/curve
// interferences, its edge from the vertices at the points of parameters p1 and p2, then
// TopOpeBRepDS_BuildTool(type) RecomputeCurves on the edge; prints the new curve index,
// the curve count, the new curve and the edge.
void bl_ds_recompute_case(std::istream& input){
 GaReader r{input};TpShapes t;
 try{t=tp_shapes(r);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';ga_end();return;}
 const TopoDS_Face f1=TopoDS::Face(t.shapes.at(r.integer())),f2=TopoDS::Face(t.shapes.at(r.integer()));
 bool walk=r.flag();int type=r.integer();bool c3d=r.flag(),pc1=r.flag(),pc2=r.flag();double p1=r.real(),p2=r.real();int n=r.integer();
 NCollection_Array1<gp_Pnt> p(1,n);NCollection_Array1<gp_Pnt2d> q1(1,n),q2(1,n);
 for(int i=1;i<=n;i++){p(i)=r.pnt();q1(i)=r.pnt2d();q2(i)=r.pnt2d();}
 occ::handle<TopOpeBRepDS_HDataStructure> hds=new TopOpeBRepDS_HDataStructure();
 TopOpeBRepDS_DataStructure& ds=hds->ChangeDS();
 int i1=ds.AddShape(f1,1),i2=ds.AddShape(f2,2);
 occ::handle<Geom_Curve> C;occ::handle<Geom2d_Curve> PC1,PC2;
 if(walk){C=TopOpeBRepTool_CurveTool::MakeBSpline1fromPnt(p);PC1=TopOpeBRepTool_CurveTool::MakeBSpline1fromPnt2d(q1);PC2=TopOpeBRepTool_CurveTool::MakeBSpline1fromPnt2d(q2);}
 else C=new Geom_Circle(gp_Ax2(gp_Pnt(0.,0.,1.),gp_Dir(gp_Dir::D::Z)),1.);
 TopOpeBRepDS_Curve dc(C,1.e-7,walk);dc.SetShapes(f1,f2);
 occ::handle<TopOpeBRepDS_SurfaceCurveInterference> s1=new TopOpeBRepDS_SurfaceCurveInterference(TopOpeBRepDS_Transition(),TopOpeBRepDS_FACE,i1,TopOpeBRepDS_CURVE,0,PC1);
 occ::handle<TopOpeBRepDS_SurfaceCurveInterference> s2=new TopOpeBRepDS_SurfaceCurveInterference(TopOpeBRepDS_Transition(),TopOpeBRepDS_FACE,i2,TopOpeBRepDS_CURVE,0,PC2);
 dc.SetSCI(s1,s2);int ic=ds.AddCurve(dc);s1->Geometry(ic);s2->Geometry(ic);
 ds.AddShapeInterference(f1,s1);ds.AddShapeInterference(f2,s2);
 TopOpeBRepDS_BuildTool bt((TopOpeBRepTool_OutCurveType)type);bt.ChangeGeomTool().Define((TopOpeBRepTool_OutCurveType)type,c3d,pc1,pc2);
 TopoDS_Shape E,V1,V2;
 try{bt.MakeEdge(E,ds.Curve(ic));
  double par[2]={p1,p2};TopoDS_Shape* vs[2]={&V1,&V2};
  for(int k=0;k<2;k++){gp_Pnt pk=C->Value(par[k]);bt.MakeVertex(*vs[k],TopOpeBRepDS_Point(pk,1.e-7));
   vs[k]->Orientation(k==0?TopAbs_FORWARD:TopAbs_REVERSED);bt.AddEdgeVertex(E,*vs[k]);bt.Parameter(E,*vs[k],par[k]);}}
 catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';ga_end();return;}
 TopoDS_Edge EE=TopoDS::Edge(E);int inew=-1;
 try{bt.RecomputeCurves(ds.Curve(ic),EE,EE,inew,hds);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
 ga_int(inew);ga_int(ds.NbCurves());
 if(inew>0)bl_ds_curve(ds.Curve(inew));
 bl_ds_curve(ds.Curve(ic));
 bl_ds_edge(EE,f1,f2);
 ga_int(ds.ShapeInterferences(f1).Extent());ga_int(ds.ShapeInterferences(f2).Extent());
 ga_end();}
// `program s1 s2 refFirst c1 c2 mode`: the same domain relation of two shapes in a
// fresh data structure (FillShapesSameDomain with refFirst, or with the configurations
// c1 c2 when mode is 1), then twice more swapped; prints the index, reference,
// orientation, index and rank of both shapes and their same domain counts.
void bl_ds_same_domain_case(std::istream& input){
 GaReader r{input};TpShapes t;
 try{t=tp_shapes(r);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';ga_end();return;}
 const TopoDS_Shape s1=t.shapes.at(r.integer()),s2=t.shapes.at(r.integer());
 bool ref=r.flag();TopOpeBRepDS_Config c1=(TopOpeBRepDS_Config)r.integer(),c2=(TopOpeBRepDS_Config)r.integer();int mode=r.integer();
 TopOpeBRepDS_DataStructure ds;
 for(int k=0;k<2;k++){
  const TopoDS_Shape& a=k==0?s1:s2;const TopoDS_Shape& b=k==0?s2:s1;
  GA_TRY(if(mode==1)ds.FillShapesSameDomain(a,b,c1,c2,ref);else ds.FillShapesSameDomain(a,b,ref))
  for(const TopoDS_Shape& s:{s1,s2}){ga_int(ds.Shape(s));ga_int(ds.SameDomainRef(s));ga_int((int)ds.SameDomainOri(s));
   ga_int(ds.SameDomainInd(s));ga_int(ds.AncestorRank(s));ga_int(ds.ShapeSameDomain(s).Extent());}}
 GA_TRY(ds.UnfillShapesSameDomain(s1,s2);ga_int(ds.ShapeSameDomain(s1).Extent());ga_int(ds.ShapeSameDomain(s2).Extent()))
 ga_end();}
// `n (param orientation)...`: curve/point interferences of the parameters and transition
// orientations on one curve; prints the indices of SortOnParameter (L1, L2) and
// MinMaxOnParameter.
void bl_ds_sort_case(std::istream& input){
 GaReader r{input};int n=r.integer();
 NCollection_List<occ::handle<TopOpeBRepDS_Interference>> l;std::vector<occ::handle<TopOpeBRepDS_Interference>> all;
 for(int i=0;i<n;i++){double par=r.real();TopAbs_Orientation o=(TopAbs_Orientation)r.integer();
  occ::handle<TopOpeBRepDS_Interference> it=new TopOpeBRepDS_CurvePointInterference(TopOpeBRepDS_Transition(o),TopOpeBRepDS_CURVE,1,TopOpeBRepDS_POINT,i+1,par);
  l.Append(it);all.push_back(it);}
 occ::handle<TopOpeBRepDS_HDataStructure> hds=new TopOpeBRepDS_HDataStructure();
 NCollection_List<occ::handle<TopOpeBRepDS_Interference>> s;
 GA_TRY(hds->SortOnParameter(l,s);ga_int(s.Extent());for(const occ::handle<TopOpeBRepDS_Interference>& i:s)ga_int(i->Geometry()))
 GA_TRY(double a=-1,b=-1;hds->MinMaxOnParameter(l,a,b);ga_real(a);ga_real(b))
 ga_end();}
