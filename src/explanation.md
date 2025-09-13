This describes a crafting system for items in a game. 

## Modifiers
A modifier is either Prefix or Suffix (AKA the "affix").  
A depending on the rarity of the item, it can have a maximum number of mods:  
- Normal: No modifiers  
- Magic: 1 prefix and 1 suffix (2 total)  
- Rare: 3 prefixes and 3 suffixes (6 total)  
A modifier can have zero or more Tags associated with it.  
A modifier has several "tiers", each of which have an associated "level" to them. High level modifiers are better.

## Currencies
A currency is used on an item to change it in some way. Optionally, several Omens can be used to change how the currency behaves. Multiple omens can be used at once, for example a Dextral Greater Exalt will add 2 suffix modifiers.  
Here are the available currencies:

Transmute:  
    Usability constraints:  
        Item must be Normal rarity  
    Effects:  
        Rarity Normal -> Magic  
        Adds 1 random modifier  
    Variants:  
        Standard - Adds modifiers of any level  
        Greater - Adds modifiers with a minimum level of 35  
        Perfect - Adds modifiers with a minimum level of 50  
Augmentation:  
    Usability constraints:  
        Item must be Magic rarity  
        Item must have <2 modifiers  
    Effects:  
        Adds 1 random modifier  
    Variants:  
        Standard - Adds modifiers of any level  
        Greater - Adds modifiers with a minimum level of 35  
        Perfect - Adds modifiers with a minimum level of 50  
Regal:  
    Usability constraints:  
        Item must be Magic rarity  
    Effects:  
        Rarity Magic -> Rare  
        Adds 1 random modifier  
    Omens:
        Homogenous - Only modifiers with at least one tag in common with existing modifiers on the item are added.
    Variants:  
        Standard - Adds modifiers of any level  
        Greater - Adds modifiers with a minimum level of 35  
        Perfect - Adds modifiers with a minimum level of 50  
Exalt:  
    Usability constraints:  
        Item must be Rare rarity  
        Item must have <6 modifiers  
    Effects:  
        Rarity Magic -> Rare  
        Adds 1 random modifier  
    Omens:
        Dextral - Only a Suffix is added  
        Sinistral - Only a Prefix is added  
        Homogenous - Only modifiers with at least one tag in common with existing modifiers on the item are added.
        Greater - Adds 2 modifiers  
    Variants:  
        Standard - Adds modifiers of any level  
        Greater - Adds modifiers with a minimum level of 35  
        Perfect - Adds modifiers with a minimum level of 50  
Chaos:  
    Usability constraints:  
        Item must be Rare rarity  
        Item must have at least 1 modifier  
    Effects:  
        Removes 1 random modifier  
        Adds 1 random modifier  
    Omens:
        Dextral - Only a Suffix is removed  
        Sinistral - Only a Prefix is removed  
        Whittling - Removes the modifier with the lowest level. If there are several with the same lowest level, one of them is chosen randomly.  
    Variants:  
        Standard - Adds modifiers of any level  
        Greater - Adds modifiers with a minimum level of 35  
        Perfect - Adds modifiers with a minimum level of 50  
Annul:  
    Usability constraints:  
        Item must have at least 1 modifier  
    Effects:  
        Removes 1 random modifier  
    Omens:
        Dextral - Only a Suffix is removed  
        Sinistral - Only a Prefix is removed  
        Greater - Removes 2 modifiers  
Greater Essence:  
    Usability constraints:  
        Item must be Magic rarity  
    Effects:  
        Rarity Magic -> Rare  
        Adds 1 of several specific modifiers depending on the essence used  
Perfect Essence:  
    Usability constraints:  
        Item must be Rare rarity  
        Item must have at least 1 modifier  
    Effects:  
        Removes 1 random modifier  
        Adds 1 of several specific modifiers depending on the essence used  
    Omens:
        Dextral - Only a Suffix is removed  
        Sinistral - Only a Prefix is removed  
    
The list of essences available are as follows:  
Perfect Essence of Electricity
        DamageasExtraLightning (Prefix), tags: {"Elemental", "Damage", "Lightning"}, available levels: [72]
Perfect Essence of Flames
        DamageasExtraFire (Prefix), tags: {"Damage", "Fire", "Elemental"}, available levels: [72]
