use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyDict, PyList};
use pyo3::exceptions::{PyIOError, PyKeyError, PyValueError};

use crate::binary::*;
use crate::item_info::structs::*;
use crate::item_info::ItemInfo;

// ── Dict helpers ───────────────────────────────────────────────────────────

fn get<'py, T>(d: &Bound<'py, PyDict>, key: &str) -> PyResult<T>
where
    for<'a> T: FromPyObject<'a, 'py, Error = PyErr>,
{
    d.get_item(key)?
        .ok_or_else(|| PyKeyError::new_err(key.to_string()))?
        .extract()
}

fn get_obj<'py>(d: &Bound<'py, PyDict>, key: &str) -> PyResult<Bound<'py, PyAny>> {
    d.get_item(key)?
        .ok_or_else(|| PyKeyError::new_err(key.to_string()))
}

// ── Binary write helpers ───────────────────────────────────────────────────

fn wr_u8(w: &mut Vec<u8>, v: u8) { w.push(v); }
fn wr_u16(w: &mut Vec<u8>, v: u16) { w.extend_from_slice(&v.to_le_bytes()); }
fn wr_u32(w: &mut Vec<u8>, v: u32) { w.extend_from_slice(&v.to_le_bytes()); }
fn wr_u64(w: &mut Vec<u8>, v: u64) { w.extend_from_slice(&v.to_le_bytes()); }
fn wr_i8(w: &mut Vec<u8>, v: i8) { w.extend_from_slice(&v.to_le_bytes()); }
fn wr_i64(w: &mut Vec<u8>, v: i64) { w.extend_from_slice(&v.to_le_bytes()); }
fn wr_f32(w: &mut Vec<u8>, v: f32) { w.extend_from_slice(&v.to_le_bytes()); }

fn wr_str(w: &mut Vec<u8>, s: &str) {
    wr_u32(w, s.len() as u32);
    w.extend_from_slice(s.as_bytes());
}

fn wr_localizable(w: &mut Vec<u8>, obj: &Bound<'_, PyAny>) -> PyResult<()> {
    let d = obj.cast::<PyDict>()?;
    wr_u8(w, get(d, "category")?);
    wr_u64(w, get(d, "index")?);
    wr_str(w, &get::<String>(d, "default")?);
    Ok(())
}

fn wr_array(
    w: &mut Vec<u8>,
    obj: &Bound<'_, PyAny>,
    f: fn(&mut Vec<u8>, &Bound<'_, PyAny>) -> PyResult<()>,
) -> PyResult<()> {
    let list = obj.cast::<PyList>()?;
    wr_u32(w, list.len() as u32);
    for item in list.iter() {
        f(w, &item)?;
    }
    Ok(())
}

fn wr_optional(
    w: &mut Vec<u8>,
    obj: &Bound<'_, PyAny>,
    f: fn(&mut Vec<u8>, &Bound<'_, PyAny>) -> PyResult<()>,
) -> PyResult<()> {
    if obj.is_none() {
        w.push(0);
    } else {
        w.push(1);
        f(w, obj)?;
    }
    Ok(())
}

fn wr_u8_elem(w: &mut Vec<u8>, obj: &Bound<'_, PyAny>) -> PyResult<()> {
    wr_u8(w, obj.extract()?); Ok(())
}
fn wr_u16_elem(w: &mut Vec<u8>, obj: &Bound<'_, PyAny>) -> PyResult<()> {
    wr_u16(w, obj.extract()?); Ok(())
}
fn wr_u32_elem(w: &mut Vec<u8>, obj: &Bound<'_, PyAny>) -> PyResult<()> {
    wr_u32(w, obj.extract()?); Ok(())
}
fn wr_str_elem(w: &mut Vec<u8>, obj: &Bound<'_, PyAny>) -> PyResult<()> {
    wr_str(w, &obj.extract::<String>()?); Ok(())
}

// ── To-Python helpers ──────────────────────────────────────────────────────

fn u32_keys<T: Copy>(items: &[T], f: fn(&T) -> u32) -> Vec<u32> {
    items.iter().map(f).collect()
}

fn u16_keys<T: Copy>(items: &[T], f: fn(&T) -> u16) -> Vec<u16> {
    items.iter().map(f).collect()
}

fn arr_to_py<'py, T>(
    py: Python<'py>,
    items: &[T],
    f: fn(Python<'py>, &T) -> PyResult<Bound<'py, PyDict>>,
) -> PyResult<Vec<Bound<'py, PyDict>>> {
    items.iter().map(|v| f(py, v)).collect()
}

// ── LocalizableString ──────────────────────────────────────────────────────

fn to_py_localizable<'py>(py: Python<'py>, v: &LocalizableString) -> PyResult<Bound<'py, PyDict>> {
    let d = PyDict::new(py);
    d.set_item("category", v.category)?;
    d.set_item("index", v.index)?;
    d.set_item("default", v.default.data)?;
    Ok(d)
}

// ── OccupiedEquipSlotData ──────────────────────────────────────────────────

fn to_py_occupied<'py>(py: Python<'py>, v: &OccupiedEquipSlotData) -> PyResult<Bound<'py, PyDict>> {
    let d = PyDict::new(py);
    d.set_item("equip_slot_name_key", v.equip_slot_name_key)?;
    // Vec<u8> would become Python bytes; convert to list of ints instead
    let idx: Vec<u32> = v.equip_slot_name_index_list.items.iter().map(|&x| x as u32).collect();
    d.set_item("equip_slot_name_index_list", idx)?;
    Ok(d)
}

fn wr_occupied(w: &mut Vec<u8>, obj: &Bound<'_, PyAny>) -> PyResult<()> {
    let d = obj.cast::<PyDict>()?;
    wr_u32(w, get(d, "equip_slot_name_key")?);
    wr_array(w, &get_obj(d, "equip_slot_name_index_list")?, wr_u8_elem)?;
    Ok(())
}

// ── ItemIconData ───────────────────────────────────────────────────────────

fn to_py_icon<'py>(py: Python<'py>, v: &ItemIconData) -> PyResult<Bound<'py, PyDict>> {
    let d = PyDict::new(py);
    d.set_item("icon_path", v.icon_path.0)?;
    d.set_item("check_exist_sealed_data", v.check_exist_sealed_data)?;
    d.set_item("gimmick_state_list", &v.gimmick_state_list.items)?;
    Ok(d)
}

fn wr_icon(w: &mut Vec<u8>, obj: &Bound<'_, PyAny>) -> PyResult<()> {
    let d = obj.cast::<PyDict>()?;
    wr_u32(w, get(d, "icon_path")?);
    wr_u8(w, get(d, "check_exist_sealed_data")?);
    wr_array(w, &get_obj(d, "gimmick_state_list")?, wr_u32_elem)?;
    Ok(())
}

// ── PassiveSkillLevel ──────────────────────────────────────────────────────

fn to_py_passive<'py>(py: Python<'py>, v: &PassiveSkillLevel) -> PyResult<Bound<'py, PyDict>> {
    let d = PyDict::new(py);
    d.set_item("skill", v.skill.0)?;
    d.set_item("level", v.level)?;
    Ok(d)
}

