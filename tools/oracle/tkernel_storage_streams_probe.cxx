// Storage stream-level operations (twin of oracle_probe/storage/streams.rs).
//
// stor_fmt <precision> <hex bits>          operator<<(double) at a precision
// stor_fmtf <precision> <hex bits>         operator<<(float) at a precision
// stor_stream <hex bytes> <op>...          libstdc++ extraction on an fstream over the bytes:
//   g get, l<N> getline, i/s/u/b int/short/unsigned short/bool, w char[100] word,
//   x<N> width, r<N> read, tg/tp tellg/tellp, c clear
// stor_magic <hex bytes>                   Storage_BaseDriver::ReadMagicNumber
// stor_b64e <hex>, stor_b64ec <cap> <hex>  FSD_Base64::Encode (cap -1: null destination)
// stor_b64d <hex>, stor_b64dc <cap> <hex>  FSD_Base64::Decode
// stor_ver, stor_migr <hex>                Storage::Version, Storage_Schema::CheckTypeMigration
#include <FSD_Base64.hxx>
#include <Storage.hxx>
#include <Storage_BaseDriver.hxx>
#include <Storage_Schema.hxx>

namespace
{
void stor_stream_state(std::ostream& out, std::istream& theStream)
{
  out << ':' << static_cast<int>(theStream.rdstate());
}

// The extraction targets of one script; each keeps its value across the script's operations.
struct StorStreamValues
{
  char           aChar   = 0;
  int            anInt   = 0;
  short          aShort  = 0;
  unsigned short aUShort = 0;
  bool           aBool   = false;
};

void stor_stream_op(const std::string& theOp,
                    std::fstream&      theStream,
                    StorStreamValues&  theValues,
                    std::ostream&      out)
{
  char&           aChar   = theValues.aChar;
  int&            anInt   = theValues.anInt;
  short&          aShort  = theValues.aShort;
  unsigned short& aUShort = theValues.aUShort;
  bool&           aBool   = theValues.aBool;
  if (theOp == "g")
  {
    theStream.get(aChar);
    out << " g" << static_cast<int>(static_cast<unsigned char>(aChar)) << ':' << theStream.gcount();
  }
  else if (theOp[0] == 'l')
  {
    const int         aSize = std::stoi(theOp.substr(1));
    std::vector<char> aBuffer(static_cast<size_t>(aSize) + 1, '\0');
    theStream.getline(aBuffer.data(), aSize, '\n');
    out << " l" << stor_hex(std::string(aBuffer.data())) << ':' << theStream.gcount();
  }
  else if (theOp == "i")
  {
    theStream >> anInt;
    out << " i" << anInt;
  }
  else if (theOp == "s")
  {
    theStream >> aShort;
    out << " s" << aShort;
  }
  else if (theOp == "u")
  {
    theStream >> aUShort;
    out << " u" << aUShort;
  }
  else if (theOp == "b")
  {
    theStream >> aBool;
    out << " b" << aBool;
  }
  else if (theOp == "w")
  {
    char aWord[100];
    aWord[0] = '\0';
    theStream >> aWord;
    out << " w" << stor_hex(std::string(aWord));
  }
  else if (theOp[0] == 'x')
  {
    theStream.width(std::stoi(theOp.substr(1)));
    out << " x";
  }
  else if (theOp[0] == 'r')
  {
    const int         aSize = std::stoi(theOp.substr(1));
    std::vector<char> aBuffer(static_cast<size_t>(aSize), '\0');
    theStream.read(aBuffer.data(), aSize);
    out << " r" << stor_hex(std::string(aBuffer.data(), static_cast<size_t>(theStream.gcount())))
        << ':' << theStream.gcount();
  }
  else if (theOp == "tg")
    out << " tg" << static_cast<long long>(theStream.tellg());
  else if (theOp == "tp")
    out << " tp" << static_cast<long long>(theStream.tellp());
  else if (theOp == "c")
  {
    theStream.clear();
    out << " c";
  }
  stor_stream_state(out, theStream);
}

void stor_stream(std::istream& in, std::ostream& out)
{
  std::string aHex;
  in >> aHex;
  const std::string aPath = stor_scratch("stream");
  stor_write_file(aPath, stor_unhex(aHex));
  std::fstream aStream(aPath, std::ios::in | std::ios::out | std::ios::binary);
  std::string      anOp;
  StorStreamValues aValues;
  while (in >> anOp)
    stor_stream_op(anOp, aStream, aValues, out);
  aStream.close();
  std::remove(aPath.c_str());
}

void stor_base64(const std::string& op, std::istream& in, std::ostream& out)
{
  long long aCapacity = 0;
  if (op == "stor_b64ec" || op == "stor_b64dc")
    in >> aCapacity;
  std::string aHex;
  in >> aHex;
  const std::string aSource = stor_unhex(aHex);
  if (op == "stor_b64e")
  {
    out << stor_ascii_hex(FSD_Base64::Encode(reinterpret_cast<const uint8_t*>(aSource.data()), aSource.size()));
    return;
  }
  if (op == "stor_b64d")
  {
    occ::handle<NCollection_Buffer> aBuffer = FSD_Base64::Decode(aSource.c_str(), aSource.size());
    if (aBuffer.IsNull())
      out << "null";
    else
      out << aBuffer->IsEmpty() << ' ' << aBuffer->Size() << ' '
          << stor_hex(std::string(reinterpret_cast<const char*>(aBuffer->Data()), aBuffer->Size()));
    return;
  }
  // One spare byte keeps a zero capacity distinct from the null destination of -1.
  const size_t      aSize = aCapacity < 0 ? 0 : static_cast<size_t>(aCapacity);
  std::vector<char> aTarget(aSize + 1, '*');
  size_t            aResult = 0;
  if (op == "stor_b64ec")
    aResult = FSD_Base64::Encode(aCapacity < 0 ? nullptr : aTarget.data(), aSize,
                                 reinterpret_cast<const uint8_t*>(aSource.data()), aSource.size());
  else
    aResult = FSD_Base64::Decode(aCapacity < 0 ? nullptr : reinterpret_cast<uint8_t*>(aTarget.data()),
                                 aSize, aSource.c_str(), aSource.size());
  out << aResult << ' ' << stor_hex(std::string(aTarget.begin(), aTarget.begin() + aSize));
}
} // namespace

