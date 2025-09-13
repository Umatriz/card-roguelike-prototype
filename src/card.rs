use std::collections::VecDeque;

use bevy::{ecs::relationship::RelatedSpawner, prelude::*};
use tiny_bail::prelude::*;

use crate::visual_actions;
// use haalka::prelude::*;

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, spawn_card);

    app.add_observer(stacked_handling);
}

fn spawn_card(mut commands: Commands) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(50.0),
                position_type: PositionType::Absolute,
                top: Val::Percent(10.0),
                padding: UiRect::new(Val::Percent(20.0), Val::Percent(20.0), Val::ZERO, Val::ZERO),
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
                                .observe(card_dragging_end)
                                .observe(child_remove_handling);
                        }
                    })),
                ))
                .observe(stack_capture_card)
                .observe(stack_determine_z_index);

            parent
                .spawn(cards_stack())
                .observe(stack_capture_card)
                .observe(stack_determine_z_index);
        });
}

pub const WIDTH: f32 = 150.0;
pub const HEIGHT: f32 = 200.0;
pub const STACK_HEIGHT: f32 = HEIGHT * 1.2;

pub mod colors {
    use bevy::color::Color;

    pub const BLACK: Color = Color::srgb_u8(26, 26, 26);
    pub const AKAROA: Color = Color::srgb_u8(220, 201, 169);
    pub const MOJO: Color = Color::srgb_u8(184, 58, 45);
    pub const FINLANDIA: Color = Color::srgb_u8(78, 104, 81);
}

const BASE_STACKED_CRAD_LAYER: i32 = -1;
const BASE_CARD_LAYER: i32 = 1;

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
        GlobalZIndex(BASE_CARD_LAYER),
        Card,
        children![(Text::new(text))],
    )
}

fn card_dragging_start(
    on_drag_start: On<Pointer<DragStart>>,
    mut z_indexes: Query<(&mut Pickable, &mut GlobalZIndex), With<Card>>,
) {
    let (mut pickable, mut z_index) = r!(z_indexes.get_mut(on_drag_start.event().entity));
    pickable.should_block_lower = false;
    z_index.0 = 1;
}

fn card_dragging(on_drag: On<Pointer<Drag>>, mut transforms: Query<&mut UiTransform, With<Card>>) {
    let Ok(mut transform) = transforms.get_mut(on_drag.event().entity) else {
        return;
    };

    transform.translation = Val2::px(on_drag.distance.x, on_drag.distance.y);
}

fn card_dragging_end(
    on_drag_end: On<Pointer<DragEnd>>,
    mut commands: Commands,
    mut query: Query<(&StackedCardOffset, &mut Pickable, &mut GlobalZIndex)>,
) {
    let (offset, mut pickable, mut z_index) = r!(query.get_mut(on_drag_end.event().entity));
    commands
        .entity(on_drag_end.event().entity)
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

fn stacked_handling(
    on_stacked: On<Insert, StackedIn>,
    stacked_query: Query<&StackedIn>,
    stacks: Query<&CardsStack>,
    mut nodes: Query<(&UiGlobalTransform, &mut UiTransform)>,
    mut commands: Commands,
) {
    let card_entity = on_stacked.event().entity;
    let stack_entity = r!(stacked_query.get(card_entity)).0;

    let [
        (stack_global_transform, _stack_transform),
        (card_global_transform, mut card_transform),
    ] = r!(nodes.get_many_mut([stack_entity, card_entity]));
    let cards_stack = r!(stacks.get(stack_entity));

    let offset = card_offset_n(0, cards_stack.max as usize);
    // get the vector pointing from the target position to the card
    let delta = card_global_transform.translation - (stack_global_transform.translation + offset);

    card_transform.translation = Val2::px(delta.x, delta.y);

    commands
        .entity(stack_entity)
        .queue(update_visible_cards)
        .queue(calculate_visible_offsets);
}

fn child_remove_handling(on_removed: On<Remove, ChildOf>, mut commands: Commands) {
    commands
        .entity(on_removed.event().entity)
        .insert(GlobalZIndex(BASE_CARD_LAYER))
        .queue(make_idle);
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

fn update_visible_cards(mut stack_enitity: EntityWorldMut) {
    let cards = r!(stack_enitity.get::<StackedCards>());
    let stack = r!(stack_enitity.get::<CardsStack>());

    // Slice `stack.max` cards from the top of the stack
    let starting_idx = cards.0.len().saturating_sub(stack.max as usize);
    let visible_cards = &cards.0[starting_idx..].to_vec();

    stack_enitity.replace_children(visible_cards);
}

fn calculate_visible_offsets(mut stack_entity: EntityWorldMut) {
    let (children, stack) = r!(stack_entity.get_components::<(&Children, &CardsStack)>());
    let visible_cards = children.to_vec();
    let max_cards = stack.max as usize;

    for (idx, cards_entity) in visible_cards.into_iter().enumerate() {
        let offset = card_offset_n(idx, max_cards);
        stack_entity.world_scope(|world: &mut World| {
            world.entity_mut(cards_entity).insert((
                GlobalZIndex(BASE_STACKED_CRAD_LAYER - idx as i32),
                StackedCardOffset(offset),
            ));
        });
    }
}

fn card_offset_n(n: usize, max_cards: usize) -> Vec2 {
    let rest = STACK_HEIGHT - HEIGHT;
    let base_offset = rest / 2.0;
    let offset = base_offset + rest / max_cards as f32 * n as f32;
    vec2(0.0, offset)
}

fn cards_stack() -> impl Bundle {
    (
        CardsStack { max: 3 },
        Node {
            width: Val::Px(WIDTH),
            height: Val::Px(STACK_HEIGHT),

            ..default()
        },
        BackgroundColor(colors::BLACK),
        Outline::new(Val::Percent(3.0), Val::ZERO, colors::AKAROA),
        Pickable::default(),
    )
}

fn stack_determine_z_index(
    on_spawn: On<Add, CardsStack>,
    stacks: Query<&CardsStack>,
    mut commands: Commands,
) {
    let entity = on_spawn.event().entity;
    let stack = r!(stacks.get(entity));

    commands.entity(entity).insert(GlobalZIndex(
        BASE_STACKED_CRAD_LAYER * (stack.max as i32 + 1),
    ));
}

pub fn stack_capture_card(on_drag_drop: On<Pointer<DragDrop>>, mut commands: Commands) {
    // TODO: Add `Card` component check
    commands
        .entity(on_drag_drop.dropped)
        .insert(StackedIn(on_drag_drop.event().entity));
}