fn wr_passive(w: &mut Vec<u8>, obj: &Bound<'_, PyAny>) -> PyResult<()> {
    let d = obj.cast::<PyDict>()?;
    wr_u32(w, get(d, "skill")?);
    wr_u32(w, get(d, "level")?);
    Ok(())
}

// ── ReserveSlotTargetData ──────────────────────────────────────────────────

fn to_py_reserve<'py>(py: Python<'py>, v: &ReserveSlotTargetData) -> PyResult<Bound<'py, PyDict>> {
    let d = PyDict::new(py);
    d.set_item("reserve_slot_info", v.reserve_slot_info.0)?;
    d.set_item("condition_info", v.condition_info.0)?;
    Ok(d)
}

fn wr_reserve(w: &mut Vec<u8>, obj: &Bound<'_, PyAny>) -> PyResult<()> {
    let d = obj.cast::<PyDict>()?;
    wr_u32(w, get(d, "reserve_slot_info")?);
    wr_u32(w, get(d, "condition_info")?);
    Ok(())
}

// ── SocketMaterialItem ─────────────────────────────────────────────────────

fn to_py_socket_mat<'py>(py: Python<'py>, v: &SocketMaterialItem) -> PyResult<Bound<'py, PyDict>> {
    let d = PyDict::new(py);
    d.set_item("item", v.item.0)?;
    d.set_item("value", v.value)?;
    Ok(d)
}

fn wr_socket_mat(w: &mut Vec<u8>, obj: &Bound<'_, PyAny>) -> PyResult<()> {
    let d = obj.cast::<PyDict>()?;
    wr_u32(w, get(d, "item")?);
    wr_u64(w, get(d, "value")?);
    Ok(())
}

// ── EnchantStatChange ──────────────────────────────────────────────────────

fn to_py_stat_change<'py>(py: Python<'py>, v: &EnchantStatChange) -> PyResult<Bound<'py, PyDict>> {
    let d = PyDict::new(py);
    d.set_item("stat", v.stat.0)?;
    d.set_item("change_mb", v.change_mb)?;
    Ok(d)
}

fn wr_stat_change(w: &mut Vec<u8>, obj: &Bound<'_, PyAny>) -> PyResult<()> {
    let d = obj.cast::<PyDict>()?;
    wr_u32(w, get(d, "stat")?);
    wr_i64(w, get(d, "change_mb")?);
    Ok(())
}

// ── EnchantLevelChange ─────────────────────────────────────────────────────

fn to_py_level_change<'py>(py: Python<'py>, v: &EnchantLevelChange) -> PyResult<Bound<'py, PyDict>> {
    let d = PyDict::new(py);
    d.set_item("stat", v.stat.0)?;
    d.set_item("change_mb", v.change_mb)?;
    Ok(d)
}

fn wr_level_change(w: &mut Vec<u8>, obj: &Bound<'_, PyAny>) -> PyResult<()> {
    let d = obj.cast::<PyDict>()?;
    wr_u32(w, get(d, "stat")?);
    wr_i8(w, get(d, "change_mb")?);
    Ok(())
}

// ── EnchantStatData ────────────────────────────────────────────────────────

fn to_py_stat_data<'py>(py: Python<'py>, v: &EnchantStatData) -> PyResult<Bound<'py, PyDict>> {
    let d = PyDict::new(py);
    d.set_item("max_stat_list", arr_to_py(py, &v.max_stat_list.items, to_py_stat_change)?)?;
    d.set_item("regen_stat_list", arr_to_py(py, &v.regen_stat_list.items, to_py_stat_change)?)?;
    d.set_item("stat_list_static", arr_to_py(py, &v.stat_list_static.items, to_py_stat_change)?)?;
    d.set_item("stat_list_static_level", arr_to_py(py, &v.stat_list_static_level.items, to_py_level_change)?)?;
    Ok(d)
}

fn wr_stat_data(w: &mut Vec<u8>, obj: &Bound<'_, PyAny>) -> PyResult<()> {
    let d = obj.cast::<PyDict>()?;
    wr_array(w, &get_obj(d, "max_stat_list")?, wr_stat_change)?;
    wr_array(w, &get_obj(d, "regen_stat_list")?, wr_stat_change)?;
    wr_array(w, &get_obj(d, "stat_list_static")?, wr_stat_change)?;
    wr_array(w, &get_obj(d, "stat_list_static_level")?, wr_level_change)?;
    Ok(())
}

// ── PriceFloor ─────────────────────────────────────────────────────────────

fn to_py_price_floor<'py>(py: Python<'py>, v: &PriceFloor) -> PyResult<Bound<'py, PyDict>> {
    let d = PyDict::new(py);
    d.set_item("price", v.price)?;
    d.set_item("sym_no", v.sym_no)?;
    d.set_item("item_info_wrapper", v.item_info_wrapper.0)?;
    Ok(d)
}

fn wr_price_floor(w: &mut Vec<u8>, obj: &Bound<'_, PyAny>) -> PyResult<()> {
    let d = obj.cast::<PyDict>()?;
    wr_u64(w, get(d, "price")?);
    wr_u32(w, get(d, "sym_no")?);
    wr_u32(w, get(d, "item_info_wrapper")?);
    Ok(())
}

// ── ItemPriceInfo ──────────────────────────────────────────────────────────

fn to_py_price_info<'py>(py: Python<'py>, v: &ItemPriceInfo) -> PyResult<Bound<'py, PyDict>> {
    let d = PyDict::new(py);
    d.set_item("key", v.key.0)?;
    d.set_item("price", to_py_price_floor(py, &v.price)?)?;
    Ok(d)
}

fn wr_price_info(w: &mut Vec<u8>, obj: &Bound<'_, PyAny>) -> PyResult<()> {
    let d = obj.cast::<PyDict>()?;
    wr_u32(w, get(d, "key")?);
    wr_price_floor(w, &get_obj(d, "price")?)?;
    Ok(())
}

// ── EquipmentBuff ──────────────────────────────────────────────────────────

fn to_py_equip_buff<'py>(py: Python<'py>, v: &EquipmentBuff) -> PyResult<Bound<'py, PyDict>> {
    let d = PyDict::new(py);
    d.set_item("buff", v.buff.0)?;
    d.set_item("level", v.level)?;
    Ok(d)
}

fn wr_equip_buff(w: &mut Vec<u8>, obj: &Bound<'_, PyAny>) -> PyResult<()> {
    let d = obj.cast::<PyDict>()?;
    wr_u32(w, get(d, "buff")?);
    wr_u32(w, get(d, "level")?);
    Ok(())
}

// ── EnchantData ────────────────────────────────────────────────────────────

fn to_py_enchant<'py>(py: Python<'py>, v: &EnchantData) -> PyResult<Bound<'py, PyDict>> {
    let d = PyDict::new(py);
    d.set_item("level", v.level)?;
    d.set_item("enchant_stat_data", to_py_stat_data(py, &v.enchant_stat_data)?)?;
    d.set_item("buy_price_list", arr_to_py(py, &v.buy_price_list.items, to_py_price_info)?)?;
    d.set_item("equip_buffs", arr_to_py(py, &v.equip_buffs.items, to_py_equip_buff)?)?;
    Ok(d)
}

