from __future__ import annotations
from dataclasses import dataclass
from bier.serialization import (
    BinarySerializable,
    u8,
    u16,
    u32,
    u64,
    i8,
    i64,
    f32,
)
from bier.serialization.options import custom
from bier.EndianedBinaryIO import EndianedReaderIOBase, EndianedFileIO, EndianedBytesIO
from typing import Type


class VariantSerializable(BinarySerializable):
    cases: dict[int, type[BinarySerializable]]

    @classmethod
    def read_from(cls, reader: EndianedReaderIOBase, context=None):
        type = reader.read_u8()
        case_cls = cls.cases[type]
        return case_cls.read_from(reader, context)


class optional[SubB](BinarySerializable):
    has_value: boolean
    value: SubB

    @classmethod
    def read_from(cls, reader: EndianedReaderIOBase, context=None):
        has_value = reader.read_u8() != 0
        if has_value:
            value = SubB.read_from(reader, context)
        else:
            value = None
        return cls(has_value, value)


class string(BinarySerializable):
    length: u32
    data: bytes

    @classmethod
    def read_from(cls, reader: EndianedReaderIOBase, context=None):
        length = reader.read_u32()
        data = reader.read(length)
        _null_bytes = reader.read(1)
        return cls(length, data)

type array[TType] = custom[list[TType], u32]


class boolean(BinarySerializable):
    value: bool

    @classmethod
    def read_from(cls, reader: EndianedReaderIOBase, context=None):
        value = reader.read_u8() != 0
        return cls(value)


@dataclass(slots=True)
class LocalizableString(BinarySerializable):
    category: u8
    index: u64
    default: string


@dataclass(frozen=True)
class BuffKey(BinarySerializable):
    value: u32


@dataclass(frozen=True)
class CategoryKey(BinarySerializable):
    value: u16


@dataclass(frozen=True)
class CharacterGroupKey(BinarySerializable):
    value: u16


@dataclass(frozen=True)
class CharacterKey(BinarySerializable):
    value: u32


@dataclass(frozen=True)
class ConditionKey(BinarySerializable):
    value: u32


@dataclass(frozen=True)
class CraftToolKey(BinarySerializable):
    value: u16


@dataclass(frozen=True)
class EffectKey(BinarySerializable):
    value: u32


@dataclass(frozen=True)
class EquipTypeKey(BinarySerializable):
    value: u32


@dataclass(frozen=True)
class GameAdviceInfoKey(BinarySerializable):
    value: u32


@dataclass(frozen=True)
class GimmickInfoKey(BinarySerializable):
    value: u32


@dataclass(frozen=True)
class InventoryKey(BinarySerializable):
    value: u16


@dataclass(frozen=True)
class ItemGroupKey(BinarySerializable):
    value: u16


@dataclass(frozen=True)
class ItemKey(BinarySerializable):
    value: u32


@dataclass(frozen=True)
class ItemUseKey(BinarySerializable):
    value: u32


@dataclass(frozen=True)
class KnowledgeKey(BinarySerializable):
    value: u32


@dataclass(frozen=True)
class LocalStringInfoKey(BinarySerializable):
    value: u32


@dataclass(frozen=True)
class MaterialMatchKey(BinarySerializable):
    value: u32


@dataclass(frozen=True)
class MissionKey(BinarySerializable):
    value: u32


@dataclass(frozen=True)
class MultiChangeKey(BinarySerializable):
    value: u32


@dataclass(frozen=True)
class ReserveSlotKey(BinarySerializable):
    value: u32


@dataclass(frozen=True)
class SkillKey(BinarySerializable):
    value: u32


@dataclass(frozen=True)
class SkillGroupKey(BinarySerializable):
    value: u16


@dataclass(frozen=True)
class StatusKey(BinarySerializable):
    value: u32


@dataclass(frozen=True)
class StringInfoKey(BinarySerializable):
    value: u32


@dataclass(frozen=True)
class TribeInfoKey(BinarySerializable):
    value: u32


@dataclass(slots=True)
class PrefabData(BinarySerializable):
    _prefabNames: array[StringInfoKey]
    _equipSlotList: array[u16]
    _tribeGenderList: array[StringInfoKey]
    _isCraftMaterial: boolean


