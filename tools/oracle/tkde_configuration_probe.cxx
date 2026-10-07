// Native oracle for the TKDE port (crates/tkde): the resource syntax of
// DE_ConfigurationContext, the text of DE_ShapeFixConfigurationNode::Save, and the
// registry order of DE_Wrapper (bind, unbind, priority, provider lookup, save, copy).
// Each line is `key|value`; crates/tkde/tests/de_native_oracle.rs prints the same lines
// from the port. Build and compare: tools/oracle/tkde_configuration_probe.md.
#include <DE_ConfigurationContext.hxx>
#include <DE_ConfigurationNode.hxx>
#include <DE_PluginHolder.hxx>
#include <DE_Provider.hxx>
#include <DE_ShapeFixConfigurationNode.hxx>
#include <DE_ValidationUtils.hxx>
#include <DE_Wrapper.hxx>
#include <Message.hxx>
#include <Message_Messenger.hxx>
#include <Message_PrinterOStream.hxx>
#include <NCollection_Buffer.hxx>
#include <Standard_Failure.hxx>
#include <TDocStd_Document.hxx>

#include <cstdio>
#include <iostream>
#include <sstream>
#include <string>

namespace
{
std::string esc(const TCollection_AsciiString& theText)
{
  std::string aResult;
  for (int anIndex = 1; anIndex <= theText.Length(); ++anIndex)
  {
    const char aChar = theText.Value(anIndex);
    if (aChar == '\n')
      aResult += "\\n";
    else if (aChar == '\t')
      aResult += "\\t";
    else if (aChar == '\r')
      aResult += "\\r";
    else if (aChar == '|')
      aResult += "\\p";
    else
      aResult += aChar;
  }
  return aResult;
}

// Joins its operands left to right: `<<` sequences its operands (C++17), `+` does not.
#define TXT(theExpr) ([&] { std::ostringstream aTxt; aTxt << theExpr; return aTxt.str(); }())

void line(const std::string& theKey, const std::string& theValue)
{
  std::cout << theKey << "|" << theValue << std::endl;
}

// A provider that only names itself, as the base class leaves every transfer unsupported.
class ProbeProvider : public DE_Provider
{
public:
  ProbeProvider(const occ::handle<DE_ConfigurationNode>& theNode,
                const TCollection_AsciiString&           theFormat,
                const TCollection_AsciiString&           theVendor)
      : DE_Provider(theNode),
        myFormat(theFormat),
        myVendor(theVendor)
  {
  }

  TCollection_AsciiString GetFormat() const override { return myFormat; }

  TCollection_AsciiString GetVendor() const override { return myVendor; }

private:
  TCollection_AsciiString myFormat;
  TCollection_AsciiString myVendor;
};

// A shape-healing node with a fixed format, vendor, extensions and capabilities.
class ProbeNode : public DE_ShapeFixConfigurationNode
{
public:
  ProbeNode(const TCollection_AsciiString& theFormat,
            const TCollection_AsciiString& theVendor,
            const TCollection_AsciiString& theExtensions,
            const bool                     theImport,
            const bool                     theExport,
            const TCollection_AsciiString& theMagic)
      : myFormat(theFormat),
        myVendor(theVendor),
        myExtensions(theExtensions),
        myImport(theImport),
        myExport(theExport),
        myMagic(theMagic)
  {
  }

  bool Load(const occ::handle<DE_ConfigurationContext>& theResource) override
  {
    myValue = theResource->IntegerVal("value",
                                      myValue,
                                      TCollection_AsciiString("provider.") + myFormat + "."
                                        + myVendor);
    return DE_ShapeFixConfigurationNode::Load(theResource);
  }

  TCollection_AsciiString Save() const override
  {
    TCollection_AsciiString aResult = TCollection_AsciiString("provider.") + myFormat + "."
                                      + myVendor + ".value :\t " + myValue + "\n";
    return aResult + DE_ShapeFixConfigurationNode::Save();
  }

  occ::handle<DE_Provider> BuildProvider() override
  {
    return new ProbeProvider(this, myFormat, myVendor);
  }

  occ::handle<DE_ConfigurationNode> Copy() const override
  {
    return new ProbeNode(myFormat, myVendor, myExtensions, myImport, myExport, myMagic);
  }

  bool IsImportSupported() const override { return myImport; }

