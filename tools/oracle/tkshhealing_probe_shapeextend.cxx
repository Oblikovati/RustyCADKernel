// ShapeExtend probe operations; the Rust twin is
// crates/tkshhealing/examples/sh_probe/shape_extend.rs.
#include <Message_MsgFile.hxx>
#include <ShapeExtend.hxx>
#include <TCollection_AsciiString.hxx>
// `flag status`: ShapeExtend::EncodeStatus(status), then DecodeStatus(flag, status).
void sh_status_case(std::istream& input){
 GaReader r{input};int flag=r.integer();ShapeExtend_Status status=ShapeExtend_Status(r.integer());
 ga_int(ShapeExtend::EncodeStatus(status));ga_flag(ShapeExtend::DecodeStatus(flag,status));
 ga_end();}
// `n (byte)...`: ShapeExtend::Init (which loads SHMessage_SHAPE_us), then HasMsg and the
// UTF-8 text of Msg for the key spelled by the n bytes. A key missing from the catalog is
// bound by Msg, so the corpus asks each missing key once.
void sh_message_case(std::istream& input){
 GaReader r{input};int n=r.integer();std::string key;
 for(int i=0;i<n;++i)key+=char(r.integer());
 GA_TRY(ShapeExtend::Init())
 ga_flag(Message_MsgFile::HasMsg(key.c_str()));
 ga_text(TCollection_AsciiString(Message_MsgFile::Msg(key.c_str())).ToCString());
 ga_end();}
