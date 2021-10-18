use crate::*;
use rltk::{Point, Rltk, VirtualKeyCode};
use specs::prelude::*;
use std::cmp::{max, min};

pub fn player_input(gs: &mut State, ctx: &mut Rltk) -> RunState {
    // Player movement
    match ctx.key {
        None => RunState::AwaitingInput,
        Some(key) => match key {
            VirtualKeyCode::H | VirtualKeyCode::Numpad4 | VirtualKeyCode::Left => {
                try_move_player(-1, 0, &mut gs.ecs);
                RunState::PlayerTurn
            }
            VirtualKeyCode::L | VirtualKeyCode::Numpad6 | VirtualKeyCode::Right => {
                try_move_player(1, 0, &mut gs.ecs);
                RunState::PlayerTurn
            }
            VirtualKeyCode::K | VirtualKeyCode::Numpad8 | VirtualKeyCode::Up => {
                try_move_player(0, -1, &mut gs.ecs);
                RunState::PlayerTurn
            }
            VirtualKeyCode::J | VirtualKeyCode::Numpad2 | VirtualKeyCode::Down => {
                try_move_player(0, 1, &mut gs.ecs);
                RunState::PlayerTurn
            }

            VirtualKeyCode::Y | VirtualKeyCode::Numpad9 => {
                try_move_player(-1, -1, &mut gs.ecs);
                RunState::PlayerTurn
            }

            VirtualKeyCode::U | VirtualKeyCode::Numpad7 => {
                try_move_player(1, -1, &mut gs.ecs);
                RunState::PlayerTurn
            }

            VirtualKeyCode::N | VirtualKeyCode::Numpad3 => {
                try_move_player(1, 1, &mut gs.ecs);
                RunState::PlayerTurn
            }

            VirtualKeyCode::B | VirtualKeyCode::Numpad1 => {
                try_move_player(-1, 1, &mut gs.ecs);
                RunState::PlayerTurn
            }

            VirtualKeyCode::G => {
                get_item(&mut gs.ecs);
                RunState::PlayerTurn
            }

            VirtualKeyCode::Period => {
                if try_next_level(&mut gs.ecs) {
                    return RunState::NextLevel;
                }
                RunState::PlayerTurn
            }

            VirtualKeyCode::I => RunState::ShowInventory,
            VirtualKeyCode::D => RunState::ShowDropItem,
            VirtualKeyCode::Escape => RunState::SaveGame,
            VirtualKeyCode::Numpad5 | VirtualKeyCode::Space => skip_turn(&mut gs.ecs),

            _ => RunState::PlayerTurn,
        },
    }
}

fn try_move_player(delta_x: i32, delta_y: i32, ecs: &mut World) {
    let mut positions = ecs.write_storage::<Position>();
    let mut players = ecs.write_storage::<Player>();
    let mut viewsheds = ecs.write_storage::<Viewshed>();
    let mut ppos = ecs.write_resource::<Point>();

    let entities = ecs.entities();
    let mut wants_to_melee = ecs.write_storage::<WantsToMelee>();

    let combat_stats = ecs.read_storage::<CombatStats>();
    let map = ecs.fetch::<Map>();

    for (entity, _player, pos, viewshed) in
        (&entities, &mut players, &mut positions, &mut viewsheds).join()
    {
        if pos.x + delta_x < 1
            || pos.x + delta_x > map.width - 1
            || pos.y + delta_y < 1
            || pos.y + delta_y > map.width - 1
        {
            return;
        }
        let destination_idx = map.xy_idx(pos.x + delta_x, pos.y + delta_y);

        for potential_target in map.tile_content[destination_idx].iter() {
            if combat_stats.get(*potential_target).is_some() {
                wants_to_melee
                    .insert(
                        entity,
                        WantsToMelee {
                            target: *potential_target,
                        },
                    )
                    .expect("Add target failed");
                return;
            }
        }
        if !map.blocked[destination_idx] {
            pos.x = min(79, max(0, pos.x + delta_x));
            pos.y = min(49, max(0, pos.y + delta_y));
            ppos.x = pos.x;
            ppos.y = pos.y;
            viewshed.dirty = true;
        }
    }
}

fn get_item(ecs: &mut World) {
    let player_pos = ecs.fetch::<Point>();
    let player_entity = ecs.fetch::<Entity>();
    let entities = ecs.entities();
    let items = ecs.read_storage::<Position>();
    let positions = ecs.read_storage::<Position>();
    let mut gamelog = ecs.fetch_mut::<GameLog>();

    let mut target_item: Option<Entity> = None;
    for (item_entity, _item, position) in (&entities, &items, &positions).join() {
        if position.x == player_pos.x && position.y == player_pos.y && item_entity != *player_entity
        {
            target_item = Some(item_entity);
        }
    }

    match target_item {
        None => gamelog
            .entries
            .push("There is nothing here to pick up.".to_string()),
        Some(item) => {
            let mut pickup = ecs.write_storage::<WantsToPickupItem>();
            pickup
                .insert(
                    *player_entity,
                    WantsToPickupItem {
                        collected_by: *player_entity,
                        item,
                    },
                )
                .expect("Unalbe to insert want to pickup");
        }
    }
}

fn skip_turn(ecs: &mut World) -> RunState {
    let player_entity = ecs.fetch::<Entity>();
    let viewshed_components = ecs.read_storage::<Viewshed>();
    let monsters = ecs.read_storage::<Monster>();
    let worldmap_resource = ecs.fetch::<Map>();

    let mut can_heal = true;

    let viewshed = viewshed_components.get(*player_entity).unwrap();

    for tile in viewshed.visible_tiles.iter() {
        let idx = worldmap_resource.xy_idx(tile.x, tile.y);
        for entity_id in worldmap_resource.tile_content[idx].iter() {
            if monsters.get(*entity_id).is_some() {
                can_heal = false;
                break;
            }
        }
    }
    if can_heal {
        let mut health_components = ecs.write_storage::<CombatStats>();
        let player_hp = health_components.get_mut(*player_entity).unwrap();
        player_hp.hp = i32::min(player_hp.hp + 1, player_hp.max_hp);
    }

    RunState::PlayerTurn
}
