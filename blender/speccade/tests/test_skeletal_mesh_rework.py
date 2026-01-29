import unittest
from pathlib import Path
import sys


# Allow `import speccade.*` from repo root.
_BLENDER_DIR = Path(__file__).resolve().parents[2]
if str(_BLENDER_DIR) not in sys.path:
    sys.path.insert(0, str(_BLENDER_DIR))


class TestSkeletalMeshReworkHelpers(unittest.TestCase):
    def test_kind_dispatch_allows_known_kinds(self) -> None:
        from speccade.skeletal_mesh_rework import classify_skeletal_mesh_kind

        self.assertEqual(
            classify_skeletal_mesh_kind("skeletal_mesh.armature_driven_v1"),
            "skeletal_mesh.armature_driven_v1",
        )
        self.assertEqual(
            classify_skeletal_mesh_kind("skeletal_mesh.skinned_mesh_v1"),
            "skeletal_mesh.skinned_mesh_v1",
        )

    def test_kind_dispatch_rejects_unknown_skeletal_mesh_kind(self) -> None:
        from speccade.skeletal_mesh_rework import classify_skeletal_mesh_kind

        with self.assertRaises(ValueError) as ctx:
            classify_skeletal_mesh_kind("skeletal_mesh.some_future_thing")
        self.assertIn("Unsupported skeletal_mesh recipe kind", str(ctx.exception))

    def test_kind_dispatch_rejects_non_skeletal_mesh_kind(self) -> None:
        from speccade.skeletal_mesh_rework import classify_skeletal_mesh_kind

        with self.assertRaises(ValueError):
            classify_skeletal_mesh_kind("mesh.static_mesh_v1")

    def test_safe_vertex_group_rename_plan_handles_cycle(self) -> None:
        from speccade.skeletal_mesh_rework import compute_safe_rename_plan

        plan = compute_safe_rename_plan({"A": "B", "B": "A"}, existing_names=["A", "B"])

        # Simulate applying the plan and ensure no step overwrites an existing name.
        names = set(["A", "B"])
        for src, dst in plan:
            self.assertIn(src, names)
            self.assertNotIn(dst, names)
            names.remove(src)
            names.add(dst)
        self.assertEqual(names, set(["A", "B"]))

    def test_safe_vertex_group_rename_plan_filters_missing_sources(self) -> None:
        from speccade.skeletal_mesh_rework import compute_safe_rename_plan

        plan = compute_safe_rename_plan(
            {"Arm.L": "arm_upper_L", "Arm.R": "arm_upper_R"},
            existing_names=["Arm.L"],
        )
        self.assertEqual(plan, [("Arm.L", "arm_upper_L")])

    def test_safe_vertex_group_rename_plan_rejects_duplicate_destinations(self) -> None:
        from speccade.skeletal_mesh_rework import compute_safe_rename_plan

        with self.assertRaises(ValueError) as ctx:
            compute_safe_rename_plan({"A": "X", "B": "X"}, existing_names=["A", "B"])
        self.assertIn("merge required", str(ctx.exception))


if __name__ == "__main__":
    unittest.main()
