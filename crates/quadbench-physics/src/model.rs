use std::f64::consts::{FRAC_1_SQRT_2, PI};

const GRAVITY_MPS2: f64 = 9.80665;

const MAX_TILT_RAD: f64 = 75.0_f64.to_radians();

const ANGULAR_DAMPING_PER_SECOND: f64 = 1.6;

const RATE_SLEEP_THRESHOLD_RAD_S: f64 = 0.0005;

const MAX_ANGULAR_RATE_RAD_S: f64 = 2_000.0_f64.to_radians();

#[derive(Debug, Clone, Copy)]
pub struct QuadParameters {
    pub mass_kg: f64,
    pub motor_radius_m: f64,
    pub inertia_kg_m2: [f64; 3],
    pub max_motor_thrust_n: f64,
    pub yaw_torque_per_thrust_m: f64,
}

impl Default for QuadParameters {
    fn default() -> Self {
        Self {
            mass_kg: 0.70,
            motor_radius_m: 0.110,
            inertia_kg_m2: [0.0050, 0.0050, 0.0090],
            max_motor_thrust_n: 22.0,
            yaw_torque_per_thrust_m: 0.015,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PhysicsSnapshot {
    pub parameters: QuadParameters,
    pub attitude_rad: [f64; 3],
    pub angular_velocity_rad_s: [f64; 3],
    pub angular_acceleration_rad_s2: [f64; 3],
    pub linear_acceleration_enu_mps2: [f64; 3],
    pub velocity_enu_mps: [f64; 3],
    pub position_enu_m: [f64; 3],
    pub motor_commands: [f32; 4],
    pub motor_mix: [f64; 3],
    pub motor_thrust_n: [f64; 4],
    pub total_thrust_n: f64,
    pub vertical_thrust_n: f64,
    pub body_torque_nm: [f64; 3],
    pub on_ground: bool,
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

    pub fn weight_n(self) -> f64 {
        self.parameters.mass_kg * GRAVITY_MPS2
    }

    pub fn thrust_to_weight(self) -> f64 {
        let weight = self.weight_n();

        if weight <= 0.0 {
            0.0
        } else {
            self.total_thrust_n / weight
        }
    }

    pub fn hover_motor_command(self) -> f64 {
        let maximum_total_thrust = self.parameters.max_motor_thrust_n * 4.0;

        if maximum_total_thrust <= 0.0 {
            return 0.0;
        }

        (self.weight_n() / maximum_total_thrust)
            .sqrt()
            .clamp(0.0, 1.0)
    }
}

#[derive(Debug, Clone)]
pub struct PhysicsModel {
    parameters: QuadParameters,
    attitude_rad: [f64; 3],
    angular_velocity_rad_s: [f64; 3],
    angular_acceleration_rad_s2: [f64; 3],
    linear_acceleration_enu_mps2: [f64; 3],
    velocity_enu_mps: [f64; 3],
    position_enu_m: [f64; 3],
    motor_commands: [f32; 4],
    motor_mix: [f64; 3],
    motor_thrust_n: [f64; 4],
    total_thrust_n: f64,
    vertical_thrust_n: f64,
    body_torque_nm: [f64; 3],
    on_ground: bool,
}

impl Default for PhysicsModel {
    fn default() -> Self {
        Self {
            parameters: QuadParameters::default(),
            attitude_rad: [0.0; 3],
            angular_velocity_rad_s: [0.0; 3],
            angular_acceleration_rad_s2: [0.0; 3],
            linear_acceleration_enu_mps2: [0.0; 3],
            velocity_enu_mps: [0.0; 3],
            position_enu_m: [0.0; 3],
            motor_commands: [0.0; 4],
            motor_mix: [0.0; 3],
            motor_thrust_n: [0.0; 4],
            total_thrust_n: 0.0,
            vertical_thrust_n: 0.0,
            body_torque_nm: [0.0; 3],
            on_ground: true,
        }
    }
}

impl PhysicsModel {
    pub fn snapshot(&self) -> PhysicsSnapshot {
        PhysicsSnapshot {
            parameters: self.parameters,
            attitude_rad: self.attitude_rad,
            angular_velocity_rad_s: self.angular_velocity_rad_s,
            angular_acceleration_rad_s2: self.angular_acceleration_rad_s2,
            linear_acceleration_enu_mps2: self.linear_acceleration_enu_mps2,
            velocity_enu_mps: self.velocity_enu_mps,
            position_enu_m: self.position_enu_m,
            motor_commands: self.motor_commands,
            motor_mix: self.motor_mix,
            motor_thrust_n: self.motor_thrust_n,
            total_thrust_n: self.total_thrust_n,
            vertical_thrust_n: self.vertical_thrust_n,
            body_torque_nm: self.body_torque_nm,
            on_ground: self.on_ground,
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

        self.update_linear_dynamics(dt_seconds);
    }

    pub fn set_motor_commands(&mut self, motor_commands: [f32; 4]) {
        self.motor_commands = motor_commands.map(|command| command.clamp(0.0, 1.0));
    }

    fn update_motor_dynamics(&mut self) {
        let thrust_fraction = self.motor_commands.map(|command| {
            let command = f64::from(command);

            command * command
        });

        self.motor_thrust_n =
            thrust_fraction.map(|fraction| fraction * self.parameters.max_motor_thrust_n);

        self.total_thrust_n = self.motor_thrust_n.iter().sum();

        let roll_mix =
            -thrust_fraction[0] - thrust_fraction[1] + thrust_fraction[2] + thrust_fraction[3];

        let pitch_mix =
            thrust_fraction[0] - thrust_fraction[1] + thrust_fraction[2] - thrust_fraction[3];

        let yaw_mix =
            -thrust_fraction[0] + thrust_fraction[1] + thrust_fraction[2] - thrust_fraction[3];

        self.motor_mix = [roll_mix, pitch_mix, yaw_mix];

        let arm_component_m = self.parameters.motor_radius_m * FRAC_1_SQRT_2;

        let roll_torque_nm = (-self.motor_thrust_n[0] - self.motor_thrust_n[1]
            + self.motor_thrust_n[2]
            + self.motor_thrust_n[3])
            * arm_component_m;

        let pitch_torque_nm = (self.motor_thrust_n[0] - self.motor_thrust_n[1]
            + self.motor_thrust_n[2]
            - self.motor_thrust_n[3])
            * arm_component_m;

        let yaw_torque_nm =
            (-self.motor_thrust_n[0] + self.motor_thrust_n[1] + self.motor_thrust_n[2]
                - self.motor_thrust_n[3])
                * self.parameters.yaw_torque_per_thrust_m;

        self.body_torque_nm = [roll_torque_nm, pitch_torque_nm, yaw_torque_nm];

        for axis in 0..3 {
            let inertia = self.parameters.inertia_kg_m2[axis].max(f64::EPSILON);

            self.angular_acceleration_rad_s2[axis] = self.body_torque_nm[axis] / inertia;
        }
    }

    fn update_linear_dynamics(&mut self, dt_seconds: f64) {
        let vertical_factor = self.attitude_rad[0].cos() * self.attitude_rad[1].cos();

        self.vertical_thrust_n = self.total_thrust_n * vertical_factor.max(0.0);

        let mass_kg = self.parameters.mass_kg.max(f64::EPSILON);

        let free_vertical_accel = self.vertical_thrust_n / mass_kg - GRAVITY_MPS2;

        let resting_on_ground = self.position_enu_m[2] <= 0.0
            && self.velocity_enu_mps[2] <= 0.0
            && free_vertical_accel <= 0.0;

        if resting_on_ground {
            self.position_enu_m[2] = 0.0;

            self.velocity_enu_mps[2] = 0.0;

            self.linear_acceleration_enu_mps2 = [0.0; 3];

            self.on_ground = true;

            return;
        }

        self.linear_acceleration_enu_mps2 = [0.0, 0.0, free_vertical_accel];

        self.velocity_enu_mps[2] += free_vertical_accel * dt_seconds;

        self.position_enu_m[2] += self.velocity_enu_mps[2] * dt_seconds;

        if self.position_enu_m[2] <= 0.0 {
            self.position_enu_m[2] = 0.0;

            if self.velocity_enu_mps[2] < 0.0 {
                self.velocity_enu_mps[2] = 0.0;
            }

            self.linear_acceleration_enu_mps2[2] = 0.0;

            self.on_ground = true;
        } else {
            self.on_ground = false;
        }
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

        let snapshot = model.snapshot();

        assert!(snapshot.motor_mix[0] < 0.0);

        assert!(snapshot.motor_mix[1] > 0.0);

        assert!(snapshot.motor_mix[2] < 0.0);

        assert!(snapshot.body_torque_nm[0] < 0.0);

        assert!(snapshot.body_torque_nm[1] > 0.0);

        assert!(snapshot.body_torque_nm[2] < 0.0);
    }

    #[test]
    fn zero_thrust_stays_on_ground() {
        let mut model = PhysicsModel::default();

        model.step(0.05);

        let snapshot = model.snapshot();

        assert_eq!(snapshot.position_enu_m[2], 0.0,);

        assert_eq!(snapshot.velocity_enu_mps[2], 0.0,);

        assert!(snapshot.on_ground);
    }

    #[test]
    fn hover_command_balances_weight() {
        let mut model = PhysicsModel::default();

        let hover = model.snapshot().hover_motor_command() as f32;

        model.set_motor_commands([hover; 4]);

        model.step(0.01);

        let snapshot = model.snapshot();

        assert!(snapshot.linear_acceleration_enu_mps2[2].abs() < 0.001);
    }

    #[test]
    fn above_hover_lifts_off() {
        let mut model = PhysicsModel::default();

        let hover = model.snapshot().hover_motor_command() as f32;

        model.set_motor_commands([hover * 1.20; 4]);

        model.step(0.05);

        let snapshot = model.snapshot();

        assert!(snapshot.position_enu_m[2] > 0.0);

        assert!(snapshot.velocity_enu_mps[2] > 0.0);

        assert!(!snapshot.on_ground);
    }
}
