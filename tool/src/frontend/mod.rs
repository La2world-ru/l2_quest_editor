mod hunting_zone;
mod item;
mod item_set;
mod npc;
mod quest;
mod recipe;
mod region;
mod skill;
mod spawn_editor;
mod util;

use crate::backend::{
    Backend, ChangeTrackedParams, CurrentEntity, Dialog, DialogAnswer, LogHolder, LogHolderParams,
    LogLevel, LogLevelFilter, WindowParams,
};
use crate::data::{ItemId, Location, NpcId, Position, QuestId};
use crate::entity::Entity;
use crate::frontend::spawn_editor::SpawnEditor;
use crate::frontend::util::num_value::NumberValue;
use crate::frontend::util::{combo_box_row, num_row, Draw, DrawActioned, DrawAsTooltip};
use crate::holder::DataHolder;
use crate::logs;
use copypasta::{ClipboardContext, ClipboardProvider};
use eframe::egui::{
    Button, Color32, FontFamily, Image, Response, RichText, ScrollArea, TextureId, Ui,
};
use eframe::{egui, glow};
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

const QUEST_ICON: &[u8] = include_bytes!("../../../files/quest.png");
const SKILL_ICON: &[u8] = include_bytes!("../../../files/skill.png");
const NPC_ICON: &[u8] = include_bytes!("../../../files/npc.png");
const WEAPON_ICON: &[u8] = include_bytes!("../../../files/weapon.png");
const ARMOR_ICON: &[u8] = include_bytes!("../../../files/armor.png");
const ETC_ICON: &[u8] = include_bytes!("../../../files/etc.png");
const SET_ICON: &[u8] = include_bytes!("../../../files/set.png");
const RECIPE_ICON: &[u8] = include_bytes!("../../../files/recipe.png");
const REGION_ICON: &[u8] = include_bytes!("../../../files/region.png");
const HUNTING_ZONE_ICON: &[u8] = include_bytes!("../../../files/hunting_zone.png");

pub const WORLD_MAP: &[u8] = include_bytes!("../../../files/map_s.png");

const DELETE_ICON: &str = "🗑";
const ADD_ICON: &str = "➕";

lazy_static! {
    pub static ref IS_SAVING: Arc<RwLock<bool>> = Arc::new(RwLock::new(false));
}

trait DrawEntity<Action, Params> {
    fn draw_entity(
        &mut self,
        ui: &mut Ui,
        ctx: &egui::Context,
        action: &RwLock<Action>,
        holders: &mut DataHolder,
        params: &mut Params,
    );
}

trait DrawWindow {
    fn draw_window(&mut self, ui: &mut Ui, ctx: &egui::Context, holders: &mut DataHolder);
}

impl<Inner: DrawEntity<Action, Params>, OriginalId, Action, Params> DrawWindow
    for WindowParams<Inner, OriginalId, Action, Params>
{
    fn draw_window(&mut self, ui: &mut Ui, ctx: &egui::Context, holders: &mut DataHolder) {
        self.inner
            .draw_entity(ui, ctx, &self.action, holders, &mut self.params);
    }
}

impl<Inner: DrawEntity<Action, Params>, OriginalId, Action, Params> DrawWindow
    for ChangeTrackedParams<Inner, OriginalId, Action, Params>
{
    fn draw_window(&mut self, ui: &mut Ui, ctx: &egui::Context, holders: &mut DataHolder) {
        self.inner.draw_window(ui, ctx, holders);
    }
}

impl Draw for Location {
    fn draw(&mut self, ui: &mut Ui, _holders: &DataHolder) -> Response {
        ui.horizontal(|ui| {
            ui.label("X");
            ui.add(NumberValue::new(&mut self.x));

            ui.label("Y");
            ui.add(NumberValue::new(&mut self.y));

            ui.label("Z");
            ui.add(NumberValue::new(&mut self.z));
        })
        .response
    }
}

impl Draw for Position {
    fn draw(&mut self, ui: &mut Ui, _holders: &DataHolder) -> Response {
        ui.horizontal(|ui| {
            ui.label("X");
            ui.add(NumberValue::new(&mut self.x));

            ui.label("Y");
            ui.add(NumberValue::new(&mut self.y));

            ui.label("Z");
            ui.add(NumberValue::new(&mut self.z));
        })
        .response
    }
}

