//! `ac2-launcher`: the launcher window, in ThwargLauncher's way. The player's servers (a local
//! Pyrealium to start with) and the published server list are on the left; a server is added by its
//! name, connect string (`host:port`) and type. On the right are the chosen server, the account
//! and the password, and Play: the launcher builds the client's command line for the server's
//! type and starts it (`ac2_launcher::launch`). The settings live in the AC2 configuration
//! directory; passwords are never kept.

use std::sync::{Arc, Mutex};
use std::time::Duration;

use ac2_launcher::launch::{self, Started};
use ac2_launcher::servers::{self, Emu, Server, Settings, Source};
use eframe::egui;

/// How long the launcher waits for the client to take its ticket.
const HANDOFF_WAIT: Duration = Duration::from_secs(60);

mod cli;

fn main() -> eframe::Result {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if let Some(outcome) = cli::run(&args) {
        match outcome {
            Ok(text) => println!("{text}"),
            Err(e) => {
                eprintln!("ac2-launcher: {e}");
                std::process::exit(1);
            }
        }
        return Ok(());
    }
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("AC2 Launcher")
            .with_inner_size([880.0, 560.0])
            .with_min_inner_size([660.0, 440.0]),
        ..Default::default()
    };
    eframe::run_native(
        "ac2-launcher",
        options,
        Box::new(|cc| {
            cc.egui_ctx.set_visuals(theme());
            Ok(Box::new(Launcher::new(Settings::load())))
        }),
    )
}

/// The modern HUD's colours: dark teal glass, bronze lines and gold accents.
fn theme() -> egui::Visuals {
    let mut v = egui::Visuals::dark();
    let glass = egui::Color32::from_rgb(14, 34, 38);
    let bronze = egui::Color32::from_rgb(120, 92, 52);
    let gold = egui::Color32::from_rgb(217, 194, 122);
    v.panel_fill = glass;
    v.window_fill = glass;
    v.extreme_bg_color = egui::Color32::from_rgb(8, 22, 25);
    v.faint_bg_color = egui::Color32::from_rgb(18, 42, 46);
    v.selection.bg_fill = egui::Color32::from_rgb(70, 60, 30);
    v.selection.stroke = egui::Stroke::new(1.0, gold);
    v.hyperlink_color = gold;
    v.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0, bronze);
    v.widgets.inactive.bg_fill = egui::Color32::from_rgb(24, 52, 56);
    v.widgets.inactive.weak_bg_fill = egui::Color32::from_rgb(24, 52, 56);
    v.widgets.hovered.bg_stroke = egui::Stroke::new(1.0, gold);
    v.widgets.active.bg_stroke = egui::Stroke::new(1.0, gold);
    v
}

/// Which list the chosen server is in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Pick {
    Mine(usize),
    Listed(usize),
}

/// The Add Server form (ThwargLauncher's: a name, a connect string and a type).
#[derive(Default)]
struct AddForm {
    open: bool,
    name: String,
    connect: String,
    emu: Option<Emu>,
    login_server: String,
    note: String,
}

struct Launcher {
    settings: Settings,
    listed: Vec<Server>,
    list_note: String,
    pick: Option<Pick>,
    account: String,
    password: String,
    status: Arc<Mutex<String>>,
    busy: Arc<Mutex<bool>>,
    show_settings: bool,
    add: AddForm,
}

impl Launcher {
    fn new(settings: Settings) -> Self {
        let pick = settings
            .selected
            .as_ref()
            .and_then(|key| settings.favorites.iter().position(|f| &f.key() == key))
            .or((!settings.favorites.is_empty()).then_some(0))
            .map(Pick::Mine);
        let show_settings = settings.dat_dir.is_empty();
        let mut launcher = Self {
            settings,
            listed: Vec::new(),
            list_note: String::new(),
            pick,
            account: String::new(),
            password: String::new(),
            status: Arc::new(Mutex::new(String::new())),
            busy: Arc::new(Mutex::new(false)),
            show_settings,
            add: AddForm::default(),
        };
        launcher.load_account();
        if !launcher.settings.list_url.is_empty() {
            launcher.refresh_list();
        }
        launcher
    }

