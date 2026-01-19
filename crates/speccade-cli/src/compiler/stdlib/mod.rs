//! Starlark stdlib module for SpecCade.
//!
//! This module provides helper functions that emit canonical IR-compatible structures,
//! reducing boilerplate and improving authoring ergonomics for Starlark specs.
//!
//! ## Design Principles
//!
//! 1. **Flat, explicit parameters** - No magic defaults; LLMs understand keyword args well
//! 2. **Composable** - Functions return dicts that can be modified or combined
//! 3. **Deterministic** - No random, time, or IO functions
//! 4. **Minimal** - Core functions covering 80% of use cases
//! 5. **Domain-prefixed** - `audio_*`, `texture_*`, `mesh_*` to avoid conflicts
//!
//! ## Function Categories
//!
//! - **Core**: `spec()`, `output()` - Spec scaffolding
//! - **Audio**: `envelope()`, `oscillator()`, `fm_synth()`, etc. - Audio synthesis
//! - **Texture**: `noise_node()`, `gradient_node()`, etc. - Texture generation
//! - **Mesh**: `mesh_primitive()`, `mesh_recipe()` - Mesh generation

pub mod audio;
pub mod character;
pub mod core;
pub mod mesh;
pub mod music;
pub mod texture;
mod validation;

use starlark::environment::GlobalsBuilder;

/// Registers all SpecCade stdlib functions into a GlobalsBuilder.
///
/// This should be called when building the Starlark evaluation environment
/// to make all stdlib functions available.
///
/// # Example
///
/// ```ignore
/// use starlark::environment::GlobalsBuilder;
/// use speccade_cli::compiler::stdlib::register_stdlib;
///
/// let globals = GlobalsBuilder::standard()
///     .with(register_stdlib)
///     .build();
/// ```
pub fn register_stdlib(builder: &mut GlobalsBuilder) {
    core::register(builder);
    audio::register(builder);
    music::register(builder);
    texture::register(builder);
    mesh::register(builder);
    character::register(builder);
}

#[cfg(test)]
mod tests {
    use super::*;
    use starlark::environment::Module;
    use starlark::eval::Evaluator;
    use starlark::syntax::{AstModule, Dialect};

    /// Helper to evaluate a Starlark expression with stdlib and convert to JSON.
    pub fn eval_to_json(source: &str) -> Result<serde_json::Value, String> {
        let ast = AstModule::parse("test.star", source.to_string(), &Dialect::Standard)
            .map_err(|e| e.to_string())?;
        let module = Module::new();
        let globals = GlobalsBuilder::standard().with(register_stdlib).build();
        let mut eval = Evaluator::new(&module);
        let value = eval.eval_module(ast, &globals).map_err(|e| e.to_string())?;
        crate::compiler::convert::starlark_to_json(value).map_err(|e| e.to_string())
    }

    #[test]
    fn test_stdlib_registered() {
        // Test that stdlib functions are available
        let result = eval_to_json("envelope()");
        assert!(
            result.is_ok(),
            "envelope() should be available: {:?}",
            result
        );

        let result = eval_to_json("oscillator(440)");
        assert!(
            result.is_ok(),
            "oscillator() should be available: {:?}",
            result
        );

        let result = eval_to_json("output(\"test.wav\", \"wav\")");
        assert!(result.is_ok(), "output() should be available: {:?}", result);
    }
}