impl Draw for NpcId {
    fn draw(&mut self, ui: &mut Ui, holders: &DataHolder) -> Response {
        ui.add(NumberValue::new(&mut self.0)).on_hover_ui(|ui| {
            holders
                .game_data_holder
                .npc_holder
                .get(self)
                .draw_as_tooltip(ui)
        })
    }
}

impl Draw for ItemId {
    fn draw(&mut self, ui: &mut Ui, holders: &DataHolder) -> Response {
        ui.add(NumberValue::new(&mut self.0)).on_hover_ui(|ui| {
            holders
                .game_data_holder
                .item_holder
                .get(self)
                .draw_as_tooltip(ui)
        })
    }
}

struct GlobalSearchParams {
    pub search_showing: bool,
    pub current_entity: Entity,
}

pub struct Frontend {
    backend: Backend,
    search_params: GlobalSearchParams,
    spawn_editor: SpawnEditor,
    allow_close: bool,
    ask_close: bool,
}

impl Frontend {
    fn update_npc_spawn_path(&mut self, path: PathBuf) {
        if path.is_dir() {
            let mut c = HashMap::new();

            for npc in self.backend.holders.game_data_holder.npc_holder.values() {
                c.insert(npc.id.0, format!("{} [{}]", npc.name, npc.id.0));
            }

            self.spawn_editor.update_spawn_path(
                path.to_str().unwrap(),
                Box::new(move |v| {
                    if let Some(n) = c.get(&v) {
                        n.clone()
                    } else {
                        format!("Not Exist [{v}]")
                    }
                }),
            );
        }

        self.backend.update_npc_spawn_path(path);
    }

    fn draw_editor(&mut self, ui: &mut Ui, ctx: &egui::Context) {
        match self.backend.edit_params.current_entity {
            CurrentEntity::Npc(index) => self.backend.edit_params.npcs.opened[index].draw_window(
                ui,
                ctx,
                &mut self.backend.holders,
            ),

            CurrentEntity::Quest(index) => self.backend.edit_params.quests.opened[index]
                .draw_window(ui, ctx, &mut self.backend.holders),

            CurrentEntity::Skill(index) => self.backend.edit_params.skills.opened[index]
                .draw_window(ui, ctx, &mut self.backend.holders),

            CurrentEntity::Weapon(index) => self.backend.edit_params.weapons.opened[index]
                .draw_window(ui, ctx, &mut self.backend.holders),

            CurrentEntity::EtcItem(index) => self.backend.edit_params.etc_items.opened[index]
                .draw_window(ui, ctx, &mut self.backend.holders),

            CurrentEntity::Armor(index) => self.backend.edit_params.armor.opened[index]
                .draw_window(ui, ctx, &mut self.backend.holders),

            CurrentEntity::ItemSet(index) => self.backend.edit_params.item_sets.opened[index]
                .draw_window(ui, ctx, &mut self.backend.holders),

            CurrentEntity::Recipe(index) => self.backend.edit_params.recipes.opened[index]
                .draw_window(ui, ctx, &mut self.backend.holders),

            CurrentEntity::HuntingZone(index) => self.backend.edit_params.hunting_zones.opened
                [index]
                .draw_window(ui, ctx, &mut self.backend.holders),

            CurrentEntity::Region(index) => self.backend.edit_params.regions.opened[index]
                .draw_window(ui, ctx, &mut self.backend.holders),

            CurrentEntity::None => {}
        }
    }

