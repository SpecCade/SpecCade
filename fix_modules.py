#!/usr/bin/env python3
"""Script to fix audio submodules by extracting complete functions."""

# Read the original file
with open('crates/speccade-cli/src/compiler/stdlib/audio.rs', 'r', encoding='utf-8') as f:
    lines = f.readlines()

# Common header for all modules
header_template = '''//! {description}

use starlark::collections::SmallMap;
use starlark::environment::GlobalsBuilder;
use starlark::starlark_module;
use starlark::values::{{dict::Dict, none::NoneType, Heap, Value, ValueLike}};

use super::super::validation::{{
    validate_enum, validate_pan_range, validate_positive, validate_unit_range,
}};

/// Helper to create a hashed key for dict insertion.
fn hashed_key<'v>(heap: &'v Heap, key: &str) -> starlark::collections::Hashed<Value<'v>> {{
    heap.alloc_str(key)
        .to_value()
        .get_hashed()
        .expect("string hashing cannot fail")
}}

/// Helper to create an empty dict on the heap.
fn new_dict<'v>(_heap: &'v Heap) -> Dict<'v> {{
    let map: SmallMap<Value<'v>, Value<'v>> = SmallMap::new();
    Dict::new(map)
}}

/// Helper function to extract a float from a Starlark Value.
fn extract_float(value: Value, function: &str, param: &str) -> anyhow::Result<f64> {{
    if let Some(f) = value.unpack_i32() {{
        return Ok(f as f64);
    }}
    if value.get_type() == "float" {{
        if let Ok(f) = value.to_str().parse::<f64>() {{
            return Ok(f);
        }}
    }}
    Err(anyhow::anyhow!(
        "S102: {{}}(): '{{}}' expected float, got {{}}",
        function, param, value.get_type()
    ))
}}

/// Valid waveform types.
const WAVEFORMS: &[&str] = &["sine", "square", "sawtooth", "triangle", "pulse"];

/// Valid noise types.
const NOISE_TYPES: &[&str] = &["white", "pink", "brown"];

/// Valid sweep curves.
const SWEEP_CURVES: &[&str] = &["linear", "exponential", "logarithmic"];

/// Registers {module_name} functions into a GlobalsBuilder.
pub fn register(builder: &mut GlobalsBuilder) {{
    register_{module_name}_functions(builder);
}}

#[starlark_module]
fn register_{module_name}_functions(builder: &mut GlobalsBuilder) {{
'''

# Categorize functions by name (from the previous analysis)
function_categories = {
    'envelope': 'modulation',
    'lfo': 'modulation',
    'lfo_modulation': 'modulation',
    'pitch_envelope': 'modulation',

    'oscillator': 'synthesis',
    'fm_synth': 'synthesis',
    'am_synth': 'synthesis',
    'karplus_strong': 'synthesis',
    'noise_burst': 'synthesis',
    'additive': 'synthesis',
    'supersaw_unison': 'synthesis',
    'wavetable': 'synthesis',
    'granular': 'synthesis',
    'granular_source': 'synthesis',
    'ring_mod_synth': 'synthesis',
    'multi_oscillator': 'synthesis',
    'oscillator_config': 'synthesis',
    'membrane_drum': 'synthesis',
    'feedback_fm': 'synthesis',
    'pd_synth': 'synthesis',
    'modal': 'synthesis',
    'modal_mode': 'synthesis',
    'metallic': 'synthesis',
    'comb_filter_synth': 'synthesis',
    'vocoder': 'synthesis',
    'vocoder_band': 'synthesis',
    'formant_synth': 'synthesis',
    'formant_config': 'synthesis',
    'vector_synth': 'synthesis',
    'vector_source': 'synthesis',
    'vector_path_point': 'synthesis',
    'waveguide': 'synthesis',
    'bowed_string': 'synthesis',
    'pulsar': 'synthesis',
    'vosim': 'synthesis',
    'spectral_freeze': 'synthesis',
    'spectral_source': 'synthesis',
    'pitched_body': 'synthesis',

    'lowpass': 'filters',
    'highpass': 'filters',
    'bandpass': 'filters',
    'notch': 'filters',
    'allpass': 'filters',
    'comb_filter': 'filters',
    'formant_filter': 'filters',
    'ladder': 'filters',
    'shelf_low': 'filters',
    'shelf_high': 'filters',

    'reverb': 'effects',
    'delay': 'effects',
    'compressor': 'effects',
    'chorus': 'effects',
    'phaser': 'effects',
    'bitcrush': 'effects',
    'limiter': 'effects',
    'flanger': 'effects',
    'waveshaper': 'effects',
    'parametric_eq': 'effects',
    'eq_band': 'effects',
    'stereo_widener': 'effects',
    'delay_tap': 'effects',
    'multi_tap_delay': 'effects',
    'tape_saturation': 'effects',
    'transient_shaper': 'effects',
    'auto_filter': 'effects',
    'cabinet_sim': 'effects',
    'rotary_speaker': 'effects',
    'ring_modulator': 'effects',
    'granular_delay': 'effects',

    'audio_layer': 'layers',
}

