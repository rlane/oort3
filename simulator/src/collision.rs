use rapier2d_f64::prelude::*;

const WALL_COLLISION_GROUP: u32 = 0;
const SHIP_COLLISION_GROUP: u32 = 1;

pub fn wall_interaction_groups() -> InteractionGroups {
    InteractionGroups::new(1 << WALL_COLLISION_GROUP, 1 << SHIP_COLLISION_GROUP)
}

pub fn ship_interaction_groups(_team: i32) -> InteractionGroups {
    InteractionGroups::new(
        1 << SHIP_COLLISION_GROUP,
        1 << WALL_COLLISION_GROUP | 1 << SHIP_COLLISION_GROUP,
    )
}
