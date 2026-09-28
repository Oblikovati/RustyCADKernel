// Handle scripts (reference counts, identity, deletion) and TColStd_HPackedMapOfInteger cases.
// Included by tkernel_standard_runtime_probe.cxx.
namespace standard_runtime_probe
{
// Transients that count their destructions, so a script observes Delete().
class RuntimeProbeCounted : public Standard_Transient
{
public:
  ~RuntimeProbeCounted() override { ++Deleted(); }

  static int& Deleted()
  {
    static int THE_COUNT = 0;
    return THE_COUNT;
  }

  DEFINE_STANDARD_RTTI_INLINE(RuntimeProbeCounted, Standard_Transient)
};

class RuntimeProbeDerived : public RuntimeProbeCounted
{
public:
  DEFINE_STANDARD_RTTI_INLINE(RuntimeProbeDerived, RuntimeProbeCounted)
};

using Slots = std::vector<occ::handle<Standard_Transient>>;

// One script operation on three handle slots: a letter then one or two slot digits.
static void run_handle_op(Slots& theSlots, const std::string& theOp)
{
  const int i = theOp[1] - '0', j = theOp.size() > 2 ? theOp[2] - '0' : 0;
  switch (theOp[0])
  {
    case 'T': theSlots[i] = new RuntimeProbeCounted(); break;
    case 'D': theSlots[i] = new RuntimeProbeDerived(); break;
    case 'P': theSlots[i] = new Standard_Persistent(); break;
    case 'c': theSlots[i] = theSlots[j]; break;
    case 'm': theSlots[i] = std::move(theSlots[j]); break;
    case 'x': theSlots[i].Nullify(); break;
    case 'd': theSlots[i] = occ::down_cast<RuntimeProbeDerived>(theSlots[j]); break;
    case 's': theSlots[i] = occ::handle<Standard_Transient>::DownCast(theSlots[j]); break;
    case 'p': theSlots[i] = theSlots[j].get(); break;
    case 'r': theSlots[i].reset(new RuntimeProbeCounted()); break;
    case 'z': theSlots[i].reset(nullptr); break;
    default: throw std::runtime_error("unknown handle op " + theOp);
  }
}

// Counts, null flags, pairwise equality and destructions after one operation.
static std::string handle_state(const Slots& theSlots)
{
  std::string aState;
  for (const auto& aSlot : theSlots)
    aState += std::to_string(aSlot.IsNull() ? 0 : aSlot->GetRefCount()) + '.';
  for (const auto& aSlot : theSlots)
    aState += bit(!aSlot);
  aState += std::string{':', bit(theSlots[0] == theSlots[1]), bit(theSlots[0] != theSlots[2]),
                        bit(theSlots[1] == theSlots[2].get()), ':'};
  return aState + std::to_string(RuntimeProbeCounted::Deleted());
}

// "runtime_handle_script <op>...": the state after every operation, then after clearing.
static std::string handle_script_case(std::istream& theInput)
{
  RuntimeProbeCounted::Deleted() = 0;
  std::string aTrace;
  {
    Slots       aSlots(3);
    std::string anOp;
    while (theInput >> anOp)
    {
      run_handle_op(aSlots, anOp);
      aTrace += handle_state(aSlots) + ' ';
    }
  }
  return aTrace + "end:" + std::to_string(RuntimeProbeCounted::Deleted());
}

// "runtime_hpacked <nb buckets> <value>...": construction, sharing edits, copies and RTTI.
static std::string hpacked_case(std::istream& theInput)
{
  int aBuckets = 0;
  theInput >> aBuckets;
  const occ::handle<TColStd_HPackedMapOfInteger> aMap = new TColStd_HPackedMapOfInteger(aBuckets);
  const occ::handle<TColStd_HPackedMapOfInteger> aHolder = aMap;
  std::string aResult = std::to_string(aMap->Map().NbBuckets()) + ' ';
  std::vector<int> aValues;
  for (int aValue = 0; theInput >> aValue;)
  {
    aValues.push_back(aValue);
    aResult += bit(aHolder->ChangeMap().Add(aValue));
  }
  aResult += ' ' + std::to_string(aMap->Map().Extent()) + ' ';
  for (int aValue : aValues)
    aResult += bit(aMap->Map().Contains(aValue));
  TColStd_HPackedMapOfInteger aCopy(aMap->Map());
  aCopy.ChangeMap().Add(123456789);
  TColStd_PackedMapOfInteger  aSource(aMap->Map());
  TColStd_HPackedMapOfInteger aMoved(std::move(aSource));
  return aResult + ' ' + std::to_string(aCopy.Map().Extent()) + bit(aMap->Map().Contains(123456789)) + ' '
         + std::to_string(aMoved.Map().Extent()) + ' ' + aMap->DynamicType()->Name() + ' '
         + aMap->DynamicType()->Parent()->Name();
}
} // namespace standard_runtime_probe
