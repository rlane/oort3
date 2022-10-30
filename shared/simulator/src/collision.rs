use rapier2d_f64::prelude::*;

const WALL_COLLISION_GROUP: Group = Group::GROUP_1;
const SHIP_COLLISION_GROUP: Group = Group::GROUP_2;
const BULLET_GROUPS: &[Group] = &[
    Group::GROUP_3,
    Group::GROUP_4,
    Group::GROUP_5,
    Group::GROUP_6,
    Group::GROUP_7,
    Group::GROUP_8,
    Group::GROUP_9,
    Group::GROUP_10,
    Group::GROUP_11,
    Group::GROUP_12,
];

fn bullet_group(team: i32) -> Group {
    BULLET_GROUPS[team as usize]
}

fn all_bullet_groups() -> Group {
    let mut r = Group::empty();
    r.extend(BULLET_GROUPS.iter().cloned());
    r
}

pub fn bullet_interaction_groups(team: i32) -> InteractionGroups {
    InteractionGroups::new(
        bullet_group(team),
        WALL_COLLISION_GROUP | SHIP_COLLISION_GROUP,
    )
}

pub fn wall_interaction_groups() -> InteractionGroups {
    InteractionGroups::new(
        WALL_COLLISION_GROUP,
        SHIP_COLLISION_GROUP | all_bullet_groups(),
    )
}

pub fn ship_interaction_groups(team: i32) -> InteractionGroups {
    let bullet_groups = all_bullet_groups() ^ bullet_group(team);
    InteractionGroups::new(
        SHIP_COLLISION_GROUP,
        WALL_COLLISION_GROUP | SHIP_COLLISION_GROUP | bullet_groups,
    )
}
