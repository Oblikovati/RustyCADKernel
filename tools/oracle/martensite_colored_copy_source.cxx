// Actual copy constructors, live source slots and exception repair; no synthetic appearance facade.
#include "martensite_colored_copy_source.hxx"
static void copy_source_attempt(const occ::handle<AIS_Shape>& slot, const ColoredExpansionInputs& input)
{
  try
  {
    const occ::handle<AIS_ColoredShape> copy = new AIS_ColoredShape(slot);
    copy_source_result("copy-result", copy, input);
  }
  catch (const Standard_Failure& error) { std::cout << "copy-failure " << error.ExceptionType() << '\n'; }
  catch (const std::runtime_error&) { std::cout << "copy-failure unrelated\n"; }
}
static void copy_source_case(const ColoredExpansionInputs& input, int stage, bool owned, int failure)
{
  const occ::handle<ColoredCopySourceProbe> original = new ColoredCopySourceProbe(input.base);
  const occ::handle<AIS_Shape> incoming = new AIS_ColoredShape(input.external);
  occ::handle<AIS_Shape> slot(original.get());
  copy_source_appearance(slot, false);
  if (owned) copy_source_appearance(incoming, true);
  original->slot = &slot; original->incoming = &incoming;
  original->stage = stage; original->failure = failure;
  std::cout << "case " << stage << ' ' << owned << ' ' << failure << '\n';
  copy_source_attempt(slot, input);
  std::cout << "after-copy " << original->GetRefCount() << ' ' << incoming->GetRefCount()
    << ' ' << input.base.TShape()->GetRefCount() << ' ' << input.external.TShape()->GetRefCount() << '\n';
  const occ::handle<AIS_ColoredShape> repair = new AIS_ColoredShape(slot);
  copy_source_result("repair-result", repair, input);
}
int main()
{
  const ColoredExpansionInputs input;
  for (int stage : {3,4,5})
    for (bool owned : {false,true})
      for (int failure : {0,1,2}) copy_source_case(input, stage, owned, failure);
}
