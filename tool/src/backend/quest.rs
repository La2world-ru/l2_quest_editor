use crate::backend::{Backend, CurrentOpenedEntity, EditParams};
use crate::data::QuestId;
use crate::entity::quest::Quest;
use crate::holders::{FHashMap, QuestInfo};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Serialize, Deserialize, Debug, Copy, Clone, Default)]
pub enum StepAction {
    #[default]
    None,
    RemoveGoal(usize),
    RemoveAdditionalLocation(usize),
    RemovePrevStepIndex(usize),
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub enum QuestAction {
    #[default]
    None,
    RemoveStep(usize),
    RemoveStartNpcId(usize),
    RemoveReward(usize),
    RemoveQuestItem(usize),
}

impl EditParams {
    pub fn get_opened_quests_info(&self) -> Vec<(String, QuestId)> {
        self.quests.get_opened_info()
    }

    pub fn open_quest(&mut self, id: QuestId, holder: &mut FHashMap<QuestId, Quest>) {
        for (i, q) in self.quests.opened.iter().enumerate() {
            if q.original_id == id {
                self.current_opened_entity = CurrentOpenedEntity::Quest(i);

                return;
            }
        }

        if let Some(q) = holder.get(&id) {
            self.current_opened_entity =
                CurrentOpenedEntity::Quest(self.quests.add(q.clone(), q.id));
        }
    }

    pub fn set_current_quest(&mut self, index: usize) {
        if index < self.quests.opened.len() {
            self.current_opened_entity = CurrentOpenedEntity::Quest(index);
        }
    }

    pub fn close_quest(&mut self, index: usize) {
        self.quests.opened.remove(index);

        if let CurrentOpenedEntity::Quest(curr_index) = self.current_opened_entity {
            if self.quests.opened.is_empty() {
                self.find_opened_entity();
            } else if curr_index >= index {
                self.current_opened_entity = CurrentOpenedEntity::Quest(curr_index.max(1) - 1)
            }
        }
    }

    pub fn create_new_quest(&mut self) {
        self.current_opened_entity = CurrentOpenedEntity::Quest(self.quests.add_new());
    }
}

impl Backend {
    pub fn filter_quests(&mut self) {
        let s = self.filter_params.quest_filter_string.clone();
        let fun: Box<dyn Fn(&&Quest) -> bool> = if s.is_empty() {
            Box::new(|_: &&Quest| true)
        } else if let Ok(id) = u32::from_str(&s) {
            Box::new(move |v: &&Quest| v.id == QuestId(id))
        } else {
            Box::new(move |v: &&Quest| v.title.contains(&s))
        };

        self.filter_params.quest_catalog = self
            .holders
            .game_data_holder
            .quest_holder
            .values()
            .filter(fun)
            .map(QuestInfo::from)
            .collect();
        self.filter_params
            .quest_catalog
            .sort_by(|a, b| a.id.cmp(&b.id))
    }

    pub fn save_quest_from_dlg(&mut self, quest_id: QuestId) {
        if let CurrentOpenedEntity::Quest(index) = self.edit_params.current_opened_entity {
            let new_quest = self.edit_params.quests.opened.get(index).unwrap();

            if new_quest.inner.id != quest_id {
                return;
            }

            self.save_quest_force(new_quest.inner.clone());
        }
    }

    pub(crate) fn save_quest_force(&mut self, mut quest: Quest) {
        if let Some(java_class) = quest.java_class {
            self.holders.server_data_holder.save_java_class(
                quest.id,
                &quest.title,
                java_class.inner,
                &self.config.server_quests_java_classes_path,
            )
        }

        quest.java_class = None;

        self.holders
            .game_data_holder
            .quest_holder
            .insert(quest.id, quest);
        self.filter_quests();
    }
}