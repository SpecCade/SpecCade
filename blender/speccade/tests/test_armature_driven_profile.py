import unittest
from pathlib import Path
import sys


# Allow `import speccade.*` from repo root.
_BLENDER_DIR = Path(__file__).resolve().parents[2]
if str(_BLENDER_DIR) not in sys.path:
    sys.path.insert(0, str(_BLENDER_DIR))


class TestArmatureDrivenProfileParsing(unittest.TestCase):
    def test_parse_none_defaults_to_circle_12(self) -> None:
        from speccade.armature_driven import _parse_profile

        self.assertEqual(_parse_profile(None), ("circle", 12))

    def test_parse_circle_with_segments(self) -> None:
        from speccade.armature_driven import _parse_profile

        self.assertEqual(_parse_profile("circle(12)"), ("circle", 12))

    def test_parse_hexagon_aliases_to_circle(self) -> None:
        from speccade.armature_driven import _parse_profile

        self.assertEqual(_parse_profile("hexagon(6)"), ("circle", 6))

    def test_parse_square_defaults_to_4_segments(self) -> None:
        from speccade.armature_driven import _parse_profile

        self.assertEqual(_parse_profile("square"), ("square", 4))

    def test_parse_rectangle_defaults_to_4_segments(self) -> None:
        from speccade.armature_driven import _parse_profile

        self.assertEqual(_parse_profile("rectangle"), ("rectangle", 4))

    def test_parse_circle_invalid_segments_raises_value_error(self) -> None:
        from speccade.armature_driven import _parse_profile

        with self.assertRaises(ValueError):
            _parse_profile("circle(2)")

    def test_unknown_profile_mentions_accepted_forms(self) -> None:
        from speccade.armature_driven import _parse_profile

        with self.assertRaises(ValueError) as ctx:
            _parse_profile("triangle")
        self.assertIn("circle(N)", str(ctx.exception))

    def test_non_str_non_none_raises_type_error(self) -> None:
        from speccade.armature_driven import _parse_profile

        with self.assertRaises(TypeError) as ctx:
            _parse_profile(123)  # type: ignore[arg-type]
        self.assertIn("profile must be a str or None", str(ctx.exception))


if __name__ == "__main__":
    unittest.main()
