//! Tests for sprite recipe types.

use super::*;

mod sheet_tests {
    use super::*;

    #[test]
    fn sprite_sheet_params_roundtrip() {
        let params = SpriteSheetV1Params {
            resolution: [512, 512],
            padding: 2,
            frames: vec![
                SpriteFrame {
                    id: "idle_0".to_string(),
                    width: 64,
                    height: 64,
                    pivot: [0.5, 0.0],
                    source: SpriteFrameSource::Color {
                        color: [0.5, 0.5, 0.5, 1.0],
                    },
                },
                SpriteFrame {
                    id: "idle_1".to_string(),
                    width: 64,
                    height: 64,
                    pivot: [0.5, 0.0],
                    source: SpriteFrameSource::Color {
                        color: [0.6, 0.6, 0.6, 1.0],
                    },
                },
            ],
        };

        let json = serde_json::to_string_pretty(&params).unwrap();
        let parsed: SpriteSheetV1Params = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, params);
    }

    #[test]
    fn sprite_sheet_params_from_json() {
        let json = r#"
        {
          "resolution": [512, 512],
          "padding": 2,
          "frames": [
            { "id": "idle_0", "width": 64, "height": 64, "color": [0.5, 0.5, 0.5, 1.0] },
            { "id": "idle_1", "width": 64, "height": 64, "pivot": [0.5, 0.0], "color": [0.6, 0.6, 0.6, 1.0] }
          ]
        }
        "#;

        let params: SpriteSheetV1Params = serde_json::from_str(json).unwrap();
        assert_eq!(params.resolution, [512, 512]);
        assert_eq!(params.padding, 2);
        assert_eq!(params.frames.len(), 2);
        assert_eq!(params.frames[0].id, "idle_0");
        assert_eq!(params.frames[0].width, 64);
        // First frame should have default pivot
        assert_eq!(params.frames[0].pivot, [0.5, 0.5]);
        // Second frame has explicit pivot
        assert_eq!(params.frames[1].pivot, [0.5, 0.0]);
    }

    #[test]
    fn sprite_sheet_default_padding() {
        let json = r#"
        {
          "resolution": [256, 256],
          "frames": []
        }
        "#;

        let params: SpriteSheetV1Params = serde_json::from_str(json).unwrap();
        assert_eq!(params.padding, 2);
    }

    #[test]
    fn sprite_frame_builder() {
        let frame = SpriteFrame::solid("test", 32, 32, [1.0, 0.0, 0.0, 1.0]).with_pivot([0.0, 1.0]);

        assert_eq!(frame.id, "test");
        assert_eq!(frame.width, 32);
        assert_eq!(frame.height, 32);
        assert_eq!(frame.pivot, [0.0, 1.0]);
        assert!(matches!(
            frame.source,
            SpriteFrameSource::Color { color } if color == [1.0, 0.0, 0.0, 1.0]
        ));
    }

    #[test]
    fn sprite_sheet_builder() {
        let params = SpriteSheetV1Params::new(256, 256)
            .with_padding(4)
            .with_frame(SpriteFrame::solid("a", 32, 32, [1.0, 0.0, 0.0, 1.0]))
            .with_frame(SpriteFrame::solid("b", 64, 64, [0.0, 1.0, 0.0, 1.0]));

        assert_eq!(params.resolution, [256, 256]);
        assert_eq!(params.padding, 4);
        assert_eq!(params.frames.len(), 2);
    }

    #[test]
    fn sprite_frame_uv_serialization() {
        let uv = SpriteFrameUv {
            id: "test".to_string(),
            u_min: 0.0,
            v_min: 0.0,
            u_max: 0.25,
            v_max: 0.25,
            width: 64,
            height: 64,
            pivot: [0.5, 0.5],
        };

        let json = serde_json::to_string(&uv).unwrap();
        let parsed: SpriteFrameUv = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, uv);
    }

    #[test]
    fn sprite_sheet_metadata_serialization() {
        let metadata = SpriteSheetMetadata {
            atlas_width: 512,
            atlas_height: 512,
            padding: 2,
            frames: vec![SpriteFrameUv {
                id: "idle_0".to_string(),
                u_min: 0.0,
                v_min: 0.0,
                u_max: 0.125,
                v_max: 0.125,
                width: 64,
                height: 64,
                pivot: [0.5, 0.0],
            }],
        };

        let json = serde_json::to_string_pretty(&metadata).unwrap();
        let parsed: SpriteSheetMetadata = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, metadata);
    }

    #[test]
    fn sprite_frame_node_ref() {
        let json = r#"
        {
          "resolution": [256, 256],
          "frames": [
            { "id": "procedural", "width": 128, "height": 128, "node_ref": "noise_output" }
          ]
        }
        "#;

        let params: SpriteSheetV1Params = serde_json::from_str(json).unwrap();
        assert_eq!(params.frames.len(), 1);
        assert!(matches!(
            &params.frames[0].source,
            SpriteFrameSource::NodeRef { node_ref } if node_ref == "noise_output"
        ));
    }
}

