//! The window: the page list on the left (quests by level, NPCs, monsters, guides, or reward
//! items), the page or the route map in the middle, the navigator on the right.

use ac2_guide::{html, Guide, Kind, MapCoord, Page};
use eframe::egui::{self, RichText};

use crate::map_view::MapView;
use crate::navigator::Navigator;
use crate::page_view::{self, Action, Images};

/// A coordinate the player can steer to, with where it came from.
#[derive(Clone)]
pub struct Target {
    pub at: MapCoord,
    pub text: String,
    pub page: Option<u32>,
}

#[derive(PartialEq, Eq, Clone, Copy)]
enum Centre {
    Page,
    Map,
}

/// What the left list shows.
#[derive(PartialEq, Eq, Clone, Copy)]
enum Mode {
    Pages(Option<Kind>),
    Rewards,
}

/// What the middle shows.
#[derive(PartialEq, Eq, Clone, Copy)]
pub enum View {
    Page(u32),
    Item(u32),
}

enum Row {
    Header(String),
    Page(u32),
    Item(u32),
}

pub struct App {
    guide: Guide,
    images: Images,
    /// The quests that reward each item.
    item_quests: Vec<Vec<u32>>,
    search: String,
    reward: String,
    mode: Mode,
    area: String,
    areas: Vec<String>,
    levels: (u32, u32),
    by_level: bool,
    rows: Vec<Row>,
    dirty: bool,
    view: Option<View>,
    /// The view the last frame drew; when it changes the old page's decoded images are freed.
    drawn: Option<View>,
    back: Vec<View>,
    centre: Centre,
    nav: Navigator,
    map: MapView,
}

impl App {
    pub fn new(ctx: &egui::Context, guide: Guide) -> Self {
        egui_extras::install_image_loaders(ctx);
        let mut areas: Vec<String> = guide
            .pages
            .iter()
            .map(|p| p.area.clone())
            .filter(|a| !a.is_empty())
            .collect();
        areas.sort();
        areas.dedup();
        let mut item_quests = vec![Vec::new(); guide.items.len()];
        for (i, page) in guide.pages.iter().enumerate() {
            for r in page.quest.iter().flat_map(|q| &q.rewards) {
                if let Some(item) = r.item {
                    let quests: &mut Vec<u32> = &mut item_quests[item as usize];
                    if quests.last() != Some(&(i as u32)) {
                        quests.push(i as u32);
                    }
                }
            }
        }
        Self {
            images: Images::new(&guide),
            map: MapView::new(&guide),
            guide,
            item_quests,
            search: String::new(),
            reward: String::new(),
            mode: Mode::Pages(Some(Kind::Quest)),
            area: String::new(),
            areas,
            levels: (1, 100),
            by_level: true,
            rows: Vec::new(),
            dirty: true,
            view: None,
            drawn: None,
            back: Vec::new(),
            centre: Centre::Page,
            nav: Navigator::default(),
        }
    }

    /// Opens the first page whose title contains `title` (any case).
    pub fn open_titled(&mut self, title: &str) {
        let title = title.to_lowercase();
        if let Some(i) = self
            .guide
            .pages
            .iter()
            .position(|p| p.title.to_lowercase().contains(&title))
        {
            self.show(View::Page(i as u32));
        }
    }

    fn show(&mut self, view: View) {
        if let Some(current) = self.view.filter(|&c| c != view) {
            self.back.push(current);
        }
        self.view = Some(view);
        self.centre = Centre::Page;
    }

