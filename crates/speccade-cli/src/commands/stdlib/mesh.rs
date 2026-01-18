//! Mesh primitive and modifier functions

use super::{func, param, FunctionInfo};

pub(super) fn register_functions() -> Vec<FunctionInfo> {
    vec![
        func!(
            "mesh_primitive",
            "mesh",
            "Creates a base mesh primitive.",
            vec![
                param!("primitive_type", "string", req, enum: &["cube", "sphere", "cylinder", "cone", "torus", "plane"]),
                param!("size", "list", opt, [1.0, 1.0, 1.0]),
            ],
            "A primitive dict.",
            "mesh_primitive(\"cube\", [2.0, 2.0, 2.0])"
        ),
        func!(
            "mesh_recipe",
            "mesh",
            "Creates a complete mesh recipe.",
            vec![
                param!("primitive_type", "string", req, enum: &["cube", "sphere", "cylinder", "cone", "torus", "plane"]),
                param!("size", "list", opt, [1.0, 1.0, 1.0]),
                param!("modifiers", "list", opt_none),
            ],
            "A mesh recipe dict.",
            "mesh_recipe(\"cube\", [2.0, 2.0, 2.0], [bevel_modifier(0.1, 3)])"
        ),
        func!(
            "bevel_modifier",
            "mesh.modifiers",
            "Creates a bevel modifier configuration.",
            vec![
                param!("width", "float", req, range: Some(0.0), None),
                param!("segments", "int", opt, 1, range: Some(1.0), None),
            ],
            "A modifier dict.",
            "bevel_modifier(0.1, 3)"
        ),
        func!(
            "subdivision_modifier",
            "mesh.modifiers",
            "Creates a subdivision surface modifier.",
            vec![param!("levels", "int", opt, 1, range: Some(1.0), Some(6.0))],
            "A modifier dict.",
            "subdivision_modifier(2)"
        ),
    ]
}
