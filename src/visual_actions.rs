use bevy::prelude::*;
use tiny_bail::prelude::*;

pub fn plugin(app: &mut App) {
    app.register_type::<Move>().register_type::<MoveData>();

    app.add_systems(Update, (tick_move_timers, interpolate_movements).chain());
}

#[derive(Reflect, Debug)]
pub struct Move {
    pub end: Vec2,
    pub ease_fn: EaseFunction,
    pub duration: f32,
}

impl Move {
    pub fn new(end: Vec2, ease_fn: EaseFunction, duration: f32) -> Self {
        Self {
            end,
            ease_fn,
            duration,
        }
    }
}

impl EntityCommand for Move {
    fn apply(self, mut entity: EntityWorldMut) {
        let transform = r!(entity.get::<UiTransform>());

        let Val2 {
            x: Val::Px(x),
            y: Val::Px(y),
        } = transform.translation
        else {
            return;
        };

        entity.insert(MoveData {
            curve: EasingCurve::new(vec2(x, y), self.end, self.ease_fn),
            timer: Timer::from_seconds(self.duration, TimerMode::Once),
        });
    }
}

#[derive(Reflect, Component)]
pub struct MoveData {
    pub curve: EasingCurve<Vec2>,
    pub timer: Timer,
}

fn tick_move_timers(mut query: Query<&mut MoveData>, time: Res<Time>) {
    for mut data in &mut query {
        data.timer.tick(time.delta());
    }
}

fn interpolate_movements(
    mut query: Query<(Entity, &MoveData, &mut UiTransform)>,
    mut commands: Commands,
) {
    for (entity, data, mut transform) in &mut query {
        let pos_opt = data
            .curve
            .sample(data.timer.elapsed_secs() / data.timer.duration().as_secs_f32());
        let pos = c!(pos_opt);
        transform.translation = Val2::px(pos.x, pos.y);

        if data.timer.is_finished() {
            commands.entity(entity).remove::<MoveData>();
        }
    }
}
