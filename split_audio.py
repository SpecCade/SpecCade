#!/usr/bin/env python3
"""Script to split audio.rs into submodules."""

import re

# Read the original file
with open('crates/speccade-cli/src/compiler/stdlib/audio.rs', 'r', encoding='utf-8') as f:
    lines = f.readlines()

# Find the starlark_module block start and end
module_start = None
module_end = None
for i, line in enumerate(lines):
    if '#[starlark_module]' in line:
        module_start = i
    if module_start and line.strip() == '}' and i > module_start + 10:
        # This is likely the end of the module
        # Check if next non-empty line is extract_float or #[cfg(test)]
        for j in range(i+1, min(i+20, len(lines))):
            if lines[j].strip() and not lines[j].strip().startswith('//'):
                if 'fn extract_float' in lines[j] or '#[cfg(test)]' in lines[j]:
                    module_end = i
                    break
        if module_end:
            break

print(f"Module block: lines {module_start+1} to {module_end+1}")
print(f"Total lines in file: {len(lines)}")

# Extract function definitions and categorize them
synthesis_funcs = [
    'oscillator', 'fm_synth', 'am_synth', 'karplus_strong', 'noise_burst',
    'additive', 'supersaw_unison', 'wavetable', 'granular', 'granular_source',
    'ring_mod_synth', 'multi_oscillator', 'oscillator_config',
    'membrane_drum', 'feedback_fm', 'pd_synth', 'modal', 'modal_mode',
    'metallic', 'comb_filter_synth',
    'vocoder', 'vocoder_band', 'formant_synth', 'formant_config',
    'vector_synth', 'vector_source', 'vector_path_point',
    'waveguide', 'bowed_string', 'pulsar', 'vosim',
    'spectral_freeze', 'spectral_source', 'pitched_body'
]

filter_funcs = [
    'lowpass', 'highpass', 'bandpass', 'notch', 'allpass',
    'comb_filter', 'formant_filter', 'ladder', 'shelf_low', 'shelf_high'
]

effect_funcs = [
    'reverb', 'delay', 'compressor', 'chorus', 'phaser', 'bitcrush',
    'limiter', 'flanger', 'waveshaper', 'parametric_eq', 'eq_band',
    'stereo_widener', 'delay_tap', 'multi_tap_delay', 'tape_saturation',
    'transient_shaper', 'auto_filter', 'cabinet_sim', 'rotary_speaker',
    'ring_modulator', 'granular_delay'
]

modulation_funcs = ['envelope', 'lfo', 'lfo_modulation', 'pitch_envelope']

layer_funcs = ['audio_layer']

# Find all function starts within the starlark_module block
functions = {}
current_func = None
func_start = None

for i in range(module_start, module_end + 1):
    line = lines[i]

    # Check for function definition
    match = re.match(r'\s+fn (\w+)<', line)
    if match:
        func_name = match.group(1)
        if current_func:
            # Save previous function
            functions[current_func] = (func_start, i - 1)
        current_func = func_name
        func_start = i

# Save last function
if current_func:
    functions[current_func] = (func_start, module_end)

print(f"\nFound {len(functions)} functions")
for name, (start, end) in sorted(functions.items(), key=lambda x: x[1][0]):
    category = 'unknown'
    if name in synthesis_funcs:
        category = 'synthesis'
    elif name in filter_funcs:
        category = 'filters'
    elif name in effect_funcs:
        category = 'effects'
    elif name in modulation_funcs:
        category = 'modulation'
    elif name in layer_funcs:
        category = 'layers'
    print(f"  {name:25s} lines {start+1:4d}-{end+1:4d}  ({end-start+1:4d} lines)  [{category}]")
