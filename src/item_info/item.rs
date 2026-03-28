use std::io::{self, Write};

use super::keys::*;
use super::structs::*;
use crate::binary::*;

#[derive(Debug)]
pub struct ItemInfo<'a> {
    pub key: ItemKey,
    pub string_key: CString<'a>,
    pub is_blocked: u8,
    pub max_stack_count: u64,
    pub item_name: LocalizableString<'a>,
    pub broken_item_prefix_string: LocalStringInfoKey,
    pub inventory_info: InventoryKey,
    pub equip_type_info: EquipTypeKey,
    pub occupied_equip_slot_data_list: CArray<OccupiedEquipSlotData>,
    pub item_tag_list: CArray<u32>,
    pub equipable_hash: u32,
    pub consumable_type_list: CArray<u32>,
    pub item_use_info_list: CArray<ItemUseKey>,
    pub item_icon_list: CArray<ItemIconData>,
    pub map_icon_path: StringInfoKey,
    pub money_icon_path: StringInfoKey,
    pub use_map_icon_alert: u8,
    pub item_type: u8,
    pub material_key: u32,
    pub material_match_info: MaterialMatchKey,
    pub item_desc: LocalizableString<'a>,
    pub item_desc2: LocalizableString<'a>,
    pub equipable_level: u32,
    pub category_info: CategoryKey,
    pub knowledge_info: KnowledgeKey,
    pub knowledge_obtain_type: u8,
    pub destroy_effec_info: EffectKey,
    pub equip_passive_skill_list: CArray<PassiveSkillLevel>,
    pub use_immediately: u8,
    pub apply_max_stack_cap: u8,
    pub extract_multi_change_info: MultiChangeKey,
    pub item_memo: CString<'a>,
    pub filter_type: CString<'a>,
    pub gimmick_info: GimmickInfoKey,
    pub gimmick_tag_list: CArray<CString<'a>>,
    pub max_drop_result_sub_item_count: u32,
    pub use_drop_set_target: u8,
    pub is_all_gimmick_sealable: u8,
    pub sealable_item_info_list: CArray<SealableItemInfo<'a>>,
    pub sealable_character_info_list: CArray<SealableItemInfo<'a>>,
    pub sealable_gimmick_info_list: CArray<SealableItemInfo<'a>>,
    pub sealable_gimmick_tag_list: CArray<SealableItemInfo<'a>>,
    pub sealable_tribe_info_list: CArray<SealableItemInfo<'a>>,
    pub sealable_money_info_list: CArray<ItemKey>,
    pub delete_by_gimmick_unlock: u8,
    pub gimmick_unlock_message_local_string_info: LocalStringInfoKey,
    pub can_disassemble: u8,
    pub transmutation_material_gimmick_list: CArray<GimmickInfoKey>,
    pub transmutation_material_item_list: CArray<ItemKey>,
    pub transmutation_material_item_group_list: CArray<ItemGroupKey>,
    pub is_register_trade_market: u8,
    pub multi_change_info_list: CArray<MultiChangeKey>,
    pub is_editor_usable: u8,
    pub discardable: u8,
    pub is_dyeable: u8,
    pub is_editable_grime: u8,
    pub is_destroy_when_broken: u8,
    pub quick_slot_index: u8,
    pub reserve_slot_target_data_list: CArray<ReserveSlotTargetData>,
    pub item_tier: u8,
    pub is_important_item: u8,
    pub apply_drop_stat_type: u8,
    pub drop_default_data: DropDefaultData,
    pub prefab_data_list: CArray<PrefabData>,
    pub enchant_data_list: CArray<EnchantData>,
    pub gimmick_visual_prefab_data_list: CArray<GimmickVisualPrefabData>,
    pub price_list: CArray<ItemPriceInfo>,
    pub docking_child_data: COptional<DockingChildData<'a>>,
    pub inventory_change_data: COptional<InventoryChangeData>,
    pub fixed_page_data_list: CArray<PageData<'a>>,
    pub dynamic_page_data_list: CArray<PageData<'a>>,
    pub inspect_data_list: CArray<InspectData<'a>>,
    pub inspect_action: InspectAction<'a>,
    pub default_sub_item: SubItem,
    pub cooltime: i64,
    pub item_charge_type: u8,
    pub sharpness_data: ItemInfoSharpnessData,
    pub max_charged_useable_count: u32,
    pub hackable_character_group_info_list: CArray<CharacterGroupKey>,
    pub item_group_info_list: CArray<ItemGroupKey>,
    pub discard_offset_y: f32,
    pub hide_from_inventory_on_pop_item: u8,
    pub is_shield_item: u8,
    pub is_tower_shield_item: u8,
    pub is_wild: u8,
    pub packed_item_info: ItemKey,
    pub unpacked_item_info: ItemKey,
    pub convert_item_info_by_drop_npc: ItemKey,
    pub look_detail_game_advice_info_wrapper: GameAdviceInfoKey,
    pub look_detail_mission_info: MissionKey,
    pub enable_alert_system_to_ui: u8,
    pub usable_alert: u8,
    pub is_save_game_data_at_use_item: u8,
    pub is_logout_at_use_item: u8,
    pub shared_cool_time_group_name_hash: u32,
    pub item_bundle_data_list: CArray<ItemBundleData>,
    pub money_type_define: COptional<MoneyTypeDefine<'a>>,
    pub emoji_texture_id: CString<'a>,
    pub enable_equip_in_clone_actor: u8,
    pub is_blocked_store_sell: u8,
    pub is_preorder_item: u8,
    pub respawn_time_seconds: i64,
    pub max_endurance: u16,
    pub repair_data_list: CArray<RepairData>,
}

