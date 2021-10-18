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
pub mod random_table;
pub mod rect;
pub mod saveload_system;
pub mod spawner;
pub mod visibility_system;

pub use components::*;
pub use gamelog::*;
pub use map::{try_next_level, Map, MAPHEIGHT, MAPWIDTH};
pub use spawner::*;

use specs::prelude::*;
pub use specs::saveload::{SimpleMarker, SimpleMarkerAllocator};

#[derive(PartialEq, Copy, Clone)]
pub enum RunState {
    AwaitingInput,
    PreRun,
    PlayerTurn,
    MonsterTurn,
    ShowInventory,
    ShowDropItem,
    ShowTargeting {
        range: i32,
        item: Entity,
    },
    MainMenu {
        menu_selection: gui::MainMenuSelection,
    },
    SaveGame,
    NextLevel,
}