fn wr_enchant(w: &mut Vec<u8>, obj: &Bound<'_, PyAny>) -> PyResult<()> {
    let d = obj.cast::<PyDict>()?;
    wr_u16(w, get(d, "level")?);
    wr_stat_data(w, &get_obj(d, "enchant_stat_data")?)?;
    wr_array(w, &get_obj(d, "buy_price_list")?, wr_price_info)?;
    wr_array(w, &get_obj(d, "equip_buffs")?, wr_equip_buff)?;
    Ok(())
}

// ── GimmickVisualPrefabData ────────────────────────────────────────────────

fn to_py_gimmick_visual<'py>(py: Python<'py>, v: &GimmickVisualPrefabData) -> PyResult<Bound<'py, PyDict>> {
    let d = PyDict::new(py);
    d.set_item("tag_name_hash", v.tag_name_hash)?;
    d.set_item("scale", v.scale.to_vec())?;
    d.set_item("prefab_names", u32_keys(&v.prefab_names.items, |k| k.0))?;
    d.set_item("animation_path_list", u32_keys(&v.animation_path_list.items, |k| k.0))?;
    d.set_item("use_gimmick_prefab", v.use_gimmick_prefab)?;
    Ok(d)
}

fn wr_gimmick_visual(w: &mut Vec<u8>, obj: &Bound<'_, PyAny>) -> PyResult<()> {
    let d = obj.cast::<PyDict>()?;
    wr_u32(w, get(d, "tag_name_hash")?);
    let scale: Vec<f32> = get(d, "scale")?;
    for v in &scale { wr_f32(w, *v); }
    wr_array(w, &get_obj(d, "prefab_names")?, wr_u32_elem)?;
    wr_array(w, &get_obj(d, "animation_path_list")?, wr_u32_elem)?;
    wr_u8(w, get(d, "use_gimmick_prefab")?);
    Ok(())
}

// ── GameEventExecuteData ───────────────────────────────────────────────────

fn to_py_game_event<'py>(py: Python<'py>, v: &GameEventExecuteData) -> PyResult<Bound<'py, PyDict>> {
    let d = PyDict::new(py);
    d.set_item("game_event_type", v.game_event_type)?;
    d.set_item("player_condition", v.player_condition.0)?;
    d.set_item("target_condition", v.target_condition.0)?;
    d.set_item("event_condition", v.event_condition.0)?;
    Ok(d)
}

fn wr_game_event(w: &mut Vec<u8>, obj: &Bound<'_, PyAny>) -> PyResult<()> {
    let d = obj.cast::<PyDict>()?;
    wr_u8(w, get(d, "game_event_type")?);
    wr_u32(w, get(d, "player_condition")?);
    wr_u32(w, get(d, "target_condition")?);
    wr_u32(w, get(d, "event_condition")?);
    Ok(())
}

// ── InventoryChangeData ────────────────────────────────────────────────────

fn to_py_inv_change<'py>(py: Python<'py>, v: &InventoryChangeData) -> PyResult<Bound<'py, PyDict>> {
    let d = PyDict::new(py);
    d.set_item("game_event_execute_data", to_py_game_event(py, &v.game_event_execute_data)?)?;
    d.set_item("to_inventory_info", v.to_inventory_info.0)?;
    Ok(d)
}

fn wr_inv_change(w: &mut Vec<u8>, obj: &Bound<'_, PyAny>) -> PyResult<()> {
    let d = obj.cast::<PyDict>()?;
    wr_game_event(w, &get_obj(d, "game_event_execute_data")?)?;
    wr_u16(w, get(d, "to_inventory_info")?);
    Ok(())
}

// ── PageData ───────────────────────────────────────────────────────────────

fn to_py_page<'py>(py: Python<'py>, v: &PageData) -> PyResult<Bound<'py, PyDict>> {
    let d = PyDict::new(py);
    d.set_item("left_page_texture_path", v.left_page_texture_path.data)?;
    d.set_item("right_page_texture_path", v.right_page_texture_path.data)?;
    d.set_item("left_page_related_knowledge_info", v.left_page_related_knowledge_info.0)?;
    d.set_item("right_page_related_knowledge_info", v.right_page_related_knowledge_info.0)?;
    Ok(d)
}

fn wr_page(w: &mut Vec<u8>, obj: &Bound<'_, PyAny>) -> PyResult<()> {
    let d = obj.cast::<PyDict>()?;
    wr_str(w, &get::<String>(d, "left_page_texture_path")?);
    wr_str(w, &get::<String>(d, "right_page_texture_path")?);
    wr_u32(w, get(d, "left_page_related_knowledge_info")?);
    wr_u32(w, get(d, "right_page_related_knowledge_info")?);
    Ok(())
}

// ── InspectData ────────────────────────────────────────────────────────────

fn to_py_inspect<'py>(py: Python<'py>, v: &InspectData) -> PyResult<Bound<'py, PyDict>> {
    let d = PyDict::new(py);
    d.set_item("item_info", v.item_info.0)?;
    d.set_item("gimmick_info", v.gimmick_info.0)?;
    d.set_item("character_info", v.character_info.0)?;
    d.set_item("spawn_reason_hash", v.spawn_reason_hash)?;
    d.set_item("socket_name", v.socket_name.data)?;
    d.set_item("speak_character_info", v.speak_character_info.0)?;
    d.set_item("inspect_target_tag", v.inspect_target_tag)?;
    d.set_item("reward_own_knowledge", v.reward_own_knowledge)?;
    d.set_item("reward_knowledge_info", v.reward_knowledge_info.0)?;
    d.set_item("item_desc", to_py_localizable(py, &v.item_desc)?)?;
    d.set_item("board_key", v.board_key)?;
    d.set_item("inspect_action_type", v.inspect_action_type)?;
    d.set_item("gimmick_state_name_hash", v.gimmick_state_name_hash)?;
    d.set_item("target_page_index", v.target_page_index)?;
    d.set_item("is_left_page", v.is_left_page)?;
    d.set_item("target_page_related_knowledge_info", v.target_page_related_knowledge_info.0)?;
    d.set_item("enable_read_after_reward", v.enable_read_after_reward)?;
    d.set_item("refer_to_left_page_inspect_data", v.refer_to_left_page_inspect_data)?;
    d.set_item("inspect_effect_info_key", v.inspect_effect_info_key.0)?;
    d.set_item("inspect_complete_effect_info_key", v.inspect_complete_effect_info_key.0)?;
    Ok(d)
}

fn wr_inspect(w: &mut Vec<u8>, obj: &Bound<'_, PyAny>) -> PyResult<()> {
    let d = obj.cast::<PyDict>()?;
    wr_u32(w, get(d, "item_info")?);
    wr_u32(w, get(d, "gimmick_info")?);
    wr_u32(w, get(d, "character_info")?);
    wr_u32(w, get(d, "spawn_reason_hash")?);
    wr_str(w, &get::<String>(d, "socket_name")?);
    wr_u32(w, get(d, "speak_character_info")?);
    wr_u32(w, get(d, "inspect_target_tag")?);
    wr_u8(w, get(d, "reward_own_knowledge")?);
    wr_u32(w, get(d, "reward_knowledge_info")?);
    wr_localizable(w, &get_obj(d, "item_desc")?)?;
    wr_u32(w, get(d, "board_key")?);
    wr_u8(w, get(d, "inspect_action_type")?);
    wr_u32(w, get(d, "gimmick_state_name_hash")?);
    wr_u32(w, get(d, "target_page_index")?);
    wr_u8(w, get(d, "is_left_page")?);
    wr_u32(w, get(d, "target_page_related_knowledge_info")?);
    wr_u8(w, get(d, "enable_read_after_reward")?);
    wr_u8(w, get(d, "refer_to_left_page_inspect_data")?);
    wr_u32(w, get(d, "inspect_effect_info_key")?);
    wr_u32(w, get(d, "inspect_complete_effect_info_key")?);
    Ok(())
}

