use rltk::{GameState, Point, Rltk, VirtualKeyCode, RGB};
use specs::prelude::*;
use specs_derive::Component;
use std::cmp::{max, min};

use blast::damage_system::DamageSystem;
use blast::inventory_system::ItemCollectionSystem;
use blast::map::{Map, TileType};
use blast::map_indexing_system::MapIndexingSystem;
use blast::melee_combat_system::MeleeCombatSystem;
use blast::monster_ai_system::MonsterAI;
use blast::visibility_system::VisibilitySystem;
use blast::*;

fn draw_map(ecs: &World, ctx: &mut Rltk) {
    let map = ecs.fetch::<Map>();
    let mut y = 0;
    let mut x = 0;
    for (idx, tile) in map.tiles.iter().enumerate() {
        if map.revealed_tiles[idx] {
            let (glyph, mut fg) = match tile {
                TileType::Floor => (rltk::to_cp437('.'), RGB::from_f32(0.5, 0.5, 0.5)),
                TileType::Wall => (rltk::to_cp437('#'), RGB::from_f32(0.0, 1.0, 0.0)),
            };
            if !map.visible_tiles[idx] {
                fg = fg.to_greyscale();
            }
            ctx.set(x, y, fg, RGB::from_f32(0., 0., 0.), glyph);
        }

        x += 1;
        if x > 79 {
            x = 0;
            y += 1;
        }
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

fn player_input(gs: &mut State, ctx: &mut Rltk) -> RunState {
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

            _ => RunState::PlayerTurn,
        },
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
        if position.x == player_pos.x && position.y == player_pos.y && item_entity != *player_entity {
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

#[derive(Component)]
struct State {
    ecs: World,
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        ctx.cls();

        let oldrunstate = *self.ecs.fetch::<RunState>();
        let newrunstate = match oldrunstate {
            RunState::PreRun => {
                self.run_systems();
                RunState::AwaitingInput
            }
            RunState::AwaitingInput => player_input(self, ctx),
            RunState::PlayerTurn => {
                self.run_systems();
                RunState::MonsterTurn
            }
            RunState::MonsterTurn => {
                self.run_systems();
                RunState::AwaitingInput
            }
        };

        {
            let mut runwriter = self.ecs.write_resource::<RunState>();
            *runwriter = newrunstate;
        }

        damage_system::delete_the_dead(&mut self.ecs);

        draw_map(&self.ecs, ctx);

        let positions = self.ecs.read_storage::<Position>();
        let renderables = self.ecs.read_storage::<Renderable>();
        let map = self.ecs.fetch::<Map>();

        for (pos, render) in (&positions, &renderables).join() {
            let idx = map.xy_idx(pos.x, pos.y);
            if map.visible_tiles[idx] {
                ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph);
            }
        }

        gui::draw_ui(&self.ecs, ctx);
    }
}

impl State {
    fn run_systems(&mut self) {
        let mut vis = VisibilitySystem {};
        vis.run_now(&self.ecs);

        let mut mob = MonsterAI {};
        mob.run_now(&self.ecs);

        let mut mapindex = MapIndexingSystem {};
        mapindex.run_now(&self.ecs);

        let mut melee = MeleeCombatSystem {};
        melee.run_now(&self.ecs);

        let mut damage = DamageSystem {};
        damage.run_now(&self.ecs);

        let mut pickup = ItemCollectionSystem {};
        pickup.run_now(&self.ecs);

        self.ecs.maintain();
    }
}

fn main() -> rltk::BError {
    use rltk::RltkBuilder;
    let context = RltkBuilder::simple80x50()
        .with_title("Roguelike Tutorial")
        .build()?;
    let mut gs = State { ecs: World::new() };
    gs.ecs.register::<Position>();
    gs.ecs.register::<Renderable>();
    gs.ecs.register::<Player>();
    gs.ecs.register::<Viewshed>();
    gs.ecs.register::<Monster>();
    gs.ecs.register::<Name>();
    gs.ecs.register::<BlocksTile>();
    gs.ecs.register::<CombatStats>();
    gs.ecs.register::<WantsToMelee>();
    gs.ecs.register::<SufferDamage>();
    gs.ecs.register::<Item>();
    gs.ecs.register::<Potion>();
    gs.ecs.register::<WantsToPickupItem>();
    gs.ecs.register::<InBackpack>();

    gs.ecs.insert(rltk::RandomNumberGenerator::new());
    gs.ecs.insert(RunState::PreRun);
    gs.ecs.insert(gamelog::GameLog {
        entries: vec!["Welcome to Rusty Roguelike".to_string()],
    });

    let (map, room) = Map::new_map();
    let (px, py) = room.center();

    for room in map.rooms.iter().skip(1) {
        spawner::spawn_room(&mut gs.ecs, room);
    }
    gs.ecs.insert(map);

    let player_entity = spawner::player(&mut gs.ecs, px, py);
    gs.ecs.insert(player_entity);

    gs.ecs.insert(Point::new(px, py));

    rltk::main_loop(context, gs)
}
