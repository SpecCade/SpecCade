import unittest
from pathlib import Path
import sys


# Allow `import speccade.*` from repo root.
_BLENDER_DIR = Path(__file__).resolve().parents[2]
if str(_BLENDER_DIR) not in sys.path:
    sys.path.insert(0, str(_BLENDER_DIR))


class TestArmatureDrivenBoolShapes(unittest.TestCase):
    def test_resolve_bool_shape_mirror(self) -> None:
        from speccade.armature_driven import _resolve_mirrors

        defs = {
            "eye_socket_L": {"primitive": "sphere", "dimensions": [0.1, 0.2, 0.1], "position": [0.1, 0.2, 0.3]},
            "eye_socket_R": {"mirror": "eye_socket_L"},
        }
        resolved = _resolve_mirrors(defs)
        self.assertEqual(resolved["eye_socket_R"]["primitive"], "sphere")


if __name__ == "__main__":
    unittest.main()
