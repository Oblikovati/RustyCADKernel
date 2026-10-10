#include "tkv3d_probe_sensitive_display_driver.cxx"
#include "tkv3d_probe_sensitive_display_scene.cxx"
static void v3_display_trace(const V3DisplayTrace& trace){ga_int(trace.events.size());for(double value:trace.events)ga_real(value);}
static void v3_display_structure_state(const occ::handle<Graphic3d_Structure>& prs){
  ga_int(prs->Groups().Size());ga_int(prs->DisplayPriority());ga_flag(prs->IsDisplayed());
  ga_int(prs->TransformPersistence().IsNull()?0:int(prs->TransformPersistence()->Mode()));
}
void v3_sensitive_prs(std::istream& input){
  GaReader r{input};int kind=r.integer(),transform=r.integer(),flags=r.integer(),repeats=r.integer(),fail=r.integer();
  V3DisplayTrace trace;trace.fail=fail;occ::handle<V3d_Viewer> viewer=new V3d_Viewer(new V3DisplayDriver(trace));
  occ::handle<Graphic3d_Structure> prs=new Graphic3d_Structure(viewer->StructureManager());auto selection=v3_display_selection(kind,flags);if(flags&8)trace.appendSelection=selection;
  occ::handle<Graphic3d_TransformPers> persistence;if(flags&2)persistence=new Graphic3d_TransformPers(Graphic3d_TMF_ZoomPers,gp_Pnt(1,2,3));
  for(int i=0;i<repeats;++i){GA_TRY(SelectMgr::ComputeSensitivePrs(prs,selection,v3_display_location(transform),persistence))}
  v3_display_trace(trace);v3_display_structure_state(prs);ga_end();
}
class V3DisplaySelector:public SelectMgr_ViewerSelector {
public:
  void state(){ga_int(myStructs.Length());for(NCollection_Sequence<occ::handle<Graphic3d_Structure>>::Iterator it(myStructs);it.More();it.Next())v3_display_structure_state(it.Value());}
};
static void v3_display_objects(V3DisplaySelector& selector,const occ::handle<SelectMgr_Selection>& selection,int transform,int flags,V3DisplayTrace& trace){
  occ::handle<V3TraversalObject> object=new V3TraversalObject();object->SetLocalTransformation(v3_display_location(transform));
  if(flags&2)object->SetTransformPersistence(new Graphic3d_TransformPers(Graphic3d_TMF_ZoomPers,gp_Pnt(1,2,3)));
  object->AddSelection(selection,0);if(flags&16)trace.appendObject=object;selector.AddSelectableObject(object);selector.AddSelectableObject(new V3TraversalObject());
}
void v3_sensitive_display(std::istream& input){
  GaReader r{input};int kind=r.integer(),transform=r.integer(),flags=r.integer(),mode=r.integer(),repeats=r.integer(),fail=r.integer();
  V3DisplayTrace trace;trace.fail=fail;occ::handle<V3d_Viewer> viewer=new V3d_Viewer(new V3DisplayDriver(trace));auto view=viewer->CreateView();
  occ::handle<Aspect_NeutralWindow> window=new Aspect_NeutralWindow();window->SetSize(80,60);view->SetWindow(window);
  auto selection=v3_display_selection(kind,flags);selection->SetSelectionState(flags&4?SelectMgr_SOS_Deactivated:SelectMgr_SOS_Activated);
  V3DisplaySelector selector;v3_display_objects(selector,selection,transform,flags,trace);if(flags&8)trace.appendSelection=selection;
  for(int i=0;i<repeats;++i){int action=mode==3?i%3:mode;
    GA_TRY(if(action==0)selector.DisplaySensitive(view);else selector.DisplaySensitive(selection,v3_display_location(transform),view,action==2))
  }
  v3_display_trace(trace);selector.state();ga_end();
}
