// NCollection lists probe operations (twin of oracle_probe/ncollection/lists.rs).
//
// A request is a script of commands run on one list or sequence of ints and one iterator.
// Each command prints its result token (or `throw <exception class>`); the final state is
// printed after `|`. Commands that would leave a native iterator dangling or with a stale
// `myPrevious` reset it, so the script never exercises undefined behaviour.
#include <NCollection_HSequence.hxx>
#include <NCollection_List.hxx>
#include <NCollection_Sequence.hxx>
#include <Standard_Failure.hxx>
#include <istream>
#include <ostream>
#include <string>

namespace
{
template <class Container>
Container ncoll_lists_read(std::istream& in)
{
  int       aCount = 0;
  Container aResult;
  in >> aCount;
  for (int anIndex = 0; anIndex < aCount; ++anIndex)
  {
    int aValue = 0;
    in >> aValue;
    aResult.Append(aValue);
  }
  return aResult;
}

template <class Container>
void ncoll_lists_print(std::ostream& out, const Container& theItems)
{
  out << "[";
  for (const int aValue : theItems)
  {
    out << ' ' << aValue;
  }
  out << " ]";
}

template <class Iterator>
void ncoll_lists_print_iterator(std::ostream& out, const Iterator& theIter)
{
  if (theIter.More())
    out << " it=" << theIter.Value();
  else
    out << " it=end";
}

// Runs one list command; returns false on an unknown command.
bool ncoll_lists_list_command(const std::string&        theCmd,
                              std::istream&             in,
                              std::ostream&             out,
                              NCollection_List<int>&    theList,
                              NCollection_List<int>::Iterator& theIter)
{
  int aValue = 0;
  if (theCmd == "ap") { in >> aValue; out << theList.Append(aValue); }
  else if (theCmd == "pp") { in >> aValue; out << theList.Prepend(aValue); theIter.Init(theList); }
  else if (theCmd == "aa") { in >> aValue; theList.Append(aValue, theIter); out << '.'; }
  else if (theCmd == "rf") { theList.RemoveFirst(); theIter.Init(theList); out << '.'; }
  else if (theCmd == "rm") { theList.Remove(theIter); out << '.'; }
  else if (theCmd == "ib") { in >> aValue; out << theList.InsertBefore(aValue, theIter); }
  else if (theCmd == "ia") { in >> aValue; out << theList.InsertAfter(aValue, theIter); }
  else if (theCmd == "nx") { theIter.Next(); out << '.'; }
  else if (theCmd == "it") { theIter.Init(theList); out << '.'; }
  else if (theCmd == "rv") { theList.Reverse(); theIter.Init(theList); out << '.'; }
  else if (theCmd == "rx") { in >> aValue; out << theList.Remove(aValue); theIter.Init(theList); }
  else if (theCmd == "ct") { in >> aValue; out << theList.Contains(aValue); }
  else if (theCmd == "fl") { out << theList.First() << ',' << theList.Last(); }
  else if (theCmd == "lb") { auto aSrc = ncoll_lists_read<NCollection_List<int>>(in); theList.InsertBefore(aSrc, theIter); out << aSrc.Extent(); }
  else if (theCmd == "la") { auto aSrc = ncoll_lists_read<NCollection_List<int>>(in); theList.InsertAfter(aSrc, theIter); out << aSrc.Extent(); }
  else if (theCmd == "al") { auto aSrc = ncoll_lists_read<NCollection_List<int>>(in); theList.Append(aSrc); out << aSrc.Extent(); }
  else if (theCmd == "pl") { auto aSrc = ncoll_lists_read<NCollection_List<int>>(in); theList.Prepend(aSrc); theIter.Init(theList); out << aSrc.Extent(); }
  else return false;
  return true;
}

// Runs one sequence command; returns false on an unknown command.
bool ncoll_lists_seq_command(const std::string&                   theCmd,
                             std::istream&                        in,
                             std::ostream&                        out,
                             NCollection_Sequence<int>&           theSeq,
                             NCollection_Sequence<int>::Iterator& theIter)
{
  size_t anI = 0, aJ = 0;
  int    aValue = 0;
  if (theCmd == "ap") { in >> aValue; theSeq.Append(aValue); out << '.'; }
  else if (theCmd == "pp") { in >> aValue; theSeq.Prepend(aValue); out << '.'; }
  else if (theCmd == "ib") { in >> anI >> aValue; theSeq.InsertBefore(anI, aValue); out << '.'; }
  else if (theCmd == "ia") { in >> anI >> aValue; theSeq.InsertAfter(anI, aValue); out << '.'; }
  else if (theCmd == "ea") { in >> anI >> aValue; out << theSeq.EmplaceAfter(anI, aValue); }
  else if (theCmd == "eb") { in >> anI >> aValue; out << theSeq.EmplaceBefore(anI, aValue); }
  else if (theCmd == "rm") { in >> anI; theSeq.Remove(anI); theIter.Init(theSeq); out << '.'; }
  else if (theCmd == "rr") { in >> anI >> aJ; theSeq.Remove(anI, aJ); theIter.Init(theSeq); out << '.'; }
  else if (theCmd == "ex") { in >> anI >> aJ; theSeq.Exchange(anI, aJ); out << '.'; }
  else if (theCmd == "vl") { in >> anI; out << theSeq.Value(anI); }
  else if (theCmd == "at") { in >> anI; out << theSeq.At(anI); }
  else if (theCmd == "sv") { in >> anI >> aValue; theSeq.SetValue(anI, aValue); out << '.'; }
  else if (theCmd == "rv") { theSeq.Reverse(); out << '.'; }
  else if (theCmd == "fl") { out << theSeq.First() << ',' << theSeq.Last(); }
  else if (theCmd == "i0") { theIter.Init(theSeq, true); out << '.'; }
  else if (theCmd == "i1") { theIter.Init(theSeq, false); out << '.'; }
  else if (theCmd == "nx") { theIter.Next(); out << '.'; }
  else if (theCmd == "pv") { theIter.Previous(); out << '.'; }
  else if (theCmd == "ri") { theSeq.Remove(theIter); out << '.'; }
  else if (theCmd == "ic") { in >> aValue; theSeq.InsertAfter(theIter, aValue); out << '.'; }
  else if (theCmd == "ec") { in >> aValue; out << theSeq.EmplaceAfter(theIter, aValue); }
  else if (theCmd == "sp")
  {
    in >> anI;
    NCollection_Sequence<int> aTail;
    aTail.Append(-7);
    try { theSeq.Split(anI, aTail); } catch (const Standard_Failure&) { out << "tail" << aTail.Length() << ' '; throw; }
    // The moved tail dies with this scope; restart before the iterator can name it.
    theIter.Init(theSeq);
    ncoll_lists_print(out, aTail);
  }
  else if (theCmd == "qa") { in >> anI; auto aSrc = ncoll_lists_read<NCollection_Sequence<int>>(in); theSeq.InsertAfter(anI, aSrc); out << aSrc.Length(); }
  else if (theCmd == "qb") { in >> anI; auto aSrc = ncoll_lists_read<NCollection_Sequence<int>>(in); theSeq.InsertBefore(anI, aSrc); out << aSrc.Length(); }
  else if (theCmd == "sa") { auto aSrc = ncoll_lists_read<NCollection_Sequence<int>>(in); theSeq.Append(aSrc); out << aSrc.Length(); }
  else if (theCmd == "sq") { auto aSrc = ncoll_lists_read<NCollection_Sequence<int>>(in); theSeq.Prepend(aSrc); out << aSrc.Length(); }
  else return false;
  return true;
}

template <class Container, class Iterator, class Command>
void ncoll_lists_script(std::istream& in, std::ostream& out, Command theCommand)
{
  Container aItems = ncoll_lists_read<Container>(in);
  Iterator  anIter(aItems);
  std::string aCmd;
  while (in >> aCmd)
  {
    out << aCmd << '=';
    try
    {
      if (!theCommand(aCmd, in, out, aItems, anIter))
        throw std::runtime_error("unknown lists probe command " + aCmd);
    }
    catch (const Standard_Failure& theFailure)
    {
      out << "throw " << theFailure.ExceptionType();
    }
    out << ' ';
  }
  out << '|';
  ncoll_lists_print(out, aItems);
  out << " n=" << aItems.Size();
  ncoll_lists_print_iterator(out, anIter);
}

void ncoll_lists_hsequence(std::istream& in, std::ostream& out)
{
  const auto aFirst  = ncoll_lists_read<NCollection_Sequence<int>>(in);
  const auto aSecond = ncoll_lists_read<NCollection_Sequence<int>>(in);
  occ::handle<NCollection_HSequence<int>> aTarget = new NCollection_HSequence<int>(aFirst);
  occ::handle<NCollection_HSequence<int>> aSource = new NCollection_HSequence<int>(aSecond);
  aTarget->Append(42);
  aTarget->Append(aSource);
  aTarget->Append(aTarget);
  NCollection_Sequence<int> aMore;
  aMore.Append(-1);
  aTarget->Append(aMore);
  ncoll_lists_print(out, aTarget->Sequence());
  out << " source=" << aSource->Sequence().Length() << " more=" << aMore.Length() << ' '
      << aTarget->DynamicType()->Name();
}
} // namespace

static bool ncollection_lists_probe(const std::string& op, std::istream& in, std::ostream& out)
{
  if (op == "ncoll_lists_list")
    ncoll_lists_script<NCollection_List<int>, NCollection_List<int>::Iterator>(
      in, out, ncoll_lists_list_command);
  else if (op == "ncoll_lists_seq")
    ncoll_lists_script<NCollection_Sequence<int>, NCollection_Sequence<int>::Iterator>(
      in, out, ncoll_lists_seq_command);
  else if (op == "ncoll_lists_hseq")
    ncoll_lists_hsequence(in, out);
  else
    return false;
  return true;
}