    fn draw_tabs(&mut self, ui: &mut Ui, _ctx: &egui::Context) {
        if self.backend.edit_params.current_entity.is_some() {
            ui.horizontal(|ui| {
                ui.separator();

                let mut text = RichText::new("\u{f058}").family(FontFamily::Name("icons".into()));

                let changed = self.backend.is_current_entity_changed();

                if changed {
                    text = text.color(Color32::from_rgb(152, 80, 0));
                }

                if ui
                    .button(text)
                    .on_hover_text(if changed {
                        "Save changes\n(in memory only, will not write them to disk!)"
                    } else {
                        "No changes"
                    })
                    .clicked()
                {
                    self.backend.save_current_entity();
                }
                ui.separator();

                if ui
                    .button(RichText::new("\u{f56e}").family(FontFamily::Name("icons".into())))
                    .on_hover_text("Export in RON format")
                    .clicked()
                {
                    if let Some(v) = self.backend.current_entity_as_ron() {
                        ui.output_mut(|o| o.copied_text = v);
                    }
                }

                if ui
                    .button(RichText::new("\u{f56f}").family(FontFamily::Name("icons".into())))
                    .on_hover_text("Import from RON format")
                    .clicked()
                {
                    self.backend.fill_current_entity_from_ron(
                        &ClipboardContext::new().unwrap().get_contents().unwrap(),
                    );
                }

                ui.separator();

                ui.vertical(|ui| {
                    ui.push_id(ui.next_auto_id(), |ui| {
                        ScrollArea::horizontal().show(ui, |ui| {
                            ui.horizontal(|ui| {
                                self.draw_quest_tabs(ui);
                                self.draw_skill_tabs(ui);
                                self.draw_npc_tabs(ui);
                                self.draw_weapon_tabs(ui);
                                self.draw_armor_tabs(ui);
                                self.draw_etc_items_tabs(ui);
                                self.draw_item_set_tabs(ui);
                                self.draw_recipe_tabs(ui);
                                self.draw_hunting_zone_tabs(ui);
                                self.draw_region_tabs(ui);
                            });
                        });
                    });
                });
            });
            ui.separator();
        }
    }

    fn build_top_menu(&mut self, ui: &mut Ui, ctx: &egui::Context) {
        ui.horizontal(|ui| {
            ui.menu_button(
                RichText::new(" \u{f013} ").family(FontFamily::Name("icons".into())),
                |ui| {
                    if ui.button("Set L2 system folder").clicked() {
                        if let Some(path) = rfd::FileDialog::new().pick_folder() {
                            self.backend.update_system_path(path)
                        }
                    }
                    if ui.button("Set GS quest classes folder").clicked() {
                        if let Some(path) = rfd::FileDialog::new().pick_folder() {
                            self.backend.update_quests_java_path(path)
                        }
                    }
                    if ui.button("Set GS spawn folder").clicked() {
                        if let Some(path) = rfd::FileDialog::new().pick_folder() {
                            self.update_npc_spawn_path(path)
                        }
                    }
                },
            )
            .response
            .on_hover_text("Settings");

            let mut b =
                Button::new(RichText::new(" \u{f0c7} ").family(FontFamily::Name("icons".into())));

            if self.backend.is_changed() {
                b = b.fill(Color32::from_rgb(152, 80, 0));
            }

            if ui
                .add(b)
                .on_hover_text(if self.backend.is_changed() {
                    format!(
                        "Write changes to .dat\n\nChanged:\n{:#?}",
                        self.backend.holders.game_data_holder.changed_entites()
                    )
                } else {
                    "No changes to save".to_string()
                })
                .clicked()
                && self.backend.is_changed()
            {
                self.backend.save_to_dat();
                ui.close_menu();
            }

            if ui
                .button(RichText::new(" \u{f02d} ").family(FontFamily::Name("icons".into())))
                .on_hover_text(".dat Editor")
                .clicked()
            {
                self.search_params.search_showing = true;
            }

            if let Some(p) = &self.backend.config.server_spawn_root_folder_path {
                if ui
                    .button(RichText::new(" \u{f279} ").family(FontFamily::Name("icons".into())))
                    .on_hover_text("Spawn Viewer")
                    .clicked()
                {
                    if self.spawn_editor.editor.is_none() {
                        let mut c = HashMap::new();

                        for npc in self.backend.holders.game_data_holder.npc_holder.values() {
                            c.insert(npc.id.0, format!("{} [{}]", npc.name, npc.id.0));
                        }

                        self.spawn_editor.show(
                            p,
                            Box::new(move |v| {
                                (if let Some(n) = c.get(&v) {
                                    n
                                } else {
                                    "Not Exist"
                                })
                                .to_string()
                            }),
                        );
                    } else {
                        self.spawn_editor.showing = true;
                    }
                }
            }

            self.backend.logs.draw_as_button_tooltip(
                ui,
                ctx,
                &self.backend.holders,
                RichText::new(self.backend.logs.inner.max_log_level)
                    .family(FontFamily::Name("icons".into()))
                    .color(self.backend.logs.inner.max_log_level),
                "Logs",
                "app_logs",
                "Logs",
            );
        });
    }

