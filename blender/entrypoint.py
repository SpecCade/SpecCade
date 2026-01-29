#!/usr/bin/env python3
"""
SpecCade Blender Entrypoint

This script is executed by Blender in background mode to generate assets
from canonical JSON specs.

Usage:
    blender --background --factory-startup --python entrypoint.py -- \
        --mode <mode> --spec <path> --out-root <path> --report <path>

Modes:
    static_mesh     - Generate static mesh (blender_primitives_v1)
    modular_kit     - Generate modular kit mesh (modular_kit_v1)
    organic_sculpt  - Generate organic sculpt mesh (organic_sculpt_v1)
    shrinkwrap      - Generate shrinkwrap mesh (shrinkwrap_v1) - armor/clothing wrapping
    boolean_kit     - Generate boolean kitbash mesh (boolean_kit_v1) - hard-surface modeling
    skeletal_mesh   - Generate skeletal mesh (armature_driven_v1 / skinned_mesh_v1)
    animation       - Generate animation clip (blender_clip_v1)
    rigged_animation - Generate rigged animation (skeletal_mesh + animation)
    mesh_to_sprite  - Generate sprite sheet from mesh
    validation_grid - Generate 6-view validation grid PNG for LLM verification
"""

import sys
from pathlib import Path

# Add the blender directory to sys.path so we can import the speccade package
_blender_dir = Path(__file__).parent.resolve()
if str(_blender_dir) not in sys.path:
    sys.path.insert(0, str(_blender_dir))

from speccade.main import main

if __name__ == "__main__":
    sys.exit(main())
