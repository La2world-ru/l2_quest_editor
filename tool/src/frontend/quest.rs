use crate::backend::{Backend, Holders, QuestAction, StepAction};
use crate::data::{ItemId, NpcId, PlayerClass};
use crate::entity::quest::{
    GoalType, MarkType, Quest, QuestCategory, QuestStep, QuestType, StepGoal, Unk1, Unk2, UnkQLevel,
};
use crate::frontend::{BuildAsTooltip, Frontend};
use crate::holders::GameDataHolder;
use eframe::egui;
use eframe::egui::{Key, ScrollArea, Ui};
use strum::IntoEnumIterator;

impl BuildAsTooltip for Quest {
    fn build_as_tooltip(&self, ui: &mut Ui) {
        ui.label(format!("[{}]\n{}", self.id.0, self.title));
    }
}

impl StepGoal {
    fn build(&mut self, ui: &mut Ui, holder: &mut GameDataHolder) {
        ui.horizontal(|ui| {
            ui.label("Type");

            egui::ComboBox::from_id_source(ui.next_auto_id())
                .selected_text(format!("{}", self.goal_type))
                .show_ui(ui, |ui| {
                    ui.style_mut().wrap = Some(false);
                    ui.set_min_width(20.0);

                    for t in GoalType::iter() {
                        ui.selectable_value(&mut self.goal_type, t, format!("{t}"));
                    }
                });
        });

        ui.horizontal(|ui| {
            match self.goal_type {
                GoalType::KillNpc => {
                    ui.label("Monster Id");
                    ui.add(egui::DragValue::new(&mut self.target_id))
                        .on_hover_ui(|ui| {
                            holder
                                .npc_holder
                                .get(&NpcId(self.target_id))
                                .build_as_tooltip(ui);
                        });
                }
                GoalType::CollectItem => {
                    ui.label("Item Id");
                    ui.add(egui::DragValue::new(&mut self.target_id))
                        .on_hover_ui(|ui| {
                            holder
                                .item_holder
                                .get(&ItemId(self.target_id))
                                .build_as_tooltip(ui);
                        });
                }
                GoalType::Other => {
                    ui.label("Npc String Id");
                    ui.add(egui::DragValue::new(&mut self.target_id))
                        .on_hover_ui(|ui| {
                            holder.npc_strings.get(&self.target_id).build_as_tooltip(ui);
                        });
                }
            };
        });

        if self.goal_type != GoalType::Other {
            ui.horizontal(|ui| {
                ui.label("Count");
                ui.add(egui::DragValue::new(&mut self.count));
            });
        }
    }
}

