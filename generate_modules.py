#!/usr/bin/env python3
"""Script to split audio.rs into submodules."""

import os

# Read the original file
with open('crates/speccade-cli/src/compiler/stdlib/audio.rs', 'r', encoding='utf-8') as f:
    lines = f.readlines()

# Common header for all modules
header = '''//! {description}

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

# Line numbers are 0-indexed in Python but 1-indexed in the grep output
# So we subtract 1 from the line numbers
functions = {
    'synthesis': [(125, 215), (216, 288), (925, 995), (996, 1058), (1059, 1131),
                   (1132, 1231), (1232, 1301), (1302, 1388), (1389, 1453), (1454, 1508),
                   (1509, 1570), (1571, 1619), (1620, 1684), (1685, 1755), (1756, 1817),
                   (1818, 1854), (1855, 1897), (1898, 1944), (3324, 3414), (3415, 3462),
                   (3463, 3544), (3545, 3601), (3602, 3682), (3683, 3719), (3720, 3770),
                   (3771, 3837), (3838, 3897), (3898, 3957), (3958, 4011), (4012, 4042),
                   (4043, 4108), (4109, 4135)],
    'filters': [(385, 439), (440, 504), (1945, 1989), (1990, 2034), (2035, 2079),
                (2080, 2120), (2121, 2155), (2156, 2199), (2200, 2230), (2231, 2267)],
    'effects': [(619, 675), (676, 733), (734, 798), (2268, 2317), (2318, 2365),
                (2366, 2402), (2403, 2456), (2457, 2514), (2515, 2555), (2556, 2580),
                (2581, 2629), (2697, 2758), (2759, 2831), (2832, 2870), (2871, 2955),
                (2956, 3011), (3012, 3087), (3088, 3132), (3133, 3186), (3187, 3236),
                (3237, 3323)],
    'modulation': [(62, 124), (799, 857), (858, 924), (2630, 2696)],
    'layers': [(505, 618)],
}

modules = {
    'synthesis': 'Synthesis functions for audio generation',
    'filters': 'Filter functions for audio processing',
    'effects': 'Effect functions for audio processing',
    'modulation': 'Modulation functions (envelopes and LFOs)',
    'layers': 'Audio layer composition functions',
}

# Generate each module
os.makedirs('crates/speccade-cli/src/compiler/stdlib/audio', exist_ok=True)

for module_name, description in modules.items():
    output_lines = []

    # Add header
    output_lines.append(header.format(description=description, module_name=module_name))

    # Add functions
    for start, end in functions[module_name]:
        output_lines.extend(lines[start:end+1])

    # Close the starlark_module block
    output_lines.append('}\n')

    # Write the file
    filepath = f'crates/speccade-cli/src/compiler/stdlib/audio/{module_name}.rs'
    with open(filepath, 'w', encoding='utf-8') as f:
        f.writelines(output_lines)

    line_count = len(output_lines)
    print(f'Created {filepath} ({line_count} lines)')

# Create mod.rs
mod_rs_content = '''//! Audio stdlib functions for synthesis and effects.
//!
//! Provides helper functions for creating audio synthesis layers, envelopes,
//! oscillators, filters, and effects.

use starlark::environment::GlobalsBuilder;

mod modulation;
mod synthesis;
mod filters;
mod effects;
mod layers;

/// Registers audio stdlib functions into a GlobalsBuilder.
pub fn register(builder: &mut GlobalsBuilder) {
    modulation::register(builder);
    synthesis::register(builder);
    filters::register(builder);
    effects::register(builder);
    layers::register(builder);
}
'''

with open('crates/speccade-cli/src/compiler/stdlib/audio/mod.rs', 'w', encoding='utf-8') as f:
    f.write(mod_rs_content)

print(f'Created crates/speccade-cli/src/compiler/stdlib/audio/mod.rs ({len(mod_rs_content.split(chr(10)))} lines)')
print('\nDone! All modules created.')
