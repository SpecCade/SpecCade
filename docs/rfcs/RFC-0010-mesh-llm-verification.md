# RFC-0010: LLM Verification for Mesh and Character Assets

**Status:** Accepted
**Created:** 2026-01-17
**Finalized:** 2026-01-24
**Author:** Claude
**Related:** RFC-0008 (LLM-Native Asset Authoring)

## Abstract

This RFC proposes a verification system that enables LLMs to assess mesh and character assets through multiple feedback signals: vision-language model (VLM) analysis, geometric metrics, and constraint validation. This closes the feedback loop for LLM-generated 3D assets.

## Background: Existing Character System

SpecCade already includes a comprehensive character modeling and rigging system:

| Component | Implementation |
|-----------|----------------|
| Skeleton Presets | `HumanoidBasicV1` (20 bones), extensible |
| IK Presets | `HumanoidLegs`, `HumanoidArms`, `QuadrupedForelegs`, `QuadrupedHindlegs`, `Tentacle`, `Tail` |
| Rig Setup | IK chains, FK fallback, IK/FK switching |
| Foot Systems | Ground contact, roll, toe pivot |
| Aim Constraints | Look-at targets for heads, eyes |
| Twist Bones | Forearm/upper arm twist distribution |
| Stretch | Squash-and-stretch for cartoon rigs |

This RFC focuses on verification and feedback mechanisms, not on extending the rigging feature set.

## Motivation

When an LLM generates a mesh or character spec, it cannot currently verify:
- Does the mesh look like what was intended?
- Are proportions correct?
- Does the rig deform properly?
- Are topology and UV unwrap suitable for the use case?

Without verification, LLM-generated 3D assets require human review for every iteration, negating the automation benefits.

## Goals

1. Provide structured feedback LLMs can interpret
2. Support iterative refinement without human review
3. Detect common mesh/rig problems automatically
4. Integrate with existing character/rigging system

## Non-Goals

- Replacing human artistic direction
- Photorealistic rendering quality assessment
- Real-time game engine integration

## VLM Integration Policy (Finalized 2026-01-24)

VLM (vision-language model) integration is **experimental and opt-in** with the following constraints:

### Access Control

- **Off by default**: VLM analysis is disabled unless explicitly requested
- **Runtime credentials only**: User provides API key via `--vlm-key` flag (not persisted to disk)
- **No implicit uploads**: Only rendered images are uploaded, never mesh data or specs

### Latency and Caching

- **Batch mode only**: VLM verification is a manual trigger, not part of hot-reload
- **Expected latency**: 10-30 seconds per verification request
- **Progress indicator**: Show progress during VLM call
- **Default timeout**: 30 seconds (configurable via `--vlm-timeout`)
- **Caching**: Results cached by `spec_hash + render_settings_hash`
  - Cache location: `~/.cache/speccade/verification/`
  - Invalidation: spec change, render settings change, VLM model change
  - `--no-cache` flag to force re-verification

### Hallucination Guardrails

- **Transparent output**: Raw VLM response included in verification report
- **Geometric ground truth**: VLM cannot override deterministic geometric metrics
- **Single prompt v1**: Start simple, add ensemble prompts if hallucination becomes a problem
- **Confidence field**: Report includes confidence field for future ensemble support

### CLI Interface

```bash
# Basic VLM verification
speccade verify --spec character.star --vlm-key $ANTHROPIC_API_KEY

# With options
speccade verify --spec character.star \
    --vlm-key $ANTHROPIC_API_KEY \
    --vlm-timeout 45 \
    --no-cache
```

## Design

### Multi-Signal Verification

The verification system combines three signal types:

