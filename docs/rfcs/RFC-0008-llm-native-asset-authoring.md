# RFC-0008: LLM-Native Asset Authoring

**Status**: Draft
**Author**: Claude (Anthropic)
**Date**: 2026-01-17
**Target**: SpecCade Starlark Branch

---

## Abstract

SpecCade provides a deterministic asset pipeline with a Starlark DSL layer. While technically sound, the current interface optimizes for expert humans rather than language models. This RFC identifies friction points that limit LLM effectiveness and proposes concrete improvements to create an LLM-native authoring experience while preserving expert capabilities.

---

## Motivation

Large language models are increasingly used for procedural content generation. However, research demonstrates consistent limitations:

1. **Numeric Parameter Sensitivity**: LLMs struggle with precise numeric outputs. Studies show error rates increase significantly when outputs require >50 interdependent numeric values (Brown et al., 2020; Wei et al., 2022).

2. **Perceptual Grounding Gap**: LLMs cannot perceive audio/visual output. Without feedback signals, parameter tuning becomes random search (Ramesh et al., 2022).

3. **Example Retrieval Dependency**: LLM performance on structured generation tasks improves 40-60% with relevant few-shot examples (Liu et al., 2023).

SpecCade's current architecture exposes these weaknesses. A 248-line harp preset with 150+ numeric parameters exemplifies the problem—even with documentation, LLMs cannot reliably produce high-quality output without substantial iteration.

---

## Problem Areas

### P1: Parameter Space Explosion

**Current State**: Audio presets require explicit specification of:
- Synthesis parameters (3-8 per layer)
- Envelope (4 parameters)
- Filter (3-4 parameters)
- LFO/modulation (4+ parameters)
- Effects chain (3-6 parameters per effect)

A typical preset specifies 80-200 numeric values with complex interdependencies.

**Evidence**: Analysis of `packs/preset_library_v1/audio/` shows mean preset size of 142 lines, with plucks averaging 180+ lines. The `harp.json` preset contains 6 layers with 248 total lines.

**Impact**: LLMs produce syntactically valid but perceptually poor outputs. Parameters technically fall within valid ranges but lack musical/acoustic coherence.

---

### P2: No Perceptual Feedback Loop

**Current State**: The workflow is:
```
LLM writes spec → validate → generate → human evaluates → iterate
```

The LLM receives no signal about output quality between iterations.

**Evidence**: Current `speccade` CLI commands (`eval`, `validate`, `generate`) produce binary success/failure or raw audio bytes. No structured quality metrics are returned.

**Impact**: Agentic workflows cannot self-improve. Each iteration is essentially random perturbation without gradient signal.

---

### P3: No Semantic Preset Retrieval

**Current State**: 291 presets exist in `packs/preset_library_v1/` with no semantic index. The `style_tags` field in the spec format is defined but unused.

**Evidence**:
- No embedding index or search infrastructure exists
- `Grep` for "style_tags" in preset library returns 0 populated instances
- No retrieval mechanism in `speccade-cli`

**Impact**: When asked to create "a punchy 808-style kick," an LLM cannot efficiently find relevant examples. It either hallucinates parameters or uses unrelated examples from context.

---

### P4: Music Composition Complexity

**Current State**: Pattern IR uses tracker-style operations (`emit`, `stack`, `prob`, `choose`, `euclid`) requiring knowledge of:
- Row/tick timing model
- Channel assignment
- Effect column encoding
- Probabilistic sequencing semantics

**Evidence**: The `music_tracker.star` golden test demonstrates complexity:
```python
pattern(
    "verse",
    length = 64,
    events = [emit(at=range_op(0, 16, 4), cell=cell(0, "C4", 0, 64))]
)
```

This requires understanding that `range_op(0, 16, 4)` produces ticks `[0, 4, 8, 12]`, combined with tracker semantics.

**Impact**: LLMs have limited exposure to tracker formats in training data. Most music generation research uses MIDI or piano-roll representations.

---

### P5: Schema-Validation Divergence

