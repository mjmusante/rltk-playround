use rltk::{GameState, Point, Rltk, VirtualKeyCode, RGB};
use specs::prelude::*;
use specs_derive::Component;
use std::cmp::{max, min};

use blast::map::{Map, TileType};
use blast::monster_ai_system::MonsterAI;
use blast::visibility_system::VisibilitySystem;
use blast::{Monster, Name, Player, Position, Viewshed};

#[derive(PartialEq, Copy, Clone)]
pub enum RunState {
    Paused,
    Running,
}

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

    let map = ecs.fetch::<Map>();
    for (_player, pos, viewshed) in (&mut players, &mut positions, &mut viewsheds).join() {
        let destination_idx = map.xy_idx(pos.x + delta_x, pos.y + delta_y);
        if map.tiles[destination_idx] != TileType::Wall {
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
        None => RunState::Paused,
        Some(key) => match key {
            VirtualKeyCode::Left => {
                try_move_player(-1, 0, &mut gs.ecs);
                RunState::Running
            }
            VirtualKeyCode::Right => {
                try_move_player(1, 0, &mut gs.ecs);
                RunState::Running
            }
            VirtualKeyCode::Up => {
                try_move_player(0, -1, &mut gs.ecs);
                RunState::Running
            }
            VirtualKeyCode::Down => {
                try_move_player(0, 1, &mut gs.ecs);
                RunState::Running
            }
            _ => RunState::Paused,
        },
    }
}

#[derive(Component)]
struct Renderable {
    glyph: rltk::FontCharType,
    fg: RGB,
    bg: RGB,
}

#[derive(Component)]

struct State {
    ecs: World,
    pub runstate: RunState,
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        if self.runstate == RunState::Paused {
            self.runstate = player_input(self, ctx);
        } else {
            self.run_systems();
            self.runstate = RunState::Paused;
            ctx.cls();

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
        }
    }
}

impl State {
    fn run_systems(&mut self) {
        let mut vis = VisibilitySystem {};
        vis.run_now(&self.ecs);

        let mut mob = MonsterAI {};
        mob.run_now(&self.ecs);

        self.ecs.maintain();
    }
}

fn main() -> rltk::BError {
    use rltk::RltkBuilder;
    let context = RltkBuilder::simple80x50()
        .with_title("Roguelike Tutorial")
        .build()?;
    let mut gs = State {
        ecs: World::new(),
        runstate: RunState::Running,
    };
    gs.ecs.register::<Position>();
    gs.ecs.register::<Renderable>();
    gs.ecs.register::<Player>();
    gs.ecs.register::<Viewshed>();
    gs.ecs.register::<Monster>();
    gs.ecs.register::<Name>();

    let (map, room) = Map::new_map();
    let (px, py) = room.center();

    let mut rng = rltk::RandomNumberGenerator::new();
    for (i, room) in map.rooms.iter().skip(1).enumerate() {
        let (x, y) = room.center();

        let (name, glyph) = match rng.roll_dice(1, 2) {
            1 => ("Goblin".to_string(), rltk::to_cp437('g')),
            2 => ("Orc".to_string(), rltk::to_cp437('o')),
            _ => {
                panic!("invalid roll");
            }
        };

        gs.ecs
            .create_entity()
            .with(Position { x, y })
            .with(Renderable {
                glyph: glyph,
                fg: RGB::named(rltk::RED),
                bg: RGB::named(rltk::BLACK),
            })
            .with(Viewshed {
                visible_tiles: Vec::new(),
                range: 8,
                dirty: true,
            })
            .with(Monster {})
            .with(Name {
                name: format!("{} #{}", &name, i),
            })
            .build();
    }
    gs.ecs.insert(map);

    gs.ecs
        .create_entity()
        .with(Position { x: px, y: py })
        .with(Renderable {
            glyph: rltk::to_cp437('@'),
            fg: RGB::named(rltk::YELLOW),
            bg: RGB::named(rltk::BLACK),
        })
        .with(Player {})
        .with(Viewshed {
            visible_tiles: Vec::new(),
            range: 8,
            dirty: true,
        })
        .with(Name {
            name: "Player".to_string(),
        })
        .build();

    gs.ecs.insert(Point::new(px, py));

    rltk::main_loop(context, gs)
}