@dataclass(slots=True)
class GimmickVisualPrefabData(BinarySerializable):
    _tagNameHash: u32
    _scale: tuple[f32, f32, f32]
    _prefabNames: array[StringInfoKey]
    _animationPathList: array[StringInfoKey]
    _useGimmickPrefab: boolean


@dataclass(slots=True)
class EquipmentBuff(BinarySerializable):
    buff: BuffKey
    level: Level


@dataclass(slots=True)
class EnchantData(BinarySerializable):
    _level: u16
    _enchantStatData: EnchantStatData
    _buyPriceList: array[ItemPriceInfo]
    _equipBuffs: array[EquipmentBuff]


@dataclass(slots=True)
class ItemIconData(BinarySerializable):
    _iconPath: StringInfoKey
    _checkExistSealedData: boolean
    _gimmickStateList: array[u32]


@dataclass(slots=True)
class InspectAction(BinarySerializable):
    _actionNameHash: u32
    _catchTagNameHash: u32
    _catcherSocketName: string
    _catchTargetSocketName: string


@dataclass(slots=True)
class PageData(BinarySerializable):
    _leftPageTexturePath: string
    _rightPageTexturePath: string
    _leftPageRelatedKnowledgeInfo: KnowledgeKey
    _rightPageRelatedKnowledgeInfo: KnowledgeKey


@dataclass(slots=True)
class ReserveSlotTargetData(BinarySerializable):
    _reserveSlotInfo: ReserveSlotKey
    _conditionInfo: ConditionKey


@dataclass(slots=True)
class SocketMaterialItem(BinarySerializable):
    item: ItemKey
    value: u64


@dataclass(slots=True)
class DropDefaultData(BinarySerializable):
    _dropEnchantLevel: u16
    _socketItemList: array[ItemKey]
    _addSocketMaterialItemList: array[SocketMaterialItem]
    _defaultSubItem: SubItem
    _socketValidCount: u8
    _useSocket: boolean


@dataclass(slots=True)
class GameEventExecuteData(BinarySerializable):
    _gameEventType: u8
    _playerCondition: ConditionKey
    _targetCondition: ConditionKey
    _eventCondition: ConditionKey


@dataclass(slots=True)
class InventoryChangeData(BinarySerializable):
    _gameEventExecuteData: GameEventExecuteData
    _toInventoryInfo: InventoryKey


@dataclass(slots=True)
class UnitData(BinarySerializable):
    _uiComponent: string
    _minimum: u32
    _iconPath: StringInfoKey
    _itemName: LocalizableString
    _itemDesc: LocalizableString


@dataclass(slots=True)
class MoneyTypeDefine(BinarySerializable):
    _priceFloorValue: u64
    _unitDataListMap: array[tuple[u32, UnitData]]


@dataclass(slots=True)
class OccupiedEquipSlotData(BinarySerializable):
    _equipSlotNameKey: u32
    _equipSlotNameIndexList: array[u8]


@dataclass(slots=True)
class EnchantStatChange(BinarySerializable):
    stat: StatusKey
    change_mb: i64


@dataclass(slots=True)
class EnchantLevelChange(BinarySerializable):
    stat: StatusKey
    change_mb: i8


@dataclass(slots=True)
class EnchantStatData(BinarySerializable):
    _maxStatList_DataDefinedRegenerate: array[EnchantStatChange]
    _regenStatList_DataDefinedRegenerate: array[EnchantStatChange]
    _statList_DataDefinedStatic: array[EnchantStatChange]
    _statList_DataDefinedStaticLevel: array[EnchantLevelChange]


@dataclass(slots=True)
class ItemInfo_SharpnessData(BinarySerializable):
    _maxSharpness: u16
    _craftToolInfo: CraftToolKey
    _statData: EnchantStatData


type SlotNameKey = u32
type ItemType = u8
type MaterialKey = u32
type Level = u32
type KnowledgeObtainType = u8
type ItemTier = u8
type ApplyDropStatType = u8


@dataclass(slots=True)
class PriceFloor(BinarySerializable):
    _price: u64
    _symNo: u32
    _itemInfoWrapper: ItemKey


@dataclass(slots=True)
class ItemPriceInfo(BinarySerializable):
    key: ItemKey
    price: PriceFloor


type ItemChargeType = u8


@dataclass(slots=True)
class ItemBundleData(BinarySerializable):
    count_mb: u64
    key: GimmickInfoKey