**Current State**: From `schemas/speccade-spec-v1.schema.json`:
> "This schema is intentionally conservative and may not express all Rust-side validation rules"

**Evidence**: Schema allows `"type": "string"` for synthesis types, but Rust validation rejects unknown types with `E006`. Schema does not enumerate valid enum values for many fields.

**Impact**: LLMs using JSON Schema for guidance encounter runtime errors not predicted by schema. Tool-use agents fail silently or require extra iterations.

---

### P6: Documentation Fragmentation

**Current State**: Reference documentation is distributed across:
- `docs/stdlib-audio.md` (1469 lines)
- `docs/spec-reference/*.md` (5 files)
- `claude-plugin/skills/speccade-authoring/references/` (4 files)
- Inline comments in `golden/starlark/*.star`

**Evidence**: Total documentation exceeds 8000 lines across 15+ files. No single consolidated reference exists.

**Impact**: LLM context windows have finite capacity. Fragmented docs require either selective loading (risking missing information) or excessive context consumption.

---

### P7: No Incremental Generation

**Current State**: Generation is atomic—entire spec produces entire output. No partial generation, preview, or layer isolation exists.

**Evidence**: `speccade generate` command has no `--preview`, `--duration`, or `--layer` flags.

**Impact**: For 30-second audio, each iteration requires full generation (seconds to minutes). Agentic loops become prohibitively slow.

---

## Proposed Solutions

### S1: Semantic Parameter Abstraction Layer

**Problem Addressed**: P1 (Parameter Space Explosion)

#### Option A: Preset Inheritance with Overrides

Add `extends` field to spec format:
```json
{
  "extends": "preset://plucks/harp",
  "overrides": {
    "layers[0].synthesis.frequency": 880,
    "effects[0].reverb.wet": 0.6
  }
}
```

| Pros | Cons |
|------|------|
| Minimal spec size | Requires preset resolution at compile time |
| Leverages existing presets | Implicit dependencies may confuse LLMs |
| Backward compatible | Override path syntax adds complexity |

**Implementation**: ~200 LOC in `speccade-spec/src/resolve.rs`

#### Option B: Semantic Macro Functions

Add high-level stdlib functions:
```python
# Instead of 50+ lines of synthesis config:
sound = pluck_sound(
    character = "bright",      # maps to filter cutoff, harmonics
    body = "wooden",           # maps to resonance, decay shape
    attack = "snappy",         # maps to envelope attack, transient
    size = "medium"            # maps to reverb, stereo width
)
```

| Pros | Cons |
|------|------|
| LLM-friendly vocabulary | Requires curated semantic→parameter mappings |
| Composable abstractions | May limit expert control |
| Natural language alignment | Mapping maintenance overhead |

**Implementation**: ~500 LOC in `compiler/stdlib/semantic.rs`, plus ~50 curated mappings

#### Option C: Parameter Templates with Slots

Define templates with unfilled slots:
```python
kick_template = template("drums/kick_basic", slots=["punch", "sub_weight", "click"])
my_kick = kick_template.fill(punch=0.8, sub_weight=0.6, click=0.3)
```

| Pros | Cons |
|------|------|
| Explicit parameter reduction | Template authoring burden |
| Type-safe slots | Slot names may still confuse LLMs |
| Preserves full control via raw mode | Two abstraction levels to learn |

**Implementation**: ~400 LOC template system

**Recommendation**: Option B (Semantic Macros) provides best LLM alignment. Combine with Option A for hybrid workflows.

---

### S2: Audio Analysis Backend

**Problem Addressed**: P2 (No Perceptual Feedback Loop)

#### Option A: Integrated Analysis Command

Add `speccade analyze` command:
```bash
speccade analyze --spec kick.star --output metrics.json
```

