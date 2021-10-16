use crate::components::State;
use crate::damage_system::*;
use crate::inventory_system::{ItemCollectionSystem, ItemDropSystem, ItemUseSystem};
use crate::map::draw_map;
use crate::map_indexing_system::MapIndexingSystem;
use crate::melee_combat_system::MeleeCombatSystem;
use crate::monster_ai_system::MonsterAI;
use crate::player::player_input;
use crate::visibility_system::VisibilitySystem;
use crate::*;

use rltk::{GameState, Rltk};
use specs::prelude::*;

impl State {
    fn game_screen(&mut self, ctx: &mut Rltk) {
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
    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        ctx.cls();
        let oldrunstate = *self.ecs.fetch::<RunState>();

        match oldrunstate {
            RunState::MainMenu { .. } => (),
            _ => {
                self.game_screen(ctx);
            }
        };

        let newrunstate = match oldrunstate {
            RunState::MainMenu { .. } => match gui::main_menu(self, ctx) {
                gui::MainMenuResult::NoSelection { selected } => RunState::MainMenu {
                    menu_selection: selected,
                },
                gui::MainMenuResult::Selected { selected } => match selected {
                    gui::MainMenuSelection::NewGame => RunState::PreRun,
                    gui::MainMenuSelection::LoadGame => RunState::PreRun,
                    gui::MainMenuSelection::Quit => {
                        ::std::process::exit(0);
                    }
                },
            },
            RunState::SaveGame => RunState::MainMenu {
                menu_selection: gui::MainMenuSelection::LoadGame,
            },
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
                        let is_ranged = self.ecs.read_storage::<Ranged>();
                        if let Some(ranged_item) = is_ranged.get(item_entity) {
                            RunState::ShowTargeting {
                                range: ranged_item.range,
                                item: item_entity,
                            }
                        } else {
                            let mut intent = self.ecs.write_storage::<WantsToUseItem>();
                            intent
                                .insert(
                                    *self.ecs.fetch::<Entity>(),
                                    WantsToUseItem {
                                        item: item_entity,
                                        target: None,
                                    },
                                )
                                .expect("Unable to insert intent");
                            RunState::PlayerTurn
                        }
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
            RunState::ShowTargeting { range, item } => match gui::ranged_target(self, ctx, range) {
                (gui::ItemMenuResult::Cancel, _) => RunState::AwaitingInput,
                (gui::ItemMenuResult::NoResponse, _) => RunState::ShowTargeting { range, item },
                (gui::ItemMenuResult::Selected, target) => {
                    let mut intent = self.ecs.write_storage::<WantsToUseItem>();
                    intent
                        .insert(*self.ecs.fetch::<Entity>(), WantsToUseItem { item, target })
                        .expect("Unable to insert intent");
                    RunState::PlayerTurn
                }
            },
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

        let mut useitems = ItemUseSystem {};
        useitems.run_now(&self.ecs);

        self.ecs.maintain();
    }
}
