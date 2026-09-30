// Native model behavior, independent of the mesher itself.
void mesh_status(IMeshData_StatusOwner& owner,int mask,int unset){
    for(int bit=1;bit<=256;bit*=2)if(mask&bit)owner.SetStatus(IMeshData_Status(bit));
    ga_int(owner.GetStatusMask());
    for(int bit=0;bit<=256;bit=bit==0?1:bit*2){ga_flag(owner.IsSet(IMeshData_Status(bit)));ga_flag(owner.IsEqual(IMeshData_Status(bit)));}
    owner.UnsetStatus(IMeshData_Status(unset));ga_int(owner.GetStatusMask());
}
void mesh_edge_report(const IMeshData::IEdgeHandle& edge){
    ga_flag(edge->GetSameParam());ga_flag(edge->GetSameRange());ga_flag(edge->GetDegenerated());
    ga_real(edge->GetDeflection());ga_real(edge->GetAngularDeflection());ga_flag(edge->IsFree());
    edge->SetSameParam(false);edge->SetSameRange(false);edge->SetDegenerated(true);edge->SetDeflection(-2);edge->SetAngularDeflection(0.75);
    ga_flag(edge->GetSameParam());ga_flag(edge->GetSameRange());ga_flag(edge->GetDegenerated());ga_real(edge->GetDeflection());ga_real(edge->GetAngularDeflection());
}
void mesh_model(std::istream& input){
    int mask,unset,orientation;input>>mask>>unset>>orientation;
    BRep_Builder builder;TopoDS_Compound shape;builder.MakeCompound(shape);
    occ::handle<BRepMeshData_Model> model=new BRepMeshData_Model(shape);
    ga_flag(model->GetShape().IsSame(shape));ga_real(model->GetMaxSize());model->SetMaxSize(19);ga_real(model->GetMaxSize());
    ga_int(model->FacesNb());ga_int(model->EdgesNb());
    TopoDS_Face topoFace;builder.MakeFace(topoFace,new Geom_Plane(gp_Pln(gp_Pnt(0,0,2),gp_Dir(0,0,1))),1.e-7);
    auto face=model->AddFace(topoFace);ga_flag(model->GetFace(0)==face);ga_flag(face->GetFace().IsSame(topoFace));
    ga_pnt(face->GetSurface()->Value(2,3));ga_real(face->GetDeflection());face->SetDeflection(0.25);ga_real(face->GetDeflection());
    mesh_status(*face,mask,unset);ga_flag(face->IsValid());ga_int(face->WiresNb());
    TopoDS_Wire topoWire;builder.MakeWire(topoWire);auto wire=face->AddWire(topoWire,3);
    ga_flag(face->GetWire(0)==wire);ga_flag(wire->GetWire().IsSame(topoWire));ga_int(wire->EdgesNb());
    ga_real(wire->GetDeflection());wire->SetDeflection(0.5);ga_real(wire->GetDeflection());mesh_status(*wire,mask,unset);
    TopoDS_Edge topoEdge;builder.MakeEdge(topoEdge);auto edge=model->AddEdge(topoEdge);
    ga_flag(model->GetEdge(0)==edge);ga_flag(edge->GetEdge().IsSame(topoEdge));mesh_edge_report(edge);mesh_status(*edge,mask,unset);
    ga_int(wire->AddEdge(edge.get(),TopAbs_Orientation(orientation)));ga_flag(wire->GetEdge(0)==edge.get());ga_int(wire->GetEdgeOrientation(0));
    auto first=edge->AddPCurve(face.get(),TopAbs_FORWARD);auto second=edge->AddPCurve(face.get(),TopAbs_REVERSED);auto third=edge->AddPCurve(face.get(),TopAbs_INTERNAL);
    ga_int(edge->PCurvesNb());ga_flag(edge->IsFree());ga_flag(edge->GetPCurve(0)==first);ga_flag(edge->GetPCurve(face.get(),TopAbs_FORWARD)==first);
    ga_flag(edge->GetPCurve(face.get(),TopAbs_REVERSED)==third);ga_flag(edge->GetPCurve(face.get(),TopAbs_EXTERNAL)==third);
    ga_flag(first->GetFace()==face.get());for(const int index:edge->GetPCurves(face.get()))ga_int(index);
    for(int i=0;i<4;++i){edge->GetCurve()->AddPoint(gp_Pnt(i,0,0),i);first->AddPoint(gp_Pnt2d(i,0),i);}
    edge->Clear(true);ga_int(edge->GetCurve()->ParametersNb());ga_int(first->ParametersNb());ga_int(second->ParametersNb());edge->Clear(false);ga_int(edge->GetCurve()->ParametersNb());
    occ::handle<NCollection_IncAllocator> allocator=new NCollection_IncAllocator;auto previous=edge->GetCurve();edge->SetCurve(new (allocator) BRepMeshData_Curve(allocator));ga_flag(edge->GetCurve()!=previous);
    model->SetShape(topoFace);ga_flag(model->GetShape().IsSame(topoFace));ga_int(model->FacesNb());ga_int(model->EdgesNb());
    ga_flag(edge->IsKind(STANDARD_TYPE(IMeshData_Shape)));ga_flag(wire->IsKind(STANDARD_TYPE(IMeshData_TessellatedShape)));ga_flag(face->IsKind(STANDARD_TYPE(IMeshData_Face)));ga_end();
}
