//! Tests for VFX recipe types.

use super::*;

#[test]
fn test_flipbook_params_new() {
    let params = VfxFlipbookV1Params::new(512, 512, FlipbookEffectType::Explosion);
    assert_eq!(params.resolution, [512, 512]);
    assert_eq!(params.padding, 2);
    assert_eq!(params.effect, FlipbookEffectType::Explosion);
    assert_eq!(params.frame_count, 16);
    assert_eq!(params.fps, 24);
    assert_eq!(params.loop_mode, FlipbookLoopMode::Once);
}

#[test]
fn test_flipbook_params_builder() {
    let params = VfxFlipbookV1Params::new(512, 512, FlipbookEffectType::Smoke)
        .with_padding(4)
        .with_frame_count(32)
        .with_frame_size(128, 128)
        .with_fps(30)
        .with_loop_mode(FlipbookLoopMode::Loop);

    assert_eq!(params.padding, 4);
    assert_eq!(params.frame_count, 32);
    assert_eq!(params.frame_size, [128, 128]);
    assert_eq!(params.fps, 30);
    assert_eq!(params.loop_mode, FlipbookLoopMode::Loop);
}

#[test]
fn test_compute_total_duration_ms() {
    let params = VfxFlipbookV1Params::new(512, 512, FlipbookEffectType::Energy)
        .with_frame_count(24)
        .with_fps(24);

    assert_eq!(params.compute_total_duration_ms(), 1000); // 24 frames at 24 fps = 1 second

    let params2 = VfxFlipbookV1Params::new(512, 512, FlipbookEffectType::Dissolve)
        .with_frame_count(30)
        .with_fps(60);

    assert_eq!(params2.compute_total_duration_ms(), 500); // 30 frames at 60 fps = 0.5 seconds
}

#[test]
fn test_effect_type_serde() {
    let effect = FlipbookEffectType::Explosion;
    let json = serde_json::to_string(&effect).unwrap();
    assert_eq!(json, "\"explosion\"");

    let parsed: FlipbookEffectType = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed, effect);
}

#[test]
fn test_loop_mode_serde() {
    let mode = FlipbookLoopMode::PingPong;
    let json = serde_json::to_string(&mode).unwrap();
    assert_eq!(json, "\"ping_pong\"");

    let parsed: FlipbookLoopMode = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed, mode);
}

#[test]
fn test_params_serde_roundtrip() {
    let params = VfxFlipbookV1Params::new(1024, 1024, FlipbookEffectType::Smoke)
        .with_frame_count(48)
        .with_frame_size(96, 96)
        .with_fps(30)
        .with_loop_mode(FlipbookLoopMode::Loop);

    let json = serde_json::to_string_pretty(&params).unwrap();
    let parsed: VfxFlipbookV1Params = serde_json::from_str(&json).unwrap();

    assert_eq!(parsed, params);
}

#[test]
fn test_metadata_creation() {
    let params = VfxFlipbookV1Params::new(512, 512, FlipbookEffectType::Energy)
        .with_frame_count(8)
        .with_fps(16);

    let frame_uvs = vec![
        FlipbookFrameUv {
            index: 0,
            u_min: 0.0,
            v_min: 0.0,
            u_max: 0.5,
            v_max: 0.5,
            width: 64,
            height: 64,
        },
        FlipbookFrameUv {
            index: 1,
            u_min: 0.5,
            v_min: 0.0,
            u_max: 1.0,
            v_max: 0.5,
            width: 64,
            height: 64,
        },
    ];

    let metadata = params.to_metadata(frame_uvs.clone());

    assert_eq!(metadata.atlas_width, 512);
    assert_eq!(metadata.atlas_height, 512);
    assert_eq!(metadata.effect, FlipbookEffectType::Energy);
    assert_eq!(metadata.frame_count, 8);
    assert_eq!(metadata.fps, 16);
    assert_eq!(metadata.total_duration_ms, 500); // 8 frames at 16 fps
    assert_eq!(metadata.frames.len(), 2);
}

#[test]
fn test_effect_type_display() {
    assert_eq!(FlipbookEffectType::Explosion.to_string(), "explosion");
    assert_eq!(FlipbookEffectType::Smoke.to_string(), "smoke");
    assert_eq!(FlipbookEffectType::Energy.to_string(), "energy");
    assert_eq!(FlipbookEffectType::Dissolve.to_string(), "dissolve");
}

#[test]
fn test_loop_mode_display() {
    assert_eq!(FlipbookLoopMode::Once.to_string(), "once");
    assert_eq!(FlipbookLoopMode::Loop.to_string(), "loop");
    assert_eq!(FlipbookLoopMode::PingPong.to_string(), "ping_pong");
}
