#!/usr/bin/env python3
"""
Preset Library Feature Analysis Script
Analyzes all preset JSON files and generates feature usage statistics.
Reusable for future packs.

Usage:
    python analyze_presets.py [pack_path]

Default pack_path: packs/preset_library_v1/audio
"""

import json
import os
import sys
from collections import defaultdict
from pathlib import Path

def analyze_preset(filepath):
    """Extract features from a single preset file."""
    features = {
        'synthesis_types': [],
        'effects': [],
        'lfo_targets': [],
        'filter_types': [],
        'waveforms': [],
        'layer_count': 0,
        'effect_count': 0,
        'has_lfo': False,
        'has_filter': False,
    }

    try:
        with open(filepath, 'r', encoding='utf-8') as f:
            data = json.load(f)
    except (json.JSONDecodeError, FileNotFoundError) as e:
        print(f"Warning: Could not parse {filepath}: {e}")
        return None

    recipe = data.get('recipe', {})
    params = recipe.get('params', {})
    layers = params.get('layers', [])
    effects = params.get('effects', [])

    features['layer_count'] = len(layers)
    features['effect_count'] = len(effects)

    # Analyze layers
    for layer in layers:
        synthesis = layer.get('synthesis', {})
        synth_type = synthesis.get('type')
        if synth_type:
            features['synthesis_types'].append(synth_type)

        # Check for oscillator waveforms
        if 'waveform' in synthesis:
            features['waveforms'].append(synthesis['waveform'])
        if 'oscillators' in synthesis:
            for osc in synthesis['oscillators']:
                if 'waveform' in osc:
                    features['waveforms'].append(osc['waveform'])
        if 'carrier_type' in synthesis:
            features['waveforms'].append(synthesis['carrier_type'])

        # Check for filter
        layer_filter = layer.get('filter', {})
        if layer_filter:
            features['has_filter'] = True
            filter_type = layer_filter.get('type')
            if filter_type:
                features['filter_types'].append(filter_type)

        # Check for LFO
        lfo = layer.get('lfo', {})
        if lfo:
            features['has_lfo'] = True
            config = lfo.get('config', {})
            target = lfo.get('target', {})

            if 'waveform' in config:
                features['waveforms'].append(config['waveform'])

            lfo_target = target.get('target')
            if lfo_target:
                features['lfo_targets'].append(lfo_target)

    # Analyze effects
    for effect in effects:
        effect_type = effect.get('type')
        if effect_type:
            features['effects'].append(effect_type)

    return features

def analyze_pack(pack_path):
    """Analyze all presets in a pack directory."""
    results = {
        'total_presets': 0,
        'synthesis_counts': defaultdict(int),
        'effect_counts': defaultdict(int),
        'lfo_target_counts': defaultdict(int),
        'filter_counts': defaultdict(int),
        'waveform_counts': defaultdict(int),
        'layer_distribution': defaultdict(int),
        'effect_distribution': defaultdict(int),
        'presets_with_lfo': 0,
        'presets_with_filter': 0,
        'presets_with_effects': 0,
        'category_counts': defaultdict(int),
        'presets_by_category': defaultdict(list),
    }

    pack_path = Path(pack_path)

    for json_file in pack_path.rglob('*.json'):
        # Skip non-preset files
        if json_file.name.startswith('.'):
            continue
        if '.report.json' in json_file.name:
            continue
        if json_file.name.startswith('test_'):
            continue

        features = analyze_preset(json_file)
        if features is None:
            continue

        results['total_presets'] += 1

        # Get category from path
        rel_path = json_file.relative_to(pack_path)
        category = rel_path.parts[0] if len(rel_path.parts) > 1 else 'root'
        subcategory = rel_path.parts[1] if len(rel_path.parts) > 2 else None
        full_category = f"{category}/{subcategory}" if subcategory else category

        results['category_counts'][full_category] += 1
        results['presets_by_category'][full_category].append(json_file.stem)

        # Count synthesis types
        for synth_type in features['synthesis_types']:
            results['synthesis_counts'][synth_type] += 1

        # Count effects
        for effect in features['effects']:
            results['effect_counts'][effect] += 1

        # Count LFO targets
        for target in features['lfo_targets']:
            results['lfo_target_counts'][target] += 1

        # Count filter types
        for filter_type in features['filter_types']:
            results['filter_counts'][filter_type] += 1

        # Count waveforms
        for waveform in features['waveforms']:
            results['waveform_counts'][waveform] += 1

        # Layer/effect distributions
        results['layer_distribution'][features['layer_count']] += 1
        results['effect_distribution'][features['effect_count']] += 1

        # Presence counts
        if features['has_lfo']:
            results['presets_with_lfo'] += 1
        if features['has_filter']:
            results['presets_with_filter'] += 1
        if features['effect_count'] > 0:
            results['presets_with_effects'] += 1

    return results

