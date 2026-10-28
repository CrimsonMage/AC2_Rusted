use ac2_assets::{
    Ac2LayerLighting, Ac2LayerModifier, Ac2LayerStage, Ac2MaterialLayer, Ac2Waveform,
};
use ac2_core::DataId;

use super::{blend, layer_drawn, layer_pass, layer_pass_forced, RenderSettings, VertexProgram};

/// The common Legions object layer: lit, material diffuse, depth fog, stage 0
/// MODULATE2X(texture, diffuse) with alpha SELECTARG2(diffuse), SRCALPHA/INVSRCALPHA.
fn lit_layer() -> Ac2MaterialLayer {
    Ac2MaterialLayer {
        true_flags: 0x14_0000,
        false_flags: 0,
        stages: vec![Ac2LayerStage {
            texture: Some(DataId::new(0x4000_0001)),
            color_op: 5,
            color_arg1: 2,
            color_arg2: 0,
            alpha_op: 3,
            alpha_arg1: 2,
            alpha_arg2: 0,
            address_u: 1,
            address_v: 1,
            ..Ac2LayerStage::default()
        }],
        modifiers: Vec::new(),
        source_blend: blend::SRCALPHA,
        dest_blend: blend::INVSRCALPHA,
        depth_test: 4,
        depth_write: 1,
        cull_mode: 2,
        alpha_test_mode: 0,
        alpha_test_ref: Ac2Waveform::default(),
        lighting: Ac2LayerLighting {
            dynamic: true,
            material_diffuse: true,
            depth_fog: true,
            ..Ac2LayerLighting::default()
        },
        ambient: [0.5; 4],
        diffuse: [1.0, 0.8, 0.6, 1.0],
        specular: [1.0; 4],
        specular_power: Ac2Waveform::constant(0.0),
        options: 0,
        vertex_format: 0,
        vertex_shader: Default::default(),
        pixel_shader: Default::default(),
    }
}

#[test]
fn layers_are_gated_by_hardware_class_detail_and_fog() {
    let settings = RenderSettings {
        hardware_class: 1,
        ..RenderSettings::default()
    };
    let layer = lit_layer();
    assert!(layer_drawn(&layer, &settings));
    let class2 = Ac2MaterialLayer {
        true_flags: 0x24_0000,
        ..lit_layer()
    };
    assert!(!layer_drawn(&class2, &settings));
    assert!(layer_drawn(
        &class2,
        &RenderSettings {
            hardware_class: 2,
            ..settings
        }
    ));
    assert!(!layer_drawn(
        &layer,
        &RenderSettings {
            detail: 1,
            ..settings
        }
    ));
    let foggy = Ac2MaterialLayer {
        true_flags: 0x8_0000,
        ..lit_layer()
    };
    assert!(!layer_drawn(
        &foggy,
        &RenderSettings {
            fog_linear: false,
            ..settings
        }
    ));
    let stageless = Ac2MaterialLayer {
        stages: Vec::new(),
        ..lit_layer()
    };
    assert!(!layer_drawn(&stageless, &settings));
}

#[test]
fn setup_state_material_and_alpha_test() {
    let pass = layer_pass(&lit_layer(), [1.0, 1.0, 1.0, 0.9], 0.0);
    assert_eq!(pass.program, VertexProgram::Lit);
    assert_eq!(pass.diffuse, [1.0, 0.8, 0.6, 0.9]);
    // Specular highlights off: no specular colour, power or SPECULARENABLE.
    assert_eq!(
        (pass.specular, pass.power, pass.specular_enable),
        ([0.0; 4], 0.0, false)
    );
    // SRCALPHA enables the alpha test at reference 0; alpha 0.9 keeps depth writes.
    assert_eq!(pass.alpha_test, Some(0));
    assert!(pass.depth_write && pass.fog);
    // Below 0.8 with the alpha test off in the layer, depth writes are dropped.
    let faded = layer_pass(&lit_layer(), [1.0, 1.0, 1.0, 0.5], 0.0);
    assert!(!faded.depth_write);

    let specular = Ac2MaterialLayer {
        lighting: Ac2LayerLighting {
            specular_highlights: true,
            ..lit_layer().lighting
        },
        specular_power: Ac2Waveform::constant(12.0),
        alpha_test_mode: 1,
        alpha_test_ref: Ac2Waveform::constant(0.5),
        ..lit_layer()
    };
    let pass = layer_pass(&specular, [1.0; 4], 0.0);
    assert_eq!((pass.power, pass.specular_enable), (12.0, true));
    assert_eq!(pass.alpha_test, Some(127));
}

#[test]
fn vertex_program_selection_follows_the_fragment_compile() {
    let with = |lighting: Ac2LayerLighting| {
        layer_pass(
            &Ac2MaterialLayer {
                lighting,
                ..lit_layer()
            },
            [1.0; 4],
            0.0,
        )
        .program
    };
    let base = Ac2LayerLighting::default();
    assert_eq!(with(base), VertexProgram::Unlit);
    assert_eq!(
        with(Ac2LayerLighting {
            dynamic: true,
            vertex_diffuse: true,
            ..base
        }),
        VertexProgram::LitVertexDiffuse
    );
    assert_eq!(
        with(Ac2LayerLighting {
            dynamic: true,
            per_pixel: true,
            pixel_multiple_lights: true,
            ..base
        }),
        VertexProgram::PixelLitMultiple
    );
}

#[test]
fn stage_texture_matrix_rows_come_from_its_modifiers() {
    let layer = Ac2MaterialLayer {
        modifiers: vec![Ac2LayerModifier::UvTranslate {
            tex_coord: 0,
            u: Ac2Waveform {
                kind: ac2_math::waveform::kind::NONE,
                base_velocity: 0.5,
                ..Ac2Waveform::default()
            },
            v: Ac2Waveform::constant(0.0),
        }],
        ..lit_layer()
    };
    let pass = layer_pass(&layer, [1.0; 4], 2.0);
    // u' = u + 1 at t = 2 s: row 0 is (1, 0, 0, 1).
    assert_eq!(
        pass.stages[0].texture_matrix,
        Some([[1.0, 0.0, 0.0, 1.0], [0.0, 1.0, 0.0, 0.0]])
    );
}

/// `SceneTool::m_bForceDisableAlphaTest`: a faded alpha-tested layer of an object past 40 units
/// loses its depth write and its alpha reference even above 0.8; an unfaded one keeps both.
#[test]
fn forced_alpha_test_off_drops_a_faded_layers_test_and_depth_write() {
    let tested = Ac2MaterialLayer {
        alpha_test_mode: 1,
        alpha_test_ref: Ac2Waveform::constant(0.5),
        ..lit_layer()
    };
    let kept = layer_pass_forced(&tested, [1.0, 1.0, 1.0, 0.9], 0.0, false);
    assert!(kept.depth_write);
    assert_eq!(kept.alpha_test, Some(127));
    let forced = layer_pass_forced(&tested, [1.0, 1.0, 1.0, 0.9], 0.0, true);
    assert!(!forced.depth_write);
    assert_eq!(forced.alpha_test, Some(0));
    let opaque = layer_pass_forced(&tested, [1.0; 4], 0.0, true);
    assert!(opaque.depth_write);
    assert_eq!(opaque.alpha_test, Some(127));
}
