// Enter your pattern code here and click on the Play button
// below to execute your code. The results will then be visible
// in the Pattern Data view.
// 
// More information can be found in the documentation.
// 
// 
// Simple example:
// 
// import std.io;
// 
// struct Pattern {
//     u32 int;
//     float floating_point;
// };
// 
// Pattern my_pattern @ 0x00;
// std::print("0x{:08X}", my_pattern.int);

import std.io;
import std.mem;


struct EnhancementStat {
    u32 statId;
    if (statId == 1000002) {
        u32 value0;
        u32 value1;
    }
    if (statId == 1000007)
        u8 value0;
    if (statId == 1) {
        u32 value0;
        if (value0 == 11) {
            u32 value1;
            u32 sub_flags[3];
        } else {
           u32 sub_flags[2];
        }
    }
    if (statId == 1000009) {
        u32 value0;
    }
};

struct EnhancementStatBlock {
    u32 count;
    EnhancementStat stats[count];
    
};

struct Enhancement {
    u16 index;
    // Begin of stat blocks
    EnhancementStatBlock blocks[6];
};

struct string {
    u32 length;
    char value[length];
};

struct LocalizableString {
    u8 category;
    u64 index;
    string default;
};
struct array<T> {
    u32 length;
    T data[length];
};

struct optional<T> {
    u8 has_value;
    if (has_value) T value;
};
struct OccupiedEquipSlotData {
    u32 _equipSlotNameKey;
    array<u8> _equipSlotNameIndexList;
};

struct ItemIconData {
   u32 _iconPath; // StringInfoKey
   u8 _checkExistSealedData;
   array<u32> _gimmickStateList;
};

struct PassiveSkillLevel {
    u32 skill;
    u32 level;
};

struct SealableItemInfo {
   u8 type;
   u32 itemKey;
   u64 unknown0;
   match (type) {
    (0): u32 ItemKey; // ItemKey
    (1): u32 GimmickInfoKey;
    (2): string value;
    (3): u32 CharacterKey;
    (4): u32 TribeInfoKey;
   }
};

struct ReserveSlotTargetData {
  u32 _reserveSlotInfo;
  u32 _conditionInfo;
};

struct SocketMaterialItem {
    u32 item;
    u64 value;
};

struct SubItem {
    u8 type_id;
    match (type_id) {
     (0): u32 ItemKey;
     (3): u32 CharacterKey;
     (9): u32 GimmickInfoKey;
     // 14 -> NoSubItem
    }
};
    
struct DropDefaultData {
    u16 _dropEnchantLevel;
    array<u32>_socketItemList;
    array<SocketMaterialItem> _addSocketMaterialItemList;
    SubItem _defaultSubItem;
    u8 _socketValidCount;
    u8 _useSocket;
};

struct PrefabData{
    array<u32> _prefabNames;
    array<u16> _equipSlotList;
    array<u32> _tribeGenderList;
    u8 _isCraftMaterial;
};

struct EnchantStatChange {
    u32 stat;
    s64 change_mb;
};

struct EnchantLevelChange {
    u32 stat;
    s8 change_mb;
};


struct EnchantStatData {
    array<EnchantStatChange> _maxStatList_DataDefinedRegenerate;
    array<EnchantStatChange> _regenStatList_DataDefinedRegenerate;
    array<EnchantStatChange> _statList_DataDefinedStatic;
    array<EnchantLevelChange> _statList_DataDefinedStaticLevel;
};

struct PriceFloor{
    u64 _price;
    u32 _symNo;
    u32 _itemInfoWrapper;
    
};

struct ItemPriceInfo {
    u32 key;
    PriceFloor price;
};


struct EquipmentBuff{
    u32 buff;
    u32 level;
};
    
struct EnchantData {
    u16 _level;
    EnchantStatData _enchantStatData;
    array<ItemPriceInfo> _buyPriceList; 
    array<EquipmentBuff> _equipBuffs;
};


struct GimmickVisualPrefabData {
    u32 _tagNameHash;
    float _scale[3];
    array<u32> _prefabNames;
    array<u32> _animationPathList;
    u8 _useGimmickPrefab;
};

struct GameEventExecuteData {
    u8 _gameEventType;
    u32 _playerCondition;
    u32 _targetCondition;
    u32 _eventCondition;
};

