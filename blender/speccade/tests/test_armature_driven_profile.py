import unittest
from pathlib import Path
import sys


# Allow `import speccade.*` from repo root.
_BLENDER_DIR = Path(__file__).resolve().parents[2]
if str(_BLENDER_DIR) not in sys.path:
    sys.path.insert(0, str(_BLENDER_DIR))


class TestArmatureDrivenProfileParsing(unittest.TestCase):
    def test_parse_circle_with_segments(self) -> None:
        from speccade.armature_driven import _parse_profile

        self.assertEqual(_parse_profile("circle(12)"), ("circle", 12))

    def test_parse_hexagon_aliases_to_circle(self) -> None:
        from speccade.armature_driven import _parse_profile

        self.assertEqual(_parse_profile("hexagon(6)"), ("circle", 6))

    def test_parse_square_defaults_to_4_segments(self) -> None:
        from speccade.armature_driven import _parse_profile

        self.assertEqual(_parse_profile("square"), ("square", 4))


if __name__ == "__main__":
    unittest.main()
