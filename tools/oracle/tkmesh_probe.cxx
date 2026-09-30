// TKMesh foundation oracle at OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
#include <iostream>
#include <sstream>
#include <cstring>
#include <cstdint>
#include <vector>
#include <Standard_Failure.hxx>
#include <gp_Dir.hxx>
#include <gp_Dir2d.hxx>
#include <gp_Pnt.hxx>
#include <gp_Pnt2d.hxx>
#include "tkgeomalgo_probe_common.cxx"
#include <BRepMeshData_Curve.hxx>
#include <BRepMeshData_PCurve.hxx>
#include <BRepMeshData_Model.hxx>
#include <BRepMeshData_Face.hxx>
#include <BRepMeshData_Wire.hxx>
#include <BRep_Builder.hxx>
#include <Geom_Plane.hxx>
#include <gp_Pln.hxx>
#include <IMeshData_ParametersListArrayAdaptor.hxx>
#include "tkmesh_probe_models.cxx"
#include "tkmesh_probe_tools.cxx"
#include "tkmesh_probe_pipeline.cxx"
#include "tkmesh_probe_shape_tool.cxx"
#include "tkmesh_probe_wire.cxx"
#include "tkmesh_probe_model_builder.cxx"
#include "tkmesh_probe_deflection.cxx"
void mesh_curve(std::istream& input) {
    int count, keep; input >> count >> keep;
    occ::handle<NCollection_IncAllocator> allocator = new NCollection_IncAllocator;
    occ::handle<BRepMeshData_Curve> curve = new (allocator) BRepMeshData_Curve(allocator);
    IMeshData_ParametersListArrayAdaptor<occ::handle<BRepMeshData_Curve>> array(curve);
    ga_int(array.Lower()); ga_int(array.Upper()); ga_flag(array.IsInstance(STANDARD_TYPE(Standard_Transient)));
    for (int i=0; i<count; ++i) curve->AddPoint(gp_Pnt(i, -i, i*2), i*0.25);
    curve->InsertPoint(count/2, gp_Pnt(7,8,9), -1.25);
    curve->GetPoint(0).SetX(4.5); curve->GetParameter(0)=-0.0;
    ga_int(array.Upper()); ga_real(array.Value(0));
    for(int i=0;i<curve->ParametersNb();++i){ga_pnt(curve->GetPoint(i));ga_real(curve->GetParameter(i));}
    curve->RemovePoint(count/2);curve->Clear(keep!=0);ga_int(curve->ParametersNb());
    for(int i=0;i<curve->ParametersNb();++i){ga_pnt(curve->GetPoint(i));ga_real(curve->GetParameter(i));}
    ga_flag(curve->IsKind(STANDARD_TYPE(IMeshData_ParametersList)));ga_end();
}
void mesh_pcurve(std::istream& input) {
    int count, keep, orientation;input>>count>>keep>>orientation;
    occ::handle<NCollection_IncAllocator> allocator=new NCollection_IncAllocator;
    occ::handle<BRepMeshData_PCurve> curve=new (allocator) BRepMeshData_PCurve(nullptr,TopAbs_Orientation(orientation),allocator);
    ga_flag(curve->IsForward());ga_flag(curve->IsInternal());ga_int(curve->GetOrientation());ga_flag(curve->GetFace()==nullptr);
    for(int i=0;i<count;++i){curve->AddPoint(gp_Pnt2d(i,-i),i*0.25);curve->GetIndex(i)=i+100;}
    curve->InsertPoint(count/2,gp_Pnt2d(7,8),-1.25);curve->GetPoint(0).SetX(4.5);curve->GetParameter(0)=-0.0;
    for(int i=0;i<curve->ParametersNb();++i){ga_pnt2d(curve->GetPoint(i));ga_real(curve->GetParameter(i));ga_int(curve->GetIndex(i));}
    curve->RemovePoint(count/2);curve->Clear(keep!=0);ga_int(curve->ParametersNb());
    for(int i=0;i<curve->ParametersNb();++i){ga_pnt2d(curve->GetPoint(i));ga_real(curve->GetParameter(i));ga_int(curve->GetIndex(i));}
    GA_TRY(curve->GetPoint(-1));GA_TRY(curve->GetIndex(count+1));GA_TRY(curve->GetParameter(-1));
    ga_flag(curve->IsKind(STANDARD_TYPE(IMeshData_ParametersList)));ga_end();
}
int main(){
    std::string line;
    while(std::getline(std::cin,line)){
        std::istringstream input(line);std::string operation;input>>operation;
        if(operation=="mesh_curve")mesh_curve(input);
        else if(operation=="mesh_pcurve")mesh_pcurve(input);
        else if(operation=="mesh_model")mesh_model(input);
        else if(operation=="mesh_pipeline")mesh_pipeline(input);
        else if(operation=="mesh_shape_tool")mesh_shape_tool(input);
        else if(operation=="mesh_shape_polygons")mesh_shape_polygons(input);
        else if(operation=="mesh_wire_order")mesh_wire_order(input);
        else if(operation=="mesh_outer_wire")mesh_outer_wire(input);
        else if(operation=="mesh_model_builder")mesh_model_builder(input);
        else if(operation=="mesh_deflection")mesh_deflection(input);
        else if(operation=="mesh_consistent")mesh_consistent(input);
        else if(operation=="mesh_tools")mesh_tools(input);
        else if(operation=="mesh_explorer")mesh_explorer(input);
        else {std::cerr<<"Unknown probe operation: "<<operation<<'\n';return 2;}
    }
}
