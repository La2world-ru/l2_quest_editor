#![allow(clippy::needless_borrow)]
mod item;
mod item_set;
mod npc;
mod quest;
mod recipe;
mod skill;

use crate::data::{
    HuntingZoneId, InstantZoneId, ItemId, ItemSetId, Location, NpcId, Position, QuestId, RecipeId,
    SearchZoneId, SkillId,
};
use crate::entity::hunting_zone::HuntingZone;
use crate::entity::item::Item;
use crate::entity::npc::Npc;
use crate::entity::quest::Quest;
use crate::entity::skill::Skill;
use crate::frontend::IS_SAVING;
use crate::holder::{ChroniclesProtocol, FHashMap, GameDataHolder, Loader};
use crate::util::l2_reader::{deserialize_dat, save_dat, DatVariant};
use crate::util::{
    L2StringTable, ReadUnreal, UnrealReader, UnrealWriter, WriteUnreal, ASCF, DWORD, FLOAT, FLOC,
    STR, UVEC, WORD,
};

use crate::entity::item::armor::Armor;
use crate::entity::item::etc_item::EtcItem;
use crate::entity::item::weapon::Weapon;
use crate::entity::item_set::ItemSet;
use crate::entity::recipe::Recipe;
use crate::entity::CommonEntity;
use r#macro::{ReadUnreal, WriteUnreal};
use std::collections::hash_map::Keys;
use std::collections::HashMap;
use std::ops::Index;
use std::path::Path;
use std::thread;
use walkdir::DirEntry;

#[derive(Default, Clone)]
pub struct L2GeneralStringTable {
    was_changed: bool,
    next_index: u32,
    inner: HashMap<u32, String>,
    reverse_map: HashMap<String, u32>,
}

impl L2GeneralStringTable {
    fn to_vec(&self) -> Vec<String> {
        let mut k: Vec<_> = self.keys().collect();
        k.sort();

        let mut res = Vec::with_capacity(k.len());

        for key in k {
            res.push(self.inner.get(key).unwrap().clone());
        }

        res
    }
}

impl L2StringTable for L2GeneralStringTable {
    fn keys(&self) -> Keys<u32, String> {
        self.inner.keys()
    }

    fn get(&self, key: &u32) -> Option<&String> {
        self.inner.get(key)
    }

    fn get_o(&self, key: &u32) -> String {
        self.inner
            .get(key)
            .cloned()
            .unwrap_or_else(|| format!("NameNotFound[{}]", key))
    }

    fn from_vec(values: Vec<String>) -> Self {
        let mut s = Self::default();

        for v in values {
            s.add(v);
        }

        s
    }

    fn get_index(&mut self, mut value: &str) -> u32 {
        const NONE_STR: &str = &"None";

        if value.is_empty() {
            value = &NONE_STR
        }

        if let Some(i) = self.reverse_map.get(&value.to_lowercase()) {
            *i
        } else {
            self.was_changed = true;
            self.add(value.to_string())
        }
    }

    fn add(&mut self, value: String) -> u32 {
        self.reverse_map
            .insert(value.to_lowercase(), self.next_index);
        self.inner.insert(self.next_index, value);
        self.next_index += 1;

        self.next_index - 1
    }
}

#[derive(Default, Clone)]
pub struct L2SkillStringTable {
    next_index: u32,
    inner: HashMap<u32, String>,
    reverse_map: HashMap<String, u32>,
}

impl L2StringTable for L2SkillStringTable {
    fn keys(&self) -> Keys<u32, String> {
        self.inner.keys()
    }
    fn get(&self, key: &u32) -> Option<&String> {
        self.inner.get(key)
    }
    fn get_o(&self, key: &u32) -> String {
        self.inner
            .get(key)
            .cloned()
            .unwrap_or_else(|| format!("NameNotFound[{}]", key))
    }
    fn from_vec(values: Vec<String>) -> Self {
        let mut s = Self::default();

        for v in values {
            s.add(v);
        }

        s
    }

    fn get_index(&mut self, value: &str) -> u32 {
        if let Some(i) = self.reverse_map.get(value) {
            *i
        } else {
            self.add(value.to_string())
        }
    }

    fn add(&mut self, value: String) -> u32 {
        self.reverse_map.insert(value.clone(), self.next_index);
        self.inner.insert(self.next_index, value);
        self.next_index += 1;

        self.next_index - 1
    }
}

