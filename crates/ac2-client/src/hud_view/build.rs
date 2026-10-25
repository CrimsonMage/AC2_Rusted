//! The build path: `HudView::new` starts the workers that load the string tables and build
//! the HUD, and `arrived` takes the finished build into the view.

use std::sync::{mpsc, Arc};

use ac2_db::ResourceStore;
use ac2_gpu::{Gpu, UiOverlay};
use ac2_ui::{ChatSettings, GameplayHud};

use super::{Built, HudView};

impl HudView {
    pub(crate) fn new(
        store: Arc<ResourceStore>,
        gpu: &Gpu,
        format: wgpu::TextureFormat,
        size: (u32, u32),
    ) -> Self {
        let chat_text = crate::chat_text::ChatText::new((*store).clone());
        let images = crate::ui_images::UiImageLoader::new(Arc::clone(&store));
        let (send_strings, pending_strings) = mpsc::channel();
        let strings_store = Arc::clone(&store);
        std::thread::Builder::new()
            .name("ac2 ui strings".into())
            .spawn(move || {
                let tables = ac2_db::PreloadedStringTables::load_all(&strings_store, 0x5800_0000);
                let _ = send_strings.send(crate::ui_sources::UiStringTables::new(
                    &strings_store,
                    tables,
                ));
            })
            .expect("spawn the UI strings loader");
        let (send, receive) = mpsc::channel();
        let (width, height) = (size.0 as i32, size.1 as i32);
        std::thread::Builder::new()
            .name("ac2 hud".into())
            .spawn(move || {
                let built =
                    ac2_ui::build_gameplay_hud(&store, width, height).and_then(|mut hud| {
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
                        let images = ac2_ui::load_ui_images(&store, &hud.tree)?;
                        Ok((hud, images))
                    });
                let _ = send.send(built);
            })
            .expect("spawn the HUD loader");
        Self {
            pending: Some(receive),
            hud: None,
            reticle: None,
            names: None,
            damage: None,
            now: 0.0,
            chat: None,
            toolbar: None,
            shortcuts: None,
            skills: None,
            examination: None,
            allegiance: None,
            fellowship: None,
            vitals: None,
            effects: None,
            backpack: None,
            paper_doll: None,
            player_race: None,
            toolbar_layout: None,
            item_selection: 0,
            reticle_info: None,
            dialogs: None,
            stack: None,
            external_container: None,
            secure_trade: None,
            character: None,
            chat_options: None,
            shop: None,
            radar: None,
            radar_cell: None,
            radar_coordinates: None,
            navigation_time: None,
            navigation: None,
            overview: None,
            money: None,
            shift_held: false,
            player_messages: Vec::new(),
            strings: None,
            pending_strings: Some(pending_strings),
            player_loaded: false,
            early_lines: Vec::new(),
            chat_text,
            images,
            overlay: UiOverlay::new(gpu, format),
            layer_roots: Vec::new(),
            size,
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
                    examination,
                    allegiance,
                    fellowship,
                    vitals,
                    effects,
                    toolbar_layout,
                    dialogs,
                    stack,
                    external_container,
                    radar,
                    secure_trade,
                    character,
                    chat_options,
                    navigation,
                    overview,
                    shop,
                },
                images,
            )) => {
                tree.set_display(self.size.0 as i32, self.size.1 as i32);
                if let Some(chat) = &mut chat {
                    // The player's options are not read yet: the chat starts on
                    // `GameplayOptionsProfile::Clear`'s (no filters, default fonts).
                    chat.reload_chat_settings(&mut tree, &ChatSettings::default());
                    for (kind, text) in self.early_lines.drain(..) {
                        chat.display(kind, &text);
                    }
                }
                self.hud = Some((tree, images));
                self.reticle = reticle;
                self.names = names;
                self.damage = damage;
                self.chat = chat;
                self.toolbar = Some(toolbar);
                self.show_toolbar(true);
                self.shortcuts = shortcuts;
                self.skills = skills;
                self.examination = examination;
                self.allegiance = allegiance;
                self.fellowship = fellowship;
                self.vitals = Some(vitals);
                self.effects = effects;
                self.toolbar_layout = toolbar_layout;
                self.dialogs = Some(dialogs);
                self.stack = stack;
                self.external_container = external_container;
                self.secure_trade = secure_trade;
                self.character = character;
                self.chat_options = chat_options;
                self.shop = shop;
                self.radar = Some(radar);
                self.navigation = navigation;
                self.overview = overview;
                self.dirty = true;
            }
            Err(error) => eprintln!("ac2: the HUD did not load: {error}"),
        }
    }
}