Greater Essence of Electricity
        LocalLightningDamage (Prefix), tags: {"Damage", "Attack", "Lightning", "Elemental"}, available levels: [60]
Perfect Essence of Ice
        DamageasExtraCold (Prefix), tags: {"Elemental", "Damage", "Cold"}, available levels: [72]
Greater Essence of Battle
        LocalAccuracyRating (Prefix), tags: {"Attack"}, available levels: [58]
        IncreasedAccuracy (Prefix), tags: {"Attack"}, available levels: [58]
Greater Essence of the Infinite
        Strength (Suffix), tags: {"Attribute"}, available levels: [55]
        Dexterity (Suffix), tags: {"Attribute"}, available levels: [55]
        Intelligence (Suffix), tags: {"Attribute"}, available levels: [55]
Greater Essence of Ice
        LocalColdDamage (Prefix), tags: {"Elemental", "Damage", "Attack", "Cold"}, available levels: [60]
Greater Essence of Seeking
        LocalBaseCriticalStrikeChance (Suffix), tags: {"Attack", "Critical"}, available levels: [44]
Greater Essence of Abrasion
        LocalPhysicalDamage (Prefix), tags: {"Damage", "Physical", "Attack"}, available levels: [60]
Greater Essence of Haste
        LocalIncreasedAttackSpeed (Suffix), tags: {"Attack", "Speed"}, available levels: [60]
Greater Essence of Flames
        LocalFireDamage (Prefix), tags: {"Elemental", "Fire", "Attack", "Damage"}, available levels: [60]
Perfect Essence of Battle
        EssenceAttackSkillLevel (Suffix), tags: {"Attack"}, available levels: [72]
Perfect Essence of Abrasion
        DamageasExtraPhysical (Prefix), tags: {"Damage", "Physical"}, available levels: [72]
Perfect Essence of Haste
        EssenceOnslaughtonKill (Suffix), tags: {"Speed"}, available levels: [72]

## Costs
Here are the approximate costs of each currency. All prices are in units of 1 Standard Exalt.  
Standard Transmute: 0  
Greater Transmute: 0  
Perfect Transmute: 1  
Standard Augmentation: 0  
Greater Augmentation: 0  
Perfect Augmentation: 2  
Standard Regal: 0  
Greater Regal: 1  
Perfect Regal: 14  
Standard Exalt: 1  
Greater Exalt: 3  
Perfect Exalt: 328  
Standard Chaos: 4  
Greater Chaos: 14  
Perfect Chaos: 211  
Annul: 45  

All essences: 4  

Omens:  
Homogenous Regal: 7  
Dextral Exalt: 3  
Sinistral Exalt: 3  
Greater Exalt: 2  
Homogenous Exalt: 89  
Dextral Chaos: 250  
Sinistral Chaos: 373  
Whittling Chaos: 178  
Dextral Annul: 384  
Sinistral Annul: 492  
Dextral Essence: 30  
Sinistral Essence: 30  


## Tips, Tricks & Edge Cases
**Homogenous Omen**
A Homogenous omen can be used to target specific modifiers. It works by taking a combined set of unique tags across all modifiers on the item. The pool of modifiers that can be rolled is then filtered to those which have at least one tag in common with the set. Having more than one mod with the same tag doesn't increase the chance of rolling similar mods.  
For example, for a rare item with mods:  
- Mod1 (tags {Attack, Speed})  
- Mod2 (tags {Attack, Damage})  

And mod pool:  
- Mod3 (tags {Attack, Lightning})
- Mod4 (tags {Attack, Damage, Physical})
- Mod5 (tags {Mana})
- Mod6 (tags {})

The unique set of tags on the item is {Attack, Speed, Damage}. Mods 3 & 4 both have at least 1 tag in common with the item, while Mods 5 & 6 do not. Therefore the mod pool is limited to Mod 3 & 4.  

**Omen Combinations (non-exhaustive)**
- Greater + Dextral Exalt = Add 2 suffixes in one use
- Greater + Sinistral Exalt = Add 2 prefixes in one use
- Greater + Annul = Remove 2 modifiers 
- Whittling + directional omens (Dextral/Sinistral) + Chaos give you surgical control over mod improvement