    fn server(&self) -> Option<&Server> {
        match self.pick? {
            Pick::Mine(i) => self.settings.favorites.get(i),
            Pick::Listed(i) => self.listed.get(i),
        }
    }

    fn choose(&mut self, pick: Pick) {
        self.pick = Some(pick);
        self.password.clear();
        self.load_account();
        if let Pick::Mine(i) = pick {
            self.settings.selected = self.settings.favorites.get(i).map(Server::key);
            self.save();
        }
    }

    fn load_account(&mut self) {
        self.account = self
            .server()
            .and_then(|s| self.settings.accounts.get(&s.key()).cloned())
            .unwrap_or_default();
    }

    fn save(&mut self) {
        if let Err(e) = self.settings.save() {
            *self.status.lock().unwrap() = format!("settings not saved: {e}");
        }
    }

    fn refresh_list(&mut self) {
        match servers::fetch_list(self.settings.list_url.trim()) {
            Ok((servers, problems)) => {
                self.list_note = if problems.is_empty() {
                    format!("{} servers listed", servers.len())
                } else {
                    format!(
                        "{} servers listed; skipped: {}",
                        servers.len(),
                        problems.join("; ")
                    )
                };
                self.listed = servers;
            }
            Err(e) => {
                self.list_note = e;
                self.listed.clear();
            }
        }
        if matches!(self.pick, Some(Pick::Listed(_))) {
            self.pick = None;
        }
    }

    /// Signs in and starts the client on a worker, so the window stays live.
    fn play(&mut self) {
        let Some(server) = self.server().cloned() else {
            return;
        };
        if *self.busy.lock().unwrap() {
            return;
        }
        let account = self.account.trim().to_owned();
        if !account.is_empty() {
            self.settings.accounts.insert(server.key(), account.clone());
            self.save();
        }
        let password = std::mem::take(&mut self.password);
        let settings = self.settings.clone();
        let (status, busy) = (Arc::clone(&self.status), Arc::clone(&self.busy));
        *busy.lock().unwrap() = true;
        *status.lock().unwrap() = format!("Starting {}...", server.name);
        std::thread::spawn(move || {
            let outcome = launch::prepare(&server, &settings, &account, &password)
                .and_then(|plan| launch::start(&plan, HANDOFF_WAIT));
            *status.lock().unwrap() = match outcome {
                Ok(Started::Running) => format!("{} is starting in the client.", server.name),
                Ok(Started::Exited(why)) => format!("The client stopped ({why})."),
                Ok(Started::NoHandoff) => {
                    "The client did not take its sign-in; it was removed. Try again.".into()
                }
                Err(e) => e,
            };
            *busy.lock().unwrap() = false;
        });
    }

    fn add_form(&mut self, ui: &mut egui::Ui) -> Option<Pick> {
        let form = &mut self.add;
        egui::Grid::new("add").num_columns(2).show(ui, |ui| {
            ui.label("Name");
            ui.text_edit_singleline(&mut form.name);
            ui.end_row();
            ui.label("Connect string");
            ui.add(egui::TextEdit::singleline(&mut form.connect).hint_text("host:port"));
            ui.end_row();
            ui.label("Type");
            egui::ComboBox::from_id_salt("add_emu")
                .selected_text(form.emu.map_or("Choose", type_name))
                .show_ui(ui, |ui| {
                    for emu in Emu::ALL {
                        ui.selectable_value(&mut form.emu, Some(emu), type_name(emu));
                    }
                });
            ui.end_row();
            if form.emu.is_some_and(Emu::has_account_service) {
                ui.label("Login server");
                ui.add(
                    egui::TextEdit::singleline(&mut form.login_server)
                        .hint_text("none (developer account)"),
                );
                ui.end_row();
            }
        });
        let mut added = None;
        ui.horizontal(|ui| {
            if ui.button("Add").clicked() {
                match form.emu {
                    None => form.note = "choose the server's type".into(),
                    Some(emu) => {
                        let mut server = Server::new(&form.name, &form.connect, emu);
                        if emu.has_account_service() {
                            server.login_server_url = form.login_server.trim().to_owned();
                        }
                        match server.problem() {
                            Some(p) => form.note = p,
                            None => {
                                let key = server.key();
                                self.settings.add_favorite(server);
                                added = self
                                    .settings
                                    .favorites
                                    .iter()
                                    .position(|f| f.key() == key)
                                    .map(Pick::Mine);
                                *form = AddForm::default();
                            }
                        }
                    }
                }
            }
            if ui.button("Cancel").clicked() {
                *form = AddForm::default();
            }
        });
        if !form.note.is_empty() {
            ui.colored_label(egui::Color32::LIGHT_RED, &form.note);
        }
        if added.is_some() {
            self.save();
        }
        added
    }