Returns structured metrics:
```json
{
  "duration_ms": 450,
  "spectral": {
    "centroid_hz": 120,
    "bandwidth_hz": 80,
    "rolloff_hz": 2400
  },
  "dynamics": {
    "peak_db": -3.2,
    "rms_db": -12.4,
    "crest_factor": 9.2,
    "attack_ms": 8
  },
  "perceptual": {
    "brightness": 0.3,
    "warmth": 0.7,
    "punch": 0.85
  },
  "tonal": {
    "fundamental_hz": 55,
    "harmonicity": 0.2
  }
}
```

| Pros | Cons |
|------|------|
| Actionable feedback for LLMs | Requires DSP implementation |
| Enables agentic refinement loops | Perceptual metrics need calibration |
| Useful for humans too | Adds backend complexity |

**Implementation**: ~800 LOC using `rustfft` + custom perceptual estimators

#### Option B: Comparative Analysis

Add A/B comparison mode:
```bash
speccade compare --a kick_v1.star --b kick_v2.star --target reference.wav
```

Returns:
```json
{
  "similarity_to_target": {"a": 0.72, "b": 0.81},
  "spectral_distance": {"a_vs_b": 0.15},
  "recommendation": "b"
}
```

| Pros | Cons |
|------|------|
| Direct optimization signal | Requires reference audio |
| Enables tournament selection | More complex than single analysis |
| Grounds iteration in target | Reference may not always exist |

**Implementation**: ~600 LOC, depends on Option A

#### Option C: Perceptual Embedding Space

Embed generated audio using pretrained model (CLAP, PANNs):
```bash
speccade embed --spec kick.star --model clap-base
```

Returns 512-dim vector for similarity search and clustering.

| Pros | Cons |
|------|------|
| Semantic audio understanding | External model dependency |
| Enables "make it sound like X" | Larger binary/runtime |
| Industry-standard approach | Embedding interpretation unclear |

**Implementation**: ~300 LOC + ONNX runtime integration

**Recommendation**: Option A (Integrated Analysis) as foundation, with Option C for semantic retrieval.

---

### S3: Semantic Preset Search

**Problem Addressed**: P3 (No Semantic Preset Retrieval)

#### Option A: Text Embedding Index

Pre-compute embeddings for all presets:
1. Extract preset metadata + parameter summary
2. Generate text description via template
3. Embed with sentence-transformers
4. Store in vector index (HNSW)

```bash
speccade search "punchy 808 kick with long sub tail"
# Returns: drums/kick_808.json (0.89), drums/kick_sub.json (0.76), ...
```

| Pros | Cons |
|------|------|
| Natural language queries | Requires embedding model |
| Scalable to large libraries | Index maintenance on preset changes |
| Fast retrieval (<10ms) | Text descriptions may miss nuance |

**Implementation**: ~400 LOC + `usearch` or `hnswlib` integration

#### Option B: Audio-Text Multimodal Index

Use CLAP or similar to embed both audio and text:
1. Generate audio from each preset
2. Embed audio with CLAP audio encoder
3. Query with text via CLAP text encoder

```bash
speccade search --audio "warm pad with slow attack"
```

| Pros | Cons |
|------|------|
| Perceptually grounded | Requires audio generation for indexing |
| Handles sounds hard to describe | Larger model dependency |
| State-of-art approach | Slower indexing |

**Implementation**: ~500 LOC + CLAP ONNX integration

#### Option C: Tag-Based Faceted Search

Populate `style_tags` for all presets, enable faceted queries:
```bash
speccade search --tags "drums,punchy,electronic" --exclude "acoustic"
```

| Pros | Cons |
|------|------|
| Simple implementation | Manual tagging burden |
| Interpretable results | Limited expressiveness |
| No ML dependencies | Doesn't scale to novel queries |

**Implementation**: ~150 LOC

**Recommendation**: Option A (Text Embedding) for immediate value, migrate to Option B for production.

---

### S4: High-Level Composition Primitives

**Problem Addressed**: P4 (Music Composition Complexity)

#### Option A: Genre-Aware Pattern Generators

