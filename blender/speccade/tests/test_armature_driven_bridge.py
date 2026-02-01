# blender/speccade/tests/test_armature_driven_bridge.py

import unittest
from pathlib import Path
import sys

# Allow `import speccade.*` from repo root.
_BLENDER_DIR = Path(__file__).resolve().parents[2]
if str(_BLENDER_DIR) not in sys.path:
    sys.path.insert(0, str(_BLENDER_DIR))


class TestBridgeEdgeLoopHelpers(unittest.TestCase):
    """Test bridge edge loop helper functions (pure Python, no Blender)."""

    def test_can_import_bridge_helpers(self) -> None:
        from speccade.armature_driven import (
            get_bridge_pairs,
            are_profiles_compatible,
        )
        self.assertTrue(callable(get_bridge_pairs))
        self.assertTrue(callable(are_profiles_compatible))

    def test_are_profiles_compatible_exact_match(self) -> None:
        from speccade.armature_driven import are_profiles_compatible
        # Same segment count = compatible
        self.assertTrue(are_profiles_compatible(8, 8))
        self.assertTrue(are_profiles_compatible(12, 12))

    def test_are_profiles_compatible_double(self) -> None:
        from speccade.armature_driven import are_profiles_compatible
        # One is 2x the other = compatible
        self.assertTrue(are_profiles_compatible(8, 16))
        self.assertTrue(are_profiles_compatible(16, 8))
        self.assertTrue(are_profiles_compatible(6, 12))

    def test_are_profiles_incompatible(self) -> None:
        from speccade.armature_driven import are_profiles_compatible
        # Non-matching, non-2x = incompatible
        self.assertFalse(are_profiles_compatible(8, 12))
        self.assertFalse(are_profiles_compatible(7, 9))
        self.assertFalse(are_profiles_compatible(8, 24))  # 3x, not 2x

    def test_get_bridge_pairs_empty_when_no_bridging(self) -> None:
        from speccade.armature_driven import get_bridge_pairs

        bone_hierarchy = {
            "spine": {"parent": None, "children": ["chest"]},
            "chest": {"parent": "spine", "children": []},
        }
        bone_meshes = {
            "spine": {"profile": "circle(8)"},  # No connect_end
            "chest": {"profile": "circle(8)"},  # No connect_start
        }
        pairs = get_bridge_pairs(bone_hierarchy, bone_meshes)
        self.assertEqual(pairs, [])

    def test_get_bridge_pairs_finds_matching_bridges(self) -> None:
        from speccade.armature_driven import get_bridge_pairs

        bone_hierarchy = {
            "spine": {"parent": None, "children": ["chest"]},
            "chest": {"parent": "spine", "children": ["neck"]},
            "neck": {"parent": "chest", "children": []},
        }
        bone_meshes = {
            "spine": {"profile": "circle(8)", "connect_end": "bridge"},
            "chest": {"profile": "circle(8)", "connect_start": "bridge", "connect_end": "bridge"},
            "neck": {"profile": "circle(8)", "connect_start": "bridge"},
        }
        pairs = get_bridge_pairs(bone_hierarchy, bone_meshes)
        self.assertEqual(len(pairs), 2)
        self.assertIn(("spine", "chest"), pairs)
        self.assertIn(("chest", "neck"), pairs)

    def test_get_bridge_pairs_requires_both_sides(self) -> None:
        from speccade.armature_driven import get_bridge_pairs

        bone_hierarchy = {
            "spine": {"parent": None, "children": ["chest"]},
            "chest": {"parent": "spine", "children": []},
        }
        bone_meshes = {
            "spine": {"profile": "circle(8)", "connect_end": "bridge"},
            "chest": {"profile": "circle(8)"},  # No connect_start - asymmetric
        }
        pairs = get_bridge_pairs(bone_hierarchy, bone_meshes)
        self.assertEqual(pairs, [])  # No bridge - requires both sides


class TestEdgeLoopTracking(unittest.TestCase):
    """Test edge loop vertex group tracking (requires Blender mock or skip)."""

    def test_bridge_vertex_group_names(self) -> None:
        """Test the naming convention for bridge vertex groups."""
        from speccade.armature_driven import (
            get_bridge_head_vgroup_name,
            get_bridge_tail_vgroup_name,
        )
        self.assertEqual(get_bridge_head_vgroup_name("spine"), "_bridge_head_spine")
        self.assertEqual(get_bridge_tail_vgroup_name("spine"), "_bridge_tail_spine")


if __name__ == "__main__":
    unittest.main()
