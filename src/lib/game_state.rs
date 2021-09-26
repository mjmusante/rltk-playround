use crate::components::State;
use crate::damage_system::*;
use crate::inventory_system::{ItemCollectionSystem, ItemDropSystem};
use crate::map::draw_map;
use crate::map_indexing_system::MapIndexingSystem;
use crate::melee_combat_system::MeleeCombatSystem;
use crate::monster_ai_system::MonsterAI;
use crate::player::player_input;
use crate::visibility_system::VisibilitySystem;
use crate::*;

use rltk::{GameState, Rltk};
use specs::prelude::*;

impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        ctx.cls();

        draw_map(&self.ecs, ctx);
        {
            let positions = self.ecs.read_storage::<Position>();
            let renderables = self.ecs.read_storage::<Renderable>();
            let map = self.ecs.fetch::<Map>();

            let mut data = (&positions, &renderables).join().collect::<Vec<_>>();
            data.sort_by(|&a, &b| b.1.render_order.cmp(&a.1.render_order));
            for (pos, render) in data.iter() {
                let idx = map.xy_idx(pos.x, pos.y);
                if map.visible_tiles[idx] {
                    ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph);
                }
            }

            gui::draw_ui(&self.ecs, ctx);
        }

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
            RunState::ShowInventory => {
                let result = gui::show_inventory(self, ctx);
                match result.0 {
                    gui::ItemMenuResult::Cancel => RunState::AwaitingInput,
                    gui::ItemMenuResult::NoResponse => RunState::ShowInventory,
                    gui::ItemMenuResult::Selected => {
                        let item_entity = result.1.unwrap();
                        let names = self.ecs.read_storage::<Name>();
                        let mut gamelog = self.ecs.fetch_mut::<gamelog::GameLog>();
                        gamelog.entries.push(format!(
                            "You try to use {} but it isn't written yet",
                            names.get(item_entity).unwrap().name
                        ));
                        RunState::AwaitingInput
                    }
                }
            }
            RunState::ShowDropItem => {
                let result = gui::drop_item_menu(self, ctx);
                match result.0 {
                    gui::ItemMenuResult::Cancel => RunState::AwaitingInput,
                    gui::ItemMenuResult::NoResponse => RunState::ShowDropItem,
                    gui::ItemMenuResult::Selected => {
                        let item_entity = result.1.unwrap();
                        let mut intent = self.ecs.write_storage::<WantsToDropItem>();
                        intent
                            .insert(
                                *self.ecs.fetch::<Entity>(),
                                WantsToDropItem { item: item_entity },
                            )
                            .expect("Unable to insert intent");
                        RunState::PlayerTurn
                    }
                }
            }
        };

        {
            let mut runwriter = self.ecs.write_resource::<RunState>();
            *runwriter = newrunstate;
        }

        damage_system::delete_the_dead(&mut self.ecs);
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

        let mut drop_items = ItemDropSystem {};
        drop_items.run_now(&self.ecs);

        self.ecs.maintain();
    }
}
