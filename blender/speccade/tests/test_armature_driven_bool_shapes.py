import unittest
from pathlib import Path
import sys


# Allow `import speccade.*` from repo root.
_BLENDER_DIR = Path(__file__).resolve().parents[2]
if str(_BLENDER_DIR) not in sys.path:
    sys.path.insert(0, str(_BLENDER_DIR))


class TestArmatureDrivenBoolShapes(unittest.TestCase):
    def test_resolve_mirrors_resolves_mirror_only_dicts(self) -> None:
        from speccade.armature_driven import _resolve_mirrors

        shape_defs = {
            "eye_socket_L": {"primitive": "sphere", "dimensions": [0.1, 0.2, 0.1], "position": [0.1, 0.2, 0.3]},
            "eye_socket_R": {"mirror": "eye_socket_L"},
        }

        resolved = _resolve_mirrors(shape_defs)

        expected_L = {"primitive": "sphere", "dimensions": [0.1, 0.2, 0.1], "position": [0.1, 0.2, 0.3]}
        expected_R = {"primitive": "sphere", "dimensions": [0.1, 0.2, 0.1], "position": [0.1, 0.2, 0.3]}

        # Assert resolved values include all expected fields.
        self.assertEqual(resolved["eye_socket_L"], expected_L)
        self.assertEqual(resolved["eye_socket_R"], expected_R)

        # Resolved mirror entries should not carry the 'mirror' key.
        self.assertNotIn("mirror", resolved["eye_socket_R"])

        # Guard against aliasing: list values must be deep-copied.
        self.assertIsNot(resolved["eye_socket_L"]["dimensions"], shape_defs["eye_socket_L"]["dimensions"])
        self.assertIsNot(resolved["eye_socket_L"]["position"], shape_defs["eye_socket_L"]["position"])
        self.assertIsNot(resolved["eye_socket_R"]["dimensions"], shape_defs["eye_socket_L"]["dimensions"])
        self.assertIsNot(resolved["eye_socket_R"]["position"], shape_defs["eye_socket_L"]["position"])

        # The L and R resolved entries should not share list objects either.
        self.assertIsNot(resolved["eye_socket_L"]["dimensions"], resolved["eye_socket_R"]["dimensions"])
        self.assertIsNot(resolved["eye_socket_L"]["position"], resolved["eye_socket_R"]["position"])


if __name__ == "__main__":
    unittest.main()
