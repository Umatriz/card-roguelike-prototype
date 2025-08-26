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
                        parent
                            .spawn(card())
                            .observe(card_dragging_start)
                            .observe(card_dragging)
                            .observe(card_dragging_end);
                        parent
                            .spawn(card())
                            .observe(card_dragging_start)
                            .observe(card_dragging)
                            .observe(card_dragging_end);
                        parent
                            .spawn(card())
                            .observe(card_dragging_start)
                            .observe(card_dragging)
                            .observe(card_dragging_end);
                        parent
                            .spawn(card())
                            .observe(card_dragging_start)
                            .observe(card_dragging)
                            .observe(card_dragging_end);
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

fn card() -> impl Bundle {
    (
        Node {
            width: Val::Px(WIDTH),
            height: Val::Px(HEIGHT),
            ..default()
        },
        Pickable::default(),
        BackgroundColor(colors::MOJO),
        GlobalZIndex(0),
        Card,
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
    mut query: Query<(&mut Pickable, &mut GlobalZIndex)>,
) {
    let (mut pickable, mut z_index) = r!(query.get_mut(on_drag_end.entity()));
    commands
        .entity(on_drag_end.entity())
        .queue(visual_actions::Move::new(
            Vec2::ZERO,
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
#[relationship(relationship_target = StackedCards)]
pub struct StackedIn(Entity);

#[derive(Component)]
#[relationship_target(relationship = StackedIn, linked_spawn)]
pub struct StackedCards(Vec<Entity>);

fn stacked_children_handling(
    on_stacked: On<Insert, StackedIn>,
    stacked_query: Query<&StackedIn>,
    mut transforms: Query<(&UiGlobalTransform, &mut UiTransform)>,
    mut commands: Commands,
) {
    let card_entity = on_stacked.entity();
    let stack = r!(stacked_query.get(card_entity));
    commands.entity(stack.0).add_child(card_entity);

    let [
        (stack_global_transform, _stack_transform),
        (card_global_transform, mut card_transform),
    ] = r!(transforms.get_many_mut([stack.0, card_entity]));

    // Get the vector pointing from the stack to the card
    let delta = card_global_transform.translation - stack_global_transform.translation;
    let delta = Val2::px(delta.x, delta.y);
    card_transform.translation = delta;
}

fn cards_stack() -> impl Bundle {
    (
        CardsStack { max: 3 },
        Node {
            width: Val::Px(WIDTH),
            height: Val::Px(HEIGHT),
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