impl QuestStep {
    fn build(
        &mut self,
        ui: &mut Ui,
        step_index: u32,
        action: &mut StepAction,
        holder: &mut GameDataHolder,
    ) {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.set_width(200.);

                ui.label("Title");
                ui.text_edit_singleline(&mut self.title);
                ui.label("Label");
                ui.text_edit_singleline(&mut self.label);
                ui.label("Description");
                ui.text_edit_multiline(&mut self.desc);
            });

            ui.separator();

            ui.vertical(|ui| {
                ui.set_width(150.);
                ui.set_max_height(400.);

                ui.horizontal(|ui| {
                    ui.label(format!("Goals: {}", self.goals.len()));
                    if ui.button("+").clicked() {
                        self.add_goal();
                    }
                });

                ui.separator();

                ui.push_id(ui.next_auto_id(), |ui| {
                    ScrollArea::vertical().show(ui, |ui| {
                        for (i, v) in self.goals.iter_mut().enumerate() {
                            v.build(ui, holder);

                            if ui.button("🗑").clicked() {
                                *action = StepAction::RemoveGoal(i);
                            }

                            ui.separator();
                        }
                    });
                });
            });

            ui.separator();

            ui.vertical(|ui| {
                ui.label("Locations");

                ui.separator();

                ui.label("Base");
                self.location.build(ui);

                ui.separator();

                ui.horizontal(|ui| {
                    ui.label("Additional");

                    if ui.button("+").clicked() {
                        self.add_additional_location();
                    }
                });

                for (i, location) in self.additional_locations.iter_mut().enumerate() {
                    ui.horizontal(|ui| {
                        location.build(ui);
                        if ui.button("🗑").clicked() {
                            *action = StepAction::RemoveAdditionalLocation(i);
                        }
                    });
                }
            });
        });

        ui.separator();

        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label("Unknown 1");

                    egui::ComboBox::from_id_source(ui.next_auto_id())
                        .selected_text(format!("{}", self.unk_1))
                        .show_ui(ui, |ui| {
                            ui.style_mut().wrap = Some(false);
                            ui.set_min_width(20.0);

                            for t in Unk1::iter() {
                                ui.selectable_value(&mut self.unk_1, t, format!("{t}"));
                            }
                        });
                });

                ui.horizontal(|ui| {
                    ui.label("Unknown 2");

                    egui::ComboBox::from_id_source(ui.next_auto_id())
                        .selected_text(format!("{}", self.unk_2))
                        .show_ui(ui, |ui| {
                            ui.style_mut().wrap = Some(false);
                            ui.set_min_width(20.0);

                            for t in Unk2::iter() {
                                ui.selectable_value(&mut self.unk_2, t, format!("{t}"));
                            }
                        });
                });
            });

            ui.separator();

            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label("Unknown 3");
                    ui.menu_button("+", |ui| {
                        for v in UnkQLevel::iter() {
                            if ui.button(format!("{v}")).clicked() {
                                self.unk_q_level.push(v);
                                ui.close_menu();
                            }
                        }
                    });
                });

                ui.push_id(ui.next_auto_id(), |ui| {
                    ScrollArea::vertical().show(ui, |ui| {
                        for (i, v) in self.unk_q_level.clone().iter().enumerate() {
                            ui.horizontal(|ui| {
                                ui.label(format!("{v}"));
                                if ui.button("🗑".to_string()).clicked() {
                                    self.unk_q_level.remove(i);
                                }
                            });
                        }
                    });
                });
            });

            ui.separator();

            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label("Previous Steps");
                    if ui.button("+").clicked() {
                        self.prev_steps.push(0);
                    };
                });

                ui.push_id(ui.next_auto_id(), |ui| {
                    ScrollArea::vertical().show(ui, |ui| {
                        for (i, v) in self.prev_steps.iter_mut().enumerate() {
                            ui.horizontal(|ui| {
                                if ui.add(egui::DragValue::new(v)).changed() && *v > step_index {
                                    *v = 0;
                                }

                                if ui.button("🗑".to_string()).clicked() {
                                    *action = StepAction::RemovePrevStepIndex(i);
                                }
                            });
                        }
                    });
                });
            });
        });

        ui.separator();
    }
}