def print_report(results):
    """Print a formatted report of the analysis."""
    total = results['total_presets']

    print("=" * 70)
    print("PRESET LIBRARY FEATURE ANALYSIS REPORT")
    print("=" * 70)
    print()

    # Overview
    print("## OVERVIEW")
    print(f"Total Presets: {total}")
    print(f"Presets with LFO modulation: {results['presets_with_lfo']} ({100*results['presets_with_lfo']/total:.1f}%)")
    print(f"Presets with filters: {results['presets_with_filter']} ({100*results['presets_with_filter']/total:.1f}%)")
    print(f"Presets with effects: {results['presets_with_effects']} ({100*results['presets_with_effects']/total:.1f}%)")
    print()

    # Category breakdown
    print("## CATEGORY BREAKDOWN")
    print(f"{'Category':<30} {'Count':>8}")
    print("-" * 40)
    for category, count in sorted(results['category_counts'].items()):
        print(f"{category:<30} {count:>8}")
    print()

    # Synthesis types
    print("## SYNTHESIS TYPES (by usage count)")
    print(f"{'Synthesis Type':<25} {'Count':>8} {'% of layers':>12}")
    print("-" * 50)
    total_layers = sum(results['synthesis_counts'].values())
    for synth_type, count in sorted(results['synthesis_counts'].items(), key=lambda x: -x[1]):
        pct = 100 * count / total_layers if total_layers > 0 else 0
        print(f"{synth_type:<25} {count:>8} {pct:>11.1f}%")
    print()

    # Effects
    print("## EFFECTS (by usage count)")
    print(f"{'Effect Type':<25} {'Count':>8} {'% of effects':>12}")
    print("-" * 50)
    total_effects = sum(results['effect_counts'].values())
    for effect, count in sorted(results['effect_counts'].items(), key=lambda x: -x[1]):
        pct = 100 * count / total_effects if total_effects > 0 else 0
        print(f"{effect:<25} {count:>8} {pct:>11.1f}%")
    print()

    # LFO Targets
    print("## LFO TARGETS (by usage count)")
    print(f"{'LFO Target':<25} {'Count':>8}")
    print("-" * 35)
    for target, count in sorted(results['lfo_target_counts'].items(), key=lambda x: -x[1]):
        print(f"{target:<25} {count:>8}")
    print()

    # Filter types
    print("## FILTER TYPES (by usage count)")
    print(f"{'Filter Type':<25} {'Count':>8}")
    print("-" * 35)
    for filter_type, count in sorted(results['filter_counts'].items(), key=lambda x: -x[1]):
        print(f"{filter_type:<25} {count:>8}")
    print()

    # Waveforms
    print("## WAVEFORMS (by usage count)")
    print(f"{'Waveform':<25} {'Count':>8}")
    print("-" * 35)
    for waveform, count in sorted(results['waveform_counts'].items(), key=lambda x: -x[1]):
        print(f"{waveform:<25} {count:>8}")
    print()

    # Layer distribution
    print("## LAYER COUNT DISTRIBUTION")
    print(f"{'Layers':<10} {'Presets':>8} {'% of total':>12}")
    print("-" * 35)
    for layer_count, preset_count in sorted(results['layer_distribution'].items()):
        pct = 100 * preset_count / total if total > 0 else 0
        print(f"{layer_count:<10} {preset_count:>8} {pct:>11.1f}%")
    print()

    # Effect count distribution
    print("## EFFECT COUNT DISTRIBUTION")
    print(f"{'Effects':<10} {'Presets':>8} {'% of total':>12}")
    print("-" * 35)
    for effect_count, preset_count in sorted(results['effect_distribution'].items()):
        pct = 100 * preset_count / total if total > 0 else 0
        print(f"{effect_count:<10} {preset_count:>8} {pct:>11.1f}%")
    print()

