# SpecCade Starlark Standard Library - Mesh Functions

[← Back to Index](stdlib-reference.md)

## Mesh Functions

Mesh functions provide primitives and modifiers for static mesh generation.

## Table of Contents
- [Primitive Functions](#primitive-functions)
- [Modifier Functions](#modifier-functions)

---

## Primitive Functions

### mesh_primitive()

Creates a base mesh primitive specification.

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| primitive | str | Yes | - | "cube", "sphere", "cylinder", "cone", "torus", "plane", "ico_sphere" |
| dimensions | list | Yes | - | [x, y, z] dimensions |

**Returns:** Dict with base_primitive and dimensions.

**Example:**
```python
mesh_primitive("cube", [1.0, 1.0, 1.0])
mesh_primitive("sphere", [2.0, 2.0, 2.0])
```

### mesh_recipe()

Creates a complete static mesh recipe params.

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| primitive | str | Yes | - | Primitive type |
| dimensions | list | Yes | - | [x, y, z] dimensions |
| modifiers | list | No | None | Optional modifiers |

**Returns:** Dict matching StaticMeshBlenderPrimitivesV1Params.

**Example:**
```python
mesh_recipe("cube", [1.0, 1.0, 1.0])
mesh_recipe(
    "cube",
    [1.0, 1.0, 1.0],
    [bevel_modifier(0.02, 2), subdivision_modifier(2)]
)
```

---

## Modifier Functions

### bevel_modifier()

Creates a bevel modifier.

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| width | f64 | No | 0.02 | Bevel width |
| segments | int | No | 2 | Number of segments |
| angle_limit | f64 | No | None | Angle limit in degrees (only bevel edges below this angle) |

**Returns:** Dict matching the MeshModifier::Bevel IR structure.

**Example:**
```python
bevel_modifier()
bevel_modifier(0.05, 3)
bevel_modifier(0.02, 2, 30.0)  # Only bevel edges with angle < 30 degrees
```

### subdivision_modifier()

Creates a subdivision surface modifier.

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| levels | int | No | 2 | Subdivision levels |
| render_levels | int | No | None | Render levels (defaults to levels) |

**Returns:** Dict matching the MeshModifier::Subdivision IR structure.

**Example:**
```python
subdivision_modifier()
subdivision_modifier(3)
subdivision_modifier(2, 4)
```

### decimate_modifier()

Creates a decimate modifier.

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| ratio | f64 | No | 0.5 | Decimation ratio (0.0-1.0) |

**Returns:** Dict matching the MeshModifier::Decimate IR structure.

**Example:**
```python
decimate_modifier()
decimate_modifier(0.25)
```

### edge_split_modifier()

Creates an edge split modifier.

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| angle | f64 | Yes | - | Split angle in degrees (edges sharper than this will be split) |

**Returns:** Dict matching the MeshModifier::EdgeSplit IR structure.

**Example:**
```python
edge_split_modifier(30.0)
```

### mirror_modifier()

Creates a mirror modifier.

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| axis_x | bool | No | True | Mirror along X axis |
| axis_y | bool | No | False | Mirror along Y axis |
| axis_z | bool | No | False | Mirror along Z axis |

**Returns:** Dict matching the MeshModifier::Mirror IR structure.

**Example:**
```python
mirror_modifier()  # Mirror on X axis only
mirror_modifier(True, True, False)  # Mirror on X and Y axes
```

### array_modifier()

Creates an array modifier.

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| count | int | Yes | - | Number of copies |
| offset | list | Yes | - | Offset between copies [x, y, z] |

**Returns:** Dict matching the MeshModifier::Array IR structure.

**Example:**
```python
array_modifier(5, [1.0, 0.0, 0.0])  # 5 copies along X axis
array_modifier(10, [0.0, 2.0, 0.0])  # 10 copies along Y axis
```

### solidify_modifier()

Creates a solidify modifier.

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| thickness | f64 | Yes | - | Thickness to add |
| offset | f64 | No | 0.0 | Offset (-1.0 to 1.0) |

**Returns:** Dict matching the MeshModifier::Solidify IR structure.

**Example:**
```python
solidify_modifier(0.1)  # Add 0.1 thickness centered
solidify_modifier(0.1, -1.0)  # Add thickness inward
solidify_modifier(0.1, 1.0)  # Add thickness outward
```

---

[← Back to Index](stdlib-reference.md)
