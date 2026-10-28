//! The build path: `HudView::new` starts the workers that load the string tables and build
//! the HUD, and `arrived` takes the finished build into the view.

use std::sync::{mpsc, Arc};

use ac2_db::ResourceStore;
use ac2_gpu::{Gpu, UiOverlay};
use ac2_ui::{ChatSettings, GameplayHud};

use super::{Built, HudView};

/// The help pages from the content directory (`ac2_client::content_dir`, the DVD's
/// `ProgramFiles/Content`). `None` without one: the help window then has no pages, and that is
/// logged once.
fn help_pages(store: &ac2_db::ResourceStore) -> ac2_db::ResourceResult<Option<ac2_ui::HelpPages>> {
    static NO_CONTENT: std::sync::Once = std::sync::Once::new();
    match crate::help_content::content_dir() {
        Some(dir) => ac2_ui::load_help_pages(store, &dir),
        None => {
            NO_CONTENT.call_once(|| {
                eprintln!(
                    "help: no content directory (--content-dir, a Content folder beside the DATs, \
                     or AC2_CONTENT_DIR); the help window has no pages"
                );
            });
            Ok(None)
        }
    }
}

impl HudView {
    pub(crate) fn new(
        store: Arc<ResourceStore>,
        gpu: &Gpu,
        format: wgpu::TextureFormat,
        size: (u32, u32),
    ) -> Self {
        let chat_text = ac2_ui::chat_text::ChatText::new((*store).clone());
        let images = ac2_ui::UiImageLoader::new(Arc::clone(&store));
        let (send_strings, pending_strings) = mpsc::channel();
        let strings_store = Arc::clone(&store);
        std::thread::Builder::new()
            .name("ac2 ui strings".into())
            .spawn(move || {
                let tables = ac2_db::PreloadedStringTables::load_all(&strings_store, 0x5800_0000);
                let _ = send_strings.send(ac2_ui::UiStringTables::new(&strings_store, tables));
            })
            .expect("spawn the UI strings loader");
        let (send, receive) = mpsc::channel();
        let scale = crate::ui_scale::UiScale::fill(size);
        let (width, height) = (scale.display.0 as i32, scale.display.1 as i32);
        std::thread::Builder::new()
            .name("ac2 hud".into())
            .spawn(move || {
                let built =
                    ac2_ui::build_gameplay_hud(&store, width, height).and_then(|mut hud| {
                        // The text elements copy, cut and paste through the OS clipboard.
                        hud.tree.set_text_clipboard(crate::clipboard::shared());
                        // `CRegionDesc::GetGameMap`: the map image the navigation panel shows.
                        if let (Some(nav), Some(map)) = (
                            hud.navigation.as_mut(),
                            ac2_world::ac2_region_game_map(&store)?,
                        ) {
                            nav.set_game_map(&mut hud.tree, map.raw());
                        }
                        // `CRegionDesc::GetCurrentDetailMapTable` and its `CDetailMapDesc`: the
                        // detail view's tiles and their map notes, read on this worker.
                        if let Some(table) = ac2_world::ac2_region_detail_map_table(&store)? {
                            if let (Some(nav), Some(map)) = (
                                hud.navigation.as_mut(),
                                ac2_world::ac2_detail_map(&store, table)?,
                            ) {
                                nav.set_detail_map(std::sync::Arc::new(map));
                            }
                        }
                        // `KeyStone::OpenHelp`'s pages: the content directory's KSML files, read here
                        // so the window never reads a file on the frame thread.
                        hud.help_pages = help_pages(&store)?;
                        let images = ac2_ui::load_ui_images(&store, &hud.tree)?;
                        Ok((hud, images))
                    });
                let _ = send.send(built);
            })
            .expect("spawn the HUD loader");
        Self {
            pending: Some(receive),
            hud: None,
            panels: ac2_ui::Hud::default(),
            chat: None,
            toolbar: None,
            shortcuts: None,
            player_race: None,
            toolbar_layout: None,
            item_selection: 0,
            dialogs: None,
            death: ac2_ui::DeathPanel::default(),
            game_dialogs: ac2_ui::GameplayDialogs::default(),
            popup_backlog: Vec::new(),
            display_backlog: Vec::new(),
            radar: None,
            radar_cell: None,
            radar_coordinates: None,
            radar_uploaded: None,
            navigation_time: None,
            navigation_tooltips: Vec::new(),
            navigation: None,
            overview: None,
            book: None,
            help: None,
            help_pages: None,
            story: None,
            shift_held: false,
            gameplay_shown: true,
            story_hid_gameplay: None,
            hud_key_pending: false,
            use_key_pending: false,
            player_messages: Vec::new(),
            strings: None,
            pending_strings: Some(pending_strings),
            player_loaded: false,
            early_lines: Vec::new(),
            chat_text,
            images,
            overlay: {
                let mut overlay = UiOverlay::new(gpu, format);
                overlay.set_smooth(!scale.is_identity());
                overlay
            },
            layer_roots: Vec::new(),
            size,
            scale,
            dirty: false,
            sounds: Vec::new(),
        }
    }