// ── InspectAction ──────────────────────────────────────────────────────────

fn to_py_inspect_action<'py>(py: Python<'py>, v: &InspectAction) -> PyResult<Bound<'py, PyDict>> {
    let d = PyDict::new(py);
    d.set_item("action_name_hash", v.action_name_hash)?;
    d.set_item("catch_tag_name_hash", v.catch_tag_name_hash)?;
    d.set_item("catcher_socket_name", v.catcher_socket_name.data)?;
    d.set_item("catch_target_socket_name", v.catch_target_socket_name.data)?;
    Ok(d)
}

fn wr_inspect_action(w: &mut Vec<u8>, obj: &Bound<'_, PyAny>) -> PyResult<()> {
    let d = obj.cast::<PyDict>()?;
    wr_u32(w, get(d, "action_name_hash")?);
    wr_u32(w, get(d, "catch_tag_name_hash")?);
    wr_str(w, &get::<String>(d, "catcher_socket_name")?);
    wr_str(w, &get::<String>(d, "catch_target_socket_name")?);
    Ok(())
}

// ── ItemInfoSharpnessData ──────────────────────────────────────────────────

fn to_py_sharpness<'py>(py: Python<'py>, v: &ItemInfoSharpnessData) -> PyResult<Bound<'py, PyDict>> {
    let d = PyDict::new(py);
    d.set_item("max_sharpness", v.max_sharpness)?;
    d.set_item("craft_tool_info", v.craft_tool_info.0)?;
    d.set_item("stat_data", to_py_stat_data(py, &v.stat_data)?)?;
    Ok(d)
}

fn wr_sharpness(w: &mut Vec<u8>, obj: &Bound<'_, PyAny>) -> PyResult<()> {
    let d = obj.cast::<PyDict>()?;
    wr_u16(w, get(d, "max_sharpness")?);
    wr_u16(w, get(d, "craft_tool_info")?);
    wr_stat_data(w, &get_obj(d, "stat_data")?)?;
    Ok(())
}

// ── ItemBundleData ─────────────────────────────────────────────────────────

fn to_py_bundle<'py>(py: Python<'py>, v: &ItemBundleData) -> PyResult<Bound<'py, PyDict>> {
    let d = PyDict::new(py);
    d.set_item("count_mb", v.count_mb)?;
    d.set_item("key", v.key.0)?;
    Ok(d)
}

fn wr_bundle(w: &mut Vec<u8>, obj: &Bound<'_, PyAny>) -> PyResult<()> {
    let d = obj.cast::<PyDict>()?;
    wr_u64(w, get(d, "count_mb")?);
    wr_u32(w, get(d, "key")?);
    Ok(())
}

// ── UnitData ───────────────────────────────────────────────────────────────

fn to_py_unit<'py>(py: Python<'py>, v: &UnitData) -> PyResult<Bound<'py, PyDict>> {
    let d = PyDict::new(py);
    d.set_item("ui_component", v.ui_component.data)?;
    d.set_item("minimum", v.minimum)?;
    d.set_item("icon_path", v.icon_path.0)?;
    d.set_item("item_name", to_py_localizable(py, &v.item_name)?)?;
    d.set_item("item_desc", to_py_localizable(py, &v.item_desc)?)?;
    Ok(d)
}

fn wr_unit(w: &mut Vec<u8>, obj: &Bound<'_, PyAny>) -> PyResult<()> {
    let d = obj.cast::<PyDict>()?;
    wr_str(w, &get::<String>(d, "ui_component")?);
    wr_u32(w, get(d, "minimum")?);
    wr_u32(w, get(d, "icon_path")?);
    wr_localizable(w, &get_obj(d, "item_name")?)?;
    wr_localizable(w, &get_obj(d, "item_desc")?)?;
    Ok(())
}

// ── MoneyUnitEntry ─────────────────────────────────────────────────────────

fn to_py_money_entry<'py>(py: Python<'py>, v: &MoneyUnitEntry) -> PyResult<Bound<'py, PyDict>> {
    let d = PyDict::new(py);
    d.set_item("key", v.key)?;
    d.set_item("value", to_py_unit(py, &v.value)?)?;
    Ok(d)
}

fn wr_money_entry(w: &mut Vec<u8>, obj: &Bound<'_, PyAny>) -> PyResult<()> {
    let d = obj.cast::<PyDict>()?;
    wr_u32(w, get(d, "key")?);
    wr_unit(w, &get_obj(d, "value")?)?;
    Ok(())
}

// ── MoneyTypeDefine ────────────────────────────────────────────────────────

fn to_py_money_type<'py>(py: Python<'py>, v: &MoneyTypeDefine) -> PyResult<Bound<'py, PyDict>> {
    let d = PyDict::new(py);
    d.set_item("price_floor_value", v.price_floor_value)?;
    d.set_item("unit_data_list_map", arr_to_py(py, &v.unit_data_list_map.items, to_py_money_entry)?)?;
    Ok(d)
}

fn wr_money_type(w: &mut Vec<u8>, obj: &Bound<'_, PyAny>) -> PyResult<()> {
    let d = obj.cast::<PyDict>()?;
    wr_u64(w, get(d, "price_floor_value")?);
    wr_array(w, &get_obj(d, "unit_data_list_map")?, wr_money_entry)?;
    Ok(())
}

// ── PrefabData ─────────────────────────────────────────────────────────────

fn to_py_prefab<'py>(py: Python<'py>, v: &PrefabData) -> PyResult<Bound<'py, PyDict>> {
    let d = PyDict::new(py);
    d.set_item("prefab_names", u32_keys(&v.prefab_names.items, |k| k.0))?;
    d.set_item("equip_slot_list", &v.equip_slot_list.items)?;
    d.set_item("tribe_gender_list", u32_keys(&v.tribe_gender_list.items, |k| k.0))?;
    d.set_item("is_craft_material", v.is_craft_material)?;
    Ok(d)
}

fn wr_prefab(w: &mut Vec<u8>, obj: &Bound<'_, PyAny>) -> PyResult<()> {
    let d = obj.cast::<PyDict>()?;
    wr_array(w, &get_obj(d, "prefab_names")?, wr_u32_elem)?;
    wr_array(w, &get_obj(d, "equip_slot_list")?, wr_u16_elem)?;
    wr_array(w, &get_obj(d, "tribe_gender_list")?, wr_u32_elem)?;
    wr_u8(w, get(d, "is_craft_material")?);
    Ok(())
}

// ── DockingChildData ───────────────────────────────────────────────────────

