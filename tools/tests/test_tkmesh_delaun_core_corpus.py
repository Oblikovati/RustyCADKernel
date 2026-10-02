"""The Delaun predicate inventory must retain both exact and native-crash candidates."""
import unittest
from tkmesh_delaun_core_corpus import core_rows,polygon_rows,polygon_mesh_rows,super_mesh_rows,super_fan_rows,use_edge_rows,sort_rows,cleanup_rows,intersection_rows,next_link_rows,corpus,partition

class DelaunCoreCorpusTests(unittest.TestCase):
    """No collinear cleanup case is silently removed."""
    def test_complete_partition(self) -> None:
        """All generated requests survive exactly once in the evidence partition."""
        exact,crashes=partition()
        candidates=core_rows()+polygon_rows()+polygon_mesh_rows()+super_mesh_rows()+super_fan_rows()+use_edge_rows()+sort_rows()+cleanup_rows()+intersection_rows()+next_link_rows()
        self.assertEqual(len(exact),2237)
        self.assertEqual(len(crashes),80)
        self.assertEqual(set(candidates),set(exact+crashes))
        self.assertEqual(corpus().splitlines(),candidates)
        self.assertEqual(len(candidates),len(set(candidates)))