    /// Takes a finished build, then recomposes if anything changed.
    pub(super) fn arrived(&mut self, built: Built) {
        match built {
            Ok((
                GameplayHud {
                    mut tree,
                    reticle,
                    names,
                    damage,
                    mut chat,
                    toolbar,
                    shortcuts,
                    skills,
                    craft,
                    ingredient,
                    examination,
                    allegiance,
                    fellowship,
                    friends,
                    assistance,
                    social,
                    mut mini_fellow,
                    mut pets,
                    quests,
                    vitals,
                    effects,
                    indicators,
                    toolbar_layout,
                    dialogs,
                    stack,
                    external_container,
                    radar,
                    secure_trade,
                    character,
                    chat_options,
                    ui_table,
                    pref_table,
                    navigation,
                    overview,
                    shop,
                    catalog,
                    new_sale,
                    consignment,
                    book,
                    help,
                    help_pages,
                    story,
                },
                images,
            )) => {
                tree.set_display(self.scale.display.0 as i32, self.scale.display.1 as i32);
                if let Some(chat) = &mut chat {
                    // The player's options are not read yet: the chat starts on
                    // `GameplayOptionsProfile::Clear`'s (no filters, default fonts).
                    chat.reload_chat_settings(&mut tree, &ChatSettings::default());
                    for (kind, text) in self.early_lines.drain(..) {
                        chat.display(kind, &text);
                    }
                }
                // `gmMiniFellowUI::LocationApplied` (no saved location yet): the shown state again.
                if let Some(mini) = mini_fellow.as_mut() {
                    mini.location_applied(&mut tree);
                }
                // `gmPetUI`: shown with the HUD; its state follows the pets listed.
                if let Some(pets) = pets.as_mut() {
                    pets.show(&mut tree, true);
                }
                self.panels.mini_fellow = mini_fellow;
                self.panels.pets = pets;
                self.hud = Some((tree, images));
                self.panels = ac2_ui::Hud::new(reticle, names, damage);
                self.chat = chat;
                self.toolbar = Some(toolbar);
                self.show_toolbar(true);
                self.shortcuts = shortcuts;
                self.panels.skills = skills;
                self.panels.craft = craft;
                self.panels.ingredient = ingredient;
                self.panels.examination = examination;
                self.panels.allegiance = allegiance;
                self.panels.fellowship = fellowship;
                self.panels.friends = friends;
                self.panels.assistance = assistance;
                self.panels.social = social;
                self.panels.quests = quests;
                self.panels.vitals = Some(vitals);
                self.panels.effects = effects;
                self.panels.net = Some(indicators.net);
                self.panels.vitae = Some(indicators.vitae);
                self.panels.pk = Some(indicators.pk);
                self.toolbar_layout = toolbar_layout;
                self.dialogs = Some(dialogs);
                self.panels.stack = stack;
                self.panels.external_container = external_container;
                self.panels.secure_trade = secure_trade;
                self.panels.character = character;
                self.panels.chat_options = chat_options;
                self.panels.options.set_tables(ui_table, pref_table);
                self.panels.shop = shop;
                self.panels.catalog = catalog;
                self.panels.new_sale = new_sale;
                self.panels.consignment = consignment;
                self.radar = Some(radar);
                self.navigation = navigation;
                self.overview = overview;
                self.book = book;
                self.help = help;
                self.help_pages = help_pages;
                self.story = story;
                self.dirty = true;
            }
            Err(error) => eprintln!("ac2: the HUD did not load: {error}"),
        }
    }
}