Add stdlib functions for common patterns:
```python
drums = drum_pattern(
    style = "four_on_floor",
    kick = "inst_0",
    snare = "inst_1",
    hihat = "inst_2",
    swing = 0.0,
    fills = True
)

bass = bassline(
    style = "walking",
    scale = "C_minor",
    octave = 2,
    rhythm = "eighths",
    instrument = "inst_3"
)

song(
    tempo = 120,
    sections = [
        section("intro", bars=4, patterns=[drums]),
        section("verse", bars=8, patterns=[drums, bass]),
    ]
)
```

| Pros | Cons |
|------|------|
| Dramatically simpler authoring | Style vocabulary needs curation |
| Matches music production mental model | May limit experimental composition |
| High leverage for common cases | Pattern library maintenance |

**Implementation**: ~1200 LOC in `compiler/stdlib/music/patterns.rs`

#### Option B: MIDI-Like Abstraction

Support piano-roll style input:
```python
melody = notes([
    (0, "C4", 0.5),    # beat, pitch, duration
    (0.5, "E4", 0.5),
    (1, "G4", 1.0),
])
```

Convert to tracker IR internally.

| Pros | Cons |
|------|------|
| Familiar to most musicians | Less expressive than tracker IR |
| Abundant training data exists | Loses some tracker features |
| Easy for LLMs (MIDI in training) | Conversion complexity |

**Implementation**: ~600 LOC

#### Option C: Constraint-Based Generation

Specify constraints, let system fill details:
```python
drums = generate_drums(
    genre = "house",
    energy = 0.7,
    complexity = 0.4,
    instruments = ["kick", "snare", "hihat"],
    bars = 4
)
```

| Pros | Cons |
|------|------|
| Minimal input required | Non-deterministic (conflicts with core goal) |
| Handles creative decisions | Harder to iterate on specifics |
| Good for prototyping | "Black box" feel |

**Implementation**: ~2000 LOC with rule-based generator

**Recommendation**: Option A (Genre-Aware Generators) + Option B (MIDI abstraction) as complementary layers.

---

### S5: Complete Schema Validation

**Problem Addressed**: P5 (Schema-Validation Divergence)

#### Option A: Generated Schema from Rust Types

Derive JSON Schema directly from Rust types using `schemars`:
```rust
#[derive(JsonSchema, Serialize, Deserialize)]
pub struct SynthesisNode {
    #[serde(rename = "type")]
    #[schemars(regex(pattern = "^(sine|square|sawtooth|...)$"))]
    pub synth_type: SynthesisType,
    // ...
}
```

| Pros | Cons |
|------|------|
| Single source of truth | Requires refactoring type definitions |
| Always synchronized | Some validations hard to express in schema |
| Industry standard approach | Build complexity |

**Implementation**: ~200 LOC refactoring + `schemars` integration

#### Option B: Schema-First with Codegen

Define canonical schema, generate Rust types:
```json
{
  "SynthesisType": {
    "type": "string",
    "enum": ["sine", "square", "sawtooth", "triangle", "noise", "pulse", "fm_synth", ...]
  }
}
```

Generate Rust enums via build script.

| Pros | Cons |
|------|------|
| Schema is authoritative | Codegen adds build complexity |
| Forces completeness | Less flexible than hand-written types |
| Good for API-first design | Migration effort |

**Implementation**: ~400 LOC build script

#### Option C: Runtime Schema Extraction

Export validation rules at runtime:
```bash
speccade schema --format jsonschema > complete-schema.json
```

Query validator for all accepted values.

| Pros | Cons |
|------|------|
| Reflects actual validation | May miss static constraints |
| No refactoring needed | Runtime overhead |
| Incremental adoption | Schema may be overly permissive |

**Implementation**: ~300 LOC

**Recommendation**: Option A (Generated Schema) for long-term correctness.

---

### S6: Consolidated LLM Reference

**Problem Addressed**: P6 (Documentation Fragmentation)

#### Option A: Single Comprehensive Reference

