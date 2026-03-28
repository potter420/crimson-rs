use crate::binary::{BinaryRead, BinaryWrite};
use std::io::{self, Write};

macro_rules! define_key {
    ($name:ident, $inner:ty) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub struct $name(pub $inner);

        impl<'a> BinaryRead<'a> for $name {
            fn read_from(data: &'a [u8], offset: &mut usize) -> io::Result<Self> {
                <$inner>::read_from(data, offset).map($name)
            }
        }

        impl BinaryWrite for $name {
            fn write_to(&self, w: &mut dyn Write) -> io::Result<()> {
                self.0.write_to(w)
            }
        }
    };
}

// u32 keys
define_key!(ItemKey, u32);
define_key!(BuffKey, u32);
define_key!(CharacterKey, u32);
define_key!(ConditionKey, u32);
define_key!(EffectKey, u32);
define_key!(EquipTypeKey, u32);
define_key!(GameAdviceInfoKey, u32);
define_key!(GimmickInfoKey, u32);
define_key!(ItemUseKey, u32);
define_key!(KnowledgeKey, u32);
define_key!(LocalStringInfoKey, u32);
define_key!(MaterialMatchKey, u32);
define_key!(MissionKey, u32);
define_key!(MultiChangeKey, u32);
define_key!(ReserveSlotKey, u32);
define_key!(SkillKey, u32);
define_key!(StatusKey, u32);
define_key!(StringInfoKey, u32);
define_key!(TribeInfoKey, u32);

// u16 keys
define_key!(CategoryKey, u16);
define_key!(CharacterGroupKey, u16);
define_key!(CraftToolKey, u16);
define_key!(InventoryKey, u16);
define_key!(ItemGroupKey, u16);
define_key!(SkillGroupKey, u16);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_item_key_roundtrip() {
        let bytes = [0x98, 0x08, 0x00, 0x00]; // 2200 LE
        let mut offset = 0;
        let key = ItemKey::read_from(&bytes, &mut offset).unwrap();
        assert_eq!(key, ItemKey(2200));
        assert_eq!(offset, 4);

        let mut out = Vec::new();
        key.write_to(&mut out).unwrap();
        assert_eq!(out, bytes);
    }

    #[test]
    fn test_category_key_roundtrip() {
        let bytes = [0x06, 0x00]; // 6 LE
        let mut offset = 0;
        let key = CategoryKey::read_from(&bytes, &mut offset).unwrap();
        assert_eq!(key, CategoryKey(6));

        let mut out = Vec::new();
        key.write_to(&mut out).unwrap();
        assert_eq!(out, bytes);
    }
}
