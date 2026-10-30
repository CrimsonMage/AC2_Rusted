//! The HUD windows that only appear during play, opened for a snapshot (`--hud-window NAME`).
//!
//! Each window is raised the way the game raises it: the notice the server (or the player's
//! script) would send, fed to the HUD's frameworks as a session event (`Session::notify`), the
//! double's own objects (a chest to open, a craft registry), or the player's own gesture on the
//! real UI (a drag out of the pack, a press on a button). The sample content is made up here;
//! nothing is read from a server and nothing is sent to one.

use std::time::{Duration, Instant};

use ac2_client::{ClientFrame, Command};
use ac2_gpu::Gpu;
use ac2_net::protocol::StringInfo;
use ac2_ui::{ToolbarChild, UI_POINTER_X, UI_POINTER_Y};
use ac2_weenie::{FellowInfo, FellowshipInfo, GameplayNotice, SessionEvent, Shop, ShopSale};
use legions_server_double::{CHEST, KNOWN_RECIPE, POTIONS, SWORD};

use crate::session::Session;

/// The left mouse button's UI action.
const LEFT: u32 = 0x4000_0004;
/// Seconds of server time per frame.
const STEP: f64 = 1.0 / 30.0;
/// How long a window may take to show (the HUD's loads run on workers).
const PATIENCE: Duration = Duration::from_secs(60);
/// Real item icons (the DATs') for the sample sales.
const ICONS: [u32; 4] = [
    legions_server_double::ICON_POTION,
    legions_server_double::ICON_SWORD,
    legions_server_double::ICON_RATION,
    legions_server_double::ICON_SHIELD,
];

/// A window `--hud-window` opens.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HudWindow {
    Confirm,
    Message,
    Wait,
    Resurrect,
    Loot,
    Stack,
    Trade,
    Shop,
    Catalog,
    Consignment,
    NewSale,
    Ingredient,
    Craft,
    Book,
    Help,
    Guide,
    Tracker,
    Pets,
    Fellowship,
    FellowMini,
    ChatSettings,
    EditLayout,
    Targeting,
}

/// Every window with the name `--hud-window` takes.
pub const ALL: [(&str, HudWindow); 23] = [
    ("confirm", HudWindow::Confirm),
    ("message", HudWindow::Message),
    ("wait", HudWindow::Wait),
    ("resurrect", HudWindow::Resurrect),
    ("loot", HudWindow::Loot),
    ("stack", HudWindow::Stack),
    ("trade", HudWindow::Trade),
    ("shop", HudWindow::Shop),
    ("catalog", HudWindow::Catalog),
    ("consignment", HudWindow::Consignment),
    ("new-sale", HudWindow::NewSale),
    ("ingredient", HudWindow::Ingredient),
    ("craft", HudWindow::Craft),
    ("book", HudWindow::Book),
    ("help", HudWindow::Help),
    ("guide", HudWindow::Guide),
    ("tracker", HudWindow::Tracker),
    ("pets", HudWindow::Pets),
    ("fellowship", HudWindow::Fellowship),
    ("fellow-mini", HudWindow::FellowMini),
    ("chat-settings", HudWindow::ChatSettings),
    ("edit-layout", HudWindow::EditLayout),
    ("targeting", HudWindow::Targeting),
];

impl HudWindow {
    /// The window a command-line name asks for (any case).
    pub fn parse(name: &str) -> Option<Self> {
        let name = name.to_ascii_lowercase();
        ALL.iter().find(|(n, _)| *n == name).map(|&(_, w)| w)
    }
}

/// The session, view and frame a window is opened on.
pub struct Driver<'a> {
    pub session: &'a mut Session,
    pub view: &'a mut ac2_client::view::ClientView,
    pub gpu: &'a Gpu,
    pub frame: &'a mut ClientFrame,
}

fn text(s: &str) -> StringInfo {
    StringInfo::literal(s)
}

