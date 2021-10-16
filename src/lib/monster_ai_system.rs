use crate::{Confusion, Map, Monster, Position, RunState, Viewshed, WantsToMelee};
use rltk::Point;
use specs::prelude::*;

pub struct MonsterAI {}

type MonsterAIType<'a> = (
    WriteExpect<'a, Map>,
    ReadExpect<'a, Point>,
    ReadExpect<'a, Entity>,
    ReadExpect<'a, RunState>,
    Entities<'a>,
    WriteStorage<'a, Viewshed>,
    ReadStorage<'a, Monster>,
    WriteStorage<'a, Position>,
    WriteStorage<'a, WantsToMelee>,
    WriteStorage<'a, Confusion>,
);

impl<'a> System<'a> for MonsterAI {
    type SystemData = MonsterAIType<'a>;

    fn run(&mut self, data: Self::SystemData) {
        let (
            mut map,
            player_pos,
            player_entity,
            run_state,
            entities,
            mut viewshed,
            monster,
            mut position,
            mut wants_to_melee,
            mut confused,
        ) = data;

        if *run_state != RunState::MonsterTurn {
            return;
        }

        for (entity, mut viewshed, _monster, mut pos) in
            (&entities, &mut viewshed, &monster, &mut position).join()
        {
            if let Some(confuzzled) = confused.get_mut(entity) {
                if confuzzled.turns < 2 {
                    confused.remove(entity);
                } else {
                    confuzzled.turns -= 1;
                }
                continue;
            }

            if viewshed.visible_tiles.contains(&*player_pos) {
                let distance =
                    rltk::DistanceAlg::Pythagoras.distance2d(Point::new(pos.x, pos.y), *player_pos);
                if distance < 1.5 {
                    wants_to_melee
                        .insert(
                            entity,
                            WantsToMelee {
                                target: *player_entity,
                            },
                        )
                        .expect("unable to insert");
                } else if viewshed.visible_tiles.contains(&*player_pos) {
                    let path = rltk::a_star_search(
                        map.xy_idx(pos.x, pos.y) as i32,
                        map.xy_idx(player_pos.x, player_pos.y) as i32,
                        &*map,
                    );
                    if path.success && path.steps.len() > 1 {
                        let mut idx = map.xy_idx(pos.x, pos.y);
                        map.blocked[idx] = false;
                        pos.x = path.steps[1] as i32 % map.width;
                        pos.y = path.steps[1] as i32 / map.width;
                        idx = map.xy_idx(pos.x, pos.y);
                        map.blocked[idx] = true;
                        viewshed.dirty = true;
                    }
                }
            }
        }
    }
}
