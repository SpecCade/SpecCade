import unittest
from pathlib import Path
import sys


# Allow `import speccade.*` from repo root.
_BLENDER_DIR = Path(__file__).resolve().parents[2]
if str(_BLENDER_DIR) not in sys.path:
    sys.path.insert(0, str(_BLENDER_DIR))


class TestArmatureDrivenModule(unittest.TestCase):
    def test_can_import_armature_driven_module(self) -> None:
        from speccade.armature_driven import build_armature_driven_character_mesh

        self.assertTrue(callable(build_armature_driven_character_mesh))


if __name__ == "__main__":
    unittest.main()
