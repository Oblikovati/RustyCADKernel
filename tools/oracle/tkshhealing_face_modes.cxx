#include <ShapeFix_Face.hxx>
#include <ShapeFix_Wire.hxx>
#include <iostream>
void row(const char* name,ShapeFix_Face& tool){
 std::cout<<name;
 for(int value:{tool.FixWireMode(),tool.FixOrientationMode(),tool.FixAddNaturalBoundMode(),tool.FixMissingSeamMode(),tool.FixSmallAreaWireMode(),tool.RemoveSmallAreaFaceMode(),tool.FixIntersectingWiresMode(),tool.FixLoopWiresMode(),tool.FixSplitFaceMode(),tool.AutoCorrectPrecisionMode(),tool.FixPeriodicDegeneratedMode()})std::cout<<' '<<value;
 std::cout<<' '<<tool.FixWireTool()->ModifyTopologyMode()<<'\n';
}
int main(){ShapeFix_Face tool;row("default",tool);
 tool.FixWireMode()=3;tool.FixOrientationMode()=3;tool.FixAddNaturalBoundMode()=3;tool.FixMissingSeamMode()=3;tool.FixSmallAreaWireMode()=3;tool.RemoveSmallAreaFaceMode()=3;tool.FixIntersectingWiresMode()=3;tool.FixLoopWiresMode()=3;tool.FixSplitFaceMode()=3;tool.AutoCorrectPrecisionMode()=3;tool.FixPeriodicDegeneratedMode()=3;
 tool.FixWireTool()->ModifyTopologyMode()=true;row("changed",tool);tool.ClearModes();row("cleared",tool);
}