  bool IsExportSupported() const override { return myExport; }

  TCollection_AsciiString GetFormat() const override { return myFormat; }

  TCollection_AsciiString GetVendor() const override { return myVendor; }

  NCollection_List<TCollection_AsciiString> GetExtensions() const override
  {
    NCollection_List<TCollection_AsciiString> aList;
    TCollection_AsciiString                   aRest = myExtensions;
    for (int anIndex = 1;; ++anIndex)
    {
      const TCollection_AsciiString aToken = aRest.Token(" ", anIndex);
      if (aToken.IsEmpty())
        break;
      aList.Append(aToken);
    }
    return aList;
  }

  bool CheckContent(const occ::handle<NCollection_Buffer>& theBuffer) const override
  {
    if (myMagic.IsEmpty() || theBuffer.IsNull())
      return false;
    const char* aData = reinterpret_cast<const char*>(theBuffer->Data());
    return TCollection_AsciiString(aData).StartsWith(myMagic);
  }

  int myValue = 7;

private:
  TCollection_AsciiString myFormat;
  TCollection_AsciiString myVendor;
  TCollection_AsciiString myExtensions;
  bool                    myImport;
  bool                    myExport;
  TCollection_AsciiString myMagic;
};

void dumpContext(const std::string& theKey, const DE_ConfigurationContext& theContext)
{
  const NCollection_DataMap<TCollection_AsciiString, TCollection_AsciiString>& aMap =
    theContext.GetInternalMap();
  line(theKey + ".extent", std::to_string(aMap.Extent()));
  int anIndex = 0;
  for (NCollection_DataMap<TCollection_AsciiString, TCollection_AsciiString>::Iterator anIter(
         aMap);
       anIter.More();
       anIter.Next(), ++anIndex)
  {
    line(theKey + ".item" + std::to_string(anIndex), esc(anIter.Key()) + "=" + esc(anIter.Value()));
  }
}

void contextCase(const std::string& theKey, const char* theResource)
{
  occ::handle<DE_ConfigurationContext> aContext = new DE_ConfigurationContext();
  try
  {
    line(theKey + ".load", std::to_string(aContext->LoadStr(theResource)));
  }
  catch (const Standard_Failure& aFailure)
  {
    line(theKey + ".load", std::string("throw ") + aFailure.ExceptionType());
  }
  dumpContext(theKey, *aContext);
}

void contextCases()
{
  contextCase("ctx.simple", "a.b : 1\nc.d :\t two words \n");
  contextCase("ctx.comment", "! comment\na : 1\n  ! not a comment\n");
  contextCase("ctx.nocolon", "novalue\nk : v\n");
  contextCase("ctx.colonfirst", ": v\nk : v\n");
  contextCase("ctx.empty.value", "k :\nl : \t \nm:\n");
  contextCase("ctx.trailing.colon", "a : b:\nc:d:e\n");
  contextCase("ctx.escape", "a : \\ x\nb : \\\\y\nc : \\tz\nd : \\q\n");
  contextCase("ctx.override", "a : 1\na : 2\n");
  contextCase("ctx.crlf", "a : 1\r\nb : 2\r\n");
  contextCase("ctx.blank", "a : 1\n   \nb : 2\n");
  contextCase("ctx.noeol", "a : 1\nb : 2");
  contextCase("ctx.spaces.key", "  key with space  :  v  \n");
  contextCase("ctx.many", "z : 1\ny : 2\nx : 3\nw : 4\nv : 5\nu : 6\nt : 7\ns : 8\n");
}

void getterCases()
{
  occ::handle<DE_ConfigurationContext> aContext = new DE_ConfigurationContext();
  aContext->LoadStr("s.r1 : 1e-3\ns.r2 : 12\ns.r3 : abc\ns.r4 : 1.5x\ns.i1 : 0x10\n"
                    "s.i2 : 3.5\ns.i3 : -4\ns.b1 : 0\ns.b2 : 2\ns.b3 : yes\n"
                    "s.q1 : a b  c\ns.q2 : OCC\ns.q3 :\nplain : 9\n");
  const char* aNames[] = {"r1", "r2", "r3", "r4", "i1", "i2", "i3", "b1", "b2", "b3", "q1", "q2",
                          "q3", "zz"};
  for (const char* aName : aNames)
  {
    const std::string aKey = std::string("get.") + aName;
    double            aReal = -1.0;
    int               anInt = -1;
    bool              aBool = false;
    line(aKey + ".set", std::to_string(aContext->IsParamSet(aName, "s")));
    const bool aHasReal = aContext->GetReal(aName, aReal, "s");
    line(aKey + ".real", std::to_string(aHasReal) + " " + TCollection_AsciiString(aReal).ToCString());
    const bool aHasInt = aContext->GetInteger(aName, anInt, "s");
    line(aKey + ".int", std::to_string(aHasInt) + " " + std::to_string(anInt));
    const bool aHasBool = aContext->GetBoolean(aName, aBool, "s");
    line(aKey + ".bool", std::to_string(aHasBool) + " " + std::to_string(aBool));
    NCollection_List<TCollection_AsciiString> aSeq;
    const bool aHasSeq = aContext->GetStringSeq(aName, aSeq, "s");
    std::string aSeqText;
    for (NCollection_List<TCollection_AsciiString>::Iterator anIter(aSeq); anIter.More(); anIter.Next())
    {
      aSeqText += "[" + esc(anIter.Value()) + "]";
    }
    line(aKey + ".seq", std::to_string(aHasSeq) + " " + aSeqText);
    line(aKey + ".vals",
         std::string(TCollection_AsciiString(aContext->RealVal(aName, 2.5, "s")).ToCString()) + " "
           + std::to_string(aContext->IntegerVal(aName, 5, "s")) + " "
           + std::to_string(aContext->BooleanVal(aName, true, "s")) + " "
           + esc(aContext->StringVal(aName, "def", "s")));
  }
  line("get.noscope", std::to_string(aContext->IntegerVal("plain", -1)));
}

void shapeFixCases()
{
  occ::handle<ProbeNode> aNode = new ProbeNode("TST", "VND", "tst", true, true, "");
  const TCollection_AsciiString aDefault = aNode->Save();
  line("fix.default", esc(aDefault));
  occ::handle<DE_ConfigurationContext> aContext = new DE_ConfigurationContext();
  aContext->LoadStr("provider.TST.VND.healing.tolerance3d : 0.25\n"
                    "provider.TST.VND.healing.free.shell : 5\n"
                    "provider.TST.VND.healing.vertex.tolerance : 0\n"
                    "provider.TST.VND.healing.tail : x\n"
                    "provider.TST.VND.value : 42\n");
  line("fix.load", TXT(aNode->Load(aContext)));
  line("fix.loaded", esc(aNode->Save()));
  line("fix.copy.value", TXT(occ::down_cast<ProbeNode>(aNode->Copy())->myValue));
  line("fix.ext",
       TXT(aNode->CheckExtension(".TsT") << aNode->CheckExtension("tst") << aNode->CheckExtension("")
                                         << aNode->CheckExtension(".")
                                         << aNode->CheckExtension("..tst")));
  line("fix.flags",
       TXT(aNode->IsEnabled() << aNode->IsStreamSupported() << aNode->UpdateLoad(true, true)));
}

occ::handle<ProbeNode> node(const char* theFormat,
                            const char* theVendor,
                            const char* theExtensions,
                            bool        theImport,
                            bool        theExport,
                            const char* theMagic = "")
{
  return new ProbeNode(theFormat, theVendor, theExtensions, theImport, theExport, theMagic);
}

std::string order(const occ::handle<DE_Wrapper>& theWrapper)
{
  std::string aText;
  for (NCollection_DataMap<TCollection_AsciiString,
                           NCollection_IndexedDataMap<TCollection_AsciiString,
                                                      occ::handle<DE_ConfigurationNode>>>::Iterator
         aFormatIter(theWrapper->Nodes());
       aFormatIter.More();
       aFormatIter.Next())
  {
    aText += esc(aFormatIter.Key()) + ":";
    for (NCollection_IndexedDataMap<TCollection_AsciiString,
                                    occ::handle<DE_ConfigurationNode>>::Iterator
           aVendorIter(aFormatIter.Value());
         aVendorIter.More();
         aVendorIter.Next())
    {
      aText += esc(aVendorIter.Key()) + (aVendorIter.Value()->IsEnabled() ? "+" : "-") + ",";
    }
    aText += ";";
  }
  return aText;
}

std::string provider(const occ::handle<DE_Wrapper>& theWrapper, const char* thePath, bool theImport)
{
  occ::handle<DE_Provider> aProvider;
  if (!theWrapper->FindProvider(thePath, theImport, aProvider))
    return "none";
  return TXT(aProvider->GetFormat() << "/" << aProvider->GetVendor());
}

occ::handle<DE_Wrapper> registry()
{
  occ::handle<DE_Wrapper> aWrapper = new DE_Wrapper();
  aWrapper->Bind(node("VRML", "OCC", "vrml wrl", true, true));
  aWrapper->Bind(node("IGES", "OCC", "igs iges", true, true));
  aWrapper->Bind(node("OBJ", "OCC", "obj", true, false));
  aWrapper->Bind(node("GLTF", "OCC", "gltf glb", true, true));
  aWrapper->Bind(node("PLY", "OCC", "ply", false, true));
  aWrapper->Bind(node("XCAF", "OCC", "xbf", true, true));
  aWrapper->Bind(node("STEP", "OCC", "step stp", true, true, "ISO-10303"));
  aWrapper->Bind(node("STL", "OCC", "stl", true, true));
  aWrapper->Bind(node("BREP", "OCC", "brep rle", true, true));
  aWrapper->Bind(node("STEP", "ACME", "step stp p21", true, false, "ISO-10303"));
  aWrapper->Bind(node("STEP", "ZETA", "step", true, true));
  return aWrapper;
}

void wrapperLookups(const occ::handle<DE_Wrapper>& theWrapper)
{
  occ::handle<DE_ConfigurationNode> aFound;
  const bool        aFoundAcme   = theWrapper->Find("STEP", "ACME", aFound);
  const std::string aVendor      = aFound->GetVendor().ToCString();
  const bool        aFoundNone   = theWrapper->Find("STEP", "NONE", aFound);
  const bool        aFoundFormat = theWrapper->Find("NONE", "OCC", aFound);
  line("wrap.find",
       TXT(aFoundAcme << " " << aVendor << " " << aFoundNone << aFoundFormat << " "
                      << aFound->GetVendor()));
  line("wrap.read.stp", provider(theWrapper, "/tmp/no/such/file.stp", true));
  line("wrap.write.stp", provider(theWrapper, "/tmp/out/file.stp", false));
  line("wrap.write.p21", provider(theWrapper, "/tmp/out/file.p21", false));
  line("wrap.write.ply", provider(theWrapper, "/tmp/out/model.PLY", false));
  line("wrap.write.obj", provider(theWrapper, "/tmp/out/model.obj", false));
  line("wrap.write.none", provider(theWrapper, "/tmp/out/model", false));
}

void wrapperPriorities(const occ::handle<DE_Wrapper>& theWrapper)
{
  NCollection_List<TCollection_AsciiString> aPriority;
  aPriority.Append("ZETA");
  aPriority.Append("NONE");
  aPriority.Append("ACME");
  theWrapper->ChangePriority("STEP", aPriority, true);
  line("wrap.priority", order(theWrapper));
  line("wrap.write.stp2", provider(theWrapper, "/tmp/out/file.stp", false));
  theWrapper->ChangePriority(aPriority, false);
  line("wrap.priority.all", order(theWrapper));
  theWrapper->UpdateLoad(true);
  line("wrap.updateload", order(theWrapper));
  line("wrap.unbind",
       TXT(theWrapper->UnBind(node("STEP", "ZETA", "", false, false))
           << theWrapper->UnBind(node("STEP", "ZETA", "", false, false))
           << theWrapper->UnBind(node("NONE", "OCC", "", false, false))));
  line("wrap.after.unbind", order(theWrapper));
}

void wrapperSaves(const occ::handle<DE_Wrapper>& theWrapper)
{
  theWrapper->GlobalParameters.LengthUnit = 25.4;
  theWrapper->GlobalParameters.SystemUnit = 0.001;
  NCollection_List<TCollection_AsciiString> aFormats;
  aFormats.Append("STEP");
  aFormats.Append("STL");
  NCollection_List<TCollection_AsciiString> aVendors;
  aVendors.Append("OCC");
  line("wrap.save.short", esc(theWrapper->Save(false)));
  occ::handle<DE_Wrapper> aSmall = new DE_Wrapper();
  aSmall->Bind(node("STL", "OCC", "stl", true, true));
  aSmall->Bind(node("BREP", "OCC", "brep", true, true));
  line("wrap.save.small", esc(aSmall->Save(true)));
  line("wrap.save.filtered", esc(theWrapper->Save(true, aFormats, aVendors)));
  occ::handle<DE_ConfigurationNode> aStl;
  theWrapper->Find("STL", "OCC", aStl);
  occ::handle<DE_Provider> aProvider;
  theWrapper->FindProvider("a.stl", false, aProvider);
  line("wrap.global.copied", TXT(TCollection_AsciiString(aStl->GlobalParameters.LengthUnit)));
  line("wrap.provider.node", TXT((aProvider->GetNode() == aStl)));
}

void wrapperCases()
{
  occ::handle<DE_Wrapper> aWrapper = registry();
  line("wrap.order", order(aWrapper));
  line("wrap.rebind", TXT(aWrapper->Bind(node("STEP", "OCC", "x", false, false))));
  line("wrap.bindnull", TXT(aWrapper->Bind(occ::handle<DE_ConfigurationNode>())));
  wrapperLookups(aWrapper);
  wrapperPriorities(aWrapper);
  wrapperSaves(aWrapper);
}

void loadCases()
{
  occ::handle<DE_Wrapper> aWrapper = registry();
  const bool              aLoaded  = aWrapper->Load("global.priority.STEP : ACME OCC\n"
                                                    "global.priority.NONE : OCC\n"
                                                    "global.general.length.unit : 10\n"
                                                    "provider.STEP.OCC.value : 3\n"
                                                    "provider.STEP.ACME.healing.tolerance3d : 0.5\n"
                                                    "provider.VRML.OCC.value : 11\n");
  line("load.result", TXT(aLoaded));
  line("load.order", order(aWrapper));
  occ::handle<DE_ConfigurationNode> aNode;
  aWrapper->Find("STEP", "OCC", aNode);
  line("load.value", TXT(occ::down_cast<ProbeNode>(aNode)->myValue));
  aWrapper->Find("STEP", "ACME", aNode);
  line("load.tol",
       TXT(TCollection_AsciiString(
         occ::down_cast<DE_ShapeFixConfigurationNode>(aNode)->ShapeFixParameters.Tolerance3d)));
  line("load.unit", TXT(TCollection_AsciiString(aWrapper->GlobalParameters.LengthUnit)));
  occ::handle<DE_Wrapper> aShallow = registry();
  aShallow->Load("global.priority.STEP : ZETA\nprovider.STEP.OCC.value : 9\n", false);
  line("load.shallow", order(aShallow));
  aShallow->Find("STEP", "OCC", aNode);
  line("load.shallow.value", TXT(occ::down_cast<ProbeNode>(aNode)->myValue));
}

void copyCases()
{
  occ::handle<DE_Wrapper> aWrapper = registry();
  aWrapper->SetKeepUpdates(true);
  occ::handle<DE_Wrapper> aCopy = aWrapper->Copy();
  line("copy.order", order(aCopy));
  line("copy.keep", TXT(aCopy->KeepUpdates() << aWrapper->KeepUpdates()));
  occ::handle<DE_ConfigurationNode> aFirst, aSecond;
  aWrapper->Find("STEP", "OCC", aFirst);
  aCopy->Find("STEP", "OCC", aSecond);
  line("copy.shared", TXT((aFirst == aSecond)));
  occ::handle<DE_Wrapper> aRebound = new DE_Wrapper(aWrapper);
  line("copy.handle.order", order(aRebound));
  line("copy.handle.keep", TXT(aRebound->KeepUpdates() << aWrapper->KeepUpdates()));
  occ::handle<DE_Wrapper> aNull = new DE_Wrapper(occ::handle<DE_Wrapper>());
  line("copy.null", order(aNull));
}

void streamLookups()
{
  occ::handle<DE_Wrapper>  aWrapper = registry();
  std::istringstream       aStep("ISO-10303-21;\nHEADER;\n");
  occ::handle<DE_Provider> aProvider;
  const bool               aFound = aWrapper->FindReadProvider("model.dat", aStep, aProvider);
  line("content.stream", TXT(aFound << " " << aProvider->GetVendor() << " " << aStep.tellg()));
  std::istringstream anOther("solid x\n");
  aProvider.Nullify();
  line("content.other", TXT(aWrapper->FindReadProvider("model.dat", anOther, aProvider)));
}

void contentCases()
{
  streamLookups();
  std::string        aLong(3000, 'a');
  std::istringstream aLongStream(aLong);
  aLongStream.seekg(5);
  occ::handle<NCollection_Buffer> aBuffer;
  const bool aCreated = DE_ValidationUtils::CreateContentBuffer(aLongStream, aBuffer);
  line("content.buffer",
       TXT(aCreated << " " << aBuffer->Size() << " " << aLongStream.tellg() << " "
                    << (int)aBuffer->Data()[2047] << " " << (int)aBuffer->Data()[2046]));
  std::istringstream aShort("abc");
  const bool         aShortCreated = DE_ValidationUtils::CreateContentBuffer(aShort, aBuffer);
  line("content.short",
       TXT(aShortCreated << " " << (int)aBuffer->Data()[3] << " " << aShort.tellg()));
  const bool aMissing = DE_ValidationUtils::CreateContentBuffer("/no/such/file.stp", aBuffer);
  line("content.missing", TXT(aMissing << aBuffer.IsNull()));
}

void streamValidation()
{
  DE_Provider::ReadStreamList anEmpty;
  std::istringstream          aGood("x");
  std::istringstream          aBad("x");
  aBad.setstate(std::ios::failbit);
  DE_Provider::ReadStreamList aGoodList;
  aGoodList.Append(DE_Provider::ReadStreamNode("a", aGood));
  aGoodList.Append(DE_Provider::ReadStreamNode("b", aGood));
  DE_Provider::ReadStreamList aBadList;
  aBadList.Append(DE_Provider::ReadStreamNode("", aBad));
  line("valid.streams",
       TXT(DE_ValidationUtils::ValidateReadStreamList(anEmpty, "ctx", false)
           << DE_ValidationUtils::ValidateReadStreamList(aGoodList, "ctx", false)
           << DE_ValidationUtils::ValidateReadStreamList(aBadList, "ctx", false)));
  std::ostringstream anOut;
  std::ostringstream aBadOut;
  aBadOut.setstate(std::ios::badbit);
  DE_Provider::WriteStreamList aWriteList;
  aWriteList.Append(DE_Provider::WriteStreamNode("w", anOut));
  DE_Provider::WriteStreamList aBadWriteList;
  aBadWriteList.Append(DE_Provider::WriteStreamNode("w", aBadOut));
  DE_Provider::WriteStreamList anEmptyWrite;
  line("valid.wstreams",
       TXT(DE_ValidationUtils::ValidateWriteStreamList(anEmptyWrite, "ctx", false)
           << DE_ValidationUtils::ValidateWriteStreamList(aWriteList, "ctx", false)
           << DE_ValidationUtils::ValidateWriteStreamList(aBadWriteList, "ctx", false)));
}

void validationCases()
{
  const char* aReadablePath = "/tmp/tkde_probe_conf_read.txt";
  std::FILE*  aReadable     = std::fopen(aReadablePath, "w");
  std::fputs("x", aReadable);
  std::fclose(aReadable);
  occ::handle<DE_ConfigurationNode> aNode = node("STEP", "OCC", "step", true, true);
  occ::handle<DE_ConfigurationNode> aNull;
  line("valid.node",
       TXT(DE_ValidationUtils::ValidateConfigurationNode(aNode,
                                                          STANDARD_TYPE(DE_ShapeFixConfigurationNode),
                                                          "ctx",
                                                          false)
           << DE_ValidationUtils::ValidateConfigurationNode(aNode, STANDARD_TYPE(DE_Wrapper), "ctx", false)
           << DE_ValidationUtils::ValidateConfigurationNode(aNull,
                                                            STANDARD_TYPE(DE_ConfigurationNode),
                                                            "ctx",
                                                            false)));
  line("valid.read",
       TXT(DE_ValidationUtils::ValidateFileForReading("", "ctx", false)
           << DE_ValidationUtils::ValidateFileForReading("/no/such/file", "ctx", false)
           << DE_ValidationUtils::ValidateFileForReading(aReadablePath, "ctx", false)));
  line("valid.write",
       TXT(DE_ValidationUtils::ValidateFileForWriting("", "ctx", false)
           << DE_ValidationUtils::ValidateFileForWriting("/no/such/dir/file", "ctx", false)
           << DE_ValidationUtils::ValidateFileForWriting(aReadablePath, "ctx", false)));
  std::remove(aReadablePath);
  streamValidation();
  line("valid.unit", TXT(DE_ValidationUtils::WarnLengthUnitNotSupported(2.0, "ctx", false)));
  line("valid.doc",
       TXT(DE_ValidationUtils::ValidateDocument(occ::handle<TDocStd_Document>(), "ctx", false)));
}

void fileCases()
{
  occ::handle<DE_ConfigurationContext> aContext = new DE_ConfigurationContext();
  line("file.missing", TXT(aContext->LoadFile("/no/such/dir/conf.txt")));
  occ::handle<ProbeNode> aNode = node("STL", "OCC", "stl", true, true);
  aNode->myValue               = 13;
  const char* aPath            = "/tmp/tkde_probe_conf.txt";
  line("file.save", TXT(aNode->DE_ConfigurationNode::Save(aPath)));
  line("file.loadfile", TXT(aContext->LoadFile(aPath)));
  line("file.value", TXT(aContext->IntegerVal("provider.STL.OCC.value", -1)));
  line("file.extent", TXT(aContext->GetInternalMap().Extent()));
  occ::handle<ProbeNode> aReader     = node("STL", "OCC", "stl", true, true);
  const bool             aNodeLoaded = aReader->DE_ConfigurationNode::Load(TCollection_AsciiString(aPath));
  line("file.nodeload", TXT(aNodeLoaded << " " << aReader->myValue));
  occ::handle<DE_ConfigurationContext> aDispatched     = new DE_ConfigurationContext();
  const bool                           aDispatchLoaded = aDispatched->Load(aPath);
  line("file.load.dispatch",
       TXT(aDispatchLoaded << " " << aDispatched->GetInternalMap().Extent()));
  try
  {
    line("file.nodeload.empty",
         TXT(aReader->DE_ConfigurationNode::Load(TCollection_AsciiString(""))));
  }
  catch (const Standard_Failure& aFailure)
  {
    line("file.nodeload.empty", TXT("throw " << aFailure.ExceptionType()));
  }
  occ::handle<DE_Wrapper> aWrapper = new DE_Wrapper();
  aWrapper->Bind(aNode);
  line("file.wrapper.save", TXT(aWrapper->Save(TCollection_AsciiString(aPath))));
  line("file.wrapper.bad", TXT(aWrapper->Save(TCollection_AsciiString("/no/such/dir/x.txt"))));
  std::remove(aPath);
}

// Two vendors of one format, default-constructible for the plugin holders.
class PluginVendorA : public ProbeNode
{
public:
  PluginVendorA()
      : ProbeNode("PLG", "A", "plg", true, true, "")
  {
  }
};

class PluginVendorB : public ProbeNode
{
public:
  PluginVendorB()
      : ProbeNode("PLG", "B", "plg", true, true, "")
  {
  }
};

void pluginCases()
{
  occ::handle<DE_Wrapper> anIsolated = new DE_Wrapper();
  DE_Wrapper::SetGlobalWrapper(anIsolated);
  DE_Wrapper::SetGlobalWrapper(occ::handle<DE_Wrapper>());
  line("plugin.global.kept", TXT((DE_Wrapper::GlobalWrapper() == anIsolated)));
  {
    DE_MultiPluginHolder<PluginVendorA, PluginVendorB> aHolder;
    line("plugin.multi", order(DE_Wrapper::GlobalWrapper()));
    {
      DE_PluginHolder<PluginVendorA> aSingle;
      line("plugin.single", order(DE_Wrapper::GlobalWrapper()));
    }
    line("plugin.single.released", order(DE_Wrapper::GlobalWrapper()));
  }
  line("plugin.multi.released", order(DE_Wrapper::GlobalWrapper()));
}
} // namespace

int main()
{
  Message::DefaultMessenger()->RemovePrinters(STANDARD_TYPE(Message_PrinterOStream));
  contextCases();
  getterCases();
  shapeFixCases();
  wrapperCases();
  loadCases();
  copyCases();
  contentCases();
  validationCases();
  fileCases();
  pluginCases();
  return 0;
}