@dataclass(slots=True)
class SealableItemInfo(BinarySerializable):
    item: ItemKey
    unknown0: u64
    value: ItemKey | GimmickInfoKey | string | CharacterKey | TribeInfoKey

    @classmethod
    def read_from(cls, reader: EndianedReaderIOBase, context=None):
        type = reader.read_u8()
        item = ItemKey.read_from(reader, context)
        unknown0 = reader.read_u64()
        match type:
            case 0:
                value = ItemKey.read_from(reader, context)
            case 1:
                value = GimmickInfoKey.read_from(reader, context)
            case 2:
                value = reader.read_string(reader.read_u32())
            case 3:
                value = CharacterKey.read_from(reader, context)
            case 4:
                value = TribeInfoKey.read_from(reader, context)
            case _:
                assert False, (type, hex(reader.tell()))

        return cls(item, unknown0, value)


@dataclass(frozen=True)
class NoSubItem(BinarySerializable):
    pass


@dataclass(slots=True)
class SubItem(VariantSerializable):
    cases = {
        0: ItemKey,
        3: CharacterKey,
        9: GimmickInfoKey,
        14: NoSubItem,
    }


@dataclass(slots=True)
class PassiveSkillLevel(BinarySerializable):
    skill: SkillKey
    level: Level


@dataclass(slots=True)
class DockingChildData(BinarySerializable):
    _gimmickInfoKey: GimmickInfoKey
    _charcaterKey: CharacterKey
    _itemKey: ItemKey
    _attachParentSocketName: string
    _attachChildSocketName: string
    _dockingTagNameHash: tuple[u32, u32, u32, u32]
    _dockingEquipSlotNo: u16
    _spawnDistanceLevel: Level
    _isItemEquipDockingGimmick: boolean
    _sendDamageToParent: boolean
    _isBodyPart: boolean
    _dockingType: u8
    _isSummonerTeam: boolean
    _isPlayerOnly: boolean
    _isNpcOnly: ConditionKey
    _isSyncBreakParent: boolean
    _hitPart: boolean
    _detectedByNPC: boolean
    _isBagDocking: boolean
    _enableCollision: boolean
    _disableCollisionWithOtherGimmick: boolean
    _dockingSlotKey: string


@dataclass(slots=True)
class InspectData(BinarySerializable):
    _itemInfo: ItemKey
    _gimmickInfo: GimmickInfoKey
    _characterInfo: CharacterKey
    _spawnReaseonHash: u32
    _socketName: string
    _speakCharacterInfo: CharacterKey
    _inspectTargetTag: u32
    _rewardOwnKnowledge: boolean
    _rewardKnowledgeInfo: KnowledgeKey
    _itemDesc: LocalizableString
    _boardKey: u32
    _inspectActionType: u8
    _gimmickStateNameHash: u32
    _targetPageIndex: u32
    _isLeftPage: boolean
    _targetPageRelatedKnowledgeInfo: KnowledgeKey
    _enableReadAfterReward: boolean
    _referToLeftPageInspectData: boolean
    _inspectEffectInfoKey: EffectKey
    _inspectCompleteEffectInfoKey: EffectKey


@dataclass(slots=True)
class RepairData(BinarySerializable):
    _resourceItemInfo: ItemKey
    _repairValue: u16
    _repairStyle: u8
    _resourceItemCount: u64