impl Quest {
    pub(crate) fn build(
        &mut self,
        ui: &mut Ui,
        ctx: &egui::Context,
        action: &mut QuestAction,
        holders: &mut Holders,
    ) {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.set_width(200.);

                ui.horizontal(|ui| {
                    ui.add(egui::Label::new("Name"));
                    ui.add(egui::TextEdit::singleline(&mut self.title));
                    ui.add_space(5.);
                    ui.add(egui::Label::new("Id"));
                    ui.add(egui::DragValue::new(&mut self.id.0));
                });

                ui.label("Intro");
                ui.text_edit_multiline(&mut self.intro);
                ui.label("Requirements");
                ui.text_edit_multiline(&mut self.requirements);
            });

            ui.separator();

            ui.vertical(|ui| {
                ui.set_width(150.);

                ui.horizontal(|ui| {
                    ui.label("Quest Type");

                    egui::ComboBox::from_id_source(ui.next_auto_id())
                        .selected_text(format!("{}", self.quest_type))
                        .show_ui(ui, |ui| {
                            ui.style_mut().wrap = Some(false);
                            ui.set_min_width(20.0);

                            for t in QuestType::iter() {
                                ui.selectable_value(&mut self.quest_type, t, format!("{t}"));
                            }
                        });
                });

                ui.horizontal(|ui| {
                    ui.label("Category");

                    egui::ComboBox::from_id_source(ui.next_auto_id())
                        .selected_text(format!("{}", self.category))
                        .show_ui(ui, |ui| {
                            ui.style_mut().wrap = Some(false);
                            ui.set_min_width(20.0);

                            for t in QuestCategory::iter() {
                                ui.selectable_value(&mut self.category, t, format!("{t}"));
                            }
                        });
                });

                ui.horizontal(|ui| {
                    ui.label("Mark Type");

                    egui::ComboBox::from_id_source(ui.next_auto_id())
                        .selected_text(format!("{}", self.mark_type))
                        .show_ui(ui, |ui| {
                            ui.style_mut().wrap = Some(false);
                            ui.set_min_width(20.0);

                            for t in MarkType::iter() {
                                ui.selectable_value(&mut self.mark_type, t, format!("{t}"));
                            }
                        });
                });

                ui.horizontal(|ui| {
                    ui.set_height(20.);
                    ui.add(egui::Label::new("Min lvl"));
                    ui.add_space(2.);

                    if self.min_lvl > 0 {
                        if ui.checkbox(&mut true, "").changed() {
                            self.min_lvl = 0;
                        }

                        ui.add(egui::DragValue::new(&mut self.min_lvl));
                    } else if ui.checkbox(&mut false, "").changed() {
                        self.min_lvl = 1;
                    }
                });

                ui.horizontal(|ui| {
                    ui.set_height(20.);
                    ui.add(egui::Label::new("Max lvl"));

                    if self.max_lvl > 0 {
                        if ui.checkbox(&mut true, "").changed() {
                            self.max_lvl = 0;
                        }

                        ui.add(egui::DragValue::new(&mut self.max_lvl));
                    } else if ui.checkbox(&mut false, "").changed() {
                        self.max_lvl = 1;
                    }
                });

                ui.horizontal(|ui| {
                    ui.set_height(20.);
                    ui.add(egui::Label::new("Prev Quest ID"));

                    if self.required_completed_quest_id.0 > 0 {
                        if ui.checkbox(&mut true, "").changed() {
                            self.required_completed_quest_id.0 = 0;
                        }

                        ui.add(egui::DragValue::new(
                            &mut self.required_completed_quest_id.0,
                        ))
                        .on_hover_ui(|ui| {
                            holders
                                .game_data_holder
                                .quest_holder
                                .get(&self.required_completed_quest_id)
                                .build_as_tooltip(ui);
                        });
                    } else if ui.checkbox(&mut false, "").changed() {
                        self.required_completed_quest_id.0 = 1;
                    }
                });

                ui.horizontal(|ui| {
                    ui.set_height(20.);
                    ui.add(egui::Label::new("Search Zone ID"));

                    if self.search_zone_id.0 > 0 {
                        if ui.checkbox(&mut true, "").changed() {
                            self.search_zone_id.0 = 0;
                        }

                        ui.add(egui::DragValue::new(&mut self.search_zone_id.0))
                            .on_hover_ui(|ui| {
                                holders
                                    .game_data_holder
                                    .hunting_zone_holder
                                    .get(&self.search_zone_id)
                                    .build_as_tooltip(ui)
                            });
                    } else if ui.checkbox(&mut false, "").changed() {
                        self.search_zone_id.0 = 1;
                    }
                });

                if ui.button("Edit JAVA Class").clicked() {
                    if self.java_class.is_none() {
                        holders.set_java_class(self);
                    }

                    if let Some(v) = &mut self.java_class {
                        v.opened = true;
                    }
                }

                if let Some(class) = &mut self.java_class {
                    let theme = egui_extras::syntax_highlighting::CodeTheme::from_memory(ui.ctx());

                    let mut layouter = |ui: &egui::Ui, string: &str, wrap_width: f32| {
                        let mut layout_job = egui_extras::syntax_highlighting::highlight(
                            ui.ctx(),
                            &theme,
                            string,
                            "java",
                        );
                        layout_job.wrap.max_width = wrap_width;
                        ui.fonts(|f| f.layout_job(layout_job))
                    };

                    egui::Window::new(format!("{} Java Class", self.title))
                        .id(egui::Id::new(2_000_000 + self.id.0))
                        .open(&mut class.opened)
                        .show(ctx, |ui| {
                            ScrollArea::vertical().show(ui, |ui| {
                                ui.add(
                                    egui::TextEdit::multiline(&mut class.inner)
                                        .font(egui::TextStyle::Monospace) // for cursor height
                                        .code_editor()
                                        .desired_rows(10)
                                        .lock_focus(true)
                                        .desired_width(f32::INFINITY)
                                        .layouter(&mut layouter),
                                );
                            });
                        });
                }
            });

            ui.separator();

            ui.vertical(|ui| {
                ui.set_width(150.);

                ui.horizontal(|ui| {
                    ui.label("Allowed Classes");
                    if self.allowed_classes.is_some() {
                        if ui.checkbox(&mut true, "").changed() {
                            self.allowed_classes = None;
                        }
                    } else if ui.checkbox(&mut false, "").changed() {
                        self.allowed_classes = Some(vec![]);
                    }

                    if let Some(allowed_classes) = &mut self.allowed_classes {
                        ui.menu_button("+", |ui| {
                            ui.push_id(ui.next_auto_id(), |ui| {
                                ScrollArea::vertical().show(ui, |ui| {
                                    let mut classes: Vec<_> = PlayerClass::iter()
                                        .filter(|v| !allowed_classes.contains(v))
                                        .collect();
                                    classes.sort_by(|a, b| format!("{a}").cmp(&format!("{b}")));

                                    for v in classes {
                                        if ui.button(format!("{v}")).clicked() {
                                            allowed_classes.push(v);
                                        }
                                    }
                                });
                            });
                        });
                    }
                });

                if let Some(allowed_classes) = &mut self.allowed_classes {
                    ui.push_id(ui.next_auto_id(), |ui| {
                        ScrollArea::vertical().show(ui, |ui| {
                            for (i, class) in allowed_classes.clone().iter().enumerate() {
                                if ui.button(format!("{class}")).clicked() {
                                    allowed_classes.remove(i);
                                }
                            }
                        });
                    });
                }
            });

            ui.separator();

            ui.vertical(|ui| {
                ui.set_width(200.);

                ui.label("Start Npc Location");
                self.start_npc_loc.build(ui);

                ui.separator();

                ui.horizontal(|ui| {
                    ui.label("Start Npc Ids");
                    if ui.button("+").clicked() {
                        self.add_start_npc_id();
                    }
                });

                ui.push_id(ui.next_auto_id(), |ui| {
                    ScrollArea::vertical().show(ui, |ui| {
                        for (i, id) in self.start_npc_ids.iter_mut().enumerate() {
                            ui.horizontal(|ui| {
                                ui.add(egui::DragValue::new(&mut id.0)).on_hover_ui(|ui| {
                                    holders
                                        .game_data_holder
                                        .npc_holder
                                        .get(id)
                                        .build_as_tooltip(ui)
                                });

                                if ui.button("🗑").clicked() {
                                    *action = QuestAction::RemoveStartNpcId(i);
                                }
                            });
                        }
                    });
                });
            });

            ui.separator();
        });

        ui.separator();

        ui.horizontal(|ui| {
            ui.set_height(250.);

            ui.vertical(|ui| {
                ui.set_width(100.);
                ui.horizontal(|ui| {
                    ui.label("Quest Items");
                    if ui.button("+").clicked() {
                        self.add_quest_item();
                    }
                });

                ui.push_id(ui.next_auto_id(), |ui| {
                    ScrollArea::vertical().show(ui, |ui| {
                        for (i, id) in self.quest_items.iter_mut().enumerate() {
                            ui.horizontal(|ui| {
                                ui.add(egui::DragValue::new(&mut id.0)).on_hover_ui(|ui| {
                                    holders
                                        .game_data_holder
                                        .item_holder
                                        .get(id)
                                        .build_as_tooltip(ui);
                                });
                                if ui.button("🗑").clicked() {
                                    *action = QuestAction::RemoveQuestItem(i);
                                }
                            });
                        }
                    });
                });
            });

            ui.separator();

            ui.vertical(|ui| {
                ui.set_width(100.);

                ui.horizontal(|ui| {
                    ui.label("Rewards");
                    if ui.button("+").clicked() {
                        self.add_reward();
                    }
                });

                ui.push_id(ui.next_auto_id(), |ui| {
                    ScrollArea::vertical().show(ui, |ui| {
                        for (i, reward) in self.rewards.iter_mut().enumerate() {
                            ui.horizontal(|ui| {
                                ui.label("ID");

                                ui.add(egui::DragValue::new(&mut reward.reward_id.0))
                                    .on_hover_ui(|ui| {
                                        holders
                                            .game_data_holder
                                            .item_holder
                                            .get(&reward.reward_id)
                                            .build_as_tooltip(ui)
                                    });

                                ui.label("Count");
                                ui.add(egui::DragValue::new(&mut reward.count));

                                if ui.button("🗑").clicked() {
                                    *action = QuestAction::RemoveReward(i);
                                }
                            });
                        }
                    });
                });
            });

            ui.separator();

            ui.vertical(|ui| {
                ui.set_width(100.);

                ui.horizontal(|ui| {
                    ui.label(format!("Steps: {}", self.steps.len()));

                    if ui.button("+").clicked() {
                        self.add_normal_step();
                    }
                });

                ui.push_id(ui.next_auto_id(), |ui| {
                    ScrollArea::vertical().show(ui, |ui| {
                        for (i, step) in self.steps.iter_mut().enumerate() {
                            ui.horizontal(|ui| {
                                if ui.button(step.inner.title.to_string()).clicked() && !step.opened
                                {
                                    step.opened = true;
                                }

                                if ui.button("🗑").clicked() {
                                    *action = QuestAction::RemoveStep(i);
                                }
                            });
                        }
                    });
                });

                for (i, step) in self.steps.iter_mut().enumerate() {
                    if step.opened {
                        egui::Window::new(format!("{} [{i}]", step.inner.title))
                            .id(egui::Id::new(10000 * self.id.0 + i as u32))
                            .open(&mut step.opened)
                            .show(ctx, |ui| {
                                step.inner.build(
                                    ui,
                                    i as u32,
                                    &mut step.action,
                                    &mut holders.game_data_holder,
                                );
                            });
                    }
                }
            });
        });

        ui.separator();
    }
}

impl Frontend {
    pub(crate) fn build_quest_selector(
        backend: &mut Backend,
        ui: &mut Ui,
        max_height: f32,
        width: f32,
    ) {
        ui.vertical(|ui| {
            ui.set_width(width);
            ui.set_max_height(max_height);

            if ui.button("    New Quest    ").clicked() && !backend.dialog_showing {
                backend.edit_params.create_new_quest();
            }

            ui.horizontal(|ui| {
                let l = ui.text_edit_singleline(&mut backend.filter_params.quest_filter_string);
                if ui.button("🔍").clicked()
                    || (l.lost_focus() && l.ctx.input(|i| i.key_pressed(Key::Enter)))
                {
                    backend.filter_quests();
                }
            });

            ui.separator();

            ui.push_id(ui.next_auto_id(), |ui| {
                ScrollArea::vertical().show(ui, |ui| {
                    for q in &backend.filter_params.quest_catalog {
                        if ui.button(format!("ID: {}\n{}", q.id.0, q.name)).clicked()
                            && !backend.dialog_showing
                        {
                            backend.edit_params.open_quest(
                                q.id,
                                &mut backend.holders.game_data_holder.quest_holder,
                            );
                        }
                    }
                });
            });
        });
    }
}