```
┌────────────────────────────────────────────────────────────┐
│                    Verification Pipeline                   │
├────────────────────────────────────────────────────────────┤
│                                                            │
│   Mesh/Character Spec                                      │
│          │                                                 │
│          ▼                                                 │
│   ┌──────────────┐                                         │
│   │   Generate   │                                         │
│   │  Mesh/Rig    │                                         │
│   └──────┬───────┘                                         │
│          │                                                 │
│          ├─────────────────┬─────────────────┐             │
│          ▼                 ▼                 ▼             │
│   ┌──────────────┐  ┌──────────────┐  ┌──────────────┐    │
│   │  VLM Render  │  │  Geometric   │  │  Constraint  │    │
│   │   Analysis   │  │   Metrics    │  │  Validation  │    │
│   └──────┬───────┘  └──────┬───────┘  └──────┬───────┘    │
│          │                 │                 │             │
│          └─────────────────┼─────────────────┘             │
│                            ▼                               │
│                   ┌────────────────┐                       │
│                   │  Verification  │                       │
│                   │     Report     │                       │
│                   └────────────────┘                       │
│                                                            │
└────────────────────────────────────────────────────────────┘
```

### Signal 1: VLM Render Analysis

Render the mesh from multiple viewpoints and analyze with a vision-language model:

```json
{
  "vlm_analysis": {
    "views": ["front", "side", "top", "perspective"],
    "render_style": "clay",
    "prompts": [
      "Does this mesh match the description: '{description}'?",
      "List any anatomical or proportional issues.",
      "Rate the overall quality 1-10 with explanation."
    ],
    "model": "claude-3-opus"
  }
}
```

**View Configuration:**
| View | Camera Position | Use Case |
|------|-----------------|----------|
| Front | (0, 0, 2) | Symmetry, face, silhouette |
| Side | (2, 0, 0) | Profile, depth, posture |
| Top | (0, 2, 0) | Plan view, layout |
| 3/4 | (1.5, 1, 1.5) | Overall impression |
| Detail | Parametric | Specific regions |

**Render Styles:**
- `clay`: Uniform gray, focus on form
- `wireframe`: Topology visibility
- `uv_checker`: UV distortion detection
- `normals`: Surface direction visualization
- `weight_paint`: Rig influence visualization

### Signal 2: Geometric Metrics

Compute objective metrics without VLM:

```json
{
  "geometric_metrics": {
    "poly_count": 12450,
    "quad_percentage": 94.2,
    "edge_loops": {
      "valid": true,
      "issues": []
    },
    "manifold": {
      "is_manifold": true,
      "non_manifold_edges": 0
    },
    "symmetry": {
      "deviation": 0.001,
      "axis": "x"
    },
    "bounds": {
      "size": [1.8, 2.0, 0.5],
      "center": [0, 1.0, 0]
    },
    "uv_coverage": 0.87,
    "uv_overlap_percentage": 0.0,
    "degenerate_faces": 0
  }
}
```

**Character-Specific Metrics:**

```json
{
  "rig_metrics": {
    "bone_count": 20,
    "ik_chains": ["left_leg", "right_leg", "left_arm", "right_arm"],
    "weight_distribution": {
      "max_influences_per_vertex": 4,
      "unweighted_vertices": 0
    },
    "deformation_test": {
      "poses_tested": ["t_pose", "a_pose", "crouch", "arm_raise"],
      "volume_preservation": 0.98,
      "self_intersection": false
    }
  }
}
```

### Signal 3: Constraint Validation

User-defined constraints that must pass:

```starlark
mesh(
    name = "hero_character",
    # ... mesh definition ...

    constraints = [
        max_poly_count(15000),
        min_quad_percentage(90),
        require_manifold(),
        symmetry_tolerance(0.01),

        # Character-specific
        require_ik_chains(["legs", "arms"]),
        max_bone_influences(4),
        require_deformation_test(["t_pose", "crouch"]),
    ],
)
```

Constraint failures produce actionable messages:

```json
{
  "constraint_results": [
    {
      "constraint": "max_poly_count(15000)",
      "passed": false,
      "actual": 18234,
      "suggestion": "Reduce subdivision level or simplify detail meshes"
    },
    {
      "constraint": "require_manifold()",
      "passed": true
    }
  ]
}
```