    fn refilter(&mut self) {
        let words: Vec<String> = self
            .search
            .to_lowercase()
            .split_whitespace()
            .map(str::to_string)
            .collect();
        let matches = |text: &str| {
            let text = text.to_lowercase();
            words.iter().all(|w| text.contains(w.as_str()))
        };
        self.rows.clear();
        match self.mode {
            Mode::Rewards => {
                let mut items: Vec<u32> = (0..self.guide.items.len() as u32)
                    .filter(|&i| {
                        let item = &self.guide.items[i as usize];
                        let category = &self.guide.pages[item.page as usize].title;
                        matches(&format!("{} {} {category}", item.name, item.note))
                    })
                    .collect();
                items.sort_by_key(|&i| {
                    let item = &self.guide.items[i as usize];
                    (item.page, item.name.to_lowercase())
                });
                let mut category = None;
                for i in items {
                    let page = self.guide.items[i as usize].page;
                    if category != Some(page) {
                        category = Some(page);
                        self.rows
                            .push(Row::Header(self.guide.pages[page as usize].title.clone()));
                    }
                    self.rows.push(Row::Item(i));
                }
            }
            Mode::Pages(kind) => {
                let reward = self.reward.to_lowercase();
                let quests_only = kind == Some(Kind::Quest);
                let mut pages: Vec<u32> = self
                    .guide
                    .pages
                    .iter()
                    .enumerate()
                    .filter(|(_, p)| kind.is_none_or(|k| p.kind == k))
                    .filter(|(_, p)| self.area.is_empty() || p.area == self.area)
                    .filter(|(_, p)| matches(&p.title))
                    .filter(|(_, p)| {
                        !quests_only
                            || p.level()
                                .is_none_or(|l| l >= self.levels.0 && l <= self.levels.1)
                    })
                    .filter(|(_, p)| {
                        reward.is_empty()
                            || p.quest.as_ref().is_some_and(|q| {
                                q.rewards
                                    .iter()
                                    .any(|r| r.text.to_lowercase().contains(&reward))
                            })
                    })
                    .map(|(i, _)| i as u32)
                    .collect();
                if quests_only && self.by_level {
                    pages.sort_by_key(|&i| {
                        let p = &self.guide.pages[i as usize];
                        (p.level().unwrap_or(u32::MAX), p.title.to_lowercase())
                    });
                    let mut band = None;
                    for i in pages {
                        let b = self.guide.pages[i as usize].level().map(|l| l / 10);
                        if band != Some(b) {
                            band = Some(b);
                            self.rows.push(Row::Header(match b {
                                Some(0) => "Levels 1 to 9".into(),
                                Some(b) => format!("Levels {} to {}", b * 10, b * 10 + 9),
                                None => "Level not stated".into(),
                            }));
                        }
                        self.rows.push(Row::Page(i));
                    }
                } else {
                    self.rows.extend(pages.into_iter().map(Row::Page));
                }
            }
        }
        self.dirty = false;
    }

    fn list(&mut self, ui: &mut egui::Ui) {
        ui.heading("AC2Hero guide");
        let hint = match self.mode {
            Mode::Rewards => "Search reward items",
            Mode::Pages(_) => "Search titles",
        };
        self.dirty |= ui
            .add(egui::TextEdit::singleline(&mut self.search).hint_text(hint))
            .changed();
        ui.horizontal_wrapped(|ui| {
            self.dirty |= ui
                .selectable_value(&mut self.mode, Mode::Pages(None), "All")
                .changed();
            for kind in Kind::ALL {
                self.dirty |= ui
                    .selectable_value(&mut self.mode, Mode::Pages(Some(kind)), kind.label())
                    .changed();
            }
            self.dirty |= ui
                .selectable_value(&mut self.mode, Mode::Rewards, "Rewards")
                .changed();
        });
        if let Mode::Pages(kind) = self.mode {
            let shown = if self.area.is_empty() {
                "Any area".to_string()
            } else {
                self.area.clone()
            };
            egui::ComboBox::from_id_salt("area")
                .selected_text(shown)
                .show_ui(ui, |ui| {
                    self.dirty |= ui
                        .selectable_value(&mut self.area, String::new(), "Any area")
                        .changed();
                    for area in &self.areas {
                        self.dirty |= ui
                            .selectable_value(&mut self.area, area.clone(), area)
                            .changed();
                    }
                });
            if kind == Some(Kind::Quest) {
                ui.horizontal(|ui| {
                    ui.label("Level");
                    let (lo, hi) = &mut self.levels;
                    self.dirty |= ui.add(egui::DragValue::new(lo).range(1..=*hi)).changed();
                    ui.label("to");
                    self.dirty |= ui.add(egui::DragValue::new(hi).range(*lo..=100)).changed();
                    self.dirty |= ui.checkbox(&mut self.by_level, "group").changed();
                });
                self.dirty |= ui
                    .add(
                        egui::TextEdit::singleline(&mut self.reward)
                            .hint_text("Reward contains (e.g. boots, QP)"),
                    )
                    .changed();
            }
        }
        if self.dirty {
            self.refilter();
        }
        let count = self
            .rows
            .iter()
            .filter(|r| !matches!(r, Row::Header(_)))
            .count();
        ui.label(RichText::new(format!("{count} shown")).weak());
        ui.separator();
        let row_height = 20.0;
        let mut open = None;
        egui::ScrollArea::vertical().auto_shrink(false).show_rows(
            ui,
            row_height,
            self.rows.len(),
            |ui, range| {
                for row in &self.rows[range] {
                    match row {
                        Row::Header(text) => {
                            ui.add_sized(
                                [ui.available_width(), row_height],
                                egui::Label::new(RichText::new(text).strong()),
                            );
                        }
                        Row::Page(i) => {
                            let page = &self.guide.pages[*i as usize];
                            let mut text = RichText::new(&page.title);
                            if page.coords().next().is_some() {
                                text = text.color(ui.visuals().hyperlink_color);
                            }
                            ui.horizontal(|ui| {
                                ui.set_height(row_height);
                                if let Some(l) = page.level() {
                                    ui.label(RichText::new(format!("{l:>3}")).monospace().weak());
                                }
                                if ui
                                    .selectable_label(self.view == Some(View::Page(*i)), text)
                                    .clicked()
                                {
                                    open = Some(View::Page(*i));
                                }
                            });
                        }
                        Row::Item(i) => {
                            let item = &self.guide.items[*i as usize];
                            ui.horizontal(|ui| {
                                ui.set_height(row_height);
                                if let Some(icon) = item.icon {
                                    ui.add(
                                        self.images.image(icon).max_size(egui::vec2(18.0, 18.0)),
                                    );
                                }
                                if ui
                                    .selectable_label(self.view == Some(View::Item(*i)), &item.name)
                                    .clicked()
                                {
                                    open = Some(View::Item(*i));
                                }
                            });
                        }
                    }
                }
            },
        );
        if let Some(view) = open {
            self.show(view);
        }
    }