struct InventoryChangeData {
    GameEventExecuteData _gameEventExecuteData;
    u16 _toInventoryInfo;
};

struct PageData {
    string _leftPageTexturePath;
    string _rightPageTexturePath;
    u32 _leftPageRelatedKnowledgeInfo;
    u32 _rightPageRelatedKnowledgeInfo;
};

struct InspectData {
    u32 _itemInfo;
    u32 _gimmickInfo;
    u32 _characterInfo;
    u32 _spawnReaseonHash;
    string _socketName;
    u32 _speakCharacterInfo;
    u32 _inspectTargetTag;
    u8 _rewardOwnKnowledge;
    u32 _rewardKnowledgeInfo;
    LocalizableString _itemDesc;
    u32 _boardKey;
    u8 _inspectActionType;
    u32 _gimmickStateNameHash;
    u32 _targetPageIndex;
    u8 _isLeftPage;
    u32 _targetPageRelatedKnowledgeInfo;
    u8 _enableReadAfterReward;
    u8 _referToLeftPageInspectData;
    u32 _inspectEffectInfoKey;
    u32 _inspectCompleteEffectInfoKey;
};

struct InspectAction {
    u32 _actionNameHash;
    u32 _catchTagNameHash;
    string _catcherSocketName;
    string _catchTargetSocketName;
};

struct ItemInfo_SharpnessData {
    u16 _maxSharpness;
    u16 _craftToolInfo;
    EnchantStatData _statData;
};

struct ItemBundleData {
    u64 count_mb;
    u32 key;
};

struct UnitData {
    string _uiComponent;
    u32 _minimum;
    u32 _iconPath;
    LocalizableString _itemName;
    LocalizableString _itemDesc;
};

struct MoneyUnitEntry {
    u32 key;
    UnitData value;
};

struct MoneyTypeDefine {
    u64 _priceFloorValue;
    array<MoneyUnitEntry> _unitDataListMap;
};

struct DockingChildData {
    u32 _gimmickInfoKey;
    u32 _charcaterKey;
    u32 _itemKey;
    string _attachParentSocketName;
    string _attachChildSocketName;
    u32 _dockingTagNameHash[4];
    u16 _dockingEquipSlotNo;
    u32 _spawnDistanceLevel;
    u8 _isItemEquipDockingGimmick;
    u8 _sendDamageToParent;
    u8 _isBodyPart;
    u8 _dockingType;
    u8 _isSummonerTeam;
    u8 _isPlayerOnly;
    u32 _isNpcOnly;
    u8 _isSyncBreakParent;
    u8 _hitPart;
    u8 _detectedByNPC;
    u8 _isBagDocking;
    u8 _enableCollision;
    u8 _disableCollisionWithOtherGimmick;
    string _dockingSlotKey;
};

struct RepairData {
    u32 _resourceItemInfo;
    u16 _repairValue;
    u8 _repairStyle;
    u64 _resourceItemCount;
};