fn to_py_docking<'py>(py: Python<'py>, v: &DockingChildData) -> PyResult<Bound<'py, PyDict>> {
    let d = PyDict::new(py);
    d.set_item("gimmick_info_key", v.gimmick_info_key.0)?;
    d.set_item("character_key", v.character_key.0)?;
    d.set_item("item_key", v.item_key.0)?;
    d.set_item("attach_parent_socket_name", v.attach_parent_socket_name.data)?;
    d.set_item("attach_child_socket_name", v.attach_child_socket_name.data)?;
    d.set_item("docking_tag_name_hash", v.docking_tag_name_hash.to_vec())?;
    d.set_item("docking_equip_slot_no", v.docking_equip_slot_no)?;
    d.set_item("spawn_distance_level", v.spawn_distance_level)?;
    d.set_item("is_item_equip_docking_gimmick", v.is_item_equip_docking_gimmick)?;
    d.set_item("send_damage_to_parent", v.send_damage_to_parent)?;
    d.set_item("is_body_part", v.is_body_part)?;
    d.set_item("docking_type", v.docking_type)?;
    d.set_item("is_summoner_team", v.is_summoner_team)?;
    d.set_item("is_player_only", v.is_player_only)?;
    d.set_item("is_npc_only", v.is_npc_only.0)?;
    d.set_item("is_sync_break_parent", v.is_sync_break_parent)?;
    d.set_item("hit_part", v.hit_part)?;
    d.set_item("detected_by_npc", v.detected_by_npc)?;
    d.set_item("is_bag_docking", v.is_bag_docking)?;
    d.set_item("enable_collision", v.enable_collision)?;
    d.set_item("disable_collision_with_other_gimmick", v.disable_collision_with_other_gimmick)?;
    d.set_item("docking_slot_key", v.docking_slot_key.data)?;
    Ok(d)
}

fn wr_docking(w: &mut Vec<u8>, obj: &Bound<'_, PyAny>) -> PyResult<()> {
    let d = obj.cast::<PyDict>()?;
    wr_u32(w, get(d, "gimmick_info_key")?);
    wr_u32(w, get(d, "character_key")?);
    wr_u32(w, get(d, "item_key")?);
    wr_str(w, &get::<String>(d, "attach_parent_socket_name")?);
    wr_str(w, &get::<String>(d, "attach_child_socket_name")?);
    let tags: Vec<u32> = get(d, "docking_tag_name_hash")?;
    for v in &tags { wr_u32(w, *v); }
    wr_u16(w, get(d, "docking_equip_slot_no")?);
    wr_u32(w, get(d, "spawn_distance_level")?);
    wr_u8(w, get(d, "is_item_equip_docking_gimmick")?);
    wr_u8(w, get(d, "send_damage_to_parent")?);
    wr_u8(w, get(d, "is_body_part")?);
    wr_u8(w, get(d, "docking_type")?);
    wr_u8(w, get(d, "is_summoner_team")?);
    wr_u8(w, get(d, "is_player_only")?);
    wr_u32(w, get(d, "is_npc_only")?);
    wr_u8(w, get(d, "is_sync_break_parent")?);
    wr_u8(w, get(d, "hit_part")?);
    wr_u8(w, get(d, "detected_by_npc")?);
    wr_u8(w, get(d, "is_bag_docking")?);
    wr_u8(w, get(d, "enable_collision")?);
    wr_u8(w, get(d, "disable_collision_with_other_gimmick")?);
    wr_str(w, &get::<String>(d, "docking_slot_key")?);
    Ok(())
}

// ── RepairData ─────────────────────────────────────────────────────────────

fn to_py_repair<'py>(py: Python<'py>, v: &RepairData) -> PyResult<Bound<'py, PyDict>> {
    let d = PyDict::new(py);
    d.set_item("resource_item_info", v.resource_item_info.0)?;
    d.set_item("repair_value", v.repair_value)?;
    d.set_item("repair_style", v.repair_style)?;
    d.set_item("resource_item_count", v.resource_item_count)?;
    Ok(d)
}

fn wr_repair(w: &mut Vec<u8>, obj: &Bound<'_, PyAny>) -> PyResult<()> {
    let d = obj.cast::<PyDict>()?;
    wr_u32(w, get(d, "resource_item_info")?);
    wr_u16(w, get(d, "repair_value")?);
    wr_u8(w, get(d, "repair_style")?);
    wr_u64(w, get(d, "resource_item_count")?);
    Ok(())
}

// ── SubItem (variant) ──────────────────────────────────────────────────────

fn to_py_sub_item<'py>(py: Python<'py>, v: &SubItem) -> PyResult<Bound<'py, PyDict>> {
    let d = PyDict::new(py);
    d.set_item("type_id", v.type_id)?;
    match &v.value {
        SubItemValue::Item(k) => d.set_item("value", k.0)?,
        SubItemValue::Character(k) => d.set_item("value", k.0)?,
        SubItemValue::Gimmick(k) => d.set_item("value", k.0)?,
        SubItemValue::None => d.set_item("value", py.None())?,
    };
    Ok(d)
}

fn wr_sub_item(w: &mut Vec<u8>, obj: &Bound<'_, PyAny>) -> PyResult<()> {
    let d = obj.cast::<PyDict>()?;
    let type_id: u8 = get(d, "type_id")?;
    wr_u8(w, type_id);
    match type_id {
        0 | 3 | 9 => wr_u32(w, get(d, "value")?),
        14 => {}
        _ => return Err(PyValueError::new_err(format!("invalid SubItem type_id: {}", type_id))),
    }
    Ok(())
}

// ── DropDefaultData ────────────────────────────────────────────────────────

fn to_py_drop_default<'py>(py: Python<'py>, v: &DropDefaultData) -> PyResult<Bound<'py, PyDict>> {
    let d = PyDict::new(py);
    d.set_item("drop_enchant_level", v.drop_enchant_level)?;
    d.set_item("socket_item_list", u32_keys(&v.socket_item_list.items, |k| k.0))?;
    d.set_item("add_socket_material_item_list", arr_to_py(py, &v.add_socket_material_item_list.items, to_py_socket_mat)?)?;
    d.set_item("default_sub_item", to_py_sub_item(py, &v.default_sub_item)?)?;
    d.set_item("socket_valid_count", v.socket_valid_count)?;
    d.set_item("use_socket", v.use_socket)?;
    Ok(d)
}

fn wr_drop_default(w: &mut Vec<u8>, obj: &Bound<'_, PyAny>) -> PyResult<()> {
    let d = obj.cast::<PyDict>()?;
    wr_u16(w, get(d, "drop_enchant_level")?);
    wr_array(w, &get_obj(d, "socket_item_list")?, wr_u32_elem)?;
    wr_array(w, &get_obj(d, "add_socket_material_item_list")?, wr_socket_mat)?;
    wr_sub_item(w, &get_obj(d, "default_sub_item")?)?;
    wr_u8(w, get(d, "socket_valid_count")?);
    wr_u8(w, get(d, "use_socket")?);
    Ok(())
}

// ── SealableItemInfo (variant) ─────────────────────────────────────────────

