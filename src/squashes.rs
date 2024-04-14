use bevy::prelude::*;

#[derive(Component)]
pub struct SquishEffect {
    attack_duration: f32,
    sustain_duration: f32,
    decay_duration: f32,
    elapsed: f32,
    total_time: f32,
    base_scale: Vec3,
    squish_multiplier: Vec3,
}

impl SquishEffect {
    pub fn new(base_scale: Vec3, squish_multiplier: Vec3, attack_duration: f32, sustain_duration: f32, decay_duration: f32) -> SquishEffect {
        SquishEffect { attack_duration, sustain_duration, decay_duration, elapsed: 0., total_time: attack_duration + sustain_duration + decay_duration, base_scale, squish_multiplier }
    }

    pub fn reset(&mut self) {
        self.elapsed = 0.0;
    }
}

pub fn squish_effects(mut squishees: Query<(&mut Transform, &mut SquishEffect)>, time: Res<Time>) {
    for (mut transform, mut squish) in squishees.iter_mut() {
        if squish.elapsed < squish.total_time {
            squish.elapsed += time.delta_seconds();
            let mut adj_elapsed = squish.elapsed;
            if adj_elapsed > squish.attack_duration {
                adj_elapsed -= squish.attack_duration;

                if adj_elapsed > squish.sustain_duration {
                    adj_elapsed -= squish.sustain_duration;

                    if adj_elapsed > squish.decay_duration {
                        // animation finished, reset scale
                        transform.scale = squish.base_scale;
                    }
                    else {
                        // in decay phase, do lerp out
                        let t = adj_elapsed / squish.decay_duration;
                        transform.scale = squish.base_scale.lerp(squish.squish_multiplier, 1.0 - t);
                    }
                }
                else {
                    // in sustain phase, hold
                    transform.scale = squish.base_scale * squish.squish_multiplier;
                }
            }
            else {
                // in attack phase, do lerp in
                let t = adj_elapsed / squish.attack_duration;
                transform.scale = squish.base_scale.lerp(squish.squish_multiplier, t);
            }
            
        }
    }
} 