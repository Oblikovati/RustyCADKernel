// AIS_ConnectedInteractive and AIS_MultipleConnectedInteractive in a context: connection rules,
// presentations, selection (whole and decomposed), HLR and disconnection, the twin of
// crates/tkv3d/examples/v3_probe/ais_connected.rs. A request is `v3_ais_connected Case Arg`.
#include <algorithm>
#include <AIS_ConnectedInteractive.hxx>
#include <AIS_InteractiveContext.hxx>
#include <AIS_MultipleConnectedInteractive.hxx>
#include <AIS_Point.hxx>
#include <AIS_Shape.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRepBuilderAPI_MakeFace.hxx>
#include <Geom_CartesianPoint.hxx>
#include <Graphic3d_Camera.hxx>
#include <StdSelect_BRepOwner.hxx>
#include <gp_Cylinder.hxx>

class V3ProbeConnected : public AIS_ConnectedInteractive {
public:
  void hlr(const occ::handle<Graphic3d_Camera>& c,const occ::handle<TopLoc_Datum3D>& t,const occ::handle<Prs3d_Presentation>& p){computeHLR(c,t,p);}
};
static void v3_conn_selection(const occ::handle<SelectMgr_SelectableObject>& o,int mode){
  occ::handle<SelectMgr_Selection> sel=new SelectMgr_Selection(mode);
  o->ComputeSelection(sel,mode);ga_int(sel->Entities().Size());
  // The decomposed owners come from a map keyed by shapes hashed by address: the entities are
  // printed sorted, their order being memory dependent natively.
  std::vector<std::string> rows;
  for(const auto& item:sel->Entities()){const occ::handle<Select3D_SensitiveEntity>& e=item->BaseSensitive();
    const occ::handle<SelectMgr_EntityOwner>& owner=e->OwnerId();occ::handle<StdSelect_BRepOwner> b=occ::down_cast<StdSelect_BRepOwner>(owner);
    std::ostringstream row;row<<e->DynamicType()->Name()<<' '<<(owner->Selectable()==o)<<' '<<owner->HasLocation()<<' '<<owner->ComesFromDecomposition()
      <<' '<<owner->Priority()<<' '<<(b.IsNull()?-1:int(b->Shape().ShapeType()));rows.push_back(row.str());}
  std::sort(rows.begin(),rows.end());for(const std::string& row:rows)ga_text(row);
}
static void v3_conn_object(const occ::handle<AIS_InteractiveObject>& o){
  ga_int(o->Type());ga_int(o->Signature());ga_flag(o->AcceptShapeDecomposition());for(int m=0;m<4;++m)ga_flag(o->AcceptDisplayMode(m));
  ga_int(o->TypeOfPresentation3d());ga_flag(o->HasOwnPresentations());ga_int(o->Children().Extent());ga_flag(o->HasTransformation());
  if(o->HasTransformation()){const gp_Trsf& t=o->Transformation();ga_xyz(t.TranslationPart());}
  ga_int(o->Presentations().Size());
}
void v3_ais_connected(std::istream& input){
  GaReader r{input};const int kind=r.integer(),arg=r.integer();
  V3DisplayTrace trace;occ::handle<V3d_Viewer> viewer=new V3d_Viewer(new V3DisplayDriver(trace));
  occ::handle<AIS_InteractiveContext> ctx=new AIS_InteractiveContext(viewer);
  const gp_Ax3 ax(gp_Pnt(0,0,0),gp_Dir(0,0,1),gp_Dir(1,0,0));
  occ::handle<AIS_Shape> face=new AIS_Shape(BRepBuilderAPI_MakeFace(gp_Cylinder(ax,2.),0.,M_PI,0.,3.).Face());
  occ::handle<AIS_Shape> edge=new AIS_Shape(BRepBuilderAPI_MakeEdge(gp_Pnt(0,0,0),gp_Pnt(2,1,0)).Edge());
  occ::handle<AIS_Point> point=new AIS_Point(new Geom_CartesianPoint(1,2,3));
  gp_Trsf t;t.SetTranslation(gp_Vec(arg,2.*arg,0.5));
  switch(kind){
    case 0:{occ::handle<V3ProbeConnected> c=new V3ProbeConnected();v3_conn_object(c);
      GA_TRY(c->Connect(arg%2==0?occ::handle<AIS_InteractiveObject>(face):occ::handle<AIS_InteractiveObject>(edge),t))
      v3_conn_object(c);ga_flag(c->HasConnection());
      for(int mode:{0,1,2,4})GA_TRY(v3_conn_selection(c,mode))
      ctx->Display(c,0,0,false);ga_int(ctx->DisplayStatus(c));v3_conn_object(c);
      GA_TRY(occ::handle<Graphic3d_Camera> cam=new Graphic3d_Camera();cam->SetEyeAndCenter(gp_Pnt(10,8,6),gp_Pnt(0,0,0));
        occ::handle<Graphic3d_Structure> prs=new Graphic3d_Structure(viewer->StructureManager());c->hlr(cam,nullptr,prs);ga_int(prs->Groups().Size()))
      GA_TRY(ctx->Disconnect(c))v3_conn_object(c);break;}
    case 1:{ctx->Display(face,false);occ::handle<AIS_ConnectedInteractive> c=new AIS_ConnectedInteractive();
      GA_TRY(c->Connect(face))ga_flag(c->HasConnection());
      occ::handle<AIS_ConnectedInteractive> c2=new AIS_ConnectedInteractive();occ::handle<AIS_MultipleConnectedInteractive> m=new AIS_MultipleConnectedInteractive();
      GA_TRY(c2->Connect(m))ga_flag(c2->HasConnection());
      occ::handle<AIS_ConnectedInteractive> c3=new AIS_ConnectedInteractive();occ::handle<AIS_ConnectedInteractive> c4=new AIS_ConnectedInteractive();
      c3->Connect(edge,t);GA_TRY(c4->Connect(c3))ga_flag(c4->ConnectedTo()==edge);v3_conn_object(c4);break;}
    case 2:{occ::handle<AIS_MultipleConnectedInteractive> m=new AIS_MultipleConnectedInteractive();v3_conn_object(m);ga_flag(m->HasConnection());
      occ::handle<AIS_InteractiveObject> a=m->Connect(face,t);occ::handle<AIS_InteractiveObject> b=m->Connect(point);
      v3_conn_object(m);v3_conn_object(a);v3_conn_object(b);ga_flag(m->HasConnection());ga_flag(m->GetAssemblyOwner()==m->GlobalSelOwner());
      occ::handle<AIS_MultipleConnectedInteractive> m2=new AIS_MultipleConnectedInteractive();occ::handle<AIS_InteractiveObject> c=m2->Connect(m);
      v3_conn_object(m2);v3_conn_object(c);ga_flag(occ::down_cast<AIS_MultipleConnectedInteractive>(c)->GetAssemblyOwner()==m2->GetAssemblyOwner());
      for(int mode:{0,arg%3+1})GA_TRY(v3_conn_selection(m,mode))
      ctx->Display(m,0,arg%2,false);ga_int(ctx->DisplayStatus(m));v3_conn_object(m);ga_flag(a->HasInteractiveContext());
      NCollection_List<int> modes;ctx->ActivatedModes(m,modes);ga_int(modes.Extent());
      GA_TRY(ctx->Disconnect(m,a))v3_conn_object(m);GA_TRY(m->DisconnectAll())v3_conn_object(m);break;}
  }
  ga_int(int(trace.events.size()));ga_end();
}