fn to_py_sealable<'py>(py: Python<'py>, v: &SealableItemInfo) -> PyResult<Bound<'py, PyDict>> {
    let d = PyDict::new(py);
    d.set_item("type_tag", v.type_tag)?;
    d.set_item("item_key", v.item_key.0)?;
    d.set_item("unknown0", v.unknown0)?;
    match &v.value {
        SealableValue::Item(k) => d.set_item("value", k.0)?,
        SealableValue::Gimmick(k) => d.set_item("value", k.0)?,
        SealableValue::String(s) => d.set_item("value", s.data)?,
        SealableValue::Character(k) => d.set_item("value", k.0)?,
        SealableValue::Tribe(k) => d.set_item("value", k.0)?,
    };
    Ok(d)
}

fn wr_sealable(w: &mut Vec<u8>, obj: &Bound<'_, PyAny>) -> PyResult<()> {
    let d = obj.cast::<PyDict>()?;
    let type_tag: u8 = get(d, "type_tag")?;
    wr_u8(w, type_tag);
    wr_u32(w, get(d, "item_key")?);
    wr_u64(w, get(d, "unknown0")?);
    match type_tag {
        0 | 1 | 3 | 4 => wr_u32(w, get(d, "value")?),
        2 => wr_str(w, &get::<String>(d, "value")?),
        _ => return Err(PyValueError::new_err(format!("invalid sealable type_tag: {}", type_tag))),
    }
    Ok(())
}

// ── ItemInfo ───────────────────────────────────────────────────────────────

fn to_py_item<'py>(py: Python<'py>, v: &ItemInfo) -> PyResult<Bound<'py, PyDict>> {
    let d = PyDict::new(py);

    // identity
    d.set_item("key", v.key.0)?;
    d.set_item("string_key", v.string_key.data)?;
    d.set_item("is_blocked", v.is_blocked)?;
    d.set_item("max_stack_count", v.max_stack_count)?;
    d.set_item("item_name", to_py_localizable(py, &v.item_name)?)?;
    d.set_item("broken_item_prefix_string", v.broken_item_prefix_string.0)?;
    d.set_item("inventory_info", v.inventory_info.0)?;
    d.set_item("equip_type_info", v.equip_type_info.0)?;
    d.set_item("occupied_equip_slot_data_list", arr_to_py(py, &v.occupied_equip_slot_data_list.items, to_py_occupied)?)?;
    d.set_item("item_tag_list", &v.item_tag_list.items)?;
    d.set_item("equipable_hash", v.equipable_hash)?;
    d.set_item("consumable_type_list", &v.consumable_type_list.items)?;
    d.set_item("item_use_info_list", u32_keys(&v.item_use_info_list.items, |k| k.0))?;
    d.set_item("item_icon_list", arr_to_py(py, &v.item_icon_list.items, to_py_icon)?)?;
    d.set_item("map_icon_path", v.map_icon_path.0)?;
    d.set_item("money_icon_path", v.money_icon_path.0)?;
    d.set_item("use_map_icon_alert", v.use_map_icon_alert)?;
    d.set_item("item_type", v.item_type)?;
    d.set_item("material_key", v.material_key)?;
    d.set_item("material_match_info", v.material_match_info.0)?;
    d.set_item("item_desc", to_py_localizable(py, &v.item_desc)?)?;
    d.set_item("item_desc2", to_py_localizable(py, &v.item_desc2)?)?;
    d.set_item("equipable_level", v.equipable_level)?;
    d.set_item("category_info", v.category_info.0)?;
    d.set_item("knowledge_info", v.knowledge_info.0)?;
    d.set_item("knowledge_obtain_type", v.knowledge_obtain_type)?;
    d.set_item("destroy_effec_info", v.destroy_effec_info.0)?;
    d.set_item("equip_passive_skill_list", arr_to_py(py, &v.equip_passive_skill_list.items, to_py_passive)?)?;
    d.set_item("use_immediately", v.use_immediately)?;
    d.set_item("apply_max_stack_cap", v.apply_max_stack_cap)?;
    d.set_item("extract_multi_change_info", v.extract_multi_change_info.0)?;
    d.set_item("item_memo", v.item_memo.data)?;
    d.set_item("filter_type", v.filter_type.data)?;
    d.set_item("gimmick_info", v.gimmick_info.0)?;
    let tags: Vec<&str> = v.gimmick_tag_list.items.iter().map(|s| s.data).collect();
    d.set_item("gimmick_tag_list", tags)?;
    d.set_item("max_drop_result_sub_item_count", v.max_drop_result_sub_item_count)?;
    d.set_item("use_drop_set_target", v.use_drop_set_target)?;
    d.set_item("is_all_gimmick_sealable", v.is_all_gimmick_sealable)?;
    d.set_item("sealable_item_info_list", arr_to_py(py, &v.sealable_item_info_list.items, to_py_sealable)?)?;
    d.set_item("sealable_character_info_list", arr_to_py(py, &v.sealable_character_info_list.items, to_py_sealable)?)?;
    d.set_item("sealable_gimmick_info_list", arr_to_py(py, &v.sealable_gimmick_info_list.items, to_py_sealable)?)?;
    d.set_item("sealable_gimmick_tag_list", arr_to_py(py, &v.sealable_gimmick_tag_list.items, to_py_sealable)?)?;
    d.set_item("sealable_tribe_info_list", arr_to_py(py, &v.sealable_tribe_info_list.items, to_py_sealable)?)?;
    d.set_item("sealable_money_info_list", u32_keys(&v.sealable_money_info_list.items, |k| k.0))?;
    d.set_item("delete_by_gimmick_unlock", v.delete_by_gimmick_unlock)?;
    d.set_item("gimmick_unlock_message_local_string_info", v.gimmick_unlock_message_local_string_info.0)?;
    d.set_item("can_disassemble", v.can_disassemble)?;
    d.set_item("transmutation_material_gimmick_list", u32_keys(&v.transmutation_material_gimmick_list.items, |k| k.0))?;
    d.set_item("transmutation_material_item_list", u32_keys(&v.transmutation_material_item_list.items, |k| k.0))?;
    d.set_item("transmutation_material_item_group_list", u16_keys(&v.transmutation_material_item_group_list.items, |k| k.0))?;
    d.set_item("is_register_trade_market", v.is_register_trade_market)?;
    d.set_item("multi_change_info_list", u32_keys(&v.multi_change_info_list.items, |k| k.0))?;
    d.set_item("is_editor_usable", v.is_editor_usable)?;
    d.set_item("discardable", v.discardable)?;
    d.set_item("is_dyeable", v.is_dyeable)?;
    d.set_item("is_editable_grime", v.is_editable_grime)?;
    d.set_item("is_destroy_when_broken", v.is_destroy_when_broken)?;
    d.set_item("quick_slot_index", v.quick_slot_index)?;
    d.set_item("reserve_slot_target_data_list", arr_to_py(py, &v.reserve_slot_target_data_list.items, to_py_reserve)?)?;
    d.set_item("item_tier", v.item_tier)?;
    d.set_item("is_important_item", v.is_important_item)?;
    d.set_item("apply_drop_stat_type", v.apply_drop_stat_type)?;
    d.set_item("drop_default_data", to_py_drop_default(py, &v.drop_default_data)?)?;
    d.set_item("prefab_data_list", arr_to_py(py, &v.prefab_data_list.items, to_py_prefab)?)?;
    d.set_item("enchant_data_list", arr_to_py(py, &v.enchant_data_list.items, to_py_enchant)?)?;
    d.set_item("gimmick_visual_prefab_data_list", arr_to_py(py, &v.gimmick_visual_prefab_data_list.items, to_py_gimmick_visual)?)?;
    d.set_item("price_list", arr_to_py(py, &v.price_list.items, to_py_price_info)?)?;
    match &v.docking_child_data.value {
        Some(val) => d.set_item("docking_child_data", to_py_docking(py, val)?)?,
        None => d.set_item("docking_child_data", py.None())?,
    };
    match &v.inventory_change_data.value {
        Some(val) => d.set_item("inventory_change_data", to_py_inv_change(py, val)?)?,
        None => d.set_item("inventory_change_data", py.None())?,
    };
    d.set_item("fixed_page_data_list", arr_to_py(py, &v.fixed_page_data_list.items, to_py_page)?)?;
    d.set_item("dynamic_page_data_list", arr_to_py(py, &v.dynamic_page_data_list.items, to_py_page)?)?;
    d.set_item("inspect_data_list", arr_to_py(py, &v.inspect_data_list.items, to_py_inspect)?)?;
    d.set_item("inspect_action", to_py_inspect_action(py, &v.inspect_action)?)?;
    d.set_item("default_sub_item", to_py_sub_item(py, &v.default_sub_item)?)?;
    d.set_item("cooltime", v.cooltime)?;
    d.set_item("item_charge_type", v.item_charge_type)?;
    d.set_item("sharpness_data", to_py_sharpness(py, &v.sharpness_data)?)?;
    d.set_item("max_charged_useable_count", v.max_charged_useable_count)?;
    d.set_item("hackable_character_group_info_list", u16_keys(&v.hackable_character_group_info_list.items, |k| k.0))?;
    d.set_item("item_group_info_list", u16_keys(&v.item_group_info_list.items, |k| k.0))?;
    d.set_item("discard_offset_y", v.discard_offset_y)?;
    d.set_item("hide_from_inventory_on_pop_item", v.hide_from_inventory_on_pop_item)?;
    d.set_item("is_shield_item", v.is_shield_item)?;
    d.set_item("is_tower_shield_item", v.is_tower_shield_item)?;
    d.set_item("is_wild", v.is_wild)?;
    d.set_item("packed_item_info", v.packed_item_info.0)?;
    d.set_item("unpacked_item_info", v.unpacked_item_info.0)?;
    d.set_item("convert_item_info_by_drop_npc", v.convert_item_info_by_drop_npc.0)?;
    d.set_item("look_detail_game_advice_info_wrapper", v.look_detail_game_advice_info_wrapper.0)?;
    d.set_item("look_detail_mission_info", v.look_detail_mission_info.0)?;
    d.set_item("enable_alert_system_to_ui", v.enable_alert_system_to_ui)?;
    d.set_item("usable_alert", v.usable_alert)?;
    d.set_item("is_save_game_data_at_use_item", v.is_save_game_data_at_use_item)?;
    d.set_item("is_logout_at_use_item", v.is_logout_at_use_item)?;
    d.set_item("shared_cool_time_group_name_hash", v.shared_cool_time_group_name_hash)?;
    d.set_item("item_bundle_data_list", arr_to_py(py, &v.item_bundle_data_list.items, to_py_bundle)?)?;
    match &v.money_type_define.value {
        Some(val) => d.set_item("money_type_define", to_py_money_type(py, val)?)?,
        None => d.set_item("money_type_define", py.None())?,
    };
    d.set_item("emoji_texture_id", v.emoji_texture_id.data)?;
    d.set_item("enable_equip_in_clone_actor", v.enable_equip_in_clone_actor)?;
    d.set_item("is_blocked_store_sell", v.is_blocked_store_sell)?;
    d.set_item("is_preorder_item", v.is_preorder_item)?;
    d.set_item("respawn_time_seconds", v.respawn_time_seconds)?;
    d.set_item("max_endurance", v.max_endurance)?;
    d.set_item("repair_data_list", arr_to_py(py, &v.repair_data_list.items, to_py_repair)?)?;

    Ok(d)
}

