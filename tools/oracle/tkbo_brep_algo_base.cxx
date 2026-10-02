// Oracle for the base Shape() and option defaults, using a minimal upstream subclass.
#include <BRepAlgoAPI_Algo.hxx>
#include <Precision.hxx>

class BRepAlgoAPI_AlgoProbe final : public BRepAlgoAPI_Algo
{
public:
  BRepAlgoAPI_AlgoProbe() : BRepAlgoAPI_Algo() {}
};

int main()
{
  BRepAlgoAPI_AlgoProbe anAlgorithm;
  return anAlgorithm.Shape().IsNull() && !anAlgorithm.IsDone()
             && anAlgorithm.FuzzyValue() == Precision::Confusion()
         ? 0
         : 1;
}