    fn server_lists(&mut self, ui: &mut egui::Ui) {
        ui.heading("My servers");
        let mut chosen = None;
        for (i, server) in self.settings.favorites.iter().enumerate() {
            let picked = self.pick == Some(Pick::Mine(i));
            if ui
                .selectable_label(
                    picked,
                    format!("★ {}  ({})", server.name, type_name(server.emu)),
                )
                .clicked()
            {
                chosen = Some(Pick::Mine(i));
            }
        }
        if self.add.open {
            ui.separator();
            ui.label("Add a server");
            chosen = self.add_form(ui).or(chosen);
        } else {
            ui.horizontal(|ui| {
                if ui.button("Add server").clicked() {
                    self.add.open = true;
                }
                if let Some(Pick::Mine(i)) = self.pick {
                    if ui.button("Remove").clicked() && i < self.settings.favorites.len() {
                        self.settings.favorites.remove(i);
                        self.pick = None;
                        self.save();
                    }
                }
            });
        }
        ui.separator();
        ui.heading("Server list");
        if self.settings.list_url.is_empty() {
            ui.weak("No server list chosen (Settings).");
        } else {
            if ui.button("Refresh").clicked() {
                self.refresh_list();
            }
            ui.weak(&self.list_note);
        }
        for (i, server) in self.listed.iter().enumerate() {
            let picked = self.pick == Some(Pick::Listed(i));
            let star = if self.settings.is_favorite(server) {
                "★ "
            } else {
                ""
            };
            if ui
                .selectable_label(
                    picked,
                    format!("{star}{}  ({})", server.name, type_name(server.emu)),
                )
                .clicked()
            {
                chosen = Some(Pick::Listed(i));
            }
        }
        if let Some(pick) = chosen {
            self.choose(pick);
        }
    }

    fn details(&mut self, ui: &mut egui::Ui) {
        let Some(pick) = self.pick else {
            ui.weak("Choose a server.");
            return;
        };
        let mut changed = false;
        match pick {
            Pick::Mine(i) => {
                if let Some(server) = self.settings.favorites.get_mut(i) {
                    changed = edit_server(ui, server);
                }
            }
            Pick::Listed(i) => {
                let Some(server) = self.listed.get(i).cloned() else {
                    return;
                };
                show_server(ui, &server);
                if !self.settings.is_favorite(&server) && ui.button("★ Add to my servers").clicked()
                {
                    self.settings.add_favorite(server);
                    changed = true;
                }
            }
        }
        if changed {
            self.save();
        }
        let Some(server) = self.server().cloned() else {
            return;
        };
        ui.separator();
        let password = launch::needs_password(&server);
        egui::Grid::new("signin").num_columns(2).show(ui, |ui| {
            ui.label("Account");
            let hint = if password { "" } else { "local" };
            ui.add(egui::TextEdit::singleline(&mut self.account).hint_text(hint));
            ui.end_row();
            if password {
                ui.label("Password");
                ui.add(egui::TextEdit::singleline(&mut self.password).password(true));
                ui.end_row();
            }
        });
        if let Some(problem) = server.problem() {
            ui.colored_label(egui::Color32::LIGHT_RED, problem);
        }
        ui.add_space(8.0);
        let busy = *self.busy.lock().unwrap();
        let play = ui.add_enabled(
            !busy && server.problem().is_none(),
            egui::Button::new(egui::RichText::new("  Play  ").size(20.0)),
        );
        if play.clicked() {
            self.play();
        }
        let status = self.status.lock().unwrap().clone();
        if !status.is_empty() {
            ui.label(status);
        }
    }

