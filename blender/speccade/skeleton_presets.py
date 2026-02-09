"""
SpecCade Skeleton Presets Module

This module defines standard skeleton preset configurations for humanoid characters.
Each preset specifies bone positions, orientations, and hierarchy.

Available presets:
- humanoid_connected_v1: 20 bones, no fingers, ALL bones connected (parent tail = child head)

Note: Presets with offset bones (where parent tail != child head) have been removed
to prevent mesh gaps and skinning issues.
"""

# Humanoid connected v1 skeleton definition (20 bones, no fingers, ALL bones connected)
# All parent tail positions = child head positions (no gaps in bone chain)
# Shoulders connect from chest center, legs connect from hips center
HUMANOID_CONNECTED_V1_BONES = {
    "root": {"head": (0, 0, 0), "tail": (0, 0, 0.1), "parent": None},
    "hips": {"head": (0, 0, 0.9), "tail": (0, 0, 1.0), "parent": "root"},
    "spine": {"head": (0, 0, 1.0), "tail": (0, 0, 1.2), "parent": "hips"},
    "chest": {"head": (0, 0, 1.2), "tail": (0, 0, 1.4), "parent": "spine"},
    "neck": {"head": (0, 0, 1.4), "tail": (0, 0, 1.5), "parent": "chest"},
    "head": {"head": (0, 0, 1.5), "tail": (0, 0, 1.7), "parent": "neck"},
    # Left arm - shoulder starts at chest tail (0, 0, 1.4) and extends outward
    "shoulder_l": {"head": (0, 0, 1.4), "tail": (0.2, 0, 1.35), "parent": "chest"},
    "upper_arm_l": {"head": (0.2, 0, 1.35), "tail": (0.45, 0, 1.35), "parent": "shoulder_l"},
    "lower_arm_l": {"head": (0.45, 0, 1.35), "tail": (0.7, 0, 1.35), "parent": "upper_arm_l"},
    "hand_l": {"head": (0.7, 0, 1.35), "tail": (0.8, 0, 1.35), "parent": "lower_arm_l"},
    # Right arm - shoulder starts at chest tail (0, 0, 1.4) and extends outward
    "shoulder_r": {"head": (0, 0, 1.4), "tail": (-0.2, 0, 1.35), "parent": "chest"},
    "upper_arm_r": {"head": (-0.2, 0, 1.35), "tail": (-0.45, 0, 1.35), "parent": "shoulder_r"},
    "lower_arm_r": {"head": (-0.45, 0, 1.35), "tail": (-0.7, 0, 1.35), "parent": "upper_arm_r"},
    "hand_r": {"head": (-0.7, 0, 1.35), "tail": (-0.8, 0, 1.35), "parent": "lower_arm_r"},
    # Left leg - starts at hips tail (0, 0, 1.0) and extends downward/outward
    "upper_leg_l": {"head": (0, 0, 1.0), "tail": (0.1, 0, 0.5), "parent": "hips"},
    "lower_leg_l": {"head": (0.1, 0, 0.5), "tail": (0.1, 0, 0.1), "parent": "upper_leg_l"},
    "foot_l": {"head": (0.1, 0, 0.1), "tail": (0.1, 0.15, 0), "parent": "lower_leg_l"},
    # Right leg - starts at hips tail (0, 0, 1.0) and extends downward/outward
    "upper_leg_r": {"head": (0, 0, 1.0), "tail": (-0.1, 0, 0.5), "parent": "hips"},
    "lower_leg_r": {"head": (-0.1, 0, 0.5), "tail": (-0.1, 0, 0.1), "parent": "upper_leg_r"},
    "foot_r": {"head": (-0.1, 0, 0.1), "tail": (-0.1, 0.15, 0), "parent": "lower_leg_r"},
}

# Map preset names to their bone definitions
SKELETON_PRESETS = {
    "humanoid_connected_v1": HUMANOID_CONNECTED_V1_BONES,
    # Aliases for backward compatibility with animation helpers_v1
    "humanoid": HUMANOID_CONNECTED_V1_BONES,
    "humanoid_basic_v1": HUMANOID_CONNECTED_V1_BONES,
}
