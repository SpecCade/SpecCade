import unittest
from pathlib import Path
import sys
import math
from typing import cast


# Allow `import speccade.*` from repo root.
_BLENDER_DIR = Path(__file__).resolve().parents[2]
if str(_BLENDER_DIR) not in sys.path:
    sys.path.insert(0, str(_BLENDER_DIR))


class TestArmatureDrivenBoneRelativeLengths(unittest.TestCase):
    def test_number_is_relative_to_bone_length(self) -> None:
        from speccade.armature_driven import _resolve_bone_relative_length

        self.assertAlmostEqual(
            cast(float, _resolve_bone_relative_length(0.4, bone_length=2.5)),
            1.0,
        )

    def test_list_of_two_is_relative2_to_bone_length(self) -> None:
        from speccade.armature_driven import _resolve_bone_relative_length

        self.assertEqual(
            _resolve_bone_relative_length([0.2, 0.6], bone_length=10.0),
            (2.0, 6.0),
        )

    def test_tuple_of_two_is_relative2_to_bone_length(self) -> None:
        from speccade.armature_driven import _resolve_bone_relative_length

        self.assertEqual(
            _resolve_bone_relative_length((0.2, 0.6), bone_length=10.0),
            (2.0, 6.0),
        )

    def test_dict_absolute_is_absolute(self) -> None:
        from speccade.armature_driven import _resolve_bone_relative_length

        self.assertAlmostEqual(
            cast(float, _resolve_bone_relative_length({"absolute": 1.23}, bone_length=999.0)),
            1.23,
        )

    def test_rejects_non_positive_results(self) -> None:
        from speccade.armature_driven import _resolve_bone_relative_length

        with self.assertRaises(ValueError):
            _resolve_bone_relative_length(0.0, bone_length=1.0)

        with self.assertRaises(ValueError):
            _resolve_bone_relative_length(-0.1, bone_length=1.0)

        with self.assertRaises(ValueError):
            _resolve_bone_relative_length([0.0, 0.1], bone_length=1.0)

        with self.assertRaises(ValueError):
            _resolve_bone_relative_length([0.1, -0.1], bone_length=1.0)

        with self.assertRaises(ValueError):
            _resolve_bone_relative_length({"absolute": 0.0}, bone_length=1.0)

        with self.assertRaises(ValueError):
            _resolve_bone_relative_length({"absolute": -1.0}, bone_length=1.0)

    def test_rejects_invalid_types_and_shapes(self) -> None:
        from speccade.armature_driven import _resolve_bone_relative_length

        with self.assertRaises(TypeError):
            _resolve_bone_relative_length(True, bone_length=1.0)

        with self.assertRaises(TypeError):
            _resolve_bone_relative_length([1.0], bone_length=1.0)

        with self.assertRaises(TypeError):
            _resolve_bone_relative_length([1.0, 2.0, 3.0], bone_length=1.0)

        with self.assertRaises(TypeError):
            _resolve_bone_relative_length([1.0, "x"], bone_length=1.0)

        with self.assertRaises(ValueError):
            _resolve_bone_relative_length({"absolute": 1.0, "extra": 2.0}, bone_length=1.0)

        with self.assertRaises(ValueError):
            _resolve_bone_relative_length({"not_absolute": 1.0}, bone_length=1.0)

        with self.assertRaises(ValueError):
            _resolve_bone_relative_length({}, bone_length=1.0)

        with self.assertRaises(TypeError):
            _resolve_bone_relative_length(None, bone_length=1.0)

        with self.assertRaises(TypeError):
            _resolve_bone_relative_length("1.0", bone_length=1.0)

    def test_rejects_non_positive_bone_length(self) -> None:
        from speccade.armature_driven import _resolve_bone_relative_length

        with self.assertRaises(ValueError):
            _resolve_bone_relative_length(0.1, bone_length=0.0)

        with self.assertRaises(ValueError):
            _resolve_bone_relative_length(0.1, bone_length=-1.0)

    def test_rejects_bool_bone_length(self) -> None:
        from speccade.armature_driven import _resolve_bone_relative_length

        with self.assertRaises(TypeError):
            _resolve_bone_relative_length(0.1, bone_length=True)

    def test_rejects_nan_or_inf_bone_length(self) -> None:
        from speccade.armature_driven import _resolve_bone_relative_length

        for bad in (math.nan, math.inf, -math.inf):
            with self.assertRaises(ValueError):
                _resolve_bone_relative_length(0.1, bone_length=bad)

    def test_rejects_nan_or_inf_values(self) -> None:
        from speccade.armature_driven import _resolve_bone_relative_length

        for bad in (math.nan, math.inf, -math.inf):
            with self.assertRaises(ValueError):
                _resolve_bone_relative_length(bad, bone_length=1.0)
            with self.assertRaises(ValueError):
                _resolve_bone_relative_length({"absolute": bad}, bone_length=1.0)

    def test_rejects_nan_or_inf_in_relative2(self) -> None:
        from speccade.armature_driven import _resolve_bone_relative_length

        for bad in (math.nan, math.inf, -math.inf):
            with self.assertRaises(ValueError):
                _resolve_bone_relative_length([bad, 0.1], bone_length=1.0)
            with self.assertRaises(ValueError):
                _resolve_bone_relative_length([0.1, bad], bone_length=1.0)

    def test_rejects_overflow_to_infinity(self) -> None:
        from speccade.armature_driven import _resolve_bone_relative_length

        with self.assertRaises(ValueError):
            _resolve_bone_relative_length(1e308, bone_length=1e308)

        with self.assertRaises(ValueError):
            _resolve_bone_relative_length([1e308, 1.0], bone_length=1e308)


if __name__ == "__main__":
    unittest.main()