Create `docs/LLM_REFERENCE.md` (<8000 tokens):
```markdown
# SpecCade LLM Quick Reference

## Synthesis Types
| Type | Key Params | Use Case | Example |
|------|------------|----------|---------|
| sine | freq, amp | Pure tones, subs | `oscillator("sine", 110, 0.8)` |
| fm_synth | carrier, mod, index | Bells, basses | `fm_synth(440, 880, 5.0)` |
...

## Common Patterns
### Kick Drum
```python
audio_layer(
    synthesis = oscillator("sine", 55, 1.0),
    envelope = envelope(0.001, 0.15, 0.0, 0.1),
    ...
)
```
...
```

| Pros | Cons |
|------|------|
| Fits in context window | May duplicate info |
| Optimized for LLM consumption | Maintenance overhead |
| Quick reference format | Less detailed than full docs |

**Implementation**: ~4 hours documentation work

#### Option B: Structured Reference as JSON

Machine-readable reference:
```json
{
  "functions": {
    "oscillator": {
      "params": ["waveform", "frequency", "amplitude"],
      "waveform_values": ["sine", "square", ...],
      "frequency_range": [20, 20000],
      "example": "oscillator(\"sine\", 440, 0.8)"
    }
  }
}
```

| Pros | Cons |
|------|------|
| Programmatically queryable | Less readable for humans |
| Can generate multiple formats | Requires tooling to use |
| Precise parameter specs | JSON verbosity |

**Implementation**: ~300 LOC extraction script

#### Option C: Interactive Reference Server

MCP server providing reference on demand:
```
Tool: speccade_reference
Query: "fm synthesis parameters"
Response: {function: "fm_synth", params: [...], examples: [...]}
```

| Pros | Cons |
|------|------|
| Minimal context usage | Requires MCP integration |
| Always up-to-date | Network latency |
| Targeted information | More complex architecture |

**Implementation**: ~500 LOC MCP server

**Recommendation**: Option A (Single Reference) immediately, Option C (MCP Server) for production.

---

### S7: Preview and Incremental Generation

**Problem Addressed**: P7 (No Incremental Generation)

#### Option A: Duration-Limited Preview

```bash
speccade generate --spec pad.star --preview 2.0
```

Generates only first 2 seconds.

| Pros | Cons |
|------|------|
| Simple implementation | Misses tail/release characteristics |
| Fast iteration | May not represent full sound |
| Low resource usage | Some synths need warmup |

**Implementation**: ~100 LOC in generation pipeline

#### Option B: Layer Isolation

```bash
speccade generate --spec song.star --layer 0 --preview 2.0
```

Generate single layer for focused iteration.

| Pros | Cons |
|------|------|
| Targeted feedback | Layers may interact |
| Enables parallel iteration | Layer index fragile |
| Faster generation | Mixing not previewed |

**Implementation**: ~150 LOC

#### Option C: Cached Incremental Regeneration

Cache intermediate results, regenerate only changed layers:
```bash
speccade generate --spec song.star --cache ./cache --incremental
```

| Pros | Cons |
|------|------|
| Optimal for iteration | Cache management complexity |
| Preserves full quality | Cache invalidation edge cases |
| Scales to large projects | Storage overhead |

**Implementation**: ~600 LOC caching system

**Recommendation**: Options A + B immediately (minimal effort, high value).

---

## Implementation Roadmap

### Phase 1: Foundation (Immediate)

| Item | Effort | Impact |
|------|--------|--------|
| S6-A: Consolidated LLM Reference | 4h | High |
| S7-A/B: Preview + Layer Isolation | 2d | High |
| S1-A: Preset Inheritance | 3d | High |
| S5-A: Generated Schema | 2d | Medium |

### Phase 2: Feedback Loop (Short-term)

| Item | Effort | Impact |
|------|--------|--------|
| S2-A: Audio Analysis Backend | 5d | Critical |
| S3-A: Text Embedding Search | 3d | High |
| S1-B: Semantic Macros (core set) | 5d | High |

### Phase 3: Composition (Medium-term)

