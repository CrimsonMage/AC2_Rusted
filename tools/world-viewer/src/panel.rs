//! The viewer's control panel (egui): fly or walk, the player's race and sex, teleport, movement
//! speed and the time of day.

use crate::races::PlayerModel;
use crate::walk::{Landing, WalkStatus};
use ac2_render::scene::DAY_SECONDS;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Walk,
    Fly,
}

/// What the panel asks the viewer to do.
#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    SetMode(Mode),
    SetModel(usize),
    Teleport(Landing),
}

/// The panel's own state.
pub struct Panel {
    pub visible: bool,
    pub mode: Mode,
    pub models: Vec<PlayerModel>,
    pub model: usize,
    pub speed: u32,
    block: String,
    x: f32,
    y: f32,
}

/// What the viewer shows the panel each frame.
pub struct PanelView<'a> {
    pub status: Option<WalkStatus>,
    pub message: Option<&'a str>,
    pub fps: f32,
    pub time_of_day: &'a mut f64,
    pub particle_count: usize,
}

impl Panel {
    pub fn new(models: Vec<PlayerModel>, mode: Mode, landing: Landing) -> Self {
        Self {
            visible: true,
            mode,
            models,
            model: 0,
            speed: 1,
            block: format!("{:02X}{:02X}", landing.block.0, landing.block.1),
            x: landing.x,
            y: landing.y,
        }
    }

    /// The landblock typed in the teleport box: four hex digits, `xxyy`.
    fn parse_block(&self) -> Option<(u8, u8)> {
        let raw = u16::from_str_radix(self.block.trim().trim_start_matches("0x"), 16).ok()?;
        Some(((raw >> 8) as u8, raw as u8))
    }

    /// Sets the teleport fields (for "use my position").
    pub fn set_landing(&mut self, landing: Landing) {
        self.block = format!("{:02X}{:02X}", landing.block.0, landing.block.1);
        self.x = landing.x;
        self.y = landing.y;
    }

    pub fn show(&mut self, ui: &mut egui::Ui, view: PanelView<'_>) -> Vec<Command> {
        let mut commands = Vec::new();
        if !self.visible {
            return commands;
        }
        let frame = egui::Frame::window(&ui.ctx().global_style())
            .fill(egui::Color32::from_rgba_unmultiplied(24, 26, 30, 235));
        egui::Window::new("AC2 world viewer (offline)")
            .frame(frame)
            .default_pos([8.0, 110.0])
            .default_width(300.0)
            .show(ui.ctx(), |ui| {
                ui.horizontal(|ui| {
                    // The viewer changes `self.mode` when it carries the position across.
                    let mut mode = self.mode;
                    ui.selectable_value(&mut mode, Mode::Walk, "Walk");
                    ui.selectable_value(&mut mode, Mode::Fly, "Fly");
                    if mode != self.mode {
                        commands.push(Command::SetMode(mode));
                    }
                    ui.label(format!("{:.0} fps", view.fps));
                });
                ui.separator();
                ui.label("Player");
                let before = self.model;
                egui::ComboBox::from_id_salt("model")
                    .selected_text(
                        self.models
                            .get(self.model)
                            .map_or_else(|| "none".to_string(), PlayerModel::label),
                    )
                    .show_ui(ui, |ui| {
                        for (index, model) in self.models.iter().enumerate() {
                            ui.selectable_value(&mut self.model, index, model.label());
                        }
                    });
                if self.model != before {
                    commands.push(Command::SetModel(self.model));
                }
                ui.add(
                    egui::Slider::new(&mut self.speed, 1..=16)
                        .text("speed ×")
                        .logarithmic(true),
                );
                ui.separator();
                ui.label("Teleport");
                ui.horizontal(|ui| {
                    ui.label("block");
                    ui.add(egui::TextEdit::singleline(&mut self.block).desired_width(48.0));
                    ui.label("x");
                    ui.add(egui::DragValue::new(&mut self.x).range(0.0..=159.9));
                    ui.label("y");
                    ui.add(egui::DragValue::new(&mut self.y).range(0.0..=159.9));
                });
                let block = self.parse_block();
                ui.add_enabled_ui(block.is_some(), |ui| {
                    if ui.button("Teleport").clicked() {
                        if let Some(block) = block {
                            commands.push(Command::Teleport(Landing {
                                block,
                                x: self.x,
                                y: self.y,
                            }));
                        }
                    }
                });
                ui.separator();
                ui.add(
                    egui::Slider::new(view.time_of_day, 0.0..=DAY_SECONDS)
                        .text("time of day")
                        .show_value(false),
                );
                if let Some(status) = view.status {
                    ui.separator();
                    ui.monospace(format!(
                        "cell {:08X}  blocks {}\npos {:.1} {:.1} {:.1}\nheading {:.0}°{}{}",
                        status.cell,
                        status.resident_blocks,
                        status.origin[0],
                        status.origin[1],
                        status.origin[2],
                        status.heading_degrees,
                        if status.contact {
                            "  on ground"
                        } else {
                            "  airborne"
                        },
                        if status.in_world { "" } else { "  (entering)" },
                    ));
                }
                if view.particle_count > 0 {
                    ui.monospace(format!("particles {}", view.particle_count));
                }
                if let Some(message) = view.message {
                    ui.colored_label(egui::Color32::LIGHT_RED, message);
                }
                ui.separator();
                ui.weak(
                    "W/S walk  A/D turn  Q/E strafe  Space jump  RMB look  wheel zoom\n\
                     V first person  Home reset camera  numpad camera  Tab fly/walk  F1 panel  [ ] time",
                );
            });
        commands
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn panel(block: &str) -> Panel {
        let mut panel = Panel::new(
            Vec::new(),
            Mode::Walk,
            Landing {
                block: (0, 0),
                x: 0.0,
                y: 0.0,
            },
        );
        panel.block = block.to_string();
        panel
    }

    #[test]
    fn the_teleport_block_is_four_hex_digits() {
        assert_eq!(panel("7F7F").parse_block(), Some((0x7F, 0x7F)));
        assert_eq!(panel(" 0x10a0 ").parse_block(), Some((0x10, 0xA0)));
        assert_eq!(panel("7F").parse_block(), Some((0x00, 0x7F)));
        assert_eq!(panel("zz").parse_block(), None);
        assert_eq!(panel("12345").parse_block(), None);
    }
}
