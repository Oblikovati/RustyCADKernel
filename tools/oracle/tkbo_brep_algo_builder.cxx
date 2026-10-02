// Oracle for the upstream BRepAlgoAPI_BuilderAlgo argument and option defaults.
#include <BRepAlgoAPI_BuilderAlgo.hxx>
#include <BOPAlgo_GlueEnum.hxx>
#include <NCollection_List.hxx>
#include <TopoDS_Shape.hxx>

int main()
{
  BRepAlgoAPI_BuilderAlgo anAlgorithm;
  if (!anAlgorithm.Arguments().IsEmpty() || anAlgorithm.NonDestructive()
      || anAlgorithm.Glue() != BOPAlgo_GlueOff || !anAlgorithm.CheckInverted()
      || !anAlgorithm.HasHistory())
  {
    return 1;
  }
  NCollection_List<TopoDS_Shape> anArguments;
  anArguments.Append(TopoDS_Shape());
  anArguments.Append(TopoDS_Shape());
  anAlgorithm.SetArguments(anArguments);
  anArguments.Clear();
  if (anAlgorithm.Arguments().Extent() != 2)
  {
    return 2;
  }
  anAlgorithm.SetNonDestructive(true);
  anAlgorithm.SetGlue(BOPAlgo_GlueFull);
  anAlgorithm.SetCheckInverted(false);
  anAlgorithm.SetToFillHistory(false);
  return anAlgorithm.NonDestructive() && anAlgorithm.Glue() == BOPAlgo_GlueFull
                 && !anAlgorithm.CheckInverted() && !anAlgorithm.HasHistory()
             ? 0
             : 3;
}