impl Index<usize> for L2SkillStringTable {
    type Output = String;

    fn index(&self, index: usize) -> &Self::Output {
        self.inner.get(&(index as u32)).unwrap()
    }
}

impl Index<u32> for L2SkillStringTable {
    type Output = String;

    fn index(&self, index: u32) -> &Self::Output {
        self.inner.get(&index).unwrap()
    }
}

impl Index<usize> for L2GeneralStringTable {
    type Output = String;

    fn index(&self, index: usize) -> &Self::Output {
        self.inner.get(&(index as u32)).unwrap()
    }
}

impl Index<u32> for L2GeneralStringTable {
    type Output = String;

    fn index(&self, index: u32) -> &Self::Output {
        self.inner.get(&index).unwrap()
    }
}

impl Index<&u32> for L2GeneralStringTable {
    type Output = String;

    fn index(&self, index: &u32) -> &Self::Output {
        self.inner.get(index).unwrap()
    }
}

#[derive(Default)]
pub struct Loader110 {
    game_data_name: L2GeneralStringTable,
    dat_paths: HashMap<String, DirEntry>,

    quests: FHashMap<QuestId, Quest>,
    skills: FHashMap<SkillId, Skill>,
    npcs: FHashMap<NpcId, Npc>,

    all_items: HashMap<ItemId, Item>,
    weapons: FHashMap<ItemId, Weapon>,
    armor: FHashMap<ItemId, Armor>,
    etc_items: FHashMap<ItemId, EtcItem>,

    item_sets: FHashMap<ItemSetId, ItemSet>,
    recipes: FHashMap<RecipeId, Recipe>,

    npc_strings: FHashMap<u32, String>,
    hunting_zones: FHashMap<HuntingZoneId, HuntingZone>,
}

impl Loader for Loader110 {
    fn load(&mut self, dat_paths: HashMap<String, DirEntry>) -> Result<(), ()> {
        let Some(path) = dat_paths.get(&"l2gamedataname.dat".to_string()) else {
            return Err(());
        };

        self.game_data_name = Self::load_game_data_name(path.path())?;
        self.dat_paths = dat_paths;

        self.load_npcs()?;
        self.load_npc_strings()?;
        self.load_items()?;
        self.load_hunting_zones()?;
        self.load_quests()?;
        self.load_skills()?;
        self.load_item_sets()?;
        self.load_recipes()?;

        self.refill_all_items();

        println!("======================================");
        println!("\tLoaded {} Npcs", self.npcs.len());
        println!("\tLoaded {} Npc Strings", self.npc_strings.len());
        println!("\tLoaded {} Hunting Zones", self.hunting_zones.len());
        println!("\tLoaded {} Quests", self.quests.len());
        println!("\tLoaded {} Skills", self.skills.len());
        println!("\tLoaded {} Items", self.all_items.len());
        println!("\t\t Weapons: {}", self.weapons.len());
        println!("\t\t EtcItems: {}", self.etc_items.len());
        println!("\t\t Armor: {}", self.armor.len());
        println!("\tLoaded {} Item Sets", self.item_sets.len());
        println!("\tLoaded {} Recipes", self.recipes.len());
        println!("======================================");

        Ok(())
    }

    fn from_holder(game_data_holder: &GameDataHolder) -> Self {
        let items_changed = game_data_holder.armor_holder.was_changed
            || game_data_holder.etc_item_holder.was_changed
            || game_data_holder.weapon_holder.was_changed;

        Self {
            dat_paths: game_data_holder.initial_dat_paths.clone(),

            quests: game_data_holder.quest_holder.changed_or_empty(),

            game_data_name: game_data_holder.game_string_table.clone(),

            skills: game_data_holder.skill_holder.changed_or_empty(),

            npcs: game_data_holder.npc_holder.changed_or_empty(),

            npc_strings: game_data_holder.npc_strings.changed_or_empty(),

            all_items: Default::default(),

            weapons: if items_changed {
                game_data_holder.weapon_holder.clone()
            } else {
                FHashMap::new()
            },
            etc_items: if items_changed {
                game_data_holder.etc_item_holder.clone()
            } else {
                FHashMap::new()
            },
            armor: if items_changed {
                game_data_holder.armor_holder.clone()
            } else {
                FHashMap::new()
            },

            item_sets: game_data_holder.item_set_holder.changed_or_empty(),
            recipes: game_data_holder.recipe_holder.changed_or_empty(),

            hunting_zones: Default::default(),
        }
    }

