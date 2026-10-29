//! The middle of the window: a page (its quest card, then its lines with images, links and
//! clickable coordinates) or a reward item (its icon and the quests that give it).

use std::borrow::Cow;

use ac2_guide::{Guide, Image, Line, Span, Style};
use eframe::egui::{self, load::Bytes, Color32, RichText};

use crate::app::{Target, View};

/// The guide's images as egui sources. The baked bytes are handed to egui without a copy; egui
/// decodes one when it is first drawn, and `App` forgets the decoded images (CPU and GPU) when
/// the page changes, so only the open page's images stay decoded.
pub struct Images {
    sources: Vec<(String, Bytes)>,
}

impl Images {
    pub fn new(guide: &Guide) -> Self {
        let sources = guide
            .images
            .iter()
            .map(|i| {
                let bytes = match &i.bytes {
                    Cow::Borrowed(b) => Bytes::Static(b),
                    Cow::Owned(b) => Bytes::Shared(b.as_slice().into()),
                };
                // The uri's extension is the file's real format, which egui's loader picks the
                // decoder by.
                let ext = Image::format(&i.bytes).unwrap_or("jpg");
                (format!("bytes://ac2hero/{}.{ext}", i.path), bytes)
            })
            .collect();
        Self { sources }
    }

    pub fn image(&self, index: u32) -> egui::Image<'static> {
        let (uri, bytes) = &self.sources[index as usize];
        egui::Image::from_bytes(uri.clone(), bytes.clone())
    }

    pub fn path(&self, index: u32) -> &str {
        let uri = self.sources[index as usize]
            .0
            .trim_start_matches("bytes://ac2hero/");
        uri.rsplit_once('.').map_or(uri, |(path, _)| path)
    }
}

/// What a click in the middle asks for.
pub enum Action {
    Back,
    Show(View),
    Search(String),
    Target(Target),
    Map,
}

pub struct Ctx<'a> {
    pub guide: &'a Guide,
    pub images: &'a Images,
    pub item_quests: &'a [Vec<u32>],
    pub target: Option<&'a Target>,
    pub can_go_back: bool,
}

const GOLD: Color32 = Color32::from_rgb(230, 170, 40);

pub fn show(ui: &mut egui::Ui, ctx: &Ctx, view: Option<View>) -> Option<Action> {
    let Some(view) = view else {
        ui.centered_and_justified(|ui| {
            ui.label("Pick a page or a reward. Blue entries have map coordinates; click a gold coordinate to steer to it.");
        });
        return None;
    };
    let mut action = None;
    let title = match view {
        View::Page(p) => ctx.guide.pages[p as usize].title.as_str(),
        View::Item(i) => ctx.guide.items[i as usize].name.as_str(),
    };
    ui.horizontal(|ui| {
        if ui
            .add_enabled(ctx.can_go_back, egui::Button::new("< Back"))
            .clicked()
        {
            action = Some(Action::Back);
        }
        ui.heading(title);
    });
    egui::ScrollArea::vertical()
        .auto_shrink(false)
        .show(ui, |ui| {
            let a = match view {
                View::Page(p) => page(ui, ctx, p),
                View::Item(i) => item(ui, ctx, i),
            };
            if a.is_some() {
                action = a;
            }
        });
    action
}

fn page(ui: &mut egui::Ui, ctx: &Ctx, index: u32) -> Option<Action> {
    let page = &ctx.guide.pages[index as usize];
    let mut action = None;
    ui.label(RichText::new(format!("{} · {}", page.kind.label(), page.path)).weak());
    if let Some(q) = &page.quest {
        egui::Frame::group(ui.style()).show(ui, |ui| {
            ui.set_width(ui.available_width());
            egui::Grid::new("facts")
                .num_columns(2)
                .spacing([12.0, 4.0])
                .show(ui, |ui| {
                    for (label, value) in [
                        ("Requires", &q.requirements),
                        ("Starts", &q.start),
                        ("Time limit", &q.time_limit),
                        ("Resets", &q.reset),
                    ] {
                        if !value.is_empty() {
                            ui.label(RichText::new(label).weak());
                            ui.label(value);
                            ui.end_row();
                        }
                    }
                });
            if !q.rewards.is_empty() {
                ui.add_space(4.0);
                ui.label(RichText::new("Rewards").weak());
                ui.horizontal_wrapped(|ui| {
                    for r in &q.rewards {
                        let chip = egui::Frame::new()
                            .fill(ui.visuals().faint_bg_color)
                            .corner_radius(4.0)
                            .inner_margin(egui::Margin::symmetric(6, 3));
                        let response = chip
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    let icon =
                                        r.item.and_then(|i| ctx.guide.items[i as usize].icon);
                                    if let Some(icon) = icon {
                                        ui.add(
                                            ctx.images.image(icon).max_size(egui::vec2(24.0, 24.0)),
                                        );
                                    }
                                    let text = RichText::new(&r.text);
                                    if r.item.is_some() {
                                        ui.label(text.color(ui.visuals().hyperlink_color));
                                    } else {
                                        ui.label(text);
                                    }
                                });
                            })
                            .response;
                        if let Some(i) = r.item {
                            if response
                                .interact(egui::Sense::click())
                                .on_hover_text("Open the item")
                                .clicked()
                            {
                                action = Some(Action::Show(View::Item(i)));
                            }
                        }
                    }
                });
            }
        });
    }
    if page.coords().next().is_some() && ui.button("Show this page's route on the map").clicked() {
        action = Some(Action::Map);
    }
    ui.separator();
    for line in &page.lines {
        if let Some(a) = draw_line(ui, ctx, line, index) {
            action = Some(a);
        }
    }
    action
}

