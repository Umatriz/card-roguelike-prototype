use std::collections::VecDeque;

use bevy::{ecs::relationship::RelatedSpawner, prelude::*};
use tiny_bail::prelude::*;

use crate::visual_actions;
// use haalka::prelude::*;

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, spawn_card);

    app.add_observer(stacked_children_handling);
}

fn spawn_card(mut commands: Commands) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(30.0),
                bottom: Val::Px(0.0),
                padding: UiRect::new(Val::Percent(20.0), Val::Percent(20.0), Val::ZERO, Val::ZERO),
                position_type: PositionType::Absolute,
                align_content: AlignContent::Center,
                justify_content: JustifyContent::SpaceBetween,
                ..default()
            },
            Pickable::IGNORE,
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    cards_stack(),
                    StackedCards::spawn(SpawnWith(|parent: &mut RelatedSpawner<StackedIn>| {
                        for c in 'a'..='e' {
                            parent
                                .spawn(card(c))
                                .observe(card_dragging_start)
                                .observe(card_dragging)
                                .observe(card_dragging_end);
                        }
                    })),
                ))
                .observe(stack_capture_card);
            parent.spawn(cards_stack()).observe(stack_capture_card);
        });
}

pub const WIDTH: f32 = 150.0;
pub const HEIGHT: f32 = 200.0;

pub mod colors {
    use bevy::color::Color;

    pub const BLACK: Color = Color::srgb_u8(26, 26, 26);
    pub const AKAROA: Color = Color::srgb_u8(220, 201, 169);
    pub const MOJO: Color = Color::srgb_u8(184, 58, 45);
    pub const FINLANDIA: Color = Color::srgb_u8(78, 104, 81);
}

#[derive(Component, Debug)]
pub struct Card;

fn card(text: impl Into<String>) -> impl Bundle {
    (
        Node {
            position_type: PositionType::Absolute,
            width: Val::Px(WIDTH),
            height: Val::Px(HEIGHT),
            ..default()
        },
        Pickable::default(),
        BackgroundColor(colors::FINLANDIA),
        Outline::new(Val::Percent(3.0), Val::ZERO, colors::BLACK),
        GlobalZIndex(0),
        Card,
        children![(Text::new(text))],
    )
}

fn card_dragging_start(
    on_drag_start: On<Pointer<DragStart>>,
    mut z_indexes: Query<(&mut Pickable, &mut GlobalZIndex), With<Card>>,
) {
    let (mut pickable, mut z_index) = r!(z_indexes.get_mut(on_drag_start.entity()));
    pickable.should_block_lower = false;
    z_index.0 = 1;
}

fn card_dragging(on_drag: On<Pointer<Drag>>, mut transforms: Query<&mut UiTransform, With<Card>>) {
    let Ok(mut transform) = transforms.get_mut(on_drag.entity()) else {
        return;
    };

    transform.translation = Val2::px(on_drag.distance.x, on_drag.distance.y);
}

fn card_dragging_end(
    on_drag_end: On<Pointer<DragEnd>>,
    mut commands: Commands,
    mut query: Query<(&StackedCardOffset, &mut Pickable, &mut GlobalZIndex)>,
) {
    let (offset, mut pickable, mut z_index) = r!(query.get_mut(on_drag_end.entity()));
    commands
        .entity(on_drag_end.entity())
        .queue(visual_actions::Move::new(
            offset.0,
            EaseFunction::QuinticOut,
            0.5,
        ));
    pickable.should_block_lower = true;
    z_index.0 = 0;
}

#[derive(Component, Debug)]
pub struct CardsStack {
    max: u8,
}

#[derive(Component)]
pub struct StackedCardOffset(Vec2);

#[derive(Component)]
#[relationship(relationship_target = StackedCards)]
pub struct StackedIn(Entity);

#[derive(Component)]
#[relationship_target(relationship = StackedIn, linked_spawn)]
pub struct StackedCards(Vec<Entity>);

#[derive(Component, Default)]
pub struct VisibleCardsQueue(VecDeque<Entity>);

fn stacked_children_handling(
    on_stacked: On<Insert, StackedIn>,
    stacked_query: Query<&StackedIn>,
    stacks: Query<(&CardsStack, &Children)>,
    mut nodes: Query<(&Node, &UiGlobalTransform, &mut UiTransform)>,
    mut commands: Commands,
) {
    let card_entity = on_stacked.entity();
    let stack = r!(stacked_query.get(card_entity));
    // commands.entity(stack.0).add_child(card_entity);

    let [
        (stack_node, stack_global_transform, _stack_transform),
        (card_node, card_global_transform, mut card_transform),
    ] = r!(nodes.get_many_mut([stack.0, card_entity]));

    // // get the vector pointing from the stack to the card
    // let delta = card_global_transform.translation - stack_global_transform.translation;

    let (cards_stack, children) = r!(stacks.get(stack.0));

    let (Val::Px(stack_h), Val::Px(card_h)) = (stack_node.height, card_node.height) else {
        return;
    };

    let rest = (stack_h - card_h) / (cards_stack.max - 1) as f32;
    let max = cards_stack.max as usize;

    if children.len() == max {
        let first = r!(children.first());
        commands.trigger_targets(MakeIdle, *first);
    }

    commands
        .entity(stack.0)
        .add_one_related::<ChildOf>(card_entity);

    // let delta = Val2::px(delta.x, delta.y);
    // card_transform.translation = delta;
}

pub fn make_idle(mut entity: EntityWorldMut) {
    let mut visibility = r!(entity.get_mut::<Visibility>());
    *visibility = Visibility::Hidden;

    let mut pickable = r!(entity.get_mut::<Pickable>());
    *pickable = Pickable::IGNORE;
}

pub fn make_active(mut entity: EntityWorldMut) {
    let mut visibility = r!(entity.get_mut::<Visibility>());
    *visibility = Visibility::Visible;

    let mut pickable = r!(entity.get_mut::<Pickable>());
    *pickable = Pickable {
        // This is so it can trigger `DragEnd` event on the cards stack
        should_block_lower: false,
        is_hoverable: true,
    };
}

pub fn attach_to_stack(mut entity: EntityWorldMut) {
    let this_entity = entity.id();
    let stack = r!(entity.get::<StackedIn>());
    let stack = stack.0;

    entity.world_scope(|world: &mut World| {
        let mut entity = world.entity_mut(stack);
        let cards_stack = r!(entity.get::<CardsStack>());
        let max_cards = cards_stack.max;

        let children = r!(entity.get::<Children>());
        let first = children.first().copied();

        if children.len() == max_cards as usize {
            let first = r!(first);
            entity.world_scope(|world: &mut World| {
                let entity = world.entity_mut(first);
                make_idle(entity);
            });
            entity.remove_child(first);
        }

        entity.add_child(this_entity);
    });
}

fn cards_stack() -> impl Bundle {
    (
        CardsStack { max: 3 },
        Node {
            width: Val::Px(WIDTH),
            height: Val::Px(HEIGHT * 1.2),

            ..default()
        },
        BackgroundColor(colors::BLACK),
        Outline::new(Val::Percent(3.0), Val::ZERO, colors::AKAROA),
        Pickable::default(),
    )
}

pub fn stack_capture_card(on_drag_drop: On<Pointer<DragDrop>>, mut commands: Commands) {
    // TODO: Add `Card` component check
    commands
        .entity(on_drag_drop.dropped)
        .insert(StackedIn(on_drag_drop.entity()));
}