    fn show_dialog(&mut self, ctx: &egui::Context) {
        match &self.backend.dialog {
            Dialog::ConfirmNpcSave { message, .. }
            | Dialog::ConfirmQuestSave { message, .. }
            | Dialog::ConfirmWeaponSave { message, .. }
            | Dialog::ConfirmEtcSave { message, .. }
            | Dialog::ConfirmArmorSave { message, .. }
            | Dialog::ConfirmItemSetSave { message, .. }
            | Dialog::ConfirmRecipeSave { message, .. }
            | Dialog::ConfirmHuntingZoneSave { message, .. }
            | Dialog::ConfirmRegionSave { message, .. }
            | Dialog::ConfirmSkillSave { message, .. } => {
                let m = message.clone();

                egui::Window::new("Confirm Overwrite")
                    .id(egui::Id::new("_confirm_"))
                    .collapsible(false)
                    .resizable(false)
                    .show(ctx, |ui| {
                        ui.vertical_centered(|ui| {
                            ui.set_height(60.);
                            ui.set_width(300.);

                            ui.label(m);

                            ui.add_space(5.);

                            ui.horizontal(|ui| {
                                ui.add_space(100.);
                                if ui.button("Confirm").clicked() {
                                    self.backend.answer(DialogAnswer::Confirm);
                                }
                                if ui.button("Cancel").clicked() {
                                    self.backend.answer(DialogAnswer::Abort);
                                }
                            });
                        })
                    });
            }

            Dialog::ShowWarning(warn) => {
                let m = warn.clone();

                egui::Window::new("Warning!")
                    .id(egui::Id::new("_warn_"))
                    .collapsible(false)
                    .resizable(false)
                    .show(ctx, |ui| {
                        ui.vertical_centered(|ui| {
                            ui.label(m);

                            if ui.button("   Ok   ").clicked() {
                                self.backend.answer(DialogAnswer::Confirm);
                            }
                        })
                    });
            }

            Dialog::ConfirmClose(_) => {
                egui::Window::new("Confirm Close")
                    .id(egui::Id::new("_close_entity_"))
                    .collapsible(false)
                    .resizable(false)
                    .show(ctx, |ui| {
                        ui.set_height(65.);
                        ui.set_width(300.);
                        ui.vertical_centered(|ui| {
                            ui.label("There are unsaved changes!\nAre you sure?");

                            ui.add_space(5.);

                            ui.horizontal(|ui| {
                                ui.add_space(100.);
                                if ui.button("Close").clicked() {
                                    self.backend.answer(DialogAnswer::Confirm);
                                }
                                if ui.button("Cancel").clicked() {
                                    self.backend.answer(DialogAnswer::Abort);
                                }
                            });
                        })
                    });
            }

            Dialog::None => {}
        }
    }

    pub fn init(world_map_texture_id: TextureId) -> Self {
        let backend = Backend::init();
        let spawn_editor = SpawnEditor::init(world_map_texture_id);

        Self {
            backend,
            search_params: GlobalSearchParams {
                search_showing: false,
                current_entity: Entity::Quest,
            },
            spawn_editor,
            allow_close: false,
            ask_close: false,
        }
    }
}

impl eframe::App for Frontend {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        const LIBRARY_WIDTH: f32 = 392.;

        self.backend.on_update();

