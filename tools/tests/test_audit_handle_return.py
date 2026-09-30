"""Handle-return macros must not hide distinct native accessor identities."""
import unittest
from audit_api_rows import function_members, mask_handle_return_type, render


class HandleReturnIdentityTests(unittest.TestCase):
    def test_const_and_mutable_accessors_remain_distinct(self) -> None:
        declarations = ["const Handle(IMeshData::VectorOfVertex)& Vertices() const",
                        "Handle(IMeshData::VectorOfVertex)& ChangeVertices()"]
        members = [function_members(head, "MeshInspector", "MeshInspector")[0]
                   for head in declarations]
        self.assertEqual([render(member) for member in members],
                         ["MeshInspector::Vertices() const", "MeshInspector::ChangeVertices()"])

    def test_by_value_returns_and_parameter_handle_macros_keep_types(self) -> None:
        head = "Handle(Curve) Clone(const Handle(Curve)& theOther) const"
        member = function_members(head, "Mesh", "Mesh")[0]
        self.assertEqual(render(member), "Mesh::Clone(const Handle Curve&) const")
        self.assertIn("Handle(Curve)& theOther", mask_handle_return_type(head))

    def test_normal_declarations_and_standalone_macros_remain_unchanged(self) -> None:
        for head in ["int Value() const", "DEFINE_STANDARD_RTTIEXT(Mesh,Base)",
                     "Handle(Curve)", "void Assign(const Handle(Curve)& theOther)"]:
            self.assertEqual(mask_handle_return_type(head), head)


if __name__ == "__main__":
    unittest.main()
