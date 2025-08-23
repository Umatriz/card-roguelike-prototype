use bevy::prelude::*;
// use haalka::prelude::*;

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, spawn_card);
}

fn spawn_card(mut commands: Commands) {
    commands.spawn(card()).observe(card_dragging);
    commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(30.0),
            bottom: Val::Px(0.0),
            position_type: PositionType::Absolute,
            align_content: AlignContent::Center,
            justify_content: JustifyContent::Center,
            ..default()
        })
        .with_children(|parent| {
            parent.spawn(card_spot()).observe(card_spot_capture_card);
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
        Card,
    )
}

fn card_dragging(on_drag: On<Pointer<Drag>>, mut transforms: Query<&mut UiTransform, With<Card>>) {
    let Ok(mut transform) = transforms.get_mut(on_drag.entity()) else {
        return;
    };

    transform.translation = Val2::px(on_drag.distance.x, on_drag.distance.y);
}

#[derive(Component, Debug)]
pub struct CardSpot;

fn card_spot() -> impl Bundle {
    (
        Node {
            width: Val::Px(WIDTH),
            height: Val::Px(HEIGHT),
            ..default()
        },
        BackgroundColor(colors::BLACK),
        Outline::new(Val::Percent(3.0), Val::ZERO, colors::AKAROA),
    )
}

pub fn card_spot_capture_card(on_drag_drop: On<Pointer<DragDrop>>, mut nodes: Query<&mut Node>) {
    let Ok([mut dropped_node, spot_node]) =
        nodes.get_many_mut([on_drag_drop.dropped, on_drag_drop.entity()])
    else {
        return;
    };

    *dropped_node = spot_node.clone();
}
