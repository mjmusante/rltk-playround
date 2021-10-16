pub mod components;
pub mod damage_system;
pub mod game_state;
pub mod gamelog;
pub mod gui;
pub mod inventory_system;
pub mod map;
pub mod map_indexing_system;
pub mod melee_combat_system;
pub mod monster_ai_system;
pub mod player;
pub mod rect;
pub mod spawner;
pub mod visibility_system;

pub use components::*;
pub use gamelog::*;
pub use map::{Map, MAPHEIGHT, MAPWIDTH};
pub use spawner::*;

use specs::prelude::*;

#[derive(PartialEq, Copy, Clone)]
pub enum RunState {
    AwaitingInput,
    PreRun,
    PlayerTurn,
    MonsterTurn,
    ShowInventory,
    ShowDropItem,
    ShowTargeting { range: i32, item: Entity },
}
