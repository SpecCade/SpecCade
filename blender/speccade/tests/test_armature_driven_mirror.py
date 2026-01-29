import unittest
from pathlib import Path
import sys


# Allow `import speccade.*` from repo root.
_BLENDER_DIR = Path(__file__).resolve().parents[2]
if str(_BLENDER_DIR) not in sys.path:
    sys.path.insert(0, str(_BLENDER_DIR))


class TestArmatureDrivenMirrorResolution(unittest.TestCase):
    def test_resolve_mirrors_sorts_keys_and_resolves_basic_mirror(self) -> None:
        from speccade.armature_driven import _resolve_mirrors

        defs = {
            "B": {"mirror": "A"},
            "A": {"foo": 1, "bar": ["x", "y"]},
        }
        resolved = _resolve_mirrors(defs)

        self.assertEqual(list(resolved.keys()), ["A", "B"])
        self.assertEqual(resolved["A"], {"foo": 1, "bar": ["x", "y"]})
        self.assertEqual(resolved["B"], {"foo": 1, "bar": ["x", "y"]})
        self.assertNotIn("mirror", resolved["A"])
        self.assertNotIn("mirror", resolved["B"])

    def test_resolve_mirrors_deep_copies_resolved_dicts(self) -> None:
        from speccade.armature_driven import _resolve_mirrors

        defs = {
            "A": {"x": [1]},
            "B": {"mirror": "A"},
        }
        resolved = _resolve_mirrors(defs)

        self.assertIsNot(resolved["A"], defs["A"])
        self.assertIsNot(resolved["B"], resolved["A"])
        self.assertIsNot(resolved["A"]["x"], defs["A"]["x"])
        self.assertIsNot(resolved["B"]["x"], resolved["A"]["x"])

        resolved["B"]["x"].append(2)
        self.assertEqual(resolved["A"]["x"], [1])
        self.assertEqual(defs["A"]["x"], [1])

    def test_resolve_mirrors_raises_on_missing_target(self) -> None:
        from speccade.armature_driven import _resolve_mirrors

        with self.assertRaises(ValueError) as ctx:
            _resolve_mirrors({"A": {"mirror": "MISSING"}})
        self.assertIn("mirror", str(ctx.exception))

    def test_resolve_mirrors_raises_on_cycle(self) -> None:
        from speccade.armature_driven import _resolve_mirrors

        defs = {
            "A": {"mirror": "B"},
            "B": {"mirror": "A"},
        }
        with self.assertRaises(ValueError) as ctx:
            _resolve_mirrors(defs)
        self.assertIn("cycle", str(ctx.exception).lower())


if __name__ == "__main__":
    unittest.main()