        egui::Window::new("📚")
            .id(egui::Id::new("_search_"))
            .open(&mut self.search_params.search_showing)
            .show(ctx, |ui| {
                ui.set_width(LIBRARY_WIDTH);

                ui.horizontal(|ui| {
                    ui.set_height(32.);
                    if ui
                        .add(egui::ImageButton::new(Image::from_bytes(
                            "bytes://quest.png",
                            QUEST_ICON,
                        )))
                        .on_hover_text("Quests")
                        .clicked()
                    {
                        self.search_params.current_entity = Entity::Quest;
                    };

                    if ui
                        .add(egui::ImageButton::new(Image::from_bytes(
                            "bytes://skill.png",
                            SKILL_ICON,
                        )))
                        .on_hover_text("Skills")
                        .clicked()
                    {
                        self.search_params.current_entity = Entity::Skill;
                    };

                    if ui
                        .add(egui::ImageButton::new(Image::from_bytes(
                            "bytes://npc.png",
                            NPC_ICON,
                        )))
                        .on_hover_text("Npcs")
                        .clicked()
                    {
                        self.search_params.current_entity = Entity::Npc;
                    };

                    if ui
                        .add(egui::ImageButton::new(Image::from_bytes(
                            "bytes://weapon.png",
                            WEAPON_ICON,
                        )))
                        .on_hover_text("Weapon")
                        .clicked()
                    {
                        self.search_params.current_entity = Entity::Weapon;
                    };

                    if ui
                        .add(egui::ImageButton::new(Image::from_bytes(
                            "bytes://armor.png",
                            ARMOR_ICON,
                        )))
                        .on_hover_text("Armor/Jewelry")
                        .clicked()
                    {
                        self.search_params.current_entity = Entity::Armor;
                    };

                    if ui
                        .add(egui::ImageButton::new(Image::from_bytes(
                            "bytes://etc.png",
                            ETC_ICON,
                        )))
                        .on_hover_text("Etc Items")
                        .clicked()
                    {
                        self.search_params.current_entity = Entity::EtcItem;
                    };

                    if ui
                        .add(egui::ImageButton::new(Image::from_bytes(
                            "bytes://set.png",
                            SET_ICON,
                        )))
                        .on_hover_text("Sets")
                        .clicked()
                    {
                        self.search_params.current_entity = Entity::ItemSet;
                    };

                    if ui
                        .add(egui::ImageButton::new(Image::from_bytes(
                            "bytes://recipe.png",
                            RECIPE_ICON,
                        )))
                        .on_hover_text("Recipes")
                        .clicked()
                    {
                        self.search_params.current_entity = Entity::Recipe;
                    };

                    if ui
                        .add(egui::ImageButton::new(Image::from_bytes(
                            "bytes://hunting_zone.png",
                            HUNTING_ZONE_ICON,
                        )))
                        .on_hover_text("Hunting Zones")
                        .clicked()
                    {
                        self.search_params.current_entity = Entity::HuntingZone;
                    };

                    if ui
                        .add(egui::ImageButton::new(Image::from_bytes(
                            "bytes://region.png",
                            REGION_ICON,
                        )))
                        .on_hover_text("Regions")
                        .clicked()
                    {
                        self.search_params.current_entity = Entity::Region;
                    };
                });

                ui.separator();

                match self.search_params.current_entity {
                    Entity::Npc => Self::draw_npc_selector(&mut self.backend, ui, LIBRARY_WIDTH),

                    Entity::Quest => {
                        Self::draw_quest_selector(&mut self.backend, ui, LIBRARY_WIDTH)
                    }

                    Entity::Skill => {
                        Self::draw_skill_selector(&mut self.backend, ui, LIBRARY_WIDTH)
                    }

                    Entity::Weapon => {
                        Self::draw_weapon_selector(&mut self.backend, ui, LIBRARY_WIDTH)
                    }

                    Entity::EtcItem => {
                        Self::draw_etc_item_selector(&mut self.backend, ui, LIBRARY_WIDTH)
                    }

                    Entity::Armor => {
                        Self::draw_armor_selector(&mut self.backend, ui, LIBRARY_WIDTH)
                    }

                    Entity::ItemSet => {
                        Self::draw_item_set_selector(&mut self.backend, ui, LIBRARY_WIDTH)
                    }

                    Entity::Recipe => {
                        Self::draw_recipe_selector(&mut self.backend, ui, LIBRARY_WIDTH)
                    }

                    Entity::HuntingZone => {
                        Self::draw_hunting_zone_selector(&mut self.backend, ui, LIBRARY_WIDTH)
                    }

                    Entity::Region => {
                        Self::draw_region_selector(&mut self.backend, ui, LIBRARY_WIDTH)
                    }
                }
            });