fn wr_item(w: &mut Vec<u8>, obj: &Bound<'_, PyAny>) -> PyResult<()> {
    let d = obj.cast::<PyDict>()?;

    // identity
    wr_u32(w, get(d, "key")?);
    wr_str(w, &get::<String>(d, "string_key")?);
    wr_u8(w, get(d, "is_blocked")?);
    wr_u64(w, get(d, "max_stack_count")?);
    wr_localizable(w, &get_obj(d, "item_name")?)?;
    wr_u32(w, get(d, "broken_item_prefix_string")?);
    wr_u16(w, get(d, "inventory_info")?);
    wr_u32(w, get(d, "equip_type_info")?);
    wr_array(w, &get_obj(d, "occupied_equip_slot_data_list")?, wr_occupied)?;
    wr_array(w, &get_obj(d, "item_tag_list")?, wr_u32_elem)?;
    wr_u32(w, get(d, "equipable_hash")?);
    wr_array(w, &get_obj(d, "consumable_type_list")?, wr_u32_elem)?;
    wr_array(w, &get_obj(d, "item_use_info_list")?, wr_u32_elem)?;
    wr_array(w, &get_obj(d, "item_icon_list")?, wr_icon)?;
    wr_u32(w, get(d, "map_icon_path")?);
    wr_u32(w, get(d, "money_icon_path")?);
    wr_u8(w, get(d, "use_map_icon_alert")?);
    wr_u8(w, get(d, "item_type")?);
    wr_u32(w, get(d, "material_key")?);
    wr_u32(w, get(d, "material_match_info")?);
    wr_localizable(w, &get_obj(d, "item_desc")?)?;
    wr_localizable(w, &get_obj(d, "item_desc2")?)?;
    wr_u32(w, get(d, "equipable_level")?);
    wr_u16(w, get(d, "category_info")?);
    wr_u32(w, get(d, "knowledge_info")?);
    wr_u8(w, get(d, "knowledge_obtain_type")?);
    wr_u32(w, get(d, "destroy_effec_info")?);
    wr_array(w, &get_obj(d, "equip_passive_skill_list")?, wr_passive)?;
    wr_u8(w, get(d, "use_immediately")?);
    wr_u8(w, get(d, "apply_max_stack_cap")?);
    wr_u32(w, get(d, "extract_multi_change_info")?);
    wr_str(w, &get::<String>(d, "item_memo")?);
    wr_str(w, &get::<String>(d, "filter_type")?);
    wr_u32(w, get(d, "gimmick_info")?);
    wr_array(w, &get_obj(d, "gimmick_tag_list")?, wr_str_elem)?;
    wr_u32(w, get(d, "max_drop_result_sub_item_count")?);
    wr_u8(w, get(d, "use_drop_set_target")?);
    wr_u8(w, get(d, "is_all_gimmick_sealable")?);
    wr_array(w, &get_obj(d, "sealable_item_info_list")?, wr_sealable)?;
    wr_array(w, &get_obj(d, "sealable_character_info_list")?, wr_sealable)?;
    wr_array(w, &get_obj(d, "sealable_gimmick_info_list")?, wr_sealable)?;
    wr_array(w, &get_obj(d, "sealable_gimmick_tag_list")?, wr_sealable)?;
    wr_array(w, &get_obj(d, "sealable_tribe_info_list")?, wr_sealable)?;
    wr_array(w, &get_obj(d, "sealable_money_info_list")?, wr_u32_elem)?;
    wr_u8(w, get(d, "delete_by_gimmick_unlock")?);
    wr_u32(w, get(d, "gimmick_unlock_message_local_string_info")?);
    wr_u8(w, get(d, "can_disassemble")?);
    wr_array(w, &get_obj(d, "transmutation_material_gimmick_list")?, wr_u32_elem)?;
    wr_array(w, &get_obj(d, "transmutation_material_item_list")?, wr_u32_elem)?;
    wr_array(w, &get_obj(d, "transmutation_material_item_group_list")?, wr_u16_elem)?;
    wr_u8(w, get(d, "is_register_trade_market")?);
    wr_array(w, &get_obj(d, "multi_change_info_list")?, wr_u32_elem)?;
    wr_u8(w, get(d, "is_editor_usable")?);
    wr_u8(w, get(d, "discardable")?);
    wr_u8(w, get(d, "is_dyeable")?);
    wr_u8(w, get(d, "is_editable_grime")?);
    wr_u8(w, get(d, "is_destroy_when_broken")?);
    wr_u8(w, get(d, "quick_slot_index")?);
    wr_array(w, &get_obj(d, "reserve_slot_target_data_list")?, wr_reserve)?;
    wr_u8(w, get(d, "item_tier")?);
    wr_u8(w, get(d, "is_important_item")?);
    wr_u8(w, get(d, "apply_drop_stat_type")?);
    wr_drop_default(w, &get_obj(d, "drop_default_data")?)?;
    wr_array(w, &get_obj(d, "prefab_data_list")?, wr_prefab)?;
    wr_array(w, &get_obj(d, "enchant_data_list")?, wr_enchant)?;
    wr_array(w, &get_obj(d, "gimmick_visual_prefab_data_list")?, wr_gimmick_visual)?;
    wr_array(w, &get_obj(d, "price_list")?, wr_price_info)?;
    wr_optional(w, &get_obj(d, "docking_child_data")?, wr_docking)?;
    wr_optional(w, &get_obj(d, "inventory_change_data")?, wr_inv_change)?;
    wr_array(w, &get_obj(d, "fixed_page_data_list")?, wr_page)?;
    wr_array(w, &get_obj(d, "dynamic_page_data_list")?, wr_page)?;
    wr_array(w, &get_obj(d, "inspect_data_list")?, wr_inspect)?;
    wr_inspect_action(w, &get_obj(d, "inspect_action")?)?;
    wr_sub_item(w, &get_obj(d, "default_sub_item")?)?;
    wr_i64(w, get(d, "cooltime")?);
    wr_u8(w, get(d, "item_charge_type")?);
    wr_sharpness(w, &get_obj(d, "sharpness_data")?)?;
    wr_u32(w, get(d, "max_charged_useable_count")?);
    wr_array(w, &get_obj(d, "hackable_character_group_info_list")?, wr_u16_elem)?;
    wr_array(w, &get_obj(d, "item_group_info_list")?, wr_u16_elem)?;
    wr_f32(w, get(d, "discard_offset_y")?);
    wr_u8(w, get(d, "hide_from_inventory_on_pop_item")?);
    wr_u8(w, get(d, "is_shield_item")?);
    wr_u8(w, get(d, "is_tower_shield_item")?);
    wr_u8(w, get(d, "is_wild")?);
    wr_u32(w, get(d, "packed_item_info")?);
    wr_u32(w, get(d, "unpacked_item_info")?);
    wr_u32(w, get(d, "convert_item_info_by_drop_npc")?);
    wr_u32(w, get(d, "look_detail_game_advice_info_wrapper")?);
    wr_u32(w, get(d, "look_detail_mission_info")?);
    wr_u8(w, get(d, "enable_alert_system_to_ui")?);
    wr_u8(w, get(d, "usable_alert")?);
    wr_u8(w, get(d, "is_save_game_data_at_use_item")?);
    wr_u8(w, get(d, "is_logout_at_use_item")?);
    wr_u32(w, get(d, "shared_cool_time_group_name_hash")?);
    wr_array(w, &get_obj(d, "item_bundle_data_list")?, wr_bundle)?;
    wr_optional(w, &get_obj(d, "money_type_define")?, wr_money_type)?;
    wr_str(w, &get::<String>(d, "emoji_texture_id")?);
    wr_u8(w, get(d, "enable_equip_in_clone_actor")?);
    wr_u8(w, get(d, "is_blocked_store_sell")?);
    wr_u8(w, get(d, "is_preorder_item")?);
    wr_i64(w, get(d, "respawn_time_seconds")?);
    wr_u16(w, get(d, "max_endurance")?);
    wr_array(w, &get_obj(d, "repair_data_list")?, wr_repair)?;

    Ok(())
}

