// IMeshTools strategy wrapper and visitation oracle; no meshing is simulated.
#include <IMeshTools_Parameters.hxx>
#include <IMeshTools_ModelAlgo.hxx>
#include <IMeshTools_ShapeExplorer.hxx>
#include <Geom_Line.hxx>
#include <Geom_CylindricalSurface.hxx>
class OracleModelAlgorithm : public IMeshTools_ModelAlgo {
public:
    explicit OracleModelAlgorithm(int outcome): myOutcome(outcome) {}
protected:
    bool performInternal(const occ::handle<IMeshData_Model>&,const IMeshTools_Parameters&,const Message_ProgressRange&) override {
        if(myOutcome<0) throw Standard_Failure("deliberate oracle exception");
        return myOutcome!=0;
    }
private: int myOutcome;
};
class OracleShapeVisitor : public IMeshTools_ShapeVisitor {
public:
    std::vector<TopoDS_Shape> identities;
    void Visit(const TopoDS_Face& face) override { report(face); }
    void Visit(const TopoDS_Edge& edge) override { report(edge); }
    void report(const TopoDS_Shape& shape){
        for(size_t i=0;i<identities.size();++i)if(shape.IsPartner(identities[i])){ga_int(int(i));break;}
        ga_int(shape.ShapeType());ga_int(shape.Orientation());
        const auto vector=shape.Location().Transformation().TranslationPart();ga_xyz(vector);
    }
};
void mesh_tools(std::istream& input){
    int outcome;input>>outcome;IMeshTools_Parameters p;
    ga_int(p.MeshAlgo);ga_real(p.Angle);ga_real(p.Deflection);ga_real(p.AngleInterior);ga_real(p.DeflectionInterior);ga_real(p.MinSize);
    ga_flag(p.InParallel);ga_flag(p.Relative);ga_flag(p.InternalVerticesMode);ga_flag(p.ControlSurfaceDeflection);
    ga_flag(p.EnableControlSurfaceDeflectionAllSurfaces);ga_flag(p.CleanModel);ga_flag(p.AdjustMinSize);ga_flag(p.ForceFaceDeflection);ga_flag(p.AllowQualityDecrease);ga_real(p.RelMinSize());
    OracleModelAlgorithm algorithm(outcome);ga_flag(algorithm.Perform(nullptr,p,Message_ProgressRange()));ga_flag(algorithm.IsKind(STANDARD_TYPE(IMeshTools_ModelAlgo)));ga_end();
}
TopoDS_Face mesh_explorer_face(BRep_Builder& builder,const TopoDS_Edge& edge,bool cylinder){
    TopoDS_Wire wire;builder.MakeWire(wire);builder.Add(wire,edge);
    TopoDS_Face face;
    occ::handle<Geom_Surface> surface=cylinder?occ::handle<Geom_Surface>(new Geom_CylindricalSurface(gp_Ax3(),2)):occ::handle<Geom_Surface>(new Geom_Plane(gp_Pln()));
    builder.MakeFace(face,surface,1.e-7);builder.Add(face,wire);return face;
}
void mesh_explorer(std::istream& input){
    int orientation,duplicates;input>>orientation>>duplicates;BRep_Builder builder;
    TopoDS_Edge edge;builder.MakeEdge(edge,new Geom_Line(gp_Ax1()),1.e-7);
    TopoDS_Edge bare;builder.MakeEdge(bare);TopoDS_Face plane=mesh_explorer_face(builder,edge,false),cylinder=mesh_explorer_face(builder,edge,true);
    occ::handle<OracleShapeVisitor> visitor=new OracleShapeVisitor;visitor->identities={edge,bare,plane,cylinder};
    TopoDS_Compound shape;builder.MakeCompound(shape);gp_Trsf transform;transform.SetTranslation(gp_Vec(7,11,13));
    builder.Add(shape,edge.Oriented(TopAbs_Orientation(orientation)).Located(TopLoc_Location(transform)));builder.Add(shape,bare);
    builder.Add(shape,plane.Oriented(TopAbs_Orientation(orientation)).Located(TopLoc_Location(transform)));builder.Add(shape,cylinder);
    if(duplicates){transform.SetTranslation(gp_Vec(17,23,29));builder.Add(shape,plane.Located(TopLoc_Location(transform)));}
    IMeshTools_ShapeExplorer explorer(shape);ga_flag(explorer.GetShape().IsSame(shape));explorer.Accept(visitor);
    explorer.SetShape(TopoDS_Shape());explorer.Accept(visitor);ga_flag(explorer.GetShape().IsNull());ga_end();
}
