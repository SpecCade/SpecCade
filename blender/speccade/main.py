"""Main entry point for SpecCade Blender operations.

This module provides the main() function that serves as the CLI entry point
when running SpecCade scripts inside Blender. It parses command-line arguments,
loads spec files, and dispatches to the appropriate handler based on the mode.
"""

import argparse
import json
import sys
from pathlib import Path

from .report import write_report
from .scene import clear_scene, setup_scene
from .handlers_mesh import (
    handle_static_mesh,
    handle_modular_kit,
    handle_organic_sculpt,
    handle_shrinkwrap,
    handle_boolean_kit,
)
from .handlers_skeletal import (
    handle_skeletal_mesh,
    handle_animation,
    handle_rigged_animation,
    handle_animation_helpers,
)
from .handlers_render import (
    handle_mesh_to_sprite,
    handle_validation_grid,
)

try:
    import bpy
    BLENDER_AVAILABLE = True
except ImportError:
    BLENDER_AVAILABLE = False


def main() -> int:
    """Main entry point."""
    # Parse arguments after '--'
    try:
        argv = sys.argv[sys.argv.index("--") + 1:]
    except ValueError:
        argv = []

    parser = argparse.ArgumentParser(description="SpecCade Blender Entrypoint")
    parser.add_argument("--mode", required=True,
                        choices=["static_mesh", "modular_kit", "organic_sculpt", "shrinkwrap", "boolean_kit", "skeletal_mesh", "animation", "rigged_animation", "animation_helpers", "mesh_to_sprite", "validation_grid"],
                        help="Generation mode")
    parser.add_argument("--spec", required=True, type=Path,
                        help="Path to spec JSON file")
    parser.add_argument("--out-root", required=True, type=Path,
                        help="Output root directory")
    parser.add_argument("--report", required=True, type=Path,
                        help="Path for report JSON output")

    args = parser.parse_args(argv)

    # Ensure Blender is available
    if not BLENDER_AVAILABLE:
        write_report(args.report, ok=False,
                     error="This script must be run inside Blender")
        return 1

    # Load spec
    try:
        with open(args.spec, 'r') as f:
            spec = json.load(f)
    except Exception as e:
        write_report(args.report, ok=False, error=f"Failed to load spec: {e}")
        return 1

    # Clear and setup scene
    clear_scene()
    setup_scene()

    # Dispatch to handler
    handlers = {
        "static_mesh": handle_static_mesh,
        "modular_kit": handle_modular_kit,
        "organic_sculpt": handle_organic_sculpt,
        "shrinkwrap": handle_shrinkwrap,
        "boolean_kit": handle_boolean_kit,
        "skeletal_mesh": handle_skeletal_mesh,
        "animation": handle_animation,
        "rigged_animation": handle_rigged_animation,
        "animation_helpers": handle_animation_helpers,
        "mesh_to_sprite": handle_mesh_to_sprite,
        "validation_grid": handle_validation_grid,
    }

    handler = handlers.get(args.mode)
    if not handler:
        write_report(args.report, ok=False, error=f"Unknown mode: {args.mode}")
        return 1

    try:
        handler(spec, args.out_root, args.report)
        return 0
    except Exception as e:
        # Report already written in handler
        print(f"Error: {e}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