        if self.spawn_editor.showing {
            if let Some(v) = &mut self.spawn_editor.editor {
                ctx.show_viewport_immediate(
                    egui::ViewportId::from_hash_of("_spawn_editor_"),
                    egui::ViewportBuilder::default()
                        .with_title("Spawn Viewer")
                        .with_inner_size([600.0, 300.0]),
                    |ctx, class| {
                        assert!(
                            class == egui::ViewportClass::Immediate,
                            "This egui backend doesn't support multiple viewports"
                        );

                        egui::CentralPanel::default().show(ctx, |ui| {
                            v.show(ctx, ui);
                        });

                        if ctx.input(|i| i.viewport().close_requested()) {
                            // Tell parent viewport that we should not show next frame:
                            self.spawn_editor.showing = false;
                        }
                    },
                );
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            self.show_dialog(ctx);

            self.build_top_menu(ui, ctx);

            ui.separator();

            self.draw_tabs(ui, ctx);

            self.draw_editor(ui, ctx);

            if *IS_SAVING.read().unwrap() {
                egui::Window::new("SAVING IN PROGRESS")
                    .id(egui::Id::new("_saving_"))
                    .resizable(false)
                    .collapsible(false)
                    .show(ctx, |ui| {
                        ui.vertical_centered(|ui| {
                            ui.spinner();
                        })
                    });
            }
        });

        if ctx.input(|i| i.viewport().close_requested()) {
            if !self.allow_close {
                ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
                self.ask_close = true;
            }
        }

        if self.ask_close {
            egui::Window::new("Discard Changes?")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.set_height(65.);
                    ui.set_width(300.);
                    ui.vertical_centered(|ui| {
                        ui.label("Changes are unwritten to .dat\nAre you sure?");

                        ui.add_space(5.);

                        ui.horizontal(|ui| {
                            ui.add_space(105.);

                            if ui.button(" No ").clicked() {
                                self.allow_close = false;
                                self.ask_close = false;
                            }

                            ui.add_space(10.);

                            if ui.button(" Yes ").clicked() {
                                self.ask_close = false;
                                self.allow_close = true;
                                ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                            }
                        });
                    });
                });
        }
    }

    fn on_exit(&mut self, _gl: Option<&glow::Context>) {
        self.backend.auto_save(true);
    }
}

impl From<LogLevel> for Color32 {
    fn from(value: LogLevel) -> Self {
        match value {
            LogLevel::Info => Color32::from_rgb(196, 210, 221),
            LogLevel::Warning => Color32::from_rgb(238, 146, 62),
            LogLevel::Error => Color32::from_rgb(238, 62, 62),
        }
    }
}

impl DrawActioned<(), ()> for LogHolderParams {
    fn draw_with_action(
        &mut self,
        ui: &mut Ui,
        _holders: &DataHolder,
        _action: &RwLock<()>,
        _params: &mut (),
    ) {
        ui.horizontal(|ui| {
            combo_box_row(ui, &mut self.level_filter, "Level");
            ui.label("Producer");
            egui::ComboBox::from_id_source(ui.next_auto_id())
                .selected_text(&self.producer_filter)
                .show_ui(ui, |ui| {
                    ui.style_mut().wrap = Some(false);
                    ui.set_min_width(20.0);

                    let mut c = self.producers.iter().collect::<Vec<&String>>();
                    c.sort();
                    for t in c {
                        ui.selectable_value(&mut self.producer_filter, t.clone(), t);
                    }
                });
        });

        ui.separator();

        ui.horizontal(|ui| {
            ui.set_min_height(350.);
            ScrollArea::vertical().show(ui, |ui| {
                ui.vertical(|ui| {
                    for log in logs().logs.iter().filter(|v| {
                        let a = self.producer_filter == LogHolder::ALL
                            || self.producer_filter == v.producer;
                        let b = self.level_filter == LogLevelFilter::All
                            || self.level_filter as u8 == v.level as u8;

                        a && b
                    }) {
                        ui.horizontal(|ui| {
                            ui.label(&log.producer);
                            ui.label(RichText::new(&log.log).color(log.level));
                        });

                        ui.add_space(5.0);
                    }
                });

                ui.add_space(5.0);
            });
        });
    }
}

impl From<LogLevel> for String {
    fn from(value: LogLevel) -> Self {
        match value {
            LogLevel::Info => " \u{f0eb} ".to_string(),
            LogLevel::Warning => " \u{f071} ".to_string(),
            LogLevel::Error => " \u{f071} ".to_string(),
        }
    }
}

impl Draw for QuestId {
    fn draw(&mut self, ui: &mut Ui, holders: &DataHolder) -> Response {
        num_row(ui, &mut self.0, "Id").on_hover_ui(|ui| {
            holders
                .game_data_holder
                .quest_holder
                .get(self)
                .draw_as_tooltip(ui)
        })
    }
}
