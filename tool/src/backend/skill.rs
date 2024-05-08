use crate::backend::holder::FHashMap;
use crate::backend::{
    Backend, CommonEditorOps, CurrentEntity, EditParams, EntityEditParams, HandleAction,
    WindowParams,
};
use crate::data::SkillId;
use crate::entity::skill::{EnchantInfo, EnchantLevelInfo, Skill, SkillLevelInfo};
use serde::{Deserialize, Serialize};
use std::sync::RwLock;

pub type SkillEditor = EntityEditParams<Skill, SkillId, SkillAction, SkillEditWindowParams>;

impl HandleAction for WindowParams<Skill, SkillId, SkillAction, SkillEditWindowParams> {
    fn handle_action(&mut self) {
        let skill = self;

        let mut action = skill.action.write().unwrap();

        match *action {
            SkillAction::DeleteLevel => {
                skill
                    .inner
                    .skill_levels
                    .remove(skill.params.current_level_index);

                for level in &mut skill.inner.skill_levels[skill.params.current_level_index..] {
                    level.level -= 1
                }

                skill.params.current_level_index = skill
                    .params
                    .current_level_index
                    .min(skill.inner.skill_levels.len() - 1)
                    .max(0)
            }
            SkillAction::AddLevel => {
                let mut new_level_level = 1;
                let mut proto_index = 0;

                for (i, level) in skill.inner.skill_levels.iter().enumerate() {
                    proto_index = i;

                    if level.level > new_level_level {
                        break;
                    }

                    new_level_level += 1;
                }

                let mut new_level = if skill.inner.skill_levels.is_empty() {
                    SkillLevelInfo::default()
                } else {
                    skill.inner.skill_levels[proto_index].clone()
                };

                new_level.level = new_level_level;

                if proto_index != skill.inner.skill_levels.len() - 1 {
                    skill.inner.skill_levels.insert(proto_index, new_level);
                    skill.params.current_level_index = proto_index;
                } else {
                    skill.params.current_level_index = skill.inner.skill_levels.len();
                    skill.inner.skill_levels.push(new_level);
                }
            }
            SkillAction::AddEnchant => {
                let curr_level = &mut skill.inner.skill_levels[skill.params.current_level_index];

                curr_level.available_enchants.push(
                    if let Some(v) = curr_level.available_enchants.last() {
                        let mut r = v.inner.clone();

                        r.enchant_type = v.inner.enchant_type + 1;

                        WindowParams {
                            inner: r,
                            opened: false,
                            initial_id: (),
                            action: RwLock::new(SkillEnchantAction::None),
                            params: SkillEnchantEditWindowParams {
                                current_level_index: v.inner.enchant_levels.len() - 1,
                            },
                        }
                    } else {
                        WindowParams {
                            inner: EnchantInfo::default(),
                            opened: false,
                            initial_id: (),
                            action: RwLock::new(SkillEnchantAction::None),
                            params: SkillEnchantEditWindowParams {
                                current_level_index: 0,
                            },
                        }
                    },
                )
            }
            SkillAction::DeleteEnchant(index) => {
                let curr_level = &mut skill.inner.skill_levels[skill.params.current_level_index];
                curr_level.available_enchants.remove(index);
            }
            SkillAction::AddEnchantLevel(index) => {
                let curr_enchant = &mut skill.inner.skill_levels[skill.params.current_level_index]
                    .available_enchants[index];
                let mut new_level_level = 1;
                let mut proto_index = 0;

                for (i, level) in curr_enchant.inner.enchant_levels.iter().enumerate() {
                    proto_index = i;

                    if level.level > new_level_level {
                        break;
                    }

                    new_level_level += 1;
                }

                let mut new_level = if curr_enchant.inner.enchant_levels.is_empty() {
                    EnchantLevelInfo::default()
                } else {
                    curr_enchant.inner.enchant_levels[proto_index].clone()
                };

                new_level.level = new_level_level;

                if proto_index != curr_enchant.inner.enchant_levels.len() - 1 {
                    curr_enchant
                        .inner
                        .enchant_levels
                        .insert(proto_index, new_level);
                    curr_enchant.params.current_level_index = proto_index;
                } else {
                    curr_enchant.params.current_level_index =
                        curr_enchant.inner.enchant_levels.len();
                    curr_enchant.inner.enchant_levels.push(new_level);
                }
            }
            SkillAction::DeleteEnchantLevel(index) => {
                let curr_enchant = &mut skill.inner.skill_levels[skill.params.current_level_index]
                    .available_enchants[index];
                curr_enchant
                    .inner
                    .enchant_levels
                    .remove(curr_enchant.params.current_level_index);
                curr_enchant.params.current_level_index = curr_enchant
                    .params
                    .current_level_index
                    .min(curr_enchant.inner.enchant_levels.len() - 1)
                    .max(0)
            }

            SkillAction::None => {}
        }

        *action = SkillAction::None;

        if let Some(cond) = &mut skill.inner.use_condition {
            let mut action = cond.action.write().unwrap();

            match *action {
                SkillUceConditionAction::DeleteWeapon(i) => {
                    cond.inner.weapon_types.remove(i);
                }
                SkillUceConditionAction::DeleteEffectOnCaster(i) => {
                    cond.inner.caster_prior_skill.remove(i);
                }
                SkillUceConditionAction::DeleteEffectOnTarget(i) => {
                    cond.inner.target_prior_skill.remove(i);
                }

                SkillUceConditionAction::None => {}
            }

            *action = SkillUceConditionAction::None;
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Default, Debug)]
pub enum SkillAction {
    #[default]
    None,
    DeleteLevel,
    AddLevel,
    AddEnchant,
    DeleteEnchant(usize),
    AddEnchantLevel(usize),
    DeleteEnchantLevel(usize),
}

#[derive(Serialize, Deserialize, Clone, Default, Debug)]
pub enum SkillUceConditionAction {
    #[default]
    None,
    DeleteWeapon(usize),
    DeleteEffectOnCaster(usize),
    DeleteEffectOnTarget(usize),
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub enum SkillEnchantAction {
    #[default]
    None,
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct SkillEditWindowParams {
    pub current_level_index: usize,
}

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct SkillEnchantEditWindowParams {
    pub current_level_index: usize,
}

impl EditParams {
    pub fn get_opened_skills_info(&self) -> Vec<(String, SkillId, bool)> {
        self.skills.get_opened_info()
    }

    pub fn open_skill(&mut self, id: SkillId, holder: &mut FHashMap<SkillId, Skill>) {
        for (i, q) in self.skills.opened.iter().enumerate() {
            if q.inner.initial_id == id {
                self.current_entity = CurrentEntity::Skill(i);

                return;
            }
        }

        if let Some(q) = holder.get(&id) {
            self.current_entity = CurrentEntity::Skill(self.skills.add(q.clone(), q.id));
        }
    }

    pub fn set_current_skill(&mut self, index: usize) {
        if index < self.skills.opened.len() {
            self.current_entity = CurrentEntity::Skill(index);
        }
    }

    pub fn create_new_skill(&mut self) {
        self.current_entity = CurrentEntity::Skill(self.skills.add_new());
    }
}

impl Backend {
    pub fn filter_skills(&mut self) {
        self.entity_catalogs.skill.filter(&self.holders.game_data_holder.skill_holder);
    }

    pub fn save_skill_from_dlg(&mut self, skill_id: SkillId) {
        if let CurrentEntity::Skill(index) = self.edit_params.current_entity {
            let new_skill = self.edit_params.skills.opened.get_mut(index).unwrap();

            if new_skill.inner.inner.id != skill_id {
                return;
            }
            new_skill.inner.initial_id = new_skill.inner.inner.id;

            let skill = new_skill.inner.inner.clone();

            self.save_skill_force(skill);
        }
    }

    pub(crate) fn save_skill_force(&mut self, skill: Skill) {
        if let Some(v) = self.holders.game_data_holder.skill_holder.get(&skill.id) {
            if *v == skill{
                return;
            }
        }
        self.set_changed();

        self.holders
            .game_data_holder
            .skill_holder
            .insert(skill.id, skill);
        self.filter_skills();
    }
}

#[derive(Eq, PartialEq, Ord, PartialOrd)]
pub struct SkillInfo {
    pub(crate) id: SkillId,
    pub(crate) name: String,
}

impl From<&Skill> for SkillInfo {
    fn from(value: &Skill) -> Self {
        SkillInfo {
            id: value.id,
            name: value.name.clone(),
        }
    }
}
