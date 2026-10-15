// Selection, owners, highlight and sensitive entities, the twin of
// crates/tkmeshvs/examples/mv_probe/select.rs.
static void mv_entity(const occ::handle<Select3D_SensitiveEntity>& e){
  ga_text(e->DynamicType()->Name());const occ::handle<SelectMgr_EntityOwner>& o=e->OwnerId();ga_int(o.IsNull()?-1:o->Priority());
  auto eo=occ::down_cast<MeshVS_MeshEntityOwner>(o);ga_flag(!eo.IsNull());
  if(!eo.IsNull()){ga_int(eo->ID());ga_int(int(eo->Type()));ga_flag(eo->IsGroup());ga_flag(eo->Owner()==nullptr);}
  ga_flag(!occ::down_cast<MeshVS_MeshOwner>(o).IsNull());
  ga_int(e->NbSubElements());ga_int(e->SensitivityFactor());
  Select3D_BndBox3d b=e->BoundingBox();ga_flag(b.IsValid());
  if(b.IsValid()){for(int a=0;a<3;++a)ga_real(b.CornerMin()[a]);for(int a=0;a<3;++a)ga_real(b.CornerMax()[a]);}
  ga_pnt(e->CenterOfGeometry());}
static const double MV_RAYS[5][2]={{.7,.2},{1.5,.5},{3.5,.5},{1,0},{9,9}};
static void mv_picks(const occ::handle<Select3D_SensitiveEntity>& e,bool allow){
  GA_TRY(e->BVH())
  for(const auto& ray:MV_RAYS){
    SelectMgr_SelectingVolumeManager m;m.AllowOverlapDetection(allow);m.InitAxisSelectingVolume(gp_Ax1(gp_Pnt(ray[0],ray[1],10),gp_Dir(0,0,-1)));
    GA_TRY(m.BuildSelectingVolume())
    SelectBasics_PickResult pick;GA_TRY(bool hit=e->Matches(m,pick);ga_flag(hit);if(hit){ga_real(pick.Depth());ga_real(pick.DistToGeomCenter());})}}
// `mv_select Mode Method Variant MaxNodes HiddenNodes HiddenElems Hilighter Selectable`.
void mv_select(std::istream& input){
  GaReader r{input};const int mode=r.integer(),method=r.integer(),variant=r.integer(),maxNodes=r.integer();
  const int hiddenNodes=r.integer(),hiddenElems=r.integer();const bool hilighter=r.flag();const int selectable=r.integer();
  occ::handle<MvMesh> mesh=new MvMesh(false);occ::handle<MvSource> s=mv_source(variant);mesh->SetDataSource(s);
  mesh->GetDrawer()->SetInteger(MeshVS_DA_MaxFaceNodes,maxNodes);if(hilighter)mesh->AddBuilder(new MeshVS_MeshPrsBuilder(mesh),true);
  mesh->SetMeshSelMethod(MeshVS_MeshSelectionMethod(method));mesh->SetHiddenNodes(mv_ids(hiddenNodes,1,14));
  mesh->SetHiddenElems(mv_ids(hiddenElems,101,8));if(selectable>=0)mesh->SetSelectableNodes(mv_ids(selectable,1,14));
  occ::handle<SelectMgr_EntityOwner> firstOwner;
  for(int pass=0;pass<2;++pass){
    occ::handle<SelectMgr_Selection> sel=new SelectMgr_Selection(mode);GA_TRY(mesh->ComputeSelection(sel,mode))
    ga_int(sel->Entities().Size());
    for(const auto& entity:sel->Entities()){
      const auto& e=entity->BaseSensitive();if(pass==1){ga_flag(e->OwnerId()==firstOwner);continue;}
      if(firstOwner.IsNull())firstOwner=e->OwnerId();mv_entity(e);mv_picks(e,true);}
    ga_int(mesh->GetOwnerMaps(false).Extent());ga_int(mesh->GetOwnerMaps(true).Extent());}
  auto owners=mesh->GetOwnerMaps(false);
  for(NCollection_DataMap<int,occ::handle<SelectMgr_EntityOwner>>::Iterator it(owners);it.More();it.Next()){
    ga_int(it.Key());auto mo=occ::down_cast<MeshVS_MeshOwner>(it.Value());ga_flag(!mo.IsNull());if(!mo.IsNull())ga_flag(mo->GetDataSource()==s);}
  ga_flag(mesh->IsWholeMeshOwner(firstOwner));ga_flag(mesh->IsWholeMeshOwner(nullptr));
  ga_end();}
static void mv_prs(const occ::handle<Prs3d_Presentation>& prs){
  mv_structure(prs);if(!prs.IsNull()){ga_int(prs->GetZLayer());ga_int(int(prs->DisplayPriority()));ga_flag(prs->IsDisplayed());}}