    fn to_holder(self) -> GameDataHolder {
        let mut r = GameDataHolder {
            protocol_version: ChroniclesProtocol::GrandCrusade110,
            initial_dat_paths: self.dat_paths,
            npc_holder: self.npcs,
            npc_strings: self.npc_strings,
            item_holder: self.all_items,
            quest_holder: self.quests,
            skill_holder: self.skills,
            weapon_holder: self.weapons,
            armor_holder: self.armor,
            etc_item_holder: self.etc_items,
            item_set_holder: self.item_sets,
            recipe_holder: self.recipes,

            hunting_zone_holder: self.hunting_zones,
            game_string_table: self.game_data_name,
        };

        r.npc_holder.was_changed = false;
        r.npc_strings.was_changed = false;
        r.quest_holder.was_changed = false;
        r.skill_holder.was_changed = false;
        r.weapon_holder.was_changed = false;
        r.armor_holder.was_changed = false;
        r.etc_item_holder.was_changed = false;
        r.hunting_zone_holder.was_changed = false;
        r.item_set_holder.was_changed = false;
        r.recipe_holder.was_changed = false;

        r
    }

    fn serialize_to_binary(&mut self) -> std::io::Result<()> {
        *IS_SAVING.write().unwrap() = true;

        let skills_handle = if self.skills.was_changed {
            Some(self.serialize_skills_to_binary())
        } else {
            println!("Skills are unchanged");
            None
        };
        let quest_handle = if self.quests.was_changed {
            Some(self.serialize_quests_to_binary())
        } else {
            println!("Quests are unchanged");
            None
        };

        let npcs_handle = if self.quests.was_changed {
            Some(self.serialize_npcs_to_binary())
        } else {
            println!("Npcs are unchanged");
            None
        };

        let items_handle =
            if self.weapons.was_changed || self.etc_items.was_changed || self.armor.was_changed {
                Some(self.serialize_items_to_binary())
            } else {
                println!("Items are unchanged");
                None
            };

        let item_sets_handle = if self.item_sets.was_changed {
            Some(self.serialize_item_sets_to_binary())
        } else {
            println!("Item Sets are unchanged");
            None
        };

        let recipes_handle = if self.recipes.was_changed {
            Some(self.serialize_recipes_to_binary())
        } else {
            println!("Recipes are unchanged");
            None
        };

        let gdn_changed = self.game_data_name.was_changed;

        let l2_game_data_name_values = self.game_data_name.to_vec();
        let l2_game_data_name = self
            .dat_paths
            .get(&"l2gamedataname.dat".to_string())
            .unwrap()
            .clone();

        thread::spawn(move || {
            let gdn_handel = if gdn_changed {
                Some(thread::spawn(move || {
                    if let Err(e) = save_dat(
                        l2_game_data_name.path(),
                        DatVariant::<(), String>::Array(l2_game_data_name_values),
                    ) {
                        println!("{e:?}");
                    } else {
                        println!("Game Data Name saved");
                    }
                }))
            } else {
                println!("GameDataName is unchanged");
                None
            };

            if let Some(c) = gdn_handel {
                let _ = c.join();
            }

            if let Some(v) = skills_handle {
                let _ = v.join();
            }

            if let Some(v) = quest_handle {
                let _ = v.join();
            }

            if let Some(v) = npcs_handle {
                let _ = v.join();
            }

            if let Some(v) = items_handle {
                let _ = v.join();
            }

            if let Some(v) = item_sets_handle {
                let _ = v.join();
            }

            if let Some(v) = recipes_handle {
                let _ = v.join();
            }

            println!("Binaries Saved");

            *IS_SAVING.write().unwrap() = false;
        });

        Ok(())
    }
}

impl Loader110 {
    /**Returns cloned String from `game data name`
     */
    fn gdns_cloned(&self, index: &u32) -> String {
        self.game_data_name[index].clone()
    }

    /**Returns Vector of cloned Strings from `game data name`
     */
    fn vec_gdns_cloned(&self, indexes: &Vec<u32>) -> Vec<String> {
        indexes
            .iter()
            .map(|v| self.game_data_name[v].clone())
            .collect()
    }

