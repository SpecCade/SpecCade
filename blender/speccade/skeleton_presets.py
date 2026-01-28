"""
SpecCade Skeleton Presets Module

This module defines standard skeleton preset configurations for humanoid characters.
Each preset specifies bone positions, orientations, and hierarchy.

Available presets:
- humanoid_basic_v1: 20 bones, no fingers (minimal rig)
- humanoid_detailed_v1: 52 bones, full 3-bone fingers (high-detail rig)
- humanoid_game_v1: 40 bones, twist bones, simplified fingers (game-ready rig)
"""

# Humanoid basic v1 skeleton definition (20 bones, no fingers)
HUMANOID_BASIC_V1_BONES = {
    "root": {"head": (0, 0, 0), "tail": (0, 0, 0.1), "parent": None},
    "hips": {"head": (0, 0, 0.9), "tail": (0, 0, 1.0), "parent": "root"},
    "spine": {"head": (0, 0, 1.0), "tail": (0, 0, 1.2), "parent": "hips"},
    "chest": {"head": (0, 0, 1.2), "tail": (0, 0, 1.4), "parent": "spine"},
    "neck": {"head": (0, 0, 1.4), "tail": (0, 0, 1.5), "parent": "chest"},
    "head": {"head": (0, 0, 1.5), "tail": (0, 0, 1.7), "parent": "neck"},
    "shoulder_l": {"head": (0.1, 0, 1.35), "tail": (0.2, 0, 1.35), "parent": "chest"},
    "upper_arm_l": {"head": (0.2, 0, 1.35), "tail": (0.45, 0, 1.35), "parent": "shoulder_l"},
    "lower_arm_l": {"head": (0.45, 0, 1.35), "tail": (0.7, 0, 1.35), "parent": "upper_arm_l"},
    "hand_l": {"head": (0.7, 0, 1.35), "tail": (0.8, 0, 1.35), "parent": "lower_arm_l"},
    "shoulder_r": {"head": (-0.1, 0, 1.35), "tail": (-0.2, 0, 1.35), "parent": "chest"},
    "upper_arm_r": {"head": (-0.2, 0, 1.35), "tail": (-0.45, 0, 1.35), "parent": "shoulder_r"},
    "lower_arm_r": {"head": (-0.45, 0, 1.35), "tail": (-0.7, 0, 1.35), "parent": "upper_arm_r"},
    "hand_r": {"head": (-0.7, 0, 1.35), "tail": (-0.8, 0, 1.35), "parent": "lower_arm_r"},
    "upper_leg_l": {"head": (0.1, 0, 0.9), "tail": (0.1, 0, 0.5), "parent": "hips"},
    "lower_leg_l": {"head": (0.1, 0, 0.5), "tail": (0.1, 0, 0.1), "parent": "upper_leg_l"},
    "foot_l": {"head": (0.1, 0, 0.1), "tail": (0.1, 0.15, 0), "parent": "lower_leg_l"},
    "upper_leg_r": {"head": (-0.1, 0, 0.9), "tail": (-0.1, 0, 0.5), "parent": "hips"},
    "lower_leg_r": {"head": (-0.1, 0, 0.5), "tail": (-0.1, 0, 0.1), "parent": "upper_leg_r"},
    "foot_r": {"head": (-0.1, 0, 0.1), "tail": (-0.1, 0.15, 0), "parent": "lower_leg_r"},
}

