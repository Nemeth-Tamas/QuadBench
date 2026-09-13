use std::f64::consts::PI;

const MAX_TILT_RAD: f64 = 75.0_f64.to_radians();

const ANGULAR_DAMPING_PER_SECOND: f64 = 1.6;
const RATE_SLEEP_THRESHOLD_RAD_S: f64 = 0.0005;

const ROLL_PITCH_ACCEL_RAD_S2: f64 = 35.0;
const YAW_ACCEL_RAD_S2: f64 = 12.0;

const MAX_ANGULAR_RATE_RAD_S: f64 = 2_000.0_f64.to_radians();

#[derive(Debug, Clone, Copy)]
pub struct PhysicsSnapshot {
    pub attitude_rad: [f64; 3],
    pub angular_velocity_rad_s: [f64; 3],
    pub angular_acceleration_rad_s2: [f64; 3],
    pub linear_acceleration_enu_mps2: [f64; 3],
    pub velocity_enu_mps: [f64; 3],
    pub position_enu_m: [f64; 3],
    pub motor_commands: [f32; 4],
    pub motor_mix: [f64; 3],
}

impl PhysicsSnapshot {
    pub fn attitude_deg(self) -> [f64; 3] {
        self.attitude_rad.map(f64::to_degrees)
    }

    pub fn angular_velocity_deg_s(self) -> [f64; 3] {
        self.angular_velocity_rad_s.map(f64::to_degrees)
    }

    pub fn angular_acceleration_deg_s2(self) -> [f64; 3] {
        self.angular_acceleration_rad_s2.map(f64::to_degrees)
    }
}

#[derive(Debug, Clone)]
pub struct PhysicsModel {
    attitude_rad: [f64; 3],
    angular_velocity_rad_s: [f64; 3],
    angular_acceleration_rad_s2: [f64; 3],
    linear_acceleration_enu_mps2: [f64; 3],
    velocity_enu_mps: [f64; 3],
    position_enu_m: [f64; 3],
    motor_commands: [f32; 4],
    motor_mix: [f64; 3],
}

impl Default for PhysicsModel {
    fn default() -> Self {
        Self {
            attitude_rad: [0.0; 3],
            angular_velocity_rad_s: [0.0; 3],
            angular_acceleration_rad_s2: [0.0; 3],
            linear_acceleration_enu_mps2: [0.0; 3],
            velocity_enu_mps: [0.0; 3],
            position_enu_m: [0.0; 3],
            motor_commands: [0.0; 4],
            motor_mix: [0.0; 3],
        }
    }
}

impl PhysicsModel {
    pub fn snapshot(&self) -> PhysicsSnapshot {
        PhysicsSnapshot {
            attitude_rad: self.attitude_rad,
            angular_velocity_rad_s: self.angular_velocity_rad_s,
            angular_acceleration_rad_s2: self.angular_acceleration_rad_s2,
            linear_acceleration_enu_mps2: self.linear_acceleration_enu_mps2,
            velocity_enu_mps: self.velocity_enu_mps,
            position_enu_m: self.position_enu_m,
            motor_commands: self.motor_commands,
            motor_mix: self.motor_mix,
        }
    }

    pub fn step(&mut self, dt_seconds: f64) {
        let dt_seconds = dt_seconds.clamp(0.0, 0.05);

        if dt_seconds <= 0.0 {
            return;
        }

        self.update_motor_dynamics();

        let damping = (-ANGULAR_DAMPING_PER_SECOND * dt_seconds).exp();

        for axis in 0..3 {
            self.angular_velocity_rad_s[axis] +=
                self.angular_acceleration_rad_s2[axis] * dt_seconds;

            self.angular_velocity_rad_s[axis] *= damping;

            self.angular_velocity_rad_s[axis] = self.angular_velocity_rad_s[axis]
                .clamp(-MAX_ANGULAR_RATE_RAD_S, MAX_ANGULAR_RATE_RAD_S);

            if self.angular_velocity_rad_s[axis].abs() < RATE_SLEEP_THRESHOLD_RAD_S
                && self.angular_acceleration_rad_s2[axis].abs() < RATE_SLEEP_THRESHOLD_RAD_S
            {
                self.angular_velocity_rad_s[axis] = 0.0;
            }

            self.attitude_rad[axis] += self.angular_velocity_rad_s[axis] * dt_seconds;
        }

        self.attitude_rad[0] = self.attitude_rad[0].clamp(-MAX_TILT_RAD, MAX_TILT_RAD);

        self.attitude_rad[1] = self.attitude_rad[1].clamp(-MAX_TILT_RAD, MAX_TILT_RAD);

        self.attitude_rad[2] = wrap_angle(self.attitude_rad[2]);
    }