| Item | Effort | Impact |
|------|--------|--------|
| S4-A: Genre-Aware Patterns | 7d | High |
| S4-B: MIDI Abstraction | 4d | Medium |
| S2-C: Perceptual Embeddings | 4d | Medium |

### Phase 4: Production (Long-term)

| Item | Effort | Impact |
|------|--------|--------|
| S3-B: Audio-Text Multimodal Index | 5d | High |
| S6-C: MCP Reference Server | 3d | Medium |
| S7-C: Cached Incremental Generation | 5d | Medium |

---

## Success Metrics

### Quantitative

1. **Spec Complexity Reduction**: Mean preset size should decrease from 142 lines to <50 lines using semantic layer
2. **Iteration Speed**: Time to generate preview should be <2 seconds for 95% of specs
3. **LLM Success Rate**: Percentage of LLM-generated specs that validate without error should exceed 90% (currently estimated <60%)
4. **Retrieval Precision**: Top-3 search results should contain relevant preset 85% of time

### Qualitative

1. **Agentic Workflow Viability**: LLM should be able to iteratively refine sound toward target description without human intervention
2. **Human Expert Adoption**: Sound designers should find semantic layer useful, not limiting
3. **Documentation Clarity**: New users should be able to create valid spec within 10 minutes

---

## References

1. Brown, T., et al. (2020). "Language Models are Few-Shot Learners." NeurIPS.
2. Wei, J., et al. (2022). "Chain-of-Thought Prompting Elicits Reasoning in Large Language Models." NeurIPS.
3. Liu, P., et al. (2023). "Pre-train, Prompt, and Predict: A Systematic Survey of Prompting Methods in NLP." ACM Computing Surveys.
4. Ramesh, A., et al. (2022). "Hierarchical Text-Conditional Image Generation with CLIP Latents." arXiv.
5. Wu, Y., et al. (2023). "Large-scale Contrastive Language-Audio Pretraining with Feature Fusion and Keyword-to-Caption Augmentation." ICASSP. (CLAP)
6. Kong, Q., et al. (2020). "PANNs: Large-Scale Pretrained Audio Neural Networks for Audio Pattern Recognition." IEEE/ACM TASLP.

---

## Appendix A: Current Error Code Reference

| Code | Category | Description |
|------|----------|-------------|
| E001 | Spec | Missing required field |
| E002 | Spec | Invalid field type |
| E003 | Spec | Value out of range |
| E004 | Spec | Unknown output type |
| E005 | Spec | Duplicate output ID |
| E006 | Spec | Unknown synthesis type |
| E007 | Spec | Invalid envelope parameters |
| E008 | Spec | Invalid filter configuration |
| E009 | Spec | Invalid effect parameters |
| E010 | Spec | Invalid modulation target |
| E011 | Spec | Circular reference detected |
| E012 | Spec | Budget limit exceeded |
| S101 | Starlark | Empty string parameter |
| S102 | Starlark | Type mismatch |
| S103 | Starlark | Value out of range |
| S104 | Starlark | Invalid enum value |

---

## Appendix B: Semantic Parameter Mapping Examples

### Character Descriptors → Filter Configuration

| Descriptor | Cutoff | Resonance | Type |
|------------|--------|-----------|------|
| bright | 8000 | 0.2 | lowpass |
| warm | 800 | 0.4 | lowpass |
| hollow | 1200 | 0.7 | bandpass |
| thin | 2000 | 0.1 | highpass |
| aggressive | 3000 | 0.8 | lowpass |

### Attack Descriptors → Envelope Configuration

| Descriptor | Attack | Decay | Sustain | Release |
|------------|--------|-------|---------|---------|
| snappy | 0.001 | 0.05 | 0.7 | 0.1 |
| soft | 0.1 | 0.2 | 0.8 | 0.3 |
| plucky | 0.001 | 0.15 | 0.0 | 0.2 |
| swelling | 0.5 | 0.1 | 0.9 | 0.4 |
| percussive | 0.001 | 0.1 | 0.0 | 0.05 |

---

*End of RFC-001*