### Verification Report Format

Combined report structure that LLMs can parse and act on:

```json
{
  "verification_report": {
    "asset_name": "hero_character",
    "asset_type": "character",
    "overall_pass": false,

    "vlm_analysis": {
      "match_score": 0.85,
      "issues": [
        "Arms appear slightly too long relative to torso",
        "Hands lack finger definition"
      ],
      "quality_score": 7
    },

    "geometric_metrics": {
      "summary": "Good topology, minor UV issues",
      "warnings": ["UV island for left hand has 12% stretch"]
    },

    "constraint_results": {
      "passed": 8,
      "failed": 1,
      "failures": [
        {
          "constraint": "max_poly_count(15000)",
          "actual": 18234,
          "suggestion": "Reduce subdivision level"
        }
      ]
    },

    "suggested_actions": [
      "Shorten arm bones by 5%",
      "Add finger geometry or use preset hand_detailed",
      "Reduce body subdivision from 3 to 2"
    ]
  }
}
```

### Integration with Existing Rig System

The verification system integrates with existing rig components:

**IK Chain Verification:**
```starlark
# Existing IK preset
rig_setup(
    ik_chains = [
        ik_chain(preset = IkPreset.HumanoidLegs),
        ik_chain(preset = IkPreset.HumanoidArms),
    ],
    # Verification automatically tests:
    # - IK solve convergence
    # - Pole vector behavior
    # - IK/FK switch blending
)
```

**Deformation Testing:**
- Automatic pose battery using skeleton preset bind pose
- Volume preservation during IK solve
- Self-intersection detection in extreme poses
- Twist bone behavior verification

### Iterative Refinement Loop

LLMs can use verification reports to self-correct:

```
1. LLM generates character spec
2. Backend generates mesh + rig
3. Verification pipeline runs
4. Report returned to LLM
5. LLM reads failures and suggestions
6. LLM modifies spec
7. Repeat until all constraints pass or iteration limit
```

Example refinement prompt:
```
The character verification report shows:
- Constraint failed: max_poly_count(15000), actual: 18234
- VLM issue: "Arms appear slightly too long"

Please modify the spec to:
1. Reduce subdivision level to meet poly budget
2. Adjust arm_length parameter to 0.95 (from 1.0)
```

## Tracking

All implementation work and open questions for this RFC are tracked in `docs/ROADMAP.md` under **Mesh/Character Verification Loop (RFC-0010)**.

## Alternatives Considered

### Embedding-Based Similarity
- Compare mesh embedding to reference embeddings
- Pros: Fast, no VLM API calls
- Cons: Requires training data, less interpretable
- Decision: VLM provides more actionable feedback

### Rendering to Single Image
- One rendered view instead of multiple
- Pros: Simpler, cheaper
- Cons: Misses issues visible from other angles
- Decision: Multi-view worth the cost for 3D assets

### Human-in-the-Loop Only
- Always require human approval
- Pros: Highest quality control
- Cons: Defeats automation purpose
- Decision: Automated verification with optional human override

## Security Considerations

- VLM API calls use user credentials
- Rendered images processed locally before upload
- No mesh data sent to external services (only renders)
- Rate limiting on verification requests

## References

- [SpecCade Skeleton System](../crates/speccade-spec/src/recipe/character/skeleton.rs)
- [SpecCade IK Chains](../crates/speccade-spec/src/recipe/animation/skeletal/ik_chain.rs)
- [SpecCade Rig Setup](../crates/speccade-spec/src/recipe/animation/skeletal/rig_setup.rs)
- [Claude Vision API](https://docs.anthropic.com/claude/docs/vision)
- [OpenSCAD](https://openscad.org/) - Declarative 3D modeling reference
- [CadQuery](https://cadquery.readthedocs.io/) - Python CAD scripting reference
