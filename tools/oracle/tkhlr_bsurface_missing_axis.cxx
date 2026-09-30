// The pinned header declares Axis but neither .lxx nor any translation unit defines it.
#include <HLRBRep_BSurfaceTool.hxx>
int main(){BRepAdaptor_Surface surface;auto axis=HLRBRep_BSurfaceTool::Axis(surface);return axis.Location().X()!=0.;}
