use crate::backend::{Backend, CurrentOpenedEntity, EditParams};
use crate::data::NpcId;
use crate::entity::npc::Npc;
use crate::holders::{FHashMap, NpcInfo};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Serialize, Deserialize, Debug, Copy, Clone, Default)]
pub enum NpcMeshAction {
    #[default]
    None,
    RemoveMeshTexture(usize),
    RemoveMeshAdditionalTexture(usize),
    RemoveMeshDecoration(usize),
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, Default)]
pub enum NpcSoundAction {
    #[default]
    None,
    RemoveSoundDamage(usize),
    RemoveSoundAttack(usize),
    RemoveSoundDefence(usize),
    RemoveSoundDialog(usize),
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, Default)]
pub enum NpcSkillAnimationAction {
    #[default]
    None,
    RemoveSkillAnimation(usize),
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, Default)]
pub enum NpcAction {
    #[default]
    None,
    RemoveProperty(usize),
    RemoveQuest(usize),
}

impl EditParams {
    pub fn get_opened_npcs_info(&self) -> Vec<(String, NpcId)> {
        self.npcs.get_opened_info()
    }

    pub fn open_npc(&mut self, id: NpcId, holder: &mut FHashMap<NpcId, Npc>) {
        for (i, q) in self.npcs.opened.iter().enumerate() {
            if q.original_id == id {
                self.current_opened_entity = CurrentOpenedEntity::Npc(i);

                return;
            }
        }

        if let Some(q) = holder.get(&id) {
            self.current_opened_entity = CurrentOpenedEntity::Npc(self.npcs.add(q.clone(), q.id));
        }
    }

    pub fn set_current_npc(&mut self, index: usize) {
        if index < self.npcs.opened.len() {
            self.current_opened_entity = CurrentOpenedEntity::Npc(index);
        }
    }

    pub fn close_npc(&mut self, index: usize) {
        self.npcs.opened.remove(index);

        if let CurrentOpenedEntity::Npc(curr_index) = self.current_opened_entity {
            if self.npcs.opened.is_empty() {
                self.find_opened_entity();
            } else if curr_index >= index {
                self.current_opened_entity = CurrentOpenedEntity::Npc(curr_index.max(1) - 1)
            }
        }
    }

    pub fn create_new_npc(&mut self) {
        self.current_opened_entity = CurrentOpenedEntity::Npc(self.npcs.add_new());
    }
}

impl Backend {
    pub fn filter_npcs(&mut self) {
        let s = self.filter_params.npc_filter_string.to_lowercase();

        let fun: Box<dyn Fn(&&Npc) -> bool> = if s.is_empty() {
            Box::new(|_: &&Npc| true)
        } else if let Ok(id) = u32::from_str(&s) {
            Box::new(move |v: &&Npc| v.id == NpcId(id))
        } else {
            Box::new(move |v: &&Npc| v.name.to_lowercase().contains(&s))
        };

        self.filter_params.npc_catalog = self
            .holders
            .game_data_holder
            .npc_holder
            .values()
            .filter(fun)
            .map(NpcInfo::from)
            .collect();

        self.filter_params
            .npc_catalog
            .sort_by(|a, b| a.id.cmp(&b.id))
    }

    pub fn save_npc_from_dlg(&mut self, npc_id: NpcId) {
        if let CurrentOpenedEntity::Npc(index) = self.edit_params.current_opened_entity {
            let new_npc = self.edit_params.npcs.opened.get(index).unwrap();

            if new_npc.inner.id != npc_id {
                return;
            }

            self.save_npc_force(new_npc.inner.clone());
        }
    }

    pub(crate) fn save_npc_force(&mut self, npc: Npc) {
        self.holders.game_data_holder.npc_holder.insert(npc.id, npc);

        self.filter_npcs();
    }
}