    fn load_game_data_name(path: &Path) -> Result<L2GeneralStringTable, ()> {
        match deserialize_dat(path) {
            Ok(r) => Ok(L2GeneralStringTable::from_vec(r)),
            Err(e) => Err(e),
        }
    }

    fn load_hunting_zones(&mut self) -> Result<(), ()> {
        let vals = deserialize_dat::<HuntingZoneDat>(
            self.dat_paths
                .get(&"huntingzone-ru.dat".to_string())
                .unwrap()
                .path(),
        )?;

        for v in vals {
            self.hunting_zones.insert(
                HuntingZoneId(v.id),
                HuntingZone {
                    id: HuntingZoneId(v.id),
                    name: v.name.0.clone(),
                    desc: v.description.0.clone(),
                    _search_zone_id: SearchZoneId(v.search_zone_id),
                    _instant_zone_id: InstantZoneId(v.instance_zone_id),
                },
            );
        }

        Ok(())
    }

    fn load_npc_strings(&mut self) -> Result<(), ()> {
        let vals = deserialize_dat::<NpcStringDat>(
            self.dat_paths
                .get(&"npcstring-ru.dat".to_string())
                .unwrap()
                .path(),
        )?;

        for v in vals {
            self.npc_strings.insert(v.id, v.value.0);
        }

        Ok(())
    }

    fn refill_all_items(&mut self) {
        self.all_items.clear();

        self.all_items
            .extend(self.weapons.values().map(|v| (v.id(), v.into())));
        self.all_items
            .extend(self.etc_items.values().map(|v| (v.id(), v.into())));
        self.all_items
            .extend(self.armor.values().map(|v| (v.id(), v.into())));
    }
}

#[derive(Debug, Clone, PartialEq, ReadUnreal, WriteUnreal)]
struct L2GameDataNameDat {
    value: STR,
}

#[derive(Debug, Clone, PartialEq, ReadUnreal, WriteUnreal)]
struct NpcStringDat {
    id: DWORD,
    value: ASCF,
}

impl From<FLOC> for Location {
    fn from(val: FLOC) -> Self {
        Location {
            x: val.x as i32,
            y: val.y as i32,
            z: val.z as i32,
        }
    }
}

impl From<Location> for FLOC {
    fn from(val: Location) -> Self {
        FLOC {
            x: val.x as f32,
            y: val.y as f32,
            z: val.z as f32,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, ReadUnreal, WriteUnreal, Default)]
pub struct CoordsXYZ {
    x: FLOAT,
    y: FLOAT,
    z: FLOAT,
}

impl From<CoordsXYZ> for Position {
    fn from(value: CoordsXYZ) -> Self {
        Position {
            x: value.x,
            y: value.y,
            z: value.z,
        }
    }
}

impl From<Position> for CoordsXYZ {
    fn from(value: Position) -> Self {
        CoordsXYZ {
            x: value.x,
            y: value.y,
            z: value.z,
        }
    }
}

#[derive(Debug, Clone, PartialEq, ReadUnreal, WriteUnreal, Default)]
struct HuntingZoneDat {
    id: DWORD,
    zone_type: DWORD,
    min_recommended_level: DWORD,
    max_recommended_level: DWORD,
    start_npc_loc: FLOC,
    description: ASCF,
    search_zone_id: DWORD,
    name: ASCF,
    region_id: WORD,
    npc_id: DWORD,
    quest_ids: Vec<WORD>,
    instance_zone_id: DWORD,
}

#[derive(Debug, Clone, PartialEq, ReadUnreal, WriteUnreal)]
struct EnchantedWeaponFlowEffectDataDat {
    name: DWORD,
    groups: UVEC<DWORD, FlowEffect>,
}
#[derive(Debug, Clone, PartialEq, ReadUnreal, WriteUnreal)]
struct FlowEffect {
    start_enchant_value: DWORD,
    right_main_flow_effect: DWORD,
    left_main_flow_effect: DWORD,
    right_variation_flow_effect: DWORD,
    left_variation_flow_effect: DWORD,
}

#[derive(Debug, Clone, PartialEq, ReadUnreal, WriteUnreal)]
struct EnchantStatBonusDat {
    weapon_grade: DWORD,
    magic_weapon: DWORD,
    unk1: DWORD,
    weapon_type: Vec<DWORD>,
    soulshot_power: FLOAT,
    spiritshot_power: FLOAT,
}