When using a Greater + Dextral/Sinistral + Exalt, the item must have room for 2 suffixes or 2 prefixes. If not, then the craft cannot be performed.  


## Your task
Here is the list of available modifiers that can rolled on an item:  
IncreasedWeaponElementalDamagePercent (Prefix), tags: {"Damage", "Elemental", "Attack"}, available levels: [4, 16, 33, 46, 60]  
LocalAccuracyRating (Prefix), tags: {"Attack"}, available levels: [1, 11, 18, 26, 36, 48, 58, 67]  
LocalColdDamage (Prefix), tags: {"Cold", "Damage", "Attack", "Elemental"}, available levels: [1, 8, 16, 33, 46, 54, 60, 65, 75]  
LocalFireDamage (Prefix), tags: {"Damage", "Elemental", "Attack", "Fire"}, available levels: [1, 8, 16, 33, 46, 54, 60, 65, 75]  
LocalIncreasedPhysicalDamagePercentAndAccuracyRating (Prefix), tags: {"Damage", "Physical", "Attack"}, available levels: [1, 11, 23, 38, 54, 65, 70]  
LocalLightningDamage (Prefix), tags: {"Elemental", "Lightning", "Attack", "Damage"}, available levels: [1, 8, 16, 33, 46, 54, 60, 65, 75]  
LocalPhysicalDamage (Prefix), tags: {"Physical", "Damage", "Attack"}, available levels: [1, 8, 16, 33, 46, 54, 60, 65, 75]  
LocalPhysicalDamagePercent (Prefix), tags: {"Physical", "Attack", "Damage"}, available levels: [1, 8, 16, 33, 46, 60, 75]  
AdditionalArrows (Suffix), tags: {"Attack"}, available levels: [55]  
Dexterity (Suffix), tags: {"Attribute"}, available levels: [1, 11, 22, 33, 44, 55, 66, 74]  
GlobalIncreaseProjectileSkillGemLevelWeapon (Suffix), tags: {}, available levels: [2, 18, 36, 55]  
LifeGainPerTargetLocal (Suffix), tags: {"Life", "Attack"}, available levels: [8, 20, 30, 40]  
LifeGainedFromEnemyDeath (Suffix), tags: {"Life"}, available levels: [1, 11, 22, 33, 44, 55, 66]  
LifeLeechLocalPermyriad (Suffix), tags: {"Physical", "Attack", "Life"}, available levels: [21, 38, 54, 68]  
LocalAttributeRequirements (Suffix), tags: {}, available levels: [24, 32, 40, 52, 60]  
LocalBaseCriticalStrikeChance (Suffix), tags: {"Attack", "Critical"}, available levels: [1, 20, 30, 44, 59, 73]  
LocalCriticalStrikeMultiplier (Suffix), tags: {"Critical", "Attack", "Damage"}, available levels: [8, 21, 30, 44, 59, 73]  
LocalIncreasedAttackSpeed (Suffix), tags: {"Speed", "Attack"}, available levels: [1, 11, 22, 30, 37]  
LocalLightRadiusAndAccuracy (Suffix), tags: {"Attack"}, available levels: [8, 15, 30]  
ManaGainedFromEnemyDeath (Suffix), tags: {"Mana"}, available levels: [1, 12, 23, 34, 45, 56, 67]  
ManaLeechLocalPermyriad (Suffix), tags: {"Physical", "Attack", "Mana"}, available levels: [21, 38, 54, 68]  

I would like you to come up with a crafting strategy to create the following item:  
Rarity: Rare
Prefixes:
- LocalPhysicalDamage (level >= 65)
- LocalPhysicalDamagePercent (level >= 60)
One of:
- LocalColdDamage (level >= 65)
- LocalLightningDamage (level >= 65)
- LocalFireDamage (level >= 65)

Suffixes:
- AdditionalArrows
- GlobalIncreaseProjectileSkillGemLevelWeapon (level 55)
- LocalIncreasedAttackSpeed (level 37)

You will start with a Magic rarity item, with 1 modifier of your choosing.  
Consider the rough prices of currencies in your decisions, but don't avoid using them completely due to cost.  
If you would like to know more about the behaviour of a certain crafting step or combination, give me an example item, and which currency and omens you would like to use on it, and I will tell you.  