impl Driver<'_> {
    /// `frames` frames of the session and the view.
    pub fn run(&mut self, frames: usize) {
        for _ in 0..frames {
            self.session.frame(self.view, self.frame, STEP);
            self.view
                .update(self.gpu, self.frame, Duration::from_millis(4));
        }
    }

    /// One frame with the notices as the session's, then a few more to settle.
    fn notify(&mut self, notices: Vec<GameplayNotice>) {
        let events: Vec<SessionEvent> = notices.into_iter().map(SessionEvent::Notice).collect();
        self.session.notify(self.view, self.frame, &events);
        self.view
            .update(self.gpu, self.frame, Duration::from_millis(4));
        self.run(8);
    }

    /// Frames until `ready` holds; false when the HUD never got there.
    fn wait(&mut self, ready: impl Fn(&Self) -> bool) -> bool {
        let started = Instant::now();
        while started.elapsed() < PATIENCE {
            if ready(self) {
                return true;
            }
            self.run(1);
            std::thread::sleep(Duration::from_millis(5));
        }
        false
    }

    /// One frame with the UI `actions` into the HUD.
    fn input(&mut self, actions: &[(u32, f32, bool)]) {
        self.view.ui_input(actions, ac2_ui::Modifiers::default());
        self.run(1);
    }

    fn point(&mut self, (x, y): (i32, i32)) {
        self.input(&[
            (UI_POINTER_X, x as f32, true),
            (UI_POINTER_Y, y as f32, true),
        ]);
    }

    /// A click: the pointer there, the left button down and up.
    fn click(&mut self, at: (i32, i32)) {
        self.point(at);
        self.input(&[(LEFT, 1.0, true)]);
        self.input(&[(LEFT, 1.0, false)]);
        self.run(4);
    }

    /// A drag with the left button from `from` to `to`.
    fn drag(&mut self, from: (i32, i32), to: (i32, i32)) {
        self.point(from);
        self.input(&[(LEFT, 1.0, true)]);
        for step in 1..=4 {
            let t = step as f32 / 4.0;
            let x = from.0 as f32 + (to.0 - from.0) as f32 * t;
            let y = from.1 as f32 + (to.1 - from.1) as f32 * t;
            self.point((x.round() as i32, y.round() as i32));
        }
        self.input(&[(LEFT, 1.0, false)]);
        self.run(8);
    }

    /// Presses the first shown element called `name`; false when there is none.
    fn press(&mut self, name: &str) -> bool {
        match self.view.ui_element_center(name) {
            Some(at) => {
                self.click(at);
                true
            }
            None => false,
        }
    }

    /// Opens `window` with its sample content; false when it did not come up.
    pub fn open(&mut self, window: HudWindow) -> bool {
        // The HUD's string tables load on a worker; the windows' texts need them.
        self.wait(|d| d.view.built_panels().contains(&"strings"));
        let opened = match window {
            HudWindow::Confirm => {
                self.notify(vec![GameplayNotice::PopupYesNo {
                    object: 0x8000_0000_0000_0A09,
                    context: 1,
                    text: text("Do you want to accept the quest \"Rats in the Cellar\"?"),
                }]);
                true
            }
            HudWindow::Message => {
                self.notify(vec![GameplayNotice::DisplayMessage {
                    text: text("You have reached the end of the road. Turn back, traveller."),
                    topmost: false,
                }]);
                true
            }
            HudWindow::Wait => self.wait_dialog(),
            HudWindow::Resurrect => {
                self.notify(vec![GameplayNotice::UpdateDeathState(true)]);
                true
            }
            HudWindow::Loot => self.loot(),
            HudWindow::Stack => self.stack(),
            HudWindow::Trade => self.trade(),
            HudWindow::Shop => {
                self.notify(vec![GameplayNotice::ViewStore(Box::new(shop(
                    "Sheila's Wares",
                    "Potions and such",
                    &[
                        ("Healing Potion", 35.0, -1),
                        ("Iron Sword", 250.0, 2),
                        ("Traveller's Rations", 12.5, -1),
                        ("Gem of Light", 900.0, 1),
                    ],
                )))]);
                true
            }
            HudWindow::Catalog => {
                let store = |id, name: &str, size| Shop {
                    id,
                    name: text(name),
                    desc: text("A store"),
                    store_size: size,
                    ..Shop::default()
                };
                self.notify(vec![GameplayNotice::ViewCatalog {
                    header: text("Stores of the Market"),
                    shops: vec![
                        store(0x6F00_3200, "Sheila's Wares", 5),
                        store(0x6F00_3201, "Bob's Bits", 3),
                        store(0x6F00_3202, "The Gilded Hilt", 0),
                    ],
                }]);
                true
            }
            HudWindow::Consignment => {
                self.consignment();
                true
            }
            HudWindow::NewSale => {
                self.consignment();
                self.press("NewConsignmentButton")
            }
            HudWindow::Ingredient => self.ingredient(),
            HudWindow::Craft => {
                self.session.server().borrow_mut().send_craft_registry();
                self.run(8);
                self.view.show_panel(ToolbarChild::Skill, true);
                self.run(8);
                true
            }
            HudWindow::Book => {
                self.notify(vec![GameplayNotice::UseBook {
                    show_controls: true,
                    mode: ac2_ui::tokenize("Dialog"),
                    title: "The Traveller's Almanac".into(),
                    text: "Of the roads of Dereth.<BR>The market road runs north from the \
                           village to the standing stones.<BR>Beware the cellars after dark."
                        .into(),
                    image: 0,
                }]);
                true
            }
            HudWindow::Help => self.help(),
            HudWindow::Guide => {
                self.view.toggle_guide();
                self.run(30);
                true
            }
            HudWindow::Tracker => {
                self.view.toggle_tracker();
                self.run(30);
                true
            }
            HudWindow::Pets => {
                let pet = |id, name: &str, mode| GameplayNotice::AddPet {
                    pet: id,
                    name: text(name),
                    icon: legions_server_double::ICON_SWORD,
                    mode,
                };
                self.notify(vec![
                    pet(0x8000_0000_0000_0A01, "Tiny", 0x4000_0004),
                    pet(0x8000_0000_0000_0A02, "Fang", 0x4000_0004),
                ]);
                true
            }
            HudWindow::Fellowship => self.fellowship(false),
            HudWindow::FellowMini => self.fellowship(true),
            HudWindow::ChatSettings => self.press("OptionsButton"),
            HudWindow::EditLayout => {
                self.view.show_panel(ToolbarChild::Option, true);
                self.run(8);
                let pressed = self.press("ModernEditLayoutButton");
                self.view.show_panel(ToolbarChild::Option, false);
                self.run(8);
                pressed
            }
            HudWindow::Targeting => {
                self.notify(vec![GameplayNotice::SetTargetingPointer(true)]);
                true
            }
        };
        // The windows' images and icons load on workers: frames with a pause between them.
        for _ in 0..60 {
            self.run(1);
            std::thread::sleep(Duration::from_millis(10));
        }
        opened
    }

    /// The log off's please-wait dialog: the logout key raises it; the log off itself is not
    /// sent to the double.
    fn wait_dialog(&mut self) -> bool {
        self.run(8);
        self.input(&[(ac2_ui::LOGOUT_ACTION, 1.0, true)]);
        self.input(&[(ac2_ui::LOGOUT_ACTION, 1.0, false)]);
        self.session.forget_commands();
        true
    }

    /// The double's chest, opened as a use of it would.
    fn loot(&mut self) -> bool {
        self.session.server().borrow_mut().spawn_chest();
        self.run(8);
        self.session.command(Command::SetContainerOpen {
            container: CHEST,
            open: true,
        });
        self.wait(|d| {
            d.view
                .external_container_items()
                .is_some_and(|(_, items)| items.len() == 2)
        })
    }

    /// The potions' stack dragged out of the pack onto the world: the stack asks how many.
    fn stack(&mut self) -> bool {
        self.view.show_panel(ToolbarChild::Inventory, true);
        let ready = self.wait(|d| {
            d.view
                .item_icon_center("PackMainContainer", POTIONS)
                .is_some()
        });
        let Some(from) = ready
            .then(|| self.view.item_icon_center("PackMainContainer", POTIONS))
            .flatten()
        else {
            return false;
        };
        self.drag(from, (300, 250));
        self.view.ui_element_center("StackOkButton").is_some()
    }

    /// A trade with a made-up partner, the player's potions on the table.
    fn trade(&mut self) -> bool {
        let Some(player) = self.session.player() else {
            return false;
        };
        self.notify(vec![
            GameplayNotice::TradeRegistered {
                initiator: player,
                partner: 0x8000_0000_0000_0B01,
                partner_name: text("Ann the Trader"),
            },
            GameplayNotice::TradeOpened,
        ]);
        self.view.show_panel(ToolbarChild::Inventory, true);
        self.run(16);
        self.notify(vec![GameplayNotice::TradeOffered {
            source: player,
            item: POTIONS,
            amount: 5,
        }]);
        true
    }

    fn consignment(&mut self) {
        self.notify(vec![GameplayNotice::ViewConsignment(Box::new(shop(
            "Sheila's Consignment",
            "Goods on consignment",
            &[("Healing Potions", 20.0, 5), ("Iron Sword", 250.0, 1)],
        )))]);
    }

    /// The craft panel's ingredient window for the double's known recipe.
    fn ingredient(&mut self) -> bool {
        self.session.server().borrow_mut().send_craft_registry();
        self.run(8);
        self.notify(vec![GameplayNotice::OpenIngredientPanel(KNOWN_RECIPE)]);
        true
    }

    /// A panel's help button opens the help window with the panel's page (needs the content
    /// directory); the panel is closed again.
    fn help(&mut self) -> bool {
        self.view.show_panel(ToolbarChild::Skill, true);
        self.run(8);
        let button = self
            .view
            .element_box("SkillHelpButton")
            .map(|b| ((b.x0 + b.x1) / 2, (b.y0 + b.y1) / 2));
        if let Some(at) = button {
            self.click(at);
        }
        let shown = button.is_some() && self.wait(|d| d.view.help_window_text().is_some());
        self.view.show_panel(ToolbarChild::Skill, false);
        shown
    }

    /// A fellowship of three with the player leading: the Social panel's fellowship page, or with
    /// `mini` the fellow mini panel its button opens.
    fn fellowship(&mut self, mini: bool) -> bool {
        let Some(player) = self.session.player() else {
            return false;
        };
        let member = |name: &str, level, health, vigor| FellowInfo {
            name: text(name),
            level,
            health,
            max_health: 50,
            vigor,
            max_vigor: 40,
            ..FellowInfo::default()
        };
        self.notify(vec![GameplayNotice::UpdateFellowship(FellowshipInfo {
            name: "The Wayfarers".into(),
            leader: player,
            social: false,
            looting_method: 0,
            fellows: vec![
                (player, member("Explorer", 12, 44, 30)),
                (0x8000_0000_0000_0B01, member("Ann", 10, 31, 22)),
                (0x8000_0000_0000_0B02, member("Brand", 14, 18, 40)),
            ],
        })]);
        self.view.show_panel(ToolbarChild::Social, true);
        self.run(16);
        if mini {
            let pressed = self.press("FellowshipMiniButton");
            self.view.show_panel(ToolbarChild::Social, false);
            return pressed;
        }
        true
    }
}

/// A store with `(name, price, stock)` sales, ordered as listed.
fn shop(name: &str, desc: &str, sales: &[(&str, f32, i32)]) -> Shop {
    let items = [POTIONS, SWORD, 0, 0];
    Shop {
        id: 0x6F00_0001,
        name: text(name),
        desc: text(desc),
        sales: sales
            .iter()
            .enumerate()
            .map(|(i, &(item, price, stock))| ShopSale {
                ordinal: i as u32,
                item: items[i % items.len()],
                item_did: 0x0600_1000 + i as u32,
                icon_did: ICONS[i % ICONS.len()],
                item_desc: text(item),
                cost_desc: text("gold"),
                cost_quantity: price,
                stock,
                ..ShopSale::default()
            })
            .collect(),
        store_size: sales.len() as i32,
        ..Shop::default()
    }
}
