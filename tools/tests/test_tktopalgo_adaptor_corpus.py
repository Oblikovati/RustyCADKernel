"""The BRepTopAdaptor and IntCurvesFace corpora: face records of the new surface and edge
kinds have the length the `tp_face` reader consumes, every request carries exactly the
fields its probe operation reads, and every face is classified and intersected."""
import unittest
from tktopalgo_corpus_adaptor_faces import ADAPTOR_WINDOWS, adaptor_faces, bspline_surface_face, toleranced_square_face
from tktopalgo_corpus_breptopadaptor import append_breptopadaptor_requests
from tktopalgo_corpus_faces import BSPLINE_SURFACE, SEGMENT_3D, record_length
from tktopalgo_corpus_intcurvesface import append_intcurvesface_requests, curves

# The fields each operation reads after its face record, `n` counting the groups.
FIXED_FIELDS = {'bta_destroy': 3, 'bta_topol': 1, 'bta_tool': 0, 'bta_errors': 0}
GROUPED_FIELDS = {'bta_fclass2d': (2, 2), 'bta_classify': (2, 2), 'icf_line': (3, 8)}


def values_of(request: str) -> tuple[str, list[float]]:
    """The operation and the numeric fields of a request."""
    op, *fields = request.split()
    return op, [float(field) for field in fields]


def curve_length(values: list[float], start: int) -> int:
    """The fields of one `tp_icf_curve` record (kind, its fields, first and last)."""
    kind = int(values[start])
    if kind == 0:
        return 1 + 6 + 2
    if kind == 1:
        return 1 + 7 + 2
    return 2 + 3 * int(values[start + 1]) + 2


class AdaptorFaceTests(unittest.TestCase):
    def test_every_face_record_has_the_length_the_reader_consumes(self):
        for name, face in adaptor_faces().items():
            with self.subTest(name):
                self.assertEqual(record_length(face.numbers()), len(face.numbers()))

    def test_every_face_has_a_window(self):
        self.assertEqual(set(adaptor_faces()), set(ADAPTOR_WINDOWS))

    def test_bspline_record_counts_its_poles_and_knots(self):
        face = bspline_surface_face(c1_knot=True)
        self.assertEqual(face.surface[0], BSPLINE_SURFACE)
        self.assertEqual(face.surface[9:11], [7, 5])
        self.assertEqual(len(face.surface), 11 + 3 * 35 + 9 + 9)

    def test_toleranced_square_edges_carry_3d_lines(self):
        self.assertTrue(all(edge[0] == SEGMENT_3D for edge in toleranced_square_face().edges))
        self.assertTrue(all(edge[4] == 0.1 for edge in toleranced_square_face().edges))


class AdaptorRequestTests(unittest.TestCase):
    def setUp(self):
        self.requests: list[str] = []
        append_breptopadaptor_requests(self.requests)
        append_intcurvesface_requests(self.requests)

    def test_single_face_requests_have_the_fields_of_their_operation(self):
        for request in self.requests:
            op, values = values_of(request)
            if op in ('icf_shape', 'icf_curve'):
                continue
            rest = values[record_length(values):]
            if op in FIXED_FIELDS:
                self.assertEqual(len(rest), FIXED_FIELDS[op], request[:60])
                continue
            head, group = GROUPED_FIELDS[op]
            self.assertEqual(len(rest), head + 1 + group * int(rest[head]), request[:60])

    def test_curve_requests_end_with_their_range(self):
        for request in self.requests:
            op, values = values_of(request)
            if op != 'icf_curve':
                continue
            start = record_length(values) + 3
            self.assertEqual(len(values), start + curve_length(values, start) + 2, request[:60])

    def test_shape_requests_carry_their_faces_and_lines(self):
        for request in self.requests:
            op, values = values_of(request)
            if op != 'icf_shape':
                continue
            index = 1
            for _ in range(int(values[0])):
                index += record_length(values, index)
            self.assertEqual(len(values), index + 4 + 8 * int(values[index + 3]), request[:60])

    def test_every_face_is_classified_and_intersected(self):
        classified = {request.split(' ', 1)[1] for request in self.requests if request.startswith('bta_fclass2d ')}
        intersected = {request.split(' ', 1)[1] for request in self.requests if request.startswith('icf_line ')}
        for name, face in adaptor_faces().items():
            record = ' '.join(repr(float(v)) for v in face.numbers()) + ' '
            with self.subTest(name):
                self.assertTrue(any(line.startswith(record) for line in classified))
                if name != 'no_pcurve':
                    self.assertTrue(any(line.startswith(record) for line in intersected))

    def test_curves_cross_each_kind_of_face(self):
        for face in adaptor_faces().values():
            self.assertEqual([curve[0] for curve in curves(face)], [1, 1, 1, 0, 2])


if __name__ == '__main__':
    unittest.main()