impl<'a> BinaryRead<'a> for ItemInfo<'a> {
    fn read_from(data: &'a [u8], offset: &mut usize) -> io::Result<Self> {
        Ok(ItemInfo {
            key: BinaryRead::read_from(data, offset)?,
            string_key: BinaryRead::read_from(data, offset)?,
            is_blocked: BinaryRead::read_from(data, offset)?,
            max_stack_count: BinaryRead::read_from(data, offset)?,
            item_name: BinaryRead::read_from(data, offset)?,
            broken_item_prefix_string: BinaryRead::read_from(data, offset)?,
            inventory_info: BinaryRead::read_from(data, offset)?,
            equip_type_info: BinaryRead::read_from(data, offset)?,
            occupied_equip_slot_data_list: BinaryRead::read_from(data, offset)?,
            item_tag_list: BinaryRead::read_from(data, offset)?,
            equipable_hash: BinaryRead::read_from(data, offset)?,
            consumable_type_list: BinaryRead::read_from(data, offset)?,
            item_use_info_list: BinaryRead::read_from(data, offset)?,
            item_icon_list: BinaryRead::read_from(data, offset)?,
            map_icon_path: BinaryRead::read_from(data, offset)?,
            money_icon_path: BinaryRead::read_from(data, offset)?,
            use_map_icon_alert: BinaryRead::read_from(data, offset)?,
            item_type: BinaryRead::read_from(data, offset)?,
            material_key: BinaryRead::read_from(data, offset)?,
            material_match_info: BinaryRead::read_from(data, offset)?,
            item_desc: BinaryRead::read_from(data, offset)?,
            item_desc2: BinaryRead::read_from(data, offset)?,
            equipable_level: BinaryRead::read_from(data, offset)?,
            category_info: BinaryRead::read_from(data, offset)?,
            knowledge_info: BinaryRead::read_from(data, offset)?,
            knowledge_obtain_type: BinaryRead::read_from(data, offset)?,
            destroy_effec_info: BinaryRead::read_from(data, offset)?,
            equip_passive_skill_list: BinaryRead::read_from(data, offset)?,
            use_immediately: BinaryRead::read_from(data, offset)?,
            apply_max_stack_cap: BinaryRead::read_from(data, offset)?,
            extract_multi_change_info: BinaryRead::read_from(data, offset)?,
            item_memo: BinaryRead::read_from(data, offset)?,
            filter_type: BinaryRead::read_from(data, offset)?,
            gimmick_info: BinaryRead::read_from(data, offset)?,
            gimmick_tag_list: BinaryRead::read_from(data, offset)?,
            max_drop_result_sub_item_count: BinaryRead::read_from(data, offset)?,
            use_drop_set_target: BinaryRead::read_from(data, offset)?,
            is_all_gimmick_sealable: BinaryRead::read_from(data, offset)?,
            sealable_item_info_list: BinaryRead::read_from(data, offset)?,
            sealable_character_info_list: BinaryRead::read_from(data, offset)?,
            sealable_gimmick_info_list: BinaryRead::read_from(data, offset)?,
            sealable_gimmick_tag_list: BinaryRead::read_from(data, offset)?,
            sealable_tribe_info_list: BinaryRead::read_from(data, offset)?,
            sealable_money_info_list: BinaryRead::read_from(data, offset)?,
            delete_by_gimmick_unlock: BinaryRead::read_from(data, offset)?,
            gimmick_unlock_message_local_string_info: BinaryRead::read_from(data, offset)?,
            can_disassemble: BinaryRead::read_from(data, offset)?,
            transmutation_material_gimmick_list: BinaryRead::read_from(data, offset)?,
            transmutation_material_item_list: BinaryRead::read_from(data, offset)?,
            transmutation_material_item_group_list: BinaryRead::read_from(data, offset)?,
            is_register_trade_market: BinaryRead::read_from(data, offset)?,
            multi_change_info_list: BinaryRead::read_from(data, offset)?,
            is_editor_usable: BinaryRead::read_from(data, offset)?,
            discardable: BinaryRead::read_from(data, offset)?,
            is_dyeable: BinaryRead::read_from(data, offset)?,
            is_editable_grime: BinaryRead::read_from(data, offset)?,
            is_destroy_when_broken: BinaryRead::read_from(data, offset)?,
            quick_slot_index: BinaryRead::read_from(data, offset)?,
            reserve_slot_target_data_list: BinaryRead::read_from(data, offset)?,
            item_tier: BinaryRead::read_from(data, offset)?,
            is_important_item: BinaryRead::read_from(data, offset)?,
            apply_drop_stat_type: BinaryRead::read_from(data, offset)?,
            drop_default_data: BinaryRead::read_from(data, offset)?,
            prefab_data_list: BinaryRead::read_from(data, offset)?,
            enchant_data_list: BinaryRead::read_from(data, offset)?,
            gimmick_visual_prefab_data_list: BinaryRead::read_from(data, offset)?,
            price_list: BinaryRead::read_from(data, offset)?,
            docking_child_data: BinaryRead::read_from(data, offset)?,
            inventory_change_data: BinaryRead::read_from(data, offset)?,
            fixed_page_data_list: BinaryRead::read_from(data, offset)?,
            dynamic_page_data_list: BinaryRead::read_from(data, offset)?,
            inspect_data_list: BinaryRead::read_from(data, offset)?,
            inspect_action: BinaryRead::read_from(data, offset)?,
            default_sub_item: BinaryRead::read_from(data, offset)?,
            cooltime: BinaryRead::read_from(data, offset)?,
            item_charge_type: BinaryRead::read_from(data, offset)?,
            sharpness_data: BinaryRead::read_from(data, offset)?,
            max_charged_useable_count: BinaryRead::read_from(data, offset)?,
            hackable_character_group_info_list: BinaryRead::read_from(data, offset)?,
            item_group_info_list: BinaryRead::read_from(data, offset)?,
            discard_offset_y: BinaryRead::read_from(data, offset)?,
            hide_from_inventory_on_pop_item: BinaryRead::read_from(data, offset)?,
            is_shield_item: BinaryRead::read_from(data, offset)?,
            is_tower_shield_item: BinaryRead::read_from(data, offset)?,
            is_wild: BinaryRead::read_from(data, offset)?,
            packed_item_info: BinaryRead::read_from(data, offset)?,
            unpacked_item_info: BinaryRead::read_from(data, offset)?,
            convert_item_info_by_drop_npc: BinaryRead::read_from(data, offset)?,
            look_detail_game_advice_info_wrapper: BinaryRead::read_from(data, offset)?,
            look_detail_mission_info: BinaryRead::read_from(data, offset)?,
            enable_alert_system_to_ui: BinaryRead::read_from(data, offset)?,
            usable_alert: BinaryRead::read_from(data, offset)?,
            is_save_game_data_at_use_item: BinaryRead::read_from(data, offset)?,
            is_logout_at_use_item: BinaryRead::read_from(data, offset)?,
            shared_cool_time_group_name_hash: BinaryRead::read_from(data, offset)?,
            item_bundle_data_list: BinaryRead::read_from(data, offset)?,
            money_type_define: BinaryRead::read_from(data, offset)?,
            emoji_texture_id: BinaryRead::read_from(data, offset)?,
            enable_equip_in_clone_actor: BinaryRead::read_from(data, offset)?,
            is_blocked_store_sell: BinaryRead::read_from(data, offset)?,
            is_preorder_item: BinaryRead::read_from(data, offset)?,
            respawn_time_seconds: BinaryRead::read_from(data, offset)?,
            max_endurance: BinaryRead::read_from(data, offset)?,
            repair_data_list: BinaryRead::read_from(data, offset)?,
        })
    }
}