static bool storage_streams_probe(const std::string& op, std::istream& in, std::ostream& out)
{
  if (op == "stor_fmt" || op == "stor_fmtf")
  {
    int         aPrecision = 0;
    std::string aHex;
    in >> aPrecision >> aHex;
    std::ostringstream aText;
    aText.precision(aPrecision);
    if (op == "stor_fmt")
      aText << stor_double(aHex);
    else
      aText << stor_float(aHex);
    out << aText.str();
  }
  else if (op == "stor_stream")
    stor_stream(in, out);
  else if (op == "stor_magic")
  {
    std::string aHex;
    in >> aHex;
    std::stringstream aStream(stor_unhex(aHex), std::ios::in | std::ios::binary);
    out << stor_ascii_hex(Storage_BaseDriver::ReadMagicNumber(aStream));
  }
  else if (op.rfind("stor_b64", 0) == 0)
    stor_base64(op, in, out);
  else if (op == "stor_ver")
    out << Storage::Version();
  else if (op == "stor_migr")
  {
    std::string aHex;
    in >> aHex;
    TCollection_AsciiString aNew("unchanged");
    const bool              isRenamed =
      Storage_Schema::CheckTypeMigration(TCollection_AsciiString(stor_unhex(aHex).c_str()), aNew);
    out << isRenamed << ' ' << stor_ascii_hex(aNew);
  }
  else
    return false;
  return true;
}
