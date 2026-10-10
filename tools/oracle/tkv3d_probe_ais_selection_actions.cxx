// Martensite selection counterpart exercises the pinned OCCT context algorithms.
class V3SelectionActionsScene {
public:
  V3DetectionScene detection;
  occ::handle<AIS_GlobalStatus> status=new AIS_GlobalStatus();
  occ::handle<Prs3d_Drawer> style=new Prs3d_Drawer();
  V3SelectionActionsScene(int flags,int mask,int mode,int mutation,int failing):detection(flags,mask,mode,mutation,failing){
    detection.geometry();detection.trace.selectionEvents=true;
    detection.object->SetAutoHilight(!(flags&1024));
    detection.object->displayStatus(flags&4096?PrsMgr_DisplayStatus_Erased:PrsMgr_DisplayStatus_Displayed);
    status->SetDisplayMode(3);status->AddSelectionMode(0);status->SetSubIntensity(true);
    if(!(flags&2048))detection.ctx->registerObject(detection.object,status);
    style->SetDisplayMode(mode);style->SetTransparency(0.4f);detection.ctx->SetSelectionStyle(style);
    if(mutation==3)detection.object->SetHilightAttributes(style);
  }
  occ::handle<SelectMgr_EntityOwner> owner(int index){return index<0?occ::handle<SelectMgr_EntityOwner>():detection.owners[index];}
  NCollection_List<occ::handle<SelectMgr_EntityOwner>> owners(int mask){
    NCollection_List<occ::handle<SelectMgr_EntityOwner>> result;
    for(int i=0;i<3;++i)if(mask&(1<<i))result.Append(owner(i));return result;
  }
  NCollection_Array1<gp_Pnt2d> polygon(int offset){
    NCollection_Array1<gp_Pnt2d> result(1,4);result(1)=gp_Pnt2d(20+offset,10);result(2)=gp_Pnt2d(60+offset,10);
    result(3)=gp_Pnt2d(60+offset,50);result(4)=gp_Pnt2d(20+offset,50);return result;
  }
  void operation(int op,int a,int b){
    auto ctx=detection.ctx;auto object=detection.object;auto view=detection.view;
    switch(op){
      case 0:{auto list=owners(a);NCollection_Array1<occ::handle<SelectMgr_EntityOwner>> picked;
        if(!list.IsEmpty()){picked.Resize(1,list.Size(),false);int index=1;for(const auto& item:list)picked(index++)=item;}
        ga_int(ctx->Select(picked,AIS_SelectionScheme(b)));break;}
      case 1:ga_int(ctx->AddSelect(owner(a)));break;
      case 2:ga_int(ctx->AddSelect(object));break;
      case 3:ctx->SetSelected(owner(a),b!=0);break;
      case 4:ctx->SetSelected(object,b!=0);break;
      case 5:ctx->AddOrRemoveSelected(owner(a),b!=0);break;
      case 6:ctx->AddOrRemoveSelected(object,b!=0);break;
      case 7:ga_flag(ctx->SetSelectedState(owner(a),b!=0));break;
      case 8:ctx->HilightSelected(b!=0);break;
      case 9:ctx->UnhilightSelected(b!=0);break;
      case 10:ctx->ClearSelected(b!=0);break;
      case 11:ctx->InitSelected();break;
      case 12:ctx->NextSelected();break;
      case 13:detection.trace.mask=a;break;
      case 14:detection.trace.failing=a;break;
      case 15:ctx->pickedOwner(owner(a));break;
      case 16:ga_int(ctx->SelectDetected(AIS_SelectionScheme(b)));break;
      case 17:detection.trace.consumeClick=a!=0;break;
      case 18:ga_int(ctx->SelectPoint(NCollection_Vec2<int>(40+a,30),view,AIS_SelectionScheme(b)));break;
      case 19:ga_int(ctx->SelectRectangle(NCollection_Vec2<int>(20+a,10),NCollection_Vec2<int>(60+a,50),view,AIS_SelectionScheme(b)));break;
      case 20:ga_int(ctx->SelectPolygon(polygon(a),view,AIS_SelectionScheme(b)));break;
      case 21:ctx->highlightOwners(owners(a),b?style:occ::handle<Prs3d_Drawer>());break;
      case 22:ctx->unhighlightOwners(owners(a),b!=0);break;
      case 23:ctx->highlightSelected(owner(a));break;
      case 24:ctx->unhighlightSelected(b!=0);break;
      case 25:ctx->highlightGlobal(object,style,a);break;
      case 26:ctx->unhighlightGlobal(object);break;
      case 27:ctx->highlightWithSubintensity(object,a);break;
      case 28:ctx->highlightWithSubintensity(owner(a),b);break;
      case 29:ctx->HilightWithColor(object,style,b!=0);break;
      case 30:ctx->Unhilight(object,b!=0);break;
      case 31:case 32:{auto output=style;ga_flag(op==31?ctx->HighlightStyle(owner(a),output):ctx->HighlightStyle(object,output));ga_int(output.IsNull()?-99:output->DisplayMode());break;}
      case 33:ga_flag(ctx->IsHilighted(owner(a)));break;
      case 34:{auto output=ctx->getSelStyle(object,owner(a));ga_int(output.IsNull()?-99:output->DisplayMode());break;}
      case 35:ga_int(a==0?ctx->Select(b!=0):ctx->ShiftSelect(b!=0));break;
      case 36:ga_int(a==0?ctx->Select(20,10,60,50,view,b!=0):ctx->ShiftSelect(20,10,60,50,view,b!=0));break;
      case 37:ga_int(a==0?ctx->Select(polygon(0),view,b!=0):ctx->ShiftSelect(polygon(0),view,b!=0));break;
      case 38:{auto appended=owner(a);detection.trace.appendOnHighlight=[ctx,appended](){ctx->Selection()->AddSelect(appended);};break;}
      case 39:{V3DetectionScene foreign(0,7,2,0,0);ctx->SelectPoint(NCollection_Vec2<int>(40,30),foreign.view,AIS_SelectionScheme(b));break;}
      case 42:{occ::handle<V3DetectionObject> child=new V3DetectionObject(detection.trace);object->AddChild(child);ctx->setContextToObject(object);ga_flag(child->GetContext()==ctx.get());break;}
      case 43:{V3DetectionScene foreign(0,7,2,0,0);foreign.ctx->setContextToObject(object);break;}
      case 44:object->ClearSelections(false);break;
      case 40:ga_flag(ctx->isDetected(object));break;
      case 41:ctx->MoveTo(40+a,30,view,true);break;
    }
  }
  void state(){
    auto ctx=detection.ctx;ga_int(ctx->NbSelected());for(const auto& selected:ctx->Selection()->Objects())ga_int(detection.ownerId(selected));
    for(const auto& item:detection.owners)ga_flag(ctx->IsSelected(item));
    ga_flag(ctx->IsSelected(detection.object));ga_flag(ctx->MoreSelected());ga_int(detection.ownerId(ctx->SelectedOwner()));
    ga_flag(ctx->SelectedInteractive()==detection.object);ga_flag(ctx->FirstSelectedObject()==detection.object);
    ga_flag(ctx->IsHilighted(detection.object));ga_int(status->HilightStyle().IsNull()?-99:status->HilightStyle()->DisplayMode());
    ga_flag(ctx->MainPrsMgr()->IsImmediateModeOn());ga_flag(!ctx->LastActiveView().IsNull());ga_flag(detection.view->IsInvalidated());
    ga_int(detection.selector->NbPicked());ga_int(detection.trace.events.size());for(int event:detection.trace.events)ga_int(event);detection.trace.events.clear();
  }
};
void v3_ais_selection_actions(std::istream& input){
  GaReader r{input};int flags=r.integer(),mask=r.integer(),mode=r.integer(),mutation=r.integer(),failing=r.integer();
  V3SelectionActionsScene scene(flags,mask,mode,mutation,failing);scene.state();
  for(int count=r.integer();count>0;--count){int op=r.integer(),a=r.integer(),b=r.integer();GA_TRY(scene.operation(op,a,b))scene.state();}ga_end();
}
