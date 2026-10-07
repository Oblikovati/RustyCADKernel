#pragma once
#include <NCollection_Sequence.hxx>
#include "tkernel_sequence_copy_item.hxx"
static std::string state(const NCollection_Sequence<CopyItem>& source)
{
  std::string values, reads;
  for (NCollection_Sequence<CopyItem>::Iterator cursor(source); cursor.More(); cursor.Next()) {
    if (!values.empty()) { values += ','; reads += ','; }
    values += std::to_string(cursor.Value().number);
    reads += std::to_string(cursor.Value().reads);
  }
  return source.IsEmpty() ? "- -" : values + " " + reads;
}
