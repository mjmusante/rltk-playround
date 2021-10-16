use crate::{
    gamelog::GameLog, AreaOfEffect, CombatStats, Consumable, InBackpack, InflictsDamage, Map, Name,
    Position, ProvidesHealing, SufferDamage, WantsToDropItem, WantsToPickupItem, WantsToUseItem,
};
use specs::prelude::*;

pub struct ItemCollectionSystem {}

type CollectionData<'a> = (
    ReadExpect<'a, Entity>,
    WriteExpect<'a, GameLog>,
    WriteStorage<'a, WantsToPickupItem>,
    WriteStorage<'a, Position>,
    ReadStorage<'a, Name>,
    WriteStorage<'a, InBackpack>,
);

impl<'a> System<'a> for ItemCollectionSystem {
    type SystemData = CollectionData<'a>;

    fn run(&mut self, data: Self::SystemData) {
        let (player_entity, mut gamelog, mut wants_pickup, mut positions, names, mut backpack) =
            data;

        for pickup in wants_pickup.join() {
            positions.remove(pickup.item);
            backpack
                .insert(
                    pickup.item,
                    InBackpack {
                        owner: pickup.collected_by,
                    },
                )
                .expect("Unable to insert backpack entry");

            if pickup.collected_by == *player_entity {
                gamelog.entries.push(format!(
                    "You pick up the {}.",
                    names.get(pickup.item).unwrap().name
                ));
            }
        }

        wants_pickup.clear();
    }
}

pub struct ItemUseSystem {}

type ItemData<'a> = (
    ReadExpect<'a, Entity>,
    ReadExpect<'a, Map>,
    WriteExpect<'a, GameLog>,
    Entities<'a>,
    WriteStorage<'a, WantsToUseItem>,
    ReadStorage<'a, Name>,
    ReadStorage<'a, ProvidesHealing>,
    WriteStorage<'a, CombatStats>,
    ReadStorage<'a, Consumable>,
    ReadStorage<'a, InflictsDamage>,
    WriteStorage<'a, SufferDamage>,
    ReadStorage<'a, AreaOfEffect>,
);

impl<'a> System<'a> for ItemUseSystem {
    type SystemData = ItemData<'a>;

    fn run(&mut self, data: Self::SystemData) {
        let (
            player_entity,
            map,
            mut gamelog,
            entities,
            mut useitems,
            names,
            healing,
            mut combat_stats,
            consumables,
            inflicts_damage,
            mut suffer_damage,
            aoe,
        ) = data;

        let mut used_item = false;

        for (entity, useitem) in (&entities, &useitems).join() {
            let mut targets: Vec<Entity> = Vec::new();

            if let Some(target) = useitem.target {
                if let Some(area_effect) = aoe.get(useitem.item) {
                    let blast_tiles = rltk::field_of_view(target, area_effect.radius, &*map);
                    for tile_idx in blast_tiles.iter() {
                        let idx = map.xy_idx(tile_idx.x, tile_idx.y);
                        for mob in map.tile_content[idx].iter() {
                            targets.push(*mob);
                        }
                    }
                } else {
                    let idx = map.xy_idx(target.x, target.y);
                    for mob in map.tile_content[idx].iter() {
                        targets.push(*mob);
                    }
                }
            } else {
                targets.push(*player_entity);
            }

            if let Some(healer) = healing.get(useitem.item) {
                for target in targets.iter() {
                    if let Some(stats) = combat_stats.get_mut(*target) {
                        stats.hp = i32::min(stats.max_hp, stats.hp + healer.heal_amount);
                        if entity == *player_entity {
                            gamelog.entries.push(format!(
                                "You drink the {}, healing {} hp",
                                names.get(useitem.item).unwrap().name,
                                healer.heal_amount
                            ));
                        }
                    }
                }
            }

            if let Some(damage) = inflicts_damage.get(useitem.item) {
                for mob in targets.iter() {
                    SufferDamage::new_damage(&mut suffer_damage, *mob, damage.damage);
                    if entity == *player_entity {
                        let mob_name = names.get(*mob).unwrap();
                        let item_name = names.get(useitem.item).unwrap();
                        gamelog.entries.push(format!(
                            "You use {} on {}, inflicting {} hp.",
                            item_name.name, mob_name.name, damage.damage
                        ));
                    }
                    used_item = true;
                }
            }

            if used_item && consumables.get(useitem.item).is_some() {
                entities.delete(useitem.item).expect("Delete failed");
            }
        }
        useitems.clear();
    }
}

pub struct ItemDropSystem {}

type DropData<'a> = (
    ReadExpect<'a, Entity>,
    WriteExpect<'a, GameLog>,
    Entities<'a>,
    WriteStorage<'a, WantsToDropItem>,
    ReadStorage<'a, Name>,
    WriteStorage<'a, Position>,
    WriteStorage<'a, InBackpack>,
);

impl<'a> System<'a> for ItemDropSystem {
    type SystemData = DropData<'a>;

    fn run(&mut self, data: Self::SystemData) {
        let (
            player_entity,
            mut gamelog,
            entities,
            mut wants_drop,
            names,
            mut positions,
            mut backpack,
        ) = data;

        for (entity, to_drop) in (&entities, &wants_drop).join() {
            let mut dropper_pos: Position = Position { x: 0, y: 0 };
            {
                let dropped_pos = positions.get(entity).unwrap();
                dropper_pos.x = dropped_pos.x;
                dropper_pos.y = dropped_pos.y;
            }

            positions
                .insert(
                    to_drop.item,
                    Position {
                        x: dropper_pos.x,
                        y: dropper_pos.y,
                    },
                )
                .expect("Unable to insert position");
            backpack.remove(to_drop.item);

            if entity == *player_entity {
                gamelog.entries.push(format!(
                    "You drop the {}",
                    names.get(to_drop.item).unwrap().name
                ));
            }
        }

        wants_drop.clear();
    }
}
