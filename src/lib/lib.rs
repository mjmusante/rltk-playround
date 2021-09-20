pub mod damage_system;
pub mod map;
pub mod map_indexing_system;
pub mod melee_combat_system;
pub mod monster_ai_system;
pub mod rect;
pub mod visibility_system;
pub mod gui;

pub mod components;

pub use components::{
    BlocksTile, CombatStats, Monster, Name, Player, Position, SufferDamage, Viewshed, WantsToMelee,
};
pub use map::Map;

#[derive(PartialEq, Copy, Clone)]
pub enum RunState {
    AwaitingInput,
    PreRun,
    PlayerTurn,
    MonsterTurn,
}
