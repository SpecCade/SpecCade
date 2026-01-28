"""
SpecCade Report Generation Module

This module handles writing generation reports for Blender asset creation.
Reports include success/failure status, metrics, output paths, and timing.
"""

import json
from pathlib import Path
from typing import Dict, Optional

# Blender modules - only available when running inside Blender
try:
    import bpy
    BLENDER_AVAILABLE = True
except ImportError:
    BLENDER_AVAILABLE = False


def write_report(report_path: Path, ok: bool, error: Optional[str] = None,
                 metrics: Optional[Dict] = None, output_path: Optional[str] = None,
                 blend_path: Optional[str] = None, preview_path: Optional[str] = None,
                 duration_ms: Optional[int] = None) -> None:
    """Write the generation report JSON."""
    report = {
        "ok": ok,
    }
    if error:
        report["error"] = error
    if metrics:
        report["metrics"] = metrics
    if output_path:
        report["output_path"] = output_path
    if blend_path:
        report["blend_path"] = blend_path
    if preview_path:
        report["preview_path"] = preview_path
    if duration_ms is not None:
        report["duration_ms"] = duration_ms
    if BLENDER_AVAILABLE:
        report["blender_version"] = bpy.app.version_string

    with open(report_path, 'w') as f:
        json.dump(report, f, indent=2)