struct Item {
    u32 _key;
    string _stringKey;
    u8 _isBlocked;
    u64 _maxStackCount;
    LocalizableString _itemName;
    u32 _brokenItemPrefixString;
    u16 _inventoryInfo;
    u32 _equipTypeInfo;
    array<OccupiedEquipSlotData> _occupiedEquipSlotDataList;
    array<u32> _itemTagList;
    u32 _equipableHash;
    array<u32> _consumableTypeList;
    array<u32> _itemUseInfoList;
    array<ItemIconData> _itemIconList;
    u32 _mapIconPath;
    u32 _moneyIconPath;
    u8 _useMapIconAlert;
    u8 _itemType;
    u32 _materialKey;
    u32 _materialMatchInfo;
    LocalizableString _itemDesc;
    LocalizableString _itemDesc2;
    u32 _equipableLevel;
    u16 _categoryInfo;
    u32 _knowledgeInfo;
    u8 _knowledgeObtainType;
    u32 _destroyEffecInfo;
    array<PassiveSkillLevel> _equipPassiveSkillList;
    u8 _useImmediately;
    u8 _applyMaxStackCap;
    u32 _extractMultiChangeInfo;
    string _itemMemo;
    string _filterType;
    u32 _gimmickInfo;
    array<string> _gimmickTagList;
    u32 _maxDropResultSubItemCount;
    u8 _useDropSetTarget;
    u8 _isAllGimmickSealable;
    array<SealableItemInfo> _sealableItemInfoList;
    array<SealableItemInfo> _sealableCharacterInfoList;
    array<SealableItemInfo> _sealableGimmickInfoList;
    array<SealableItemInfo> _sealableGimmickTagList;
    array<SealableItemInfo> _sealableTribeInfoList;
    array<u32> _sealableMoneyInfoList;
    u8 _deleteByGimmickUnlock;
    u32 _gimmickUnlockMessageLocalStringInfo;
    u8 _canDisassemble;
    array<u32> _transmutationMaterialGimmickList;
    array<u32> _transmutationMaterialItemList;
    array<u16> _transmutationMaterialItemGroupList;
    u8 _isRegisterTradeMarket;
    array<u32> _multiChangeInfoList;
    u8 _isEditorUsable;
    u8 _discardable;
    u8 _isDyeable;
    u8 _isEditableGrime;
    u8 _isDestoryWhenBroken;
    u8 _quickSlotIndex;
    array<ReserveSlotTargetData> _reserveSlotTargetDataList;
    u8 _itemTier;
    u8 _isImportantItem;
    u8 _applyDropStatType;
    DropDefaultData _dropDefaultData;
    array<PrefabData> _prefabDataList;
    array<EnchantData> _enchantDataList;
    array<GimmickVisualPrefabData> _gimmickVisualPrefabDataList;
    array<ItemPriceInfo> _priceList;
    optional<DockingChildData> _dockingChildData;
    optional<InventoryChangeData> _inventoryChangeData;
    string _unkTextureRef1;
    array<PageData> _fixedPageDataList;
    array<PageData> _dynamicPageDataList;
    array<InspectData> _inspectDataList;
    InspectAction _inspectAction;
    SubItem _defaultSubItem;
    s64 _cooltime;
    u8 _itemChargeType;
    ItemInfo_SharpnessData _sharpnessData;
    u32 _maxChargedUseableCount;
    array<u16> _hackableCharacterGroupInfoList;
    array<u16> _itemGroupInfoList;
    float _discardOffsetY;
    u8 _hideFromInventoryOnPopItem;
    u8 _isShieldItem;
    u8 _isTowerShieldItem;
    u8 _isWild;
    u32 _packedItemInfo;
    u32 _unpackedItemInfo;
    u32 _convertItemInfoByDropNPC;
    u32 _lookDetailGameAdviceInfoWrapper;
    u32 _lookDetailMissionInfo;
    u8 _enableAlertSystemToUI;
    u8 _usableAlert;
    u8 _isSaveGameDataAtUseItem;
    u8 _isLogoutAtUseItem;
    u32 _sharedCoolTimeGroupNameHash;
    array<ItemBundleData> _itemBundleDataList;
    optional<MoneyTypeDefine> _moneyTypeDefine;
    string _emojiTextureID;
    u8 _enableEquipInCloneActor;
    u8 _isBlockedStoreSell;
    u8 _isPreorderItem;
    s64 _respawnTimeSeconds;
    u16 _maxEndurance;
    array<RepairData> _repairDataList;
};

Item lostCowNotice @0x00046DCE;

#pragma pattern_limit 10485760
Item items[while($ < std::mem::size())] @0x0;
// Item toad @0x00082A83;
// Item peony @0x00084746;
// Item marigold @0x000850E8;
// Item earring @0x001EB796; 


// std::print("Toad item_type {}, {}, {}", toad._itemType, peony._itemType, marigold._itemType);
// Item arrow @0x00000243;
// Item item_2 @0x255B;
// Item item_3 @0x46eab8;
// Item item_4 @0x48DB76;
// Item item_5 @0x48dd74;
// Item item_6 @0x42C75A;
// Item neck @0x001D84C7; // Karanda_Necklace
// Item item_8 @0x003BD794;
// Item item_9 @0x003BBBEC;
// Item immune_ring @0x003BB978; // Dev Red Dragon Ring Immune
// Item helm @0x001A988F;
// Item sword @0x00429474;
// Item hwando @0x00486829;
// Item sydmon @0x0048B004;
// Item helm @0x003B305D;
// Item earring @0x001E2866;
// Item dev_growth @0x003BB6FC;