    fn settings_panel(&mut self, ui: &mut egui::Ui) {
        let mut changed = false;
        egui::Grid::new("settings").num_columns(2).show(ui, |ui| {
            ui.label("DAT folder");
            changed |= ui
                .add(egui::TextEdit::singleline(&mut self.settings.dat_dir).desired_width(420.0))
                .lost_focus();
            ui.end_row();
            ui.label("Client program");
            let beside = launch::client_program(&Settings::default());
            changed |= ui
                .add(
                    egui::TextEdit::singleline(&mut self.settings.client)
                        .hint_text(beside.display().to_string())
                        .desired_width(420.0),
                )
                .lost_focus();
            ui.end_row();
            ui.label("Server list");
            changed |= ui
                .add(
                    egui::TextEdit::singleline(&mut self.settings.list_url)
                        .hint_text("a web address or a file (docs/server-list.md)")
                        .desired_width(420.0),
                )
                .lost_focus();
            ui.end_row();
        });
        if changed {
            self.save();
        }
    }
}

fn type_name(emu: Emu) -> &'static str {
    match emu {
        Emu::Pyrealium => "Pyrealium",
        Emu::Ac2e => "AC2E",
        Emu::Relaunch => "Relaunch",
    }
}

/// The editable fields of one of the player's servers. Returns whether anything changed.
fn edit_server(ui: &mut egui::Ui, server: &mut Server) -> bool {
    let before = server.clone();
    egui::Grid::new("server").num_columns(2).show(ui, |ui| {
        ui.label("Name");
        ui.text_edit_singleline(&mut server.name);
        ui.end_row();
        ui.label("Connect string");
        ui.add(egui::TextEdit::singleline(&mut server.connect).hint_text("host:port"));
        ui.end_row();
        ui.label("Type");
        egui::ComboBox::from_id_salt("emu")
            .selected_text(type_name(server.emu))
            .show_ui(ui, |ui| {
                for emu in Emu::ALL {
                    ui.selectable_value(&mut server.emu, emu, type_name(emu));
                }
            });
        ui.end_row();
        if server.emu.has_account_service() {
            ui.label("Login server");
            ui.add(
                egui::TextEdit::singleline(&mut server.login_server_url)
                    .hint_text("none (developer account)"),
            );
            ui.end_row();
        }
        ui.label("Notes");
        ui.text_edit_singleline(&mut server.description);
        ui.end_row();
    });
    *server != before
}

/// A listed server, read-only.
fn show_server(ui: &mut egui::Ui, server: &Server) {
    ui.heading(&server.name);
    if !server.description.is_empty() {
        ui.label(&server.description);
    }
    egui::Grid::new("listed").num_columns(2).show(ui, |ui| {
        ui.label("Connect string");
        ui.label(&server.connect);
        ui.end_row();
        ui.label("Type");
        ui.label(type_name(server.emu));
        ui.end_row();
        if !server.status.is_empty() {
            ui.label("Status");
            ui.label(&server.status);
            ui.end_row();
        }
    });
    for link in [&server.website_url, &server.discord_url] {
        if !link.is_empty() {
            ui.hyperlink(link);
        }
    }
    debug_assert_eq!(server.source, Source::Published);
}

impl eframe::App for Launcher {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        if *self.busy.lock().unwrap() {
            ui.ctx().request_repaint_after(Duration::from_millis(200));
        }
        egui::CentralPanel::default().show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Asheron's Call 2");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.toggle_value(&mut self.show_settings, "Settings");
                });
            });
            if self.show_settings {
                self.settings_panel(ui);
            }
            ui.separator();
            ui.horizontal_top(|ui| {
                ui.vertical(|ui| {
                    ui.set_width(280.0);
                    egui::ScrollArea::vertical()
                        .id_salt("lists")
                        .show(ui, |ui| self.server_lists(ui));
                });
                ui.separator();
                ui.vertical(|ui| {
                    egui::ScrollArea::vertical()
                        .id_salt("details")
                        .show(ui, |ui| self.details(ui));
                });
            });
        });
    }
}
