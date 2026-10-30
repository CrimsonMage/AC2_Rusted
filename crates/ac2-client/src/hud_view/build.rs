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

/// Released records the world map's own store keeps (it reads every landblock once, then the
/// ones in view).
const MAP_CACHE_BYTES: usize = 8 * 1024 * 1024;
impl HudView {
    pub(crate) fn new(
        store: Arc<ResourceStore>,
        gpu: &Gpu,
        format: wgpu::TextureFormat,
        size: (u32, u32),
        style: ac2_ui::HudStyle,
    ) -> Self {
        let store_for_rebuild = Arc::clone(&store);
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
                    ac2_ui::build_styled_hud(&store, width, height, style).and_then(|mut hud| {
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
                        // The world map's landscape: a store of its own with a small cache, so its
                        // sweep of every landblock never flushes the world's.
                        if let Some(map) = hud.world_map.as_mut() {
                            let map_store = ac2_db::ResourceStore::with_free_cache_budget(
                                std::sync::Arc::clone(store.dataset()),
                                MAP_CACHE_BYTES,
                            );
                            if let Ok(Some(world)) =
                                ac2_world::Ac2WorldSource::open(std::sync::Arc::new(map_store))
                            {
                                map.set_world(world);
                            }
                        }
                        let mut images = ac2_ui::load_ui_images(&store, &hud.tree)?;
                        // The Tracker button's round eye, made from the loaded icon.
                        ac2_ui::tracker_ui::bake_eye(&mut hud.tree, &mut images);
                        // The modern style's art (`modern_art`): the shipped panel icons,
                        // then every panel and attack icon set in its well.
                        if hud.style == ac2_ui::HudStyle::Modern {
                            images.extend(ac2_ui::ac1_buttons::shipped_icons());
                            let icons =
                                ac2_ui::load_images(&store, &ac2_ui::modern_art::SOURCE_ICONS)?;
                            ac2_ui::modern_art::bake(&mut images, &icons);
                        }
                        Ok((hud, images))
                    });
                let _ = send.send(built);
            })
            .expect("spawn the HUD loader");
        Self {
            pending: Some(receive),
            hud: None,
            style,
            store: store_for_rebuild,
            panels: {
                let mut panels = ac2_ui::Hud::default();
                panels.pending_strings = Some(pending_strings);
                panels
            },
            gameplay_shown: true,
            story_hid_gameplay: None,
            tracker: Default::default(),
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

    /// A new HUD of the same window in `style`, built from the same store (the HUD's state comes
    /// back from the player, as after a relog).
    pub(crate) fn rebuilt(
        &self,
        gpu: &Gpu,
        format: wgpu::TextureFormat,
        style: ac2_ui::HudStyle,
    ) -> Self {
        Self::new(Arc::clone(&self.store), gpu, format, self.size, style)
    }

    /// Takes a finished build, then recomposes if anything changed.
    pub(super) fn arrived(&mut self, built: Built) {
        match built {
            Ok((
                GameplayHud {
                    mut tree,
                    style,
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
                    option_panel,
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
                    inventory_layout,
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
                    tracker,
                    modern_edit,
                    modern_banner,
                    guide_arrow,
                    world_map,
                    guide,
                    story,
                    fellow_marker_image,
                },
                images,
            )) => {
                tree.set_display(self.scale.display.0 as i32, self.scale.display.1 as i32);
                self.panels.style = style;
                // The modern toolbar laid out for the display.
                toolbar.follow(&mut tree);
                if let Some(chat) = &mut chat {
                    // The player's options are not read yet: the chat starts on
                    // `GameplayOptionsProfile::Clear`'s (no filters, default fonts).
                    chat.reload_chat_settings(&mut tree, &ChatSettings::default());
                    for (kind, text) in self.panels.early_lines.drain(..) {
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
                self.hud = Some((tree, images));
                self.panels.set_overlays(reticle, names, damage);
                self.panels.mini_fellow = mini_fellow;
                self.panels.pets = pets;
                self.panels.chat = chat;
                self.panels.toolbar = Some(toolbar);
                self.show_toolbar(true);
                self.panels.shortcuts = shortcuts;
                self.panels.skills = skills;
                self.panels.craft = craft;
                self.panels.ingredient = ingredient;
                self.panels.examination = examination;
                self.panels.option_panel = option_panel;
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
                self.panels.toolbar_layout = toolbar_layout;
                self.panels.inventory_layout = inventory_layout;
                self.panels.dialogs = Some(dialogs);
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
                self.panels.radar = Some(radar);
                self.panels.navigation = navigation;
                self.panels.overview = overview;
                self.panels.book = book;
                self.panels.help = help;
                self.panels.help_pages = help_pages;
                self.panels.tracker = tracker;
                self.panels.modern_edit = modern_edit;
                self.panels.modern_banner = modern_banner;
                self.panels.guide_arrow = guide_arrow;
                self.panels.world_map = world_map;
                self.panels.guide = guide;
                self.panels.story = story;
                self.panels.fellow_markers =
                    ac2_ui::fellow_markers::FellowMarkers::new(fellow_marker_image);
                self.dirty = true;
            }
            Err(error) => eprintln!("ac2: the HUD did not load: {error}"),
        }
    }
}
