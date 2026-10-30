//! The frames before the world: the session runs with no movement input, its login events go to
//! the pregame screens (`PregameView`), whose UI takes the UI actions and text input, and whose
//! requests (enter, create, delete, quit) go out through the session; their viewports' 3D
//! scenes draw among the UI's surfaces. Once the player is in the world the screens give way to
//! the world view and its HUD (`UIFlow`'s gameplay mode).

use std::sync::Arc;
use std::time::Duration;

use ac2_client::view::ClientView;
use ac2_client::FrameInput;
use ac2_gpu::render_to;
use winit::event_loop::ActiveEventLoop;

use super::{show_world_names, App};

impl App {
    /// One frame before the world.
    pub(super) fn pregame_frame(&mut self, event_loop: &ActiveEventLoop) {
        let Some(state) = self.state.as_mut() else {
            return;
        };
        let (stamp, now) = state.now();
        let ui_input = state.controls.take_ui_input();
        let modifiers = state.controls.modifiers();
        let (actions, characters, _) = state.controls.take_text_input();
        let presses = state.controls.take_key_presses();
        let mut messages = Vec::new();
        if let Some(pregame) = &mut state.pregame {
            pregame.key_presses(&presses);
            messages = pregame.input(&ui_input);
            messages.extend(pregame.text_input(&actions, &characters, modifiers));
            state
                .controls
                .set_text_mode(pregame.text_mode(), stamp, now);
            state.controls.set_ime_allowed(pregame.ime_allowed());
            state.controls.set_copy_text(pregame.text_copy_focus());
        }
        state.controls.ui_messages(&messages, now);
        // No body to move yet: the controls' frame is dropped.
        state.controls.frame();
        let events = match self.client.use_time(&FrameInput::default()) {
            Ok(events) => events,
            Err(error) => {
                eprintln!("ac2: {error}");
                event_loop.exit();
                return;
            }
        };
        show_world_names(&state.window, &events);
        let notices = self.client.take_notices();
        match &mut state.pregame {
            Some(pregame) => {
                for event in &events {
                    pregame.event(event);
                }
                for notice in notices {
                    pregame.notice(notice);
                }
                let outcome = pregame.requests(self.client.session_mut());
                for did in outcome.music {
                    self.client.play_pregame_music(did);
                }
                for did in outcome.credit_sounds {
                    self.client.play_credit_sound(did);
                }
                pregame.set_credit_sound_busy(self.client.credit_sound_busy());
                if let Some(set) = &outcome.key_maps {
                    state.controls.replace_key_maps(set);
                }
                if let Some(set) = &outcome.saved_key_maps {
                    // `SaveKeyboardList`'s `PFileParser::SaveToFile` of the three maps.
                    if let Err(error) = ac2_ui::keymap_file::save_set(
                        &ac2_ui::keymap_file::KeymapFiles::user(),
                        set,
                    ) {
                        eprintln!("ac2: keymap files: {error}");
                    }
                }
                if outcome.quit {
                    self.client.session_mut().log_off(Duration::from_secs(3));
                    event_loop.exit();
                    return;
                }
                pregame.update(&state.gpu, now);
            }
            None if notices.contains(&ac2_client::Notice::ConnectionLost) => {
                eprintln!("ac2: connection lost");
                event_loop.exit();
                return;
            }
            None => {}
        }
        if self.client.frame().in_world {
            // `UseNewMode(Gameplay)`: the world and its HUD replace the pregame screens.
            let mut view = ClientView::new(
                Arc::clone(&self.world),
                &state.gpu,
                state.format,
                self.radius,
                (state.config.width, state.config.height),
            );
            view.set_scene_options(self.scene_options);
            view.set_stream_pipelines(true);
            state.view = Some(view);
            state.pregame = None;
            return;
        }
        let Some(output) = ac2_gpu::acquire_frame(&state.surface, &state.gpu.device, &state.config)
        else {
            // The frame's uploads still go to the GPU: nothing is presented this time.
            state.gpu.flush_writes();
            return;
        };
        let target = output.texture.create_view(&Default::default());
        match &mut state.pregame {
            Some(pregame) => {
                pregame.prepare(&state.gpu, now);
                pregame.render(&state.gpu, &target, &state.depth);
            }
            None => render_to(
                &state.gpu.device,
                &state.gpu.queue,
                &target,
                &state.depth,
                [0.0; 3],
                |_| {},
            ),
        }
        state.gpu.queue.present(output);
    }
}