fn item(ui: &mut egui::Ui, ctx: &Ctx, index: u32) -> Option<Action> {
    let item = &ctx.guide.items[index as usize];
    let mut action = None;
    ui.horizontal(|ui| {
        if let Some(icon) = item.icon {
            ui.add(
                ctx.images
                    .image(icon)
                    .fit_to_original_size(2.0)
                    .max_size(egui::vec2(96.0, 96.0)),
            );
        }
        ui.vertical(|ui| {
            let category = &ctx.guide.pages[item.page as usize];
            if ui.link(format!("Category: {}", category.title)).clicked() {
                action = Some(Action::Show(View::Page(item.page)));
            }
            if !item.note.is_empty() {
                ui.label(&item.note);
            }
        });
    });
    ui.separator();
    let quests = &ctx.item_quests[index as usize];
    if quests.is_empty() {
        ui.label(RichText::new("No quest's reward list names this item.").weak());
    } else {
        ui.label(RichText::new("Rewarded by").strong());
        for &q in quests {
            let page = &ctx.guide.pages[q as usize];
            let level = page
                .level()
                .map_or(String::new(), |l| format!("level {l} · "));
            ui.horizontal(|ui| {
                if ui.link(&page.title).clicked() {
                    action = Some(Action::Show(View::Page(q)));
                }
                ui.label(RichText::new(format!("{level}{}", page.area)).weak());
            });
        }
    }
    action
}

/// One line of a page; what a click on it asks for.
fn draw_line(ui: &mut egui::Ui, ctx: &Ctx, line: &Line, page: u32) -> Option<Action> {
    let mut action = None;
    let link = ui.visuals().hyperlink_color;
    let body = |text: &str, bold: bool| {
        let t = RichText::new(text);
        let t = if bold { t.strong() } else { t };
        if line.style == Style::Heading {
            t.heading()
        } else {
            t
        }
    };
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        if line.style == Style::Bullet {
            ui.label("  •  ");
        }
        for span in &line.spans {
            match span {
                Span::Text { text, bold } => {
                    ui.label(body(text, *bold));
                }
                Span::Page { text, page } => {
                    if ui.link(body(text, false).color(link)).clicked() {
                        action = Some(Action::Show(View::Page(*page)));
                    }
                }
                Span::Missing { text, href } => {
                    let query = href
                        .split_once("?search=")
                        .map(|(_, q)| q.replace('+', " "));
                    let r = ui.add(
                        egui::Label::new(body(text, false).underline()).sense(egui::Sense::click()),
                    );
                    let r = match &query {
                        Some(q) => r.on_hover_text(format!(
                            "Not in the mirror; click to search the guide for \"{q}\""
                        )),
                        None => r.on_hover_text(format!("Not in the guide: {href}")),
                    };
                    if r.clicked() {
                        if let Some(q) = query {
                            action = Some(Action::Search(q));
                        }
                    }
                }
                Span::Coord { text, at } => {
                    let chosen = ctx.target.is_some_and(|t| t.at == *at);
                    let mut t = RichText::new(text).color(GOLD).strong().underline();
                    if chosen {
                        t = t.background_color(Color32::from_rgba_unmultiplied(230, 170, 40, 50));
                    }
                    let r = ui
                        .add(egui::Label::new(t).sense(egui::Sense::click()))
                        .on_hover_text("Steer to this coordinate");
                    if r.clicked() {
                        action = Some(Action::Target(Target {
                            at: *at,
                            text: text.clone(),
                            page: Some(page),
                        }));
                    }
                }
                Span::Image { image } => {
                    let width = ui.available_width().max(64.0);
                    ui.add(
                        ctx.images
                            .image(*image)
                            .fit_to_original_size(1.0)
                            .max_width(width),
                    )
                    .on_hover_text(ctx.images.path(*image));
                }
            }
        }
    });
    action
}