// `mv_highlight Mode Variant Key Immediate`: the owner of an entity highlighted, then selected.
void mv_highlight(std::istream& input){
  GaReader r{input};const int mode=r.integer(),variant=r.integer(),key=r.integer();const bool immediate=r.flag();
  V3DisplayTrace trace;occ::handle<V3d_Viewer> viewer=new V3d_Viewer(new V3DisplayDriver(trace));
  occ::handle<PrsMgr_PresentationManager> manager=new PrsMgr_PresentationManager(viewer->StructureManager());
  occ::handle<MvMesh> mesh=new MvMesh(false);occ::handle<MvSource> s=mv_source(variant);mesh->SetDataSource(s);
  mesh->AddBuilder(new MeshVS_MeshPrsBuilder(mesh),true);mesh->SetHiddenNodes(mv_ids(0,1,14));
  occ::handle<SelectMgr_Selection> sel=new SelectMgr_Selection(mode);GA_TRY(mesh->ComputeSelection(sel,mode))
  occ::handle<SelectMgr_EntityOwner> owner;
  for(const auto& entity:sel->Entities()){auto o=entity->BaseSensitive()->OwnerId();auto eo=occ::down_cast<MeshVS_MeshEntityOwner>(o);
    if(owner.IsNull()&&(eo.IsNull()||eo->ID()==key))owner=o;}
  ga_flag(owner.IsNull());if(owner.IsNull()){ga_end();return;}
  auto mo=occ::down_cast<MeshVS_MeshOwner>(owner);
  if(!mo.IsNull()){mo->SetDetectedEntities(mv_ids(0x2,1,14),mv_ids(0x2,101,8));mo->AddSelectedEntities(mv_ids(0x6,1,14),nullptr);
    mo->AddSelectedEntities(mv_ids(0x9,1,14),mv_ids(0x1,101,8));ga_flag(mo->IsForcedHilight());}
  occ::handle<Prs3d_Drawer> style=new Prs3d_Drawer();style->SetColor(Quantity_NOC_ORANGE);
  if(immediate)manager->BeginImmediateDraw();
  ga_flag(owner->IsHilighted(manager,0));GA_TRY(owner->HilightWithColor(manager,style,0))
  if(!mo.IsNull()){ga_flag(mo->IsForcedHilight());mo->Unhilight(manager);ga_flag(mo->IsForcedHilight());mv_hmap(mo->GetSelectedNodes());mv_hmap(mo->GetSelectedElements());}
  mv_prs(mesh->GetHilightPresentation(nullptr));
  NCollection_Sequence<occ::handle<SelectMgr_EntityOwner>> owners;owners.Append(owner);
  GA_TRY(mesh->HilightSelected(manager,owners))
  mv_prs(mesh->GetSelectPresentation(nullptr));mesh->ClearSelected();mv_prs(mesh->GetSelectPresentation(nullptr));
  if(!mo.IsNull()){mo->ClearSelectedEntities();mv_hmap(mo->GetSelectedNodes());}
  mv_trace(trace);ga_end();}
static occ::handle<Select3D_SensitiveEntity> mv_sensitive_entity(int kind,const occ::handle<SelectMgr_EntityOwner>& owner,const occ::handle<MvMesh>& mesh){
  NCollection_Array1<gp_Pnt> p(1,6);for(int i=1;i<=6;++i)p(i)=gp_Pnt(MV_NODES[MV_ELEMENTS[5].nodes[i-1]-1][0],MV_NODES[MV_ELEMENTS[5].nodes[i-1]-1][1],MV_NODES[MV_ELEMENTS[5].nodes[i-1]-1][2]);
  NCollection_Array1<gp_Pnt> pent(0,4);for(int i=0;i<5;++i){const double* n=MV_NODES[MV_ELEMENTS[2].nodes[i]-1];pent(i)=gp_Pnt(n[0],n[1],n[2]);}
  switch(kind){
    case 0:return new MeshVS_SensitiveQuad(owner,p);
    case 1:return new MeshVS_SensitiveQuad(owner,gp_Pnt(1,0,0),gp_Pnt(2,0,.5),gp_Pnt(2,1,.25),gp_Pnt(1,1,0));
    case 2:return new MeshVS_SensitivePolyhedron(owner,p,MeshVS_DataSource3D::CreatePrismTopology(3));
    case 3:return new MeshVS_SensitiveFace(owner,pent);
    case 4:return new MeshVS_SensitiveSegment(owner,gp_Pnt(1,0,0),gp_Pnt(1,1,0));
    case 5:return new MeshVS_DummySensitiveEntity(owner);
    case 6:return new MeshVS_SensitiveMesh(new MeshVS_MeshOwner(mesh.get(),mesh->GetDataSource(),5),3);
    case 7:return new MeshVS_SensitiveMesh(owner,1);
    case 8:return new MeshVS_CommonSensitiveEntity(owner,mesh,MeshVS_MSM_NODES);
    case 9:return new MeshVS_CommonSensitiveEntity(owner,mesh,MeshVS_MSM_PRECISE);
    default:return new MeshVS_SensitiveFace(owner,pent,Select3D_TOS_BOUNDARY);
  }}
// `mv_sensitive Kind Selectable`: one sensitive entity, its picks and its connected copy.
void mv_sensitive(std::istream& input){
  GaReader r{input};const int kind=r.integer(),selectable=r.integer();
  occ::handle<MvMesh> mesh=new MvMesh(false);mesh->SetDataSource(mv_source(3));
  if(selectable>=0)mesh->SetSelectableNodes(mv_ids(selectable,1,14));
  occ::handle<SelectMgr_EntityOwner> owner=new SelectMgr_EntityOwner(mesh,2);
  occ::handle<Select3D_SensitiveEntity> e;GA_TRY(e=mv_sensitive_entity(kind,owner,mesh))
  ga_flag(e.IsNull());if(e.IsNull()){ga_end();return;}
  mv_entity(e);mv_picks(e,true);mv_picks(e,false);
  occ::handle<Select3D_SensitiveEntity> c;GA_TRY(c=e->GetConnected())ga_flag(c.IsNull());if(!c.IsNull())mv_entity(c);
  if(kind==6||kind==7)ga_int(occ::down_cast<MeshVS_SensitiveMesh>(e)->GetMode());
  e->Clear();ga_flag(e->OwnerId().IsNull());ga_flag(e->HasInitLocation());ga_end();}
