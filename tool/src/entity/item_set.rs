use crate::data::{ItemId, ItemSetId};
use crate::entity::CommonEntity;
use serde::{Deserialize, Serialize};

impl CommonEntity<ItemSetId, ()> for ItemSet {
    fn name(&self) -> String {
        self.id.0.to_string()
    }

    fn desc(&self) -> String {
        self.base_descriptions
            .iter()
            .enumerate()
            .map(|v| format!("{}: {}\n", v.0 + 1, v.1))
            .collect()
    }

    fn id(&self) -> ItemSetId {
        self.id
    }

    fn edit_params(&self) {}

    fn new(id: ItemSetId) -> Self {
        Self {
            id,
            base_items: vec![],
            base_descriptions: vec![],
            additional_items: vec![],
            additional_descriptions: vec![],
            unk1: 0,
            unk2: 0,
            enchant_info: vec![],
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ItemSetEnchantInfo {
    pub(crate) enchant_level: u32,
    pub(crate) enchant_description: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ItemSet {
    pub(crate) id: ItemSetId,

    pub(crate) base_items: Vec<Vec<ItemId>>,
    pub(crate) base_descriptions: Vec<String>,

    pub(crate) additional_items: Vec<Vec<ItemId>>,
    pub(crate) additional_descriptions: Vec<String>,

    pub(crate) unk1: u32,
    pub(crate) unk2: u32,

    pub(crate) enchant_info: Vec<ItemSetEnchantInfo>,
}