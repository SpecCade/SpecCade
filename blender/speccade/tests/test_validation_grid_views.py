import unittest
from pathlib import Path
import sys


# Allow `import speccade.*` from repo root.
_BLENDER_DIR = Path(__file__).resolve().parents[2]
if str(_BLENDER_DIR) not in sys.path:
    sys.path.insert(0, str(_BLENDER_DIR))


class TestValidationGridViews(unittest.TestCase):
    def test_validation_grid_uses_canonical_axis_labels(self) -> None:
        from speccade.handlers_render import VALIDATION_GRID_VIEWS

        by_label = {label: (azimuth, elevation) for label, azimuth, elevation in VALIDATION_GRID_VIEWS}

        # Canonical convention: +Y is FRONT, -Y is BACK.
        self.assertEqual(by_label["FRONT"], (180.0, 30.0))
        self.assertEqual(by_label["BACK"], (0.0, 30.0))

        # Character-left is -X, character-right is +X.
        self.assertEqual(by_label["LEFT"], (270.0, 30.0))
        self.assertEqual(by_label["RIGHT"], (90.0, 30.0))

        # Isometric should be front-biased (not back-biased).
        self.assertEqual(by_label["ISO"], (135.0, 35.264))


if __name__ == "__main__":
    unittest.main()