impl BinaryWrite for ItemInfo<'_> {
    fn write_to(&self, w: &mut dyn Write) -> io::Result<()> {
        self.key.write_to(w)?;
        self.string_key.write_to(w)?;
        self.is_blocked.write_to(w)?;
        self.max_stack_count.write_to(w)?;
        self.item_name.write_to(w)?;
        self.broken_item_prefix_string.write_to(w)?;
        self.inventory_info.write_to(w)?;
        self.equip_type_info.write_to(w)?;
        self.occupied_equip_slot_data_list.write_to(w)?;
        self.item_tag_list.write_to(w)?;
        self.equipable_hash.write_to(w)?;
        self.consumable_type_list.write_to(w)?;
        self.item_use_info_list.write_to(w)?;
        self.item_icon_list.write_to(w)?;
        self.map_icon_path.write_to(w)?;
        self.money_icon_path.write_to(w)?;
        self.use_map_icon_alert.write_to(w)?;
        self.item_type.write_to(w)?;
        self.material_key.write_to(w)?;
        self.material_match_info.write_to(w)?;
        self.item_desc.write_to(w)?;
        self.item_desc2.write_to(w)?;
        self.equipable_level.write_to(w)?;
        self.category_info.write_to(w)?;
        self.knowledge_info.write_to(w)?;
        self.knowledge_obtain_type.write_to(w)?;
        self.destroy_effec_info.write_to(w)?;
        self.equip_passive_skill_list.write_to(w)?;
        self.use_immediately.write_to(w)?;
        self.apply_max_stack_cap.write_to(w)?;
        self.extract_multi_change_info.write_to(w)?;
        self.item_memo.write_to(w)?;
        self.filter_type.write_to(w)?;
        self.gimmick_info.write_to(w)?;
        self.gimmick_tag_list.write_to(w)?;
        self.max_drop_result_sub_item_count.write_to(w)?;
        self.use_drop_set_target.write_to(w)?;
        self.is_all_gimmick_sealable.write_to(w)?;
        self.sealable_item_info_list.write_to(w)?;
        self.sealable_character_info_list.write_to(w)?;
        self.sealable_gimmick_info_list.write_to(w)?;
        self.sealable_gimmick_tag_list.write_to(w)?;
        self.sealable_tribe_info_list.write_to(w)?;
        self.sealable_money_info_list.write_to(w)?;
        self.delete_by_gimmick_unlock.write_to(w)?;
        self.gimmick_unlock_message_local_string_info.write_to(w)?;
        self.can_disassemble.write_to(w)?;
        self.transmutation_material_gimmick_list.write_to(w)?;
        self.transmutation_material_item_list.write_to(w)?;
        self.transmutation_material_item_group_list.write_to(w)?;
        self.is_register_trade_market.write_to(w)?;
        self.multi_change_info_list.write_to(w)?;
        self.is_editor_usable.write_to(w)?;
        self.discardable.write_to(w)?;
        self.is_dyeable.write_to(w)?;
        self.is_editable_grime.write_to(w)?;
        self.is_destroy_when_broken.write_to(w)?;
        self.quick_slot_index.write_to(w)?;
        self.reserve_slot_target_data_list.write_to(w)?;
        self.item_tier.write_to(w)?;
        self.is_important_item.write_to(w)?;
        self.apply_drop_stat_type.write_to(w)?;
        self.drop_default_data.write_to(w)?;
        self.prefab_data_list.write_to(w)?;
        self.enchant_data_list.write_to(w)?;
        self.gimmick_visual_prefab_data_list.write_to(w)?;
        self.price_list.write_to(w)?;
        self.docking_child_data.write_to(w)?;
        self.inventory_change_data.write_to(w)?;
        self.fixed_page_data_list.write_to(w)?;
        self.dynamic_page_data_list.write_to(w)?;
        self.inspect_data_list.write_to(w)?;
        self.inspect_action.write_to(w)?;
        self.default_sub_item.write_to(w)?;
        self.cooltime.write_to(w)?;
        self.item_charge_type.write_to(w)?;
        self.sharpness_data.write_to(w)?;
        self.max_charged_useable_count.write_to(w)?;
        self.hackable_character_group_info_list.write_to(w)?;
        self.item_group_info_list.write_to(w)?;
        self.discard_offset_y.write_to(w)?;
        self.hide_from_inventory_on_pop_item.write_to(w)?;
        self.is_shield_item.write_to(w)?;
        self.is_tower_shield_item.write_to(w)?;
        self.is_wild.write_to(w)?;
        self.packed_item_info.write_to(w)?;
        self.unpacked_item_info.write_to(w)?;
        self.convert_item_info_by_drop_npc.write_to(w)?;
        self.look_detail_game_advice_info_wrapper.write_to(w)?;
        self.look_detail_mission_info.write_to(w)?;
        self.enable_alert_system_to_ui.write_to(w)?;
        self.usable_alert.write_to(w)?;
        self.is_save_game_data_at_use_item.write_to(w)?;
        self.is_logout_at_use_item.write_to(w)?;
        self.shared_cool_time_group_name_hash.write_to(w)?;
        self.item_bundle_data_list.write_to(w)?;
        self.money_type_define.write_to(w)?;
        self.emoji_texture_id.write_to(w)?;
        self.enable_equip_in_clone_actor.write_to(w)?;
        self.is_blocked_store_sell.write_to(w)?;
        self.is_preorder_item.write_to(w)?;
        self.respawn_time_seconds.write_to(w)?;
        self.max_endurance.write_to(w)?;
        self.repair_data_list.write_to(w)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const BINARY_PATH: &str =
        "/mnt/e/OpensourceGame/CrimsonDesert/Crimson Browser/iteminfo_decompressed.pabgb";

    fn load_binary() -> Vec<u8> {
        std::fs::read(BINARY_PATH).expect("binary file not found")
    }

    #[test]
    fn test_parse_first_item() {
        let data = load_binary();
        let mut offset = 0;
        let item = ItemInfo::read_from(&data, &mut offset).unwrap();
        assert_eq!(item.key, ItemKey(2200));
        assert_eq!(item.string_key.data, "Pyeonjeon_Arrow");
        assert_eq!(offset, 0x243);
    }

    #[test]
    fn test_parse_second_item() {
        let data = load_binary();
        let mut offset = 0x243;
        let item = ItemInfo::read_from(&data, &mut offset).unwrap();
        assert_ne!(item.key, ItemKey(0));
        println!("Second item: key={}, name={}", item.key.0, item.string_key.data);
    }

    #[test]
    fn test_first_item_roundtrip() {
        let data = load_binary();
        let mut offset = 0;
        let item = ItemInfo::read_from(&data, &mut offset).unwrap();
        let end = offset;

        let mut out = Vec::new();
        item.write_to(&mut out).unwrap();
        assert_eq!(out.len(), end, "written size mismatch");
        assert_eq!(&out[..], &data[..end], "roundtrip bytes mismatch");
    }
}
