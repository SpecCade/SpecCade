#!/usr/bin/env python3
"""Validate all instrument JSON files and report failures."""

import subprocess
import json
from pathlib import Path

def validate_file(filepath):
    """Validate a single JSON file using speccade validate."""
    result = subprocess.run(
        ["cargo", "run", "-p", "speccade-cli", "-q", "--", "validate", "--spec", str(filepath)],
        capture_output=True,
        text=True
    )
    return result.returncode == 0, result.stderr

def main():
    base_dir = Path("packs/preset_library_v1/audio")

    passed = []
    failed = []

    for json_file in sorted(base_dir.rglob("*.json")):
        if ".report." in str(json_file):
            continue
        if "test_" in json_file.name:
            continue  # Skip test files

        success, error = validate_file(json_file)
        if success:
            passed.append(json_file)
            print(f"PASS: {json_file}")
        else:
            failed.append((json_file, error))
            print(f"FAIL: {json_file}")

    print(f"\n=== SUMMARY ===")
    print(f"Passed: {len(passed)}")
    print(f"Failed: {len(failed)}")

    if failed:
        print(f"\n=== FAILED FILES ===")
        for filepath, error in failed:
            print(f"\n{filepath}:")
            # Extract just the relevant error line
            for line in error.split('\n'):
                if 'error' in line.lower() or 'invalid' in line.lower():
                    print(f"  {line.strip()}")
                    break
            else:
                print(f"  {error[:200]}")

if __name__ == "__main__":
    main()
