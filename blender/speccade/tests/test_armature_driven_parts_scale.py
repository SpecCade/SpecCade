import unittest
from pathlib import Path
import sys


# Allow `import speccade.*` from repo root.
_BLENDER_DIR = Path(__file__).resolve().parents[2]
if str(_BLENDER_DIR) not in sys.path:
    sys.path.insert(0, str(_BLENDER_DIR))


class TestArmatureDrivenPartScale(unittest.TestCase):
    def test_defaults_to_uniform_bone_length_scaling(self) -> None:
        from speccade.armature_driven import _resolve_part_scale_factors

        self.assertEqual(_resolve_part_scale_factors(None, bone_length=2.0), (2.0, 2.0, 2.0))
        self.assertEqual(_resolve_part_scale_factors({}, bone_length=1.5), (1.5, 1.5, 1.5))

    def test_missing_axes_uses_uniform_defaults(self) -> None:
        from speccade.armature_driven import _resolve_part_scale_factors

        factors = _resolve_part_scale_factors(
            {"amount_from_z": {"x": 0.5}},
            bone_length=3.0,
        )
        self.assertEqual(factors, (2.0, 3.0, 3.0))

    def test_explicit_fixed_axes_empty(self) -> None:
        from speccade.armature_driven import _resolve_part_scale_factors

        self.assertEqual(
            _resolve_part_scale_factors(
                {"axes": []},
                bone_length=4.0,
            ),
            (1.0, 1.0, 1.0),
        )

    def test_z_only_scaling(self) -> None:
        from speccade.armature_driven import _resolve_part_scale_factors

        self.assertEqual(
            _resolve_part_scale_factors(
                {"axes": ["z"], "amount_from_z": {"z": 1.0}},
                bone_length=2.5,
            ),
            (1.0, 1.0, 2.5),
        )

    def test_hybrid_scaling(self) -> None:
        from speccade.armature_driven import _resolve_part_scale_factors

        self.assertEqual(
            _resolve_part_scale_factors(
                {"axes": ["x", "y", "z"], "amount_from_z": {"x": 0.25, "y": 0.5, "z": 1.0}},
                bone_length=2.0,
            ),
            (1.25, 1.5, 2.0),
        )

    def test_rejects_duplicate_axes(self) -> None:
        from speccade.armature_driven import _resolve_part_scale_factors

        with self.assertRaises(ValueError):
            _resolve_part_scale_factors({"axes": ["x", "x"]}, bone_length=2.0)

    def test_rejects_amount_out_of_range(self) -> None:
        from speccade.armature_driven import _resolve_part_scale_factors

        with self.assertRaises(ValueError):
            _resolve_part_scale_factors(
                {"axes": ["x"], "amount_from_z": {"x": 1.1}},
                bone_length=2.0,
            )


if __name__ == "__main__":
    unittest.main()
