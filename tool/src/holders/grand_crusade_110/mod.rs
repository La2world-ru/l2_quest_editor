#![allow(clippy::needless_borrow)]
mod item;
mod npc;
mod quest;
mod skill;

use crate::data::{
    HuntingZoneId, InstantZoneId, ItemId, Location, NpcId, Position, QuestId, SearchZoneId, SkillId,
};
use crate::entity::hunting_zone::HuntingZone;
use crate::entity::item::Item;
use crate::entity::npc::Npc;
use crate::entity::quest::Quest;
use crate::entity::skill::Skill;
use crate::frontend::IS_SAVING;
use crate::holders::{ChroniclesProtocol, FHashMap, GameDataHolder, Loader};
use crate::util::l2_reader::{deserialize_dat, save_dat, DatVariant};
use crate::util::{
    L2StringTable, ReadUnreal, UnrealReader, UnrealWriter, WriteUnreal, ASCF, DWORD, FLOAT, FLOC,
    STR, UVEC, WORD,
};

use crate::entity::item::weapon::Weapon;
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
    npcs: FHashMap<NpcId, Npc>,
    npc_strings: FHashMap<u32, String>,
    //TODO: remove when weapon, armor and etc will be ready!
    items: FHashMap<ItemId, Item>,
    weapons: FHashMap<ItemId, Weapon>,
    quests: FHashMap<QuestId, Quest>,
    hunting_zones: FHashMap<HuntingZoneId, HuntingZone>,
    skills: FHashMap<SkillId, Skill>,
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

        println!("======================================");
        println!("\tLoaded {} Npcs", self.npcs.len());
        println!("\tLoaded {} Npc Strings", self.npc_strings.len());
        println!("\tLoaded {} Hunting Zones", self.hunting_zones.len());
        println!("\tLoaded {} Quests", self.quests.len());
        println!("\tLoaded {} Skills", self.skills.len());
        println!("\tLoaded {} Items", self.items.len());
        println!("\t\t Weapons: {}", self.weapons.len());
        println!("======================================");

        Ok(())
    }

    fn from_holder(game_data_holder: &GameDataHolder) -> Self {
        Self {
            dat_paths: game_data_holder.initial_dat_paths.clone(),
            quests: if game_data_holder.quest_holder.was_changed {
                game_data_holder.quest_holder.clone()
            } else {
                FHashMap::new()
            },
            game_data_name: game_data_holder.game_string_table.clone(),
            skills: if game_data_holder.skill_holder.was_changed {
                game_data_holder.skill_holder.clone()
            } else {
                FHashMap::new()
            },
            npcs: if game_data_holder.npc_holder.was_changed {
                game_data_holder.npc_holder.clone()
            } else {
                FHashMap::new()
            },
            npc_strings: if game_data_holder.npc_strings.was_changed {
                game_data_holder.npc_strings.clone()
            } else {
                FHashMap::new()
            },
            ..Default::default()
        }
    }

    fn to_holder(self) -> GameDataHolder {
        GameDataHolder {
            protocol_version: ChroniclesProtocol::GrandCrusade110,
            initial_dat_paths: self.dat_paths,
            npc_holder: self.npcs,
            npc_strings: self.npc_strings,
            item_holder: self.items,
            quest_holder: self.quests,
            skill_holder: self.skills,
            weapon_holder: self.weapons,
            hunting_zone_holder: self.hunting_zones,
            game_string_table: self.game_data_name,
        }
    }

    fn serialize_to_binary(&mut self) -> std::io::Result<()> {
        *IS_SAVING.write().unwrap() = true;

        let skills_handle = if self.skills.was_changed {
            Some(self.serialize_skills_to_binary())
        } else {
            println!("Skills are unchanged");
            None
        };
        let quest_handle = if self.skills.was_changed {
            Some(self.serialize_quests_to_binary())
        } else {
            println!("Quests are unchanged");
            None
        };
        let npcs_handle = if self.skills.was_changed {
            Some(self.serialize_npcs_to_binary())
        } else {
            println!("Npcs are unchanged");
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