def generate_markdown_report(results, output_path):
    """Generate a markdown report file."""
    total = results['total_presets']

    with open(output_path, 'w', encoding='utf-8') as f:
        f.write("# Preset Library V1 - Feature Analysis Report\n\n")
        f.write("*Auto-generated by analyze_presets.py*\n\n")

        # Overview
        f.write("## Overview\n\n")
        f.write(f"- **Total Presets:** {total}\n")
        f.write(f"- **Presets with LFO modulation:** {results['presets_with_lfo']} ({100*results['presets_with_lfo']/total:.1f}%)\n")
        f.write(f"- **Presets with filters:** {results['presets_with_filter']} ({100*results['presets_with_filter']/total:.1f}%)\n")
        f.write(f"- **Presets with effects:** {results['presets_with_effects']} ({100*results['presets_with_effects']/total:.1f}%)\n\n")

        # Category breakdown
        f.write("## Category Breakdown\n\n")
        f.write("| Category | Count |\n")
        f.write("|----------|-------|\n")
        for category, count in sorted(results['category_counts'].items()):
            f.write(f"| {category} | {count} |\n")
        f.write("\n")

        # Synthesis types
        f.write("## Synthesis Types\n\n")
        f.write("| Synthesis Type | Count | % of Layers |\n")
        f.write("|----------------|-------|-------------|\n")
        total_layers = sum(results['synthesis_counts'].values())
        for synth_type, count in sorted(results['synthesis_counts'].items(), key=lambda x: -x[1]):
            pct = 100 * count / total_layers if total_layers > 0 else 0
            f.write(f"| {synth_type} | {count} | {pct:.1f}% |\n")
        f.write("\n")

        # Effects
        f.write("## Effects\n\n")
        f.write("| Effect Type | Count | % of Effects |\n")
        f.write("|-------------|-------|-------------|\n")
        total_effects = sum(results['effect_counts'].values())
        for effect, count in sorted(results['effect_counts'].items(), key=lambda x: -x[1]):
            pct = 100 * count / total_effects if total_effects > 0 else 0
            f.write(f"| {effect} | {count} | {pct:.1f}% |\n")
        f.write("\n")

        # LFO Targets
        f.write("## LFO Targets\n\n")
        f.write("| LFO Target | Count |\n")
        f.write("|------------|-------|\n")
        for target, count in sorted(results['lfo_target_counts'].items(), key=lambda x: -x[1]):
            f.write(f"| {target} | {count} |\n")
        f.write("\n")

        # Filter types
        f.write("## Filter Types\n\n")
        f.write("| Filter Type | Count |\n")
        f.write("|-------------|-------|\n")
        for filter_type, count in sorted(results['filter_counts'].items(), key=lambda x: -x[1]):
            f.write(f"| {filter_type} | {count} |\n")
        f.write("\n")

        # Waveforms
        f.write("## Waveforms\n\n")
        f.write("| Waveform | Count |\n")
        f.write("|----------|-------|\n")
        for waveform, count in sorted(results['waveform_counts'].items(), key=lambda x: -x[1]):
            f.write(f"| {waveform} | {count} |\n")
        f.write("\n")

        # Layer distribution
        f.write("## Layer Count Distribution\n\n")
        f.write("| Layers | Presets | % of Total |\n")
        f.write("|--------|---------|------------|\n")
        for layer_count, preset_count in sorted(results['layer_distribution'].items()):
            pct = 100 * preset_count / total if total > 0 else 0
            f.write(f"| {layer_count} | {preset_count} | {pct:.1f}% |\n")
        f.write("\n")

        # Effect count distribution
        f.write("## Effect Count Distribution\n\n")
        f.write("| Effects | Presets | % of Total |\n")
        f.write("|---------|---------|------------|\n")
        for effect_count, preset_count in sorted(results['effect_distribution'].items()):
            pct = 100 * preset_count / total if total > 0 else 0
            f.write(f"| {effect_count} | {preset_count} | {pct:.1f}% |\n")
        f.write("\n")

    print(f"Markdown report saved to: {output_path}")

def main():
    pack_path = "packs/preset_library_v1/audio"
    write_md = False
    md_out = None

    args = sys.argv[1:]
    if "--write-md" in args:
        write_md = True
        args = [a for a in args if a != "--write-md"]

    if "--md" in args:
        i = args.index("--md")
        if i + 1 >= len(args):
            raise SystemExit("Expected a path after --md")
        md_out = args[i + 1]
        write_md = True
        args = args[:i] + args[i + 2 :]

    if args:
        pack_path = args[0]

    print(f"Analyzing presets in: {pack_path}")
    print()

    results = analyze_pack(pack_path)

    if results['total_presets'] == 0:
        print(f"No presets found in {pack_path}")
        return

    print_report(results)

    if write_md:
        md_path = Path(md_out) if md_out is not None else Path(pack_path).parent / "FEATURE_ANALYSIS.md"
        generate_markdown_report(results, md_path)

if __name__ == "__main__":
    main()
