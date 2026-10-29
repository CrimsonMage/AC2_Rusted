//! The side panel: the layers (with the icon legend and counts), the level, area and search
//! filters, the picked places' details, and the search hits.

use ac2_guide::atlas::{Marker, MarkerKind};
use ac2_guide::{nav, Guide};
use eframe::egui::{self, RichText, Sense, Vec2};

use crate::canvas;
use crate::map::Filter;

/// Draws the panel; a marker to centre the map on.
pub fn show(
    ui: &mut egui::Ui,
    guide: &Guide,
    markers: &[Marker],
    shown: &[usize],
    filter: &mut Filter,
    areas: &[String],
    picked: &mut Vec<usize>,
) -> Option<usize> {
    let mut focus = None;
    ui.label(RichText::new("AC2Hero guide").strong());
    for (kind, on) in &mut filter.layers {
        let count = markers.iter().filter(|m| m.kind == *kind).count();
        ui.horizontal(|ui| {
            let (rect, _) = ui.allocate_exact_size(Vec2::splat(18.0), Sense::hover());
            canvas::icon(ui.painter(), rect.center(), *kind, 1.0);
            ui.checkbox(on, format!("{} ({count})", kind.label()));
        });
    }
    ui.horizontal(|ui| {
        ui.label("Quest level");
        let (lo, hi) = &mut filter.levels;
        ui.add(egui::DragValue::new(lo).range(1..=*hi));
        ui.label("to");
        ui.add(egui::DragValue::new(hi).range(*lo..=100));
    });
    let shown_area = if filter.area.is_empty() {
        "Any area".to_string()
    } else {
        filter.area.clone()
    };
    egui::ComboBox::from_id_salt("area")
        .selected_text(shown_area)
        .show_ui(ui, |ui| {
            ui.selectable_value(&mut filter.area, String::new(), "Any area");
            for a in areas {
                ui.selectable_value(&mut filter.area, a.clone(), a);
            }
        });
    ui.add(egui::TextEdit::singleline(&mut filter.search).hint_text("Search quests, NPCs, places"));
    ui.separator();

    egui::ScrollArea::vertical()
        .auto_shrink(false)
        .show(ui, |ui| {
            if !picked.is_empty() {
                ui.label(RichText::new(format!("Picked ({})", picked.len())).strong());
                for &i in picked.iter().take(60) {
                    if details(ui, guide, &markers[i]) {
                        focus = Some(i);
                    }
                }
                if ui.button("Clear").clicked() {
                    picked.clear();
                }
                ui.separator();
            }
            if !filter.search.trim().is_empty() {
                ui.label(RichText::new(format!("{} matches", shown.len())).strong());
                for &i in shown.iter().take(300) {
                    let m = &markers[i];
                    ui.horizontal(|ui| {
                        let (rect, _) = ui.allocate_exact_size(Vec2::splat(16.0), Sense::hover());
                        canvas::icon(ui.painter(), rect.center(), m.kind, 0.8);
                        if ui.selectable_label(picked.contains(&i), &m.title).clicked() {
                            *picked = vec![i];
                            focus = Some(i);
                        }
                    });
                }
            } else if picked.is_empty() {
                ui.label(
                    RichText::new(
                        "Click an icon for its details, or a numbered cluster to zoom into it.",
                    )
                    .weak(),
                );
            }
        });
    focus
}

/// One picked place; whether "Centre" was clicked.
fn details(ui: &mut egui::Ui, guide: &Guide, m: &Marker) -> bool {
    let mut centre = false;
    egui::Frame::group(ui.style()).show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.horizontal(|ui| {
            let (rect, _) = ui.allocate_exact_size(Vec2::splat(18.0), Sense::hover());
            canvas::icon(ui.painter(), rect.center(), m.kind, 1.0);
            ui.label(RichText::new(&m.title).strong());
        });
        let page = &guide.pages[m.page as usize];
        let mut facts = vec![nav::format(m.at)];
        if let Some(l) = m.level {
            facts.push(format!("level {l}"));
        }
        if !page.area.is_empty() {
            facts.push(page.area.clone());
        }
        ui.label(RichText::new(facts.join(" · ")).weak());
        if !m.context.is_empty() {
            let label = match m.kind {
                MarkerKind::QuestStart => format!("Start: {}", m.context),
                _ => m.context.clone(),
            };
            ui.label(label);
        }
        if let Some(q) = page
            .quest
            .as_ref()
            .filter(|_| m.kind == MarkerKind::QuestStart)
        {
            if !q.rewards.is_empty() {
                let rewards: Vec<&str> = q.rewards.iter().map(|r| r.text.as_str()).collect();
                ui.label(RichText::new(format!("Rewards: {}", rewards.join(", "))).small());
            }
        }
        ui.horizontal(|ui| {
            if ui.small_button("Centre").clicked() {
                centre = true;
            }
            if ui.small_button("Copy coordinates").clicked() {
                ui.ctx().copy_text(nav::format(m.at));
            }
        });
    });
    centre
}