# Humanoid detailed v1 skeleton definition (52 bones, full 3-bone fingers)
# Roll values set so positive X rotation = flexion (anatomically correct)
HUMANOID_DETAILED_V1_BONES = {
    # Core
    "root": {"head": (0, 0, 0), "tail": (0, 0, 0.1), "parent": None, "roll": 0},
    "hips": {"head": (0, 0, 0.9), "tail": (0, 0, 1.0), "parent": "root", "roll": 0},
    "spine": {"head": (0, 0, 1.0), "tail": (0, 0, 1.2), "parent": "hips", "roll": 0},
    "chest": {"head": (0, 0, 1.2), "tail": (0, 0, 1.4), "parent": "spine", "roll": 0},
    "neck": {"head": (0, 0, 1.4), "tail": (0, 0, 1.5), "parent": "chest", "roll": 0},
    "head": {"head": (0, 0, 1.5), "tail": (0, 0, 1.7), "parent": "neck", "roll": 0},
    # Left arm
    "shoulder_l": {"head": (0.1, 0, 1.35), "tail": (0.2, 0, 1.35), "parent": "chest", "roll": 0},
    "upper_arm_l": {"head": (0.2, 0, 1.35), "tail": (0.45, 0, 1.35), "parent": "shoulder_l", "roll": 0},
    "lower_arm_l": {"head": (0.45, 0, 1.35), "tail": (0.7, 0, 1.35), "parent": "upper_arm_l", "roll": 0},
    "hand_l": {"head": (0.7, 0, 1.35), "tail": (0.78, 0, 1.35), "parent": "lower_arm_l", "roll": 0},
    # Left thumb (offset forward in Y, rotated)
    "thumb_01_l": {"head": (0.72, 0.02, 1.34), "tail": (0.75, 0.04, 1.32), "parent": "hand_l", "roll": 45},
    "thumb_02_l": {"head": (0.75, 0.04, 1.32), "tail": (0.78, 0.05, 1.30), "parent": "thumb_01_l", "roll": 45},
    "thumb_03_l": {"head": (0.78, 0.05, 1.30), "tail": (0.80, 0.06, 1.29), "parent": "thumb_02_l", "roll": 45},
    # Left index finger
    "index_01_l": {"head": (0.78, 0.02, 1.36), "tail": (0.82, 0.02, 1.36), "parent": "hand_l", "roll": 0},
    "index_02_l": {"head": (0.82, 0.02, 1.36), "tail": (0.85, 0.02, 1.36), "parent": "index_01_l", "roll": 0},
    "index_03_l": {"head": (0.85, 0.02, 1.36), "tail": (0.87, 0.02, 1.36), "parent": "index_02_l", "roll": 0},
    # Left middle finger
    "middle_01_l": {"head": (0.78, 0, 1.35), "tail": (0.83, 0, 1.35), "parent": "hand_l", "roll": 0},
    "middle_02_l": {"head": (0.83, 0, 1.35), "tail": (0.87, 0, 1.35), "parent": "middle_01_l", "roll": 0},
    "middle_03_l": {"head": (0.87, 0, 1.35), "tail": (0.89, 0, 1.35), "parent": "middle_02_l", "roll": 0},
    # Left ring finger
    "ring_01_l": {"head": (0.78, -0.02, 1.34), "tail": (0.82, -0.02, 1.34), "parent": "hand_l", "roll": 0},
    "ring_02_l": {"head": (0.82, -0.02, 1.34), "tail": (0.85, -0.02, 1.34), "parent": "ring_01_l", "roll": 0},
    "ring_03_l": {"head": (0.85, -0.02, 1.34), "tail": (0.87, -0.02, 1.34), "parent": "ring_02_l", "roll": 0},
    # Left pinky finger
    "pinky_01_l": {"head": (0.78, -0.04, 1.33), "tail": (0.81, -0.04, 1.33), "parent": "hand_l", "roll": 0},
    "pinky_02_l": {"head": (0.81, -0.04, 1.33), "tail": (0.83, -0.04, 1.33), "parent": "pinky_01_l", "roll": 0},
    "pinky_03_l": {"head": (0.83, -0.04, 1.33), "tail": (0.85, -0.04, 1.33), "parent": "pinky_02_l", "roll": 0},
    # Right arm
    "shoulder_r": {"head": (-0.1, 0, 1.35), "tail": (-0.2, 0, 1.35), "parent": "chest", "roll": 0},
    "upper_arm_r": {"head": (-0.2, 0, 1.35), "tail": (-0.45, 0, 1.35), "parent": "shoulder_r", "roll": 0},
    "lower_arm_r": {"head": (-0.45, 0, 1.35), "tail": (-0.7, 0, 1.35), "parent": "upper_arm_r", "roll": 0},
    "hand_r": {"head": (-0.7, 0, 1.35), "tail": (-0.78, 0, 1.35), "parent": "lower_arm_r", "roll": 0},
    # Right thumb (mirrored)
    "thumb_01_r": {"head": (-0.72, 0.02, 1.34), "tail": (-0.75, 0.04, 1.32), "parent": "hand_r", "roll": -45},
    "thumb_02_r": {"head": (-0.75, 0.04, 1.32), "tail": (-0.78, 0.05, 1.30), "parent": "thumb_01_r", "roll": -45},
    "thumb_03_r": {"head": (-0.78, 0.05, 1.30), "tail": (-0.80, 0.06, 1.29), "parent": "thumb_02_r", "roll": -45},
    # Right index finger
    "index_01_r": {"head": (-0.78, 0.02, 1.36), "tail": (-0.82, 0.02, 1.36), "parent": "hand_r", "roll": 180},
    "index_02_r": {"head": (-0.82, 0.02, 1.36), "tail": (-0.85, 0.02, 1.36), "parent": "index_01_r", "roll": 180},
    "index_03_r": {"head": (-0.85, 0.02, 1.36), "tail": (-0.87, 0.02, 1.36), "parent": "index_02_r", "roll": 180},
    # Right middle finger
    "middle_01_r": {"head": (-0.78, 0, 1.35), "tail": (-0.83, 0, 1.35), "parent": "hand_r", "roll": 180},
    "middle_02_r": {"head": (-0.83, 0, 1.35), "tail": (-0.87, 0, 1.35), "parent": "middle_01_r", "roll": 180},
    "middle_03_r": {"head": (-0.87, 0, 1.35), "tail": (-0.89, 0, 1.35), "parent": "middle_02_r", "roll": 180},
    # Right ring finger
    "ring_01_r": {"head": (-0.78, -0.02, 1.34), "tail": (-0.82, -0.02, 1.34), "parent": "hand_r", "roll": 180},
    "ring_02_r": {"head": (-0.82, -0.02, 1.34), "tail": (-0.85, -0.02, 1.34), "parent": "ring_01_r", "roll": 180},
    "ring_03_r": {"head": (-0.85, -0.02, 1.34), "tail": (-0.87, -0.02, 1.34), "parent": "ring_02_r", "roll": 180},
    # Right pinky finger
    "pinky_01_r": {"head": (-0.78, -0.04, 1.33), "tail": (-0.81, -0.04, 1.33), "parent": "hand_r", "roll": 180},
    "pinky_02_r": {"head": (-0.81, -0.04, 1.33), "tail": (-0.83, -0.04, 1.33), "parent": "pinky_01_r", "roll": 180},
    "pinky_03_r": {"head": (-0.83, -0.04, 1.33), "tail": (-0.85, -0.04, 1.33), "parent": "pinky_02_r", "roll": 180},
    # Left leg
    "upper_leg_l": {"head": (0.1, 0, 0.9), "tail": (0.1, 0, 0.5), "parent": "hips", "roll": 0},
    "lower_leg_l": {"head": (0.1, 0, 0.5), "tail": (0.1, 0, 0.1), "parent": "upper_leg_l", "roll": 0},
    "foot_l": {"head": (0.1, 0, 0.1), "tail": (0.1, 0.12, 0.02), "parent": "lower_leg_l", "roll": 0},
    "toe_l": {"head": (0.1, 0.12, 0.02), "tail": (0.1, 0.18, 0), "parent": "foot_l", "roll": 0},
    # Right leg
    "upper_leg_r": {"head": (-0.1, 0, 0.9), "tail": (-0.1, 0, 0.5), "parent": "hips", "roll": 0},
    "lower_leg_r": {"head": (-0.1, 0, 0.5), "tail": (-0.1, 0, 0.1), "parent": "upper_leg_r", "roll": 0},
    "foot_r": {"head": (-0.1, 0, 0.1), "tail": (-0.1, 0.12, 0.02), "parent": "lower_leg_r", "roll": 0},
    "toe_r": {"head": (-0.1, 0.12, 0.02), "tail": (-0.1, 0.18, 0), "parent": "foot_r", "roll": 0},
}