    pub fn set_motor_commands(&mut self, motor_commands: [f32; 4]) {
        self.motor_commands = motor_commands.map(|command| command.clamp(0.0, 1.0));
    }

    fn update_motor_dynamics(&mut self) {
        let thrust = self.motor_commands.map(|command| {
            let command = f64::from(command);

            command * command
        });

        let roll_mix = -thrust[0] - thrust[1] + thrust[2] + thrust[3];

        let pitch_mix = thrust[0] - thrust[1] + thrust[2] - thrust[3];

        let yaw_mix = -thrust[0] + thrust[1] + thrust[2] - thrust[3];

        self.motor_mix = [roll_mix, pitch_mix, yaw_mix];

        self.angular_acceleration_rad_s2 = [
            roll_mix * ROLL_PITCH_ACCEL_RAD_S2,
            pitch_mix * ROLL_PITCH_ACCEL_RAD_S2,
            yaw_mix * YAW_ACCEL_RAD_S2,
        ];
    }

    pub fn set_attitude_deg(&mut self, roll_deg: f64, pitch_deg: f64, yaw_deg: f64) {
        self.attitude_rad = [
            roll_deg.to_radians().clamp(-MAX_TILT_RAD, MAX_TILT_RAD),
            pitch_deg.to_radians().clamp(-MAX_TILT_RAD, MAX_TILT_RAD),
            wrap_angle(yaw_deg.to_radians()),
        ];
    }

    pub fn add_angular_velocity_deg_s(&mut self, delta_deg_s: [f64; 3]) {
        for (rate, delta) in self.angular_velocity_rad_s.iter_mut().zip(delta_deg_s) {
            *rate += delta.to_radians();
        }
    }

    pub fn reset_attitude(&mut self) {
        self.attitude_rad = [0.0; 3];
        self.angular_velocity_rad_s = [0.0; 3];
    }

    pub fn reset_all(&mut self) {
        *self = Self::default();
    }
}

fn wrap_angle(mut angle: f64) -> f64 {
    while angle > PI {
        angle -= 2.0 * PI;
    }

    while angle < -PI {
        angle += 2.0 * PI;
    }

    angle
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_model_is_level() {
        let snapshot = PhysicsModel::default().snapshot();

        assert_eq!(snapshot.attitude_rad, [0.0; 3],);

        assert_eq!(snapshot.angular_velocity_rad_s, [0.0; 3],);
    }

    #[test]
    fn attitude_can_be_set_in_degrees() {
        let mut model = PhysicsModel::default();

        model.set_attitude_deg(20.0, -15.0, 90.0);

        let attitude = model.snapshot().attitude_deg();

        assert!((attitude[0] - 20.0).abs() < 0.001);

        assert!((attitude[1] + 15.0).abs() < 0.001);

        assert!((attitude[2] - 90.0).abs() < 0.001);
    }

    #[test]
    fn angular_impulse_changes_attitude() {
        let mut model = PhysicsModel::default();

        model.add_angular_velocity_deg_s([90.0, 0.0, 0.0]);

        model.step(0.1);

        assert!(model.snapshot().attitude_rad[0] > 0.0);
    }

    #[test]
    fn equal_motor_output_has_no_torque() {
        let mut model = PhysicsModel::default();

        model.set_motor_commands([0.5; 4]);

        model.step(0.01);

        let snapshot = model.snapshot();

        assert_eq!(snapshot.motor_mix, [0.0; 3],);
    }

    #[test]
    fn quad_x_motor_one_has_expected_mix() {
        let mut model = PhysicsModel::default();

        model.set_motor_commands([1.0, 0.0, 0.0, 0.0]);

        model.step(0.01);

        let mix = model.snapshot().motor_mix;

        assert!(mix[0] < 0.0);
        assert!(mix[1] > 0.0);
        assert!(mix[2] < 0.0);
    }
}