# Find the starlark_module block
module_start = 43  # Line 44 in 1-indexed
module_end = 4135  # Line 4136 in 1-indexed (before closing })

# Extract functions by finding `fn function_name` patterns
# We'll scan through and find complete function blocks
function_blocks = {cat: [] for cat in ['modulation', 'synthesis', 'filters', 'effects', 'layers']}

i = module_start
while i <= module_end:
    line = lines[i]

    # Look for function definition
    import re
    match = re.search(r'^\s+fn (\w+)<', line)
    if match:
        func_name = match.group(1)
        if func_name in function_categories:
            category = function_categories[func_name]

            # Find the start of this function's documentation (scan backwards)
            doc_start = i
            j = i - 1
            while j >= module_start:
                stripped = lines[j].strip()
                if stripped.startswith('///') or stripped == '':
                    doc_start = j
                    j -= 1
                elif stripped.startswith('//'):
                    # Section comment, include it
                    doc_start = j
                    j -= 1
                else:
                    break

            # Find the end of this function (scan forward for closing brace at same indent level)
            # Count braces to find matching close
            brace_count = 0
            func_end = i
            for k in range(i, module_end + 1):
                line_content = lines[k]
                # Count braces (simple approach - doesn't handle strings/comments perfectly)
                brace_count += line_content.count('{')
                brace_count -= line_content.count('}')
                if brace_count == 0 and k > i:
                    func_end = k
                    break

            # Store the function block
            function_blocks[category].append((doc_start, func_end + 1))

    i += 1

# Print statistics
print("Function blocks found:")
for cat, blocks in function_blocks.items():
    print(f"  {cat}: {len(blocks)} functions")

# Generate each module file
modules = {
    'modulation': 'Modulation functions (envelopes and LFOs)',
    'synthesis': 'Synthesis functions for audio generation',
    'filters': 'Filter functions for audio processing',
    'effects': 'Effect functions for audio processing',
    'layers': 'Audio layer composition functions',
}

for module_name, description in modules.items():
    output_lines = []

    # Add header
    output_lines.append(header_template.format(description=description, module_name=module_name))

    # Add all function blocks for this category
    for start, end in function_blocks[module_name]:
        output_lines.extend(lines[start:end])
        output_lines.append('\n')  # Add spacing between functions

    # Close the starlark_module block
    output_lines.append('}\n')

    # Write the file
    filepath = f'crates/speccade-cli/src/compiler/stdlib/audio/{module_name}.rs'
    with open(filepath, 'w', encoding='utf-8') as f:
        f.writelines(output_lines)

    line_count = len(output_lines)
    print(f'Regenerated {filepath} ({line_count} lines)')

print('\nDone!')
