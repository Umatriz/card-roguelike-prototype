use bevy::prelude::*;
use tiny_bail::prelude::*;
// use haalka::prelude::*;

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, spawn_card);
}

fn spawn_card(mut commands: Commands) {
    commands
        .spawn(card())
        .observe(card_dragging_start)
        .observe(card_dragging)
        .observe(card_dragging_end);

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(30.0),
                bottom: Val::Px(0.0),
                position_type: PositionType::Absolute,
                align_content: AlignContent::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            Pickable::IGNORE,
        ))
        .with_children(|parent| {
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
    mut query: Query<(&mut UiTransform, &mut Pickable, &mut GlobalZIndex)>,
) {
    let (mut transform, mut pickable, mut z_index) = r!(query.get_mut(on_drag_end.entity()));
    transform.translation = Val2::ZERO;
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

fn stacked_children_handling(on_stacked: On<Insert, StackedIn>, mut commands: Commands) {}

fn cards_stack() -> impl Bundle {
    (
        CardsStack { max: 5 },
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