# Humanoid game v1 skeleton definition (40 bones, twist bones, simplified fingers)
HUMANOID_GAME_V1_BONES = {
    # Core
    "root": {"head": (0, 0, 0), "tail": (0, 0, 0.1), "parent": None, "roll": 0},
    "hips": {"head": (0, 0, 0.9), "tail": (0, 0, 1.0), "parent": "root", "roll": 0},
    "spine": {"head": (0, 0, 1.0), "tail": (0, 0, 1.2), "parent": "hips", "roll": 0},
    "chest": {"head": (0, 0, 1.2), "tail": (0, 0, 1.4), "parent": "spine", "roll": 0},
    "neck": {"head": (0, 0, 1.4), "tail": (0, 0, 1.5), "parent": "chest", "roll": 0},
    "head": {"head": (0, 0, 1.5), "tail": (0, 0, 1.7), "parent": "neck", "roll": 0},
    # Left arm with twist bones
    "shoulder_l": {"head": (0.1, 0, 1.35), "tail": (0.2, 0, 1.35), "parent": "chest", "roll": 0},
    "upper_arm_l": {"head": (0.2, 0, 1.35), "tail": (0.45, 0, 1.35), "parent": "shoulder_l", "roll": 0},
    "upper_arm_twist_l": {"head": (0.35, 0, 1.35), "tail": (0.45, 0, 1.35), "parent": "upper_arm_l", "roll": 0},
    "lower_arm_l": {"head": (0.45, 0, 1.35), "tail": (0.7, 0, 1.35), "parent": "upper_arm_l", "roll": 0},
    "lower_arm_twist_l": {"head": (0.6, 0, 1.35), "tail": (0.7, 0, 1.35), "parent": "lower_arm_l", "roll": 0},
    "hand_l": {"head": (0.7, 0, 1.35), "tail": (0.78, 0, 1.35), "parent": "lower_arm_l", "roll": 0},
    # Left hand (simplified 1-bone fingers)
    "thumb_l": {"head": (0.72, 0.02, 1.34), "tail": (0.78, 0.05, 1.30), "parent": "hand_l", "roll": 45},
    "index_l": {"head": (0.78, 0.02, 1.36), "tail": (0.86, 0.02, 1.36), "parent": "hand_l", "roll": 0},
    "middle_l": {"head": (0.78, 0, 1.35), "tail": (0.88, 0, 1.35), "parent": "hand_l", "roll": 0},
    "ring_l": {"head": (0.78, -0.02, 1.34), "tail": (0.86, -0.02, 1.34), "parent": "hand_l", "roll": 0},
    "pinky_l": {"head": (0.78, -0.04, 1.33), "tail": (0.84, -0.04, 1.33), "parent": "hand_l", "roll": 0},
    # Right arm with twist bones
    "shoulder_r": {"head": (-0.1, 0, 1.35), "tail": (-0.2, 0, 1.35), "parent": "chest", "roll": 0},
    "upper_arm_r": {"head": (-0.2, 0, 1.35), "tail": (-0.45, 0, 1.35), "parent": "shoulder_r", "roll": 0},
    "upper_arm_twist_r": {"head": (-0.35, 0, 1.35), "tail": (-0.45, 0, 1.35), "parent": "upper_arm_r", "roll": 0},
    "lower_arm_r": {"head": (-0.45, 0, 1.35), "tail": (-0.7, 0, 1.35), "parent": "upper_arm_r", "roll": 0},
    "lower_arm_twist_r": {"head": (-0.6, 0, 1.35), "tail": (-0.7, 0, 1.35), "parent": "lower_arm_r", "roll": 0},
    "hand_r": {"head": (-0.7, 0, 1.35), "tail": (-0.78, 0, 1.35), "parent": "lower_arm_r", "roll": 0},
    # Right hand (simplified 1-bone fingers)
    "thumb_r": {"head": (-0.72, 0.02, 1.34), "tail": (-0.78, 0.05, 1.30), "parent": "hand_r", "roll": -45},
    "index_r": {"head": (-0.78, 0.02, 1.36), "tail": (-0.86, 0.02, 1.36), "parent": "hand_r", "roll": 180},
    "middle_r": {"head": (-0.78, 0, 1.35), "tail": (-0.88, 0, 1.35), "parent": "hand_r", "roll": 180},
    "ring_r": {"head": (-0.78, -0.02, 1.34), "tail": (-0.86, -0.02, 1.34), "parent": "hand_r", "roll": 180},
    "pinky_r": {"head": (-0.78, -0.04, 1.33), "tail": (-0.84, -0.04, 1.33), "parent": "hand_r", "roll": 180},
    # Left leg with twist bones
    "upper_leg_l": {"head": (0.1, 0, 0.9), "tail": (0.1, 0, 0.5), "parent": "hips", "roll": 0},
    "upper_leg_twist_l": {"head": (0.1, 0, 0.65), "tail": (0.1, 0, 0.5), "parent": "upper_leg_l", "roll": 0},
    "lower_leg_l": {"head": (0.1, 0, 0.5), "tail": (0.1, 0, 0.1), "parent": "upper_leg_l", "roll": 0},
    "lower_leg_twist_l": {"head": (0.1, 0, 0.25), "tail": (0.1, 0, 0.1), "parent": "lower_leg_l", "roll": 0},
    "foot_l": {"head": (0.1, 0, 0.1), "tail": (0.1, 0.12, 0.02), "parent": "lower_leg_l", "roll": 0},
    "toe_l": {"head": (0.1, 0.12, 0.02), "tail": (0.1, 0.18, 0), "parent": "foot_l", "roll": 0},
    # Right leg with twist bones
    "upper_leg_r": {"head": (-0.1, 0, 0.9), "tail": (-0.1, 0, 0.5), "parent": "hips", "roll": 0},
    "upper_leg_twist_r": {"head": (-0.1, 0, 0.65), "tail": (-0.1, 0, 0.5), "parent": "upper_leg_r", "roll": 0},
    "lower_leg_r": {"head": (-0.1, 0, 0.5), "tail": (-0.1, 0, 0.1), "parent": "upper_leg_r", "roll": 0},
    "lower_leg_twist_r": {"head": (-0.1, 0, 0.25), "tail": (-0.1, 0, 0.1), "parent": "lower_leg_r", "roll": 0},
    "foot_r": {"head": (-0.1, 0, 0.1), "tail": (-0.1, 0.12, 0.02), "parent": "lower_leg_r", "roll": 0},
    "toe_r": {"head": (-0.1, 0.12, 0.02), "tail": (-0.1, 0.18, 0), "parent": "foot_r", "roll": 0},
}

# Map preset names to their bone definitions
SKELETON_PRESETS = {
    "humanoid_basic_v1": HUMANOID_BASIC_V1_BONES,
    "humanoid_detailed_v1": HUMANOID_DETAILED_V1_BONES,
    "humanoid_game_v1": HUMANOID_GAME_V1_BONES,
}