@dataclass(slots=True)
class ItemInfo(BinarySerializable):
    _key: ItemKey
    _stringKey: string
    _isBlocked: boolean
    _maxStackCount: u64
    _itemName: LocalizableString
    _brokenItemPrefixString: LocalStringInfoKey
    _inventoryInfo: InventoryKey
    _equipTypeInfo: EquipTypeKey
    _occupiedEquipSlotDataList: array[OccupiedEquipSlotData]
    _itemTagList: array[u32]
    _equipAbleHash: u32
    _consumableTypeList: array[u32]
    _itemUseInfoList: array[ItemUseKey]
    _itemIconList: array[ItemIconData]
    _mapIconPath: StringInfoKey
    _moneyIconPath: StringInfoKey
    _useMapIconAlert: boolean
    _itemType: ItemType
    _materialKey: MaterialKey
    _materialMatchInfo: MaterialMatchKey
    _itemDesc: LocalizableString
    _itemDesc2: LocalizableString
    _equipableLevel: Level
    _categoryInfo: CategoryKey
    _knowledgeInfo: KnowledgeKey
    _knowledgeObtainType: KnowledgeObtainType
    _destroyEffecInfo: EffectKey
    _equipPassiveSkillList: array[PassiveSkillLevel]
    _useImmediately: boolean
    _applyMaxStackCap: boolean
    _extractMultiChangeInfo: MultiChangeKey
    _itemMemo: string
    _filterType: string
    _gimmickInfo: GimmickInfoKey
    _gimmickTagList: array[string]
    _maxDropResultSubItemCount: u32
    _useDropSetTarget: boolean
    _isAllGimmickSealable: boolean
    _sealableItemInfoList: array[SealableItemInfo]
    _sealableCharacterInfoList: array[SealableItemInfo]
    _sealableGimmickInfoList: array[SealableItemInfo]
    _sealableGimmickTagList: array[SealableItemInfo]
    _sealableTribeInfoList: array[SealableItemInfo]
    _sealableMoneyInfoList: array[ItemKey]
    _deleteByGimmickUnlock: boolean
    _gimmickUnlockMessageLocalStringInfo: LocalStringInfoKey
    _canDisassemble: boolean
    _transmutationMaterialGimmickList: array[GimmickInfoKey]
    _transmutationMaterialItemList: array[ItemKey]
    _transmutationMaterialItemGroupList: array[ItemGroupKey]
    _isRegisterTradeMarket: boolean
    _multiChangeInfoList: array[MultiChangeKey]
    _isEditorUsable: boolean
    _discardable: boolean
    _isDyeable: boolean
    _isEditableGrime: boolean
    _isDestoryWhenBroken: boolean
    _quickSlotIndex: u8
    _reserveSlotTargetDataList: array[ReserveSlotTargetData]
    _itemTier: ItemTier
    _isImportantItem: boolean
    _applyDropStatType: ApplyDropStatType
    _dropDefaultData: DropDefaultData
    _prefabDataList: array[PrefabData]
    _enchantDataList: array[EnchantData]
    _gimmickVisualPrefabDataList: array[GimmickVisualPrefabData]
    _priceList: array[ItemPriceInfo]
    _dockingChildData: optional[DockingChildData]
    _inventoryChangeData: optional[InventoryChangeData]
    _fixedPageDataList: array[PageData]
    _dynamicPageDataList: array[PageData]
    _inspectDataList: array[InspectData]
    _inspectAction: InspectAction
    _defaultSubItem: SubItem
    _cooltime: i64
    _itemChargeType: ItemChargeType
    _sharpnessData: ItemInfo_SharpnessData
    _maxChargedUseableCount: u32
    _hackableCharacterGroupInfoList: array[CharacterGroupKey]
    _itemGroupInfoList: array[ItemGroupKey]
    _discardOffsetY: f32
    _hideFromInventoryOnPopItem: boolean
    _isShieldItem: boolean
    _isTowerShieldItem: boolean
    _isWild: boolean
    _packedItemInfo: ItemKey
    _unpackedItemInfo: ItemKey
    _convertItemInfoByDropNPC: ItemKey
    _lookDetailGameAdviceInfoWrapper: GameAdviceInfoKey
    _lookDetailMissionInfo: MissionKey
    _enableAlertSystemToUI: boolean
    _usableAlert: boolean
    _isSaveGameDataAtUseItem: boolean
    _isLogoutAtUseItem: boolean
    _sharedCoolTimeGroupNameHash: u32
    _itemBundleDataList: array[ItemBundleData]
    _moneyTypeDefine: optional[MoneyTypeDefine]
    _emojiTextureID: string
    _enableEquipInCloneActor: boolean
    _isBlockedStoreSell: boolean
    _isPreorderItem: boolean
    _respawnTimeSeconds: i64
    _maxEndurance: u16
    _repairDataList: array[RepairData]

if __name__ == "__main__":
    FILEPATH = "/mnt/e/OpensourceGame/CrimsonDesert/Crimson Browser/iteminfo_decompressed.pabgb"
    with open(FILEPATH, "rb") as f:
        first_item = f.read(579)
        reader = EndianedBytesIO(first_item)
        item_info = ItemInfo.read_from(reader)
        print(item_info)