// ── Module functions ───────────────────────────────────────────────────────

#[pyfunction]
fn parse_file(py: Python<'_>, path: &str) -> PyResult<Py<PyAny>> {
    let data = std::fs::read(path)
        .map_err(|e| PyIOError::new_err(e.to_string()))?;
    parse_bytes_inner(py, &data)
}

#[pyfunction]
fn parse_bytes(py: Python<'_>, data: &[u8]) -> PyResult<Py<PyAny>> {
    parse_bytes_inner(py, data)
}

fn parse_bytes_inner(py: Python<'_>, data: &[u8]) -> PyResult<Py<PyAny>> {
    let mut offset = 0;
    let mut items = Vec::new();
    while offset < data.len() {
        let item = ItemInfo::read_from(data, &mut offset)
            .map_err(|e| PyValueError::new_err(
                format!("parse error at offset 0x{:08X}: {}", offset, e),
            ))?;
        items.push(to_py_item(py, &item)?);
    }
    Ok(PyList::new(py, items)?.into_any().unbind())
}

#[pyfunction]
fn write_file(items: &Bound<'_, PyList>, path: &str) -> PyResult<()> {
    let data = serialize_impl(items)?;
    std::fs::write(path, data)
        .map_err(|e| PyIOError::new_err(e.to_string()))
}

#[pyfunction]
fn serialize_items(py: Python<'_>, items: &Bound<'_, PyList>) -> PyResult<Py<PyAny>> {
    let data = serialize_impl(items)?;
    Ok(PyBytes::new(py, &data).into_any().unbind())
}

fn serialize_impl(items: &Bound<'_, PyList>) -> PyResult<Vec<u8>> {
    let mut buf = Vec::new();
    for item in items.iter() {
        wr_item(&mut buf, &item)?;
    }
    Ok(buf)
}

// ── Registration ───────────────────────────────────────────────────────────

pub fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(parse_file, m)?)?;
    m.add_function(wrap_pyfunction!(parse_bytes, m)?)?;
    m.add_function(wrap_pyfunction!(write_file, m)?)?;
    m.add_function(wrap_pyfunction!(serialize_items, m)?)?;
    Ok(())
}
