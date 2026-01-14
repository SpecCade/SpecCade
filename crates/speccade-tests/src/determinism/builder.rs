//! Builder pattern for custom determinism tests.

use crate::determinism::core::{verify_determinism, DeterminismResult};

/// Builder for custom determinism tests with explicit generation functions.
///
/// Use this when you need to test asset types that don't have a simple
/// `generate(&spec)` function, or when you need custom generation logic.
///
/// # Example
///
/// ```rust,ignore
/// use speccade_tests::determinism::DeterminismBuilder;
/// use speccade_backend_texture::generate_material_maps;
///
/// let result = DeterminismBuilder::new()
///     .runs(5)
///     .generate(|| {
///         let params = create_texture_params();
///         generate_material_maps(&params, 42)
///             .maps.get(&TextureMapType::Albedo)
///             .map(|m| m.png_data.clone())
///             .unwrap_or_default()
///     })
///     .verify();
/// ```
pub struct DeterminismBuilder<F, O>
where
    F: Fn() -> O,
    O: AsRef<[u8]>,
{
    runs: usize,
    generator: Option<F>,
    _phantom: std::marker::PhantomData<O>,
}

impl<F, O> DeterminismBuilder<F, O>
where
    F: Fn() -> O,
    O: AsRef<[u8]>,
{
    /// Create a new builder with default settings.
    pub fn new() -> Self {
        Self {
            runs: 3,
            generator: None,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Set the number of runs.
    pub fn runs(mut self, runs: usize) -> Self {
        self.runs = runs;
        self
    }

    /// Set the generation function.
    pub fn generate(mut self, f: F) -> Self {
        self.generator = Some(f);
        self
    }

    /// Verify determinism and return the result.
    ///
    /// # Panics
    /// Panics if no generator was set.
    pub fn verify(self) -> DeterminismResult {
        let generator = self
            .generator
            .expect("No generator set - call .generate() first");
        verify_determinism(generator, self.runs)
    }

    /// Verify determinism and panic on failure.
    ///
    /// # Panics
    /// Panics if no generator was set or if output is non-deterministic.
    pub fn assert(self) {
        self.verify().assert_deterministic();
    }
}

impl<F, O> Default for DeterminismBuilder<F, O>
where
    F: Fn() -> O,
    O: AsRef<[u8]>,
{
    fn default() -> Self {
        Self::new()
    }
}