mod animation_tests {
    use super::*;

    #[test]
    fn sprite_animation_params_roundtrip() {
        let params = SpriteAnimationV1Params {
            name: "idle".to_string(),
            fps: 12,
            loop_mode: AnimationLoopMode::Loop,
            frames: vec![
                AnimationFrame {
                    frame_id: "idle_0".to_string(),
                    duration_ms: Some(100),
                },
                AnimationFrame {
                    frame_id: "idle_1".to_string(),
                    duration_ms: None,
                },
            ],
        };

        let json = serde_json::to_string_pretty(&params).unwrap();
        let parsed: SpriteAnimationV1Params = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, params);
    }

    #[test]
    fn sprite_animation_params_from_json() {
        let json = r#"
        {
          "name": "walk",
          "fps": 8,
          "loop_mode": "ping_pong",
          "frames": [
            { "frame_id": "walk_0", "duration_ms": 125 },
            { "frame_id": "walk_1" },
            { "frame_id": "walk_2", "duration_ms": 125 }
          ]
        }
        "#;

        let params: SpriteAnimationV1Params = serde_json::from_str(json).unwrap();
        assert_eq!(params.name, "walk");
        assert_eq!(params.fps, 8);
        assert_eq!(params.loop_mode, AnimationLoopMode::PingPong);
        assert_eq!(params.frames.len(), 3);
        assert_eq!(params.frames[0].duration_ms, Some(125));
        assert_eq!(params.frames[1].duration_ms, None);
    }

    #[test]
    fn sprite_animation_default_fps() {
        let json = r#"
        {
          "name": "test",
          "frames": []
        }
        "#;

        let params: SpriteAnimationV1Params = serde_json::from_str(json).unwrap();
        assert_eq!(params.fps, 12);
    }

    #[test]
    fn sprite_animation_default_loop_mode() {
        let json = r#"
        {
          "name": "test",
          "frames": []
        }
        "#;

        let params: SpriteAnimationV1Params = serde_json::from_str(json).unwrap();
        assert_eq!(params.loop_mode, AnimationLoopMode::Loop);
    }

    #[test]
    fn animation_loop_mode_serialization() {
        assert_eq!(
            serde_json::to_string(&AnimationLoopMode::Loop).unwrap(),
            "\"loop\""
        );
        assert_eq!(
            serde_json::to_string(&AnimationLoopMode::Once).unwrap(),
            "\"once\""
        );
        assert_eq!(
            serde_json::to_string(&AnimationLoopMode::PingPong).unwrap(),
            "\"ping_pong\""
        );

        assert_eq!(
            serde_json::from_str::<AnimationLoopMode>("\"loop\"").unwrap(),
            AnimationLoopMode::Loop
        );
        assert_eq!(
            serde_json::from_str::<AnimationLoopMode>("\"once\"").unwrap(),
            AnimationLoopMode::Once
        );
        assert_eq!(
            serde_json::from_str::<AnimationLoopMode>("\"ping_pong\"").unwrap(),
            AnimationLoopMode::PingPong
        );
    }

    #[test]
    fn animation_frame_builder() {
        let frame1 = AnimationFrame::new("frame_0");
        assert_eq!(frame1.frame_id, "frame_0");
        assert_eq!(frame1.duration_ms, None);

        let frame2 = AnimationFrame::with_duration("frame_1", 150);
        assert_eq!(frame2.frame_id, "frame_1");
        assert_eq!(frame2.duration_ms, Some(150));
    }

    #[test]
    fn sprite_animation_builder() {
        let params = SpriteAnimationV1Params::new("attack")
            .with_fps(24)
            .with_loop_mode(AnimationLoopMode::Once)
            .with_frame(AnimationFrame::with_duration("attack_0", 50))
            .with_frame(AnimationFrame::with_duration("attack_1", 100))
            .with_frame(AnimationFrame::with_duration("attack_2", 50));

        assert_eq!(params.name, "attack");
        assert_eq!(params.fps, 24);
        assert_eq!(params.loop_mode, AnimationLoopMode::Once);
        assert_eq!(params.frames.len(), 3);
    }

    #[test]
    fn compute_total_duration_explicit() {
        let params = SpriteAnimationV1Params::new("test")
            .with_fps(10)
            .with_frame(AnimationFrame::with_duration("a", 100))
            .with_frame(AnimationFrame::with_duration("b", 200))
            .with_frame(AnimationFrame::with_duration("c", 150));

        assert_eq!(params.compute_total_duration_ms(), 450);
    }

    #[test]
    fn compute_total_duration_default() {
        let params = SpriteAnimationV1Params::new("test")
            .with_fps(10) // 100ms per frame default
            .with_frame(AnimationFrame::new("a"))
            .with_frame(AnimationFrame::new("b"))
            .with_frame(AnimationFrame::new("c"));

        assert_eq!(params.compute_total_duration_ms(), 300);
    }

    #[test]
    fn compute_total_duration_mixed() {
        let params = SpriteAnimationV1Params::new("test")
            .with_fps(10) // 100ms per frame default
            .with_frame(AnimationFrame::with_duration("a", 50))
            .with_frame(AnimationFrame::new("b")) // Uses default 100ms
            .with_frame(AnimationFrame::with_duration("c", 200));

        assert_eq!(params.compute_total_duration_ms(), 350);
    }

    #[test]
    fn to_metadata() {
        let params = SpriteAnimationV1Params::new("idle")
            .with_fps(12)
            .with_loop_mode(AnimationLoopMode::Loop)
            .with_frame(AnimationFrame::with_duration("idle_0", 100))
            .with_frame(AnimationFrame::new("idle_1")) // Default: 1000/12 = 83ms
            .with_frame(AnimationFrame::with_duration("idle_2", 200));

        let metadata = params.to_metadata();

        assert_eq!(metadata.name, "idle");
        assert_eq!(metadata.fps, 12);
        assert_eq!(metadata.loop_mode, AnimationLoopMode::Loop);
        assert_eq!(metadata.frames.len(), 3);
        assert_eq!(metadata.frames[0].duration_ms, 100);
        assert_eq!(metadata.frames[1].duration_ms, 83); // 1000/12
        assert_eq!(metadata.frames[2].duration_ms, 200);
        assert_eq!(metadata.total_duration_ms, 383);
    }

    #[test]
    fn animation_metadata_serialization() {
        let metadata = SpriteAnimationMetadata {
            name: "idle".to_string(),
            fps: 12,
            loop_mode: AnimationLoopMode::Loop,
            total_duration_ms: 500,
            frames: vec![
                AnimationFrameResolved {
                    frame_id: "idle_0".to_string(),
                    duration_ms: 100,
                },
                AnimationFrameResolved {
                    frame_id: "idle_1".to_string(),
                    duration_ms: 200,
                },
            ],
        };

        let json = serde_json::to_string_pretty(&metadata).unwrap();
        let parsed: SpriteAnimationMetadata = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, metadata);
    }

    #[test]
    fn loop_mode_display() {
        assert_eq!(AnimationLoopMode::Loop.to_string(), "loop");
        assert_eq!(AnimationLoopMode::Once.to_string(), "once");
        assert_eq!(AnimationLoopMode::PingPong.to_string(), "ping_pong");
    }
}
