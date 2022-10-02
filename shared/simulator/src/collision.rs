use rapier2d_f64::prelude::*;

const WALL_COLLISION_GROUP: u32 = 0;
const SHIP_COLLISION_GROUP: u32 = 1;
const MAX_TEAMS: u32 = 10;

fn bullet_group(team: i32) -> u32 {
    1 << (2 + team)
}

fn all_bullet_groups() -> u32 {
    let mut r: u32 = 0;
    for team in 0..(MAX_TEAMS as i32) {
        r |= bullet_group(team);
    }
    r
}

pub fn bullet_interaction_groups(team: i32) -> InteractionGroups {
    InteractionGroups::new(
        bullet_group(team),
        1 << WALL_COLLISION_GROUP | 1 << SHIP_COLLISION_GROUP,
    )
}

pub fn wall_interaction_groups() -> InteractionGroups {
    InteractionGroups::new(
        1 << WALL_COLLISION_GROUP,
        1 << SHIP_COLLISION_GROUP | all_bullet_groups(),
    )
}

pub fn ship_interaction_groups(team: i32) -> InteractionGroups {
    let bullet_groups = !bullet_group(team);
    InteractionGroups::new(
        1 << SHIP_COLLISION_GROUP,
        1 << WALL_COLLISION_GROUP | 1 << SHIP_COLLISION_GROUP | bullet_groups,
    )
}