    fn apply(&mut self, action: Action) {
        match action {
            Action::Back => self.view = self.back.pop(),
            Action::Show(view) => self.show(view),
            Action::Search(q) => {
                self.search = q;
                self.mode = Mode::Pages(None);
                self.area.clear();
                self.dirty = true;
            }
            Action::Target(t) => self.nav.target = Some(t),
            Action::Map => self.centre = Centre::Map,
        }
    }

    fn current_page(&self) -> Option<&Page> {
        match self.view {
            Some(View::Page(p)) => Some(&self.guide.pages[p as usize]),
            _ => None,
        }
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        if self.drawn != self.view {
            self.drawn = self.view;
            ui.ctx().forget_all_images();
        }
        egui::Panel::left("pages")
            .resizable(true)
            .default_size(320.0)
            .show(ui, |ui| self.list(ui));
        let route: Vec<(String, MapCoord)> = self
            .current_page()
            .map(|p| p.coords().map(|(t, c)| (t.to_string(), c)).collect())
            .unwrap_or_default();
        let page = match self.view {
            Some(View::Page(p)) => Some(p),
            _ => None,
        };
        let open = egui::Panel::right("navigator")
            .resizable(true)
            .default_size(320.0)
            .show(ui, |ui| self.nav.ui(ui, &route, page))
            .inner;
        if let Some(p) = open {
            self.show(View::Page(p));
        }
        let mut action = None;
        egui::CentralPanel::default_margins().show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.centre, Centre::Page, "Page");
                ui.selectable_value(&mut self.centre, Centre::Map, "Route map");
            });
            ui.separator();
            match self.centre {
                Centre::Page => {
                    let ctx = page_view::Ctx {
                        guide: &self.guide,
                        images: &self.images,
                        item_quests: &self.item_quests,
                        target: self.nav.target.as_ref(),
                        can_go_back: !self.back.is_empty(),
                    };
                    action = page_view::show(ui, &ctx, self.view);
                }
                Centre::Map => {
                    if let Some(open) = self.map.ui(ui, &self.guide, &route, &mut self.nav) {
                        action = Some(Action::Show(View::Page(open)));
                    }
                }
            }
        });
        if let Some(a) = action {
            self.apply(a);
        }
    }
}

/// A page's plain text, for tooltips.
pub fn page_summary(page: &Page) -> String {
    page.lines
        .iter()
        .take(4)
        .map(html::line_text)
        .collect::<Vec<_>>()
        .join("\n")
}
