use std::f64::consts::PI;

#[derive(Debug, Clone, Copy)]
pub(crate) struct Orientation {
    quat_nwu: [f64; 4],
}

impl Default for Orientation {
    fn default() -> Self {
        Self {
            quat_nwu: [1.0, 0.0, 0.0, 0.0],
        }
    }
}

impl Orientation {
    pub fn from_attitude_rad(attitude_rad: [f64; 3]) -> Self {
        let [roll, pitch, yaw] = attitude_rad;

        let half_roll = roll * 0.5;
        let half_pitch = pitch * 0.5;
        let half_yaw_nwu = -yaw * 0.5;

        let cr = half_roll.cos();
        let sr = half_roll.sin();

        let cp = half_pitch.cos();
        let sp = half_pitch.sin();

        let cy = half_yaw_nwu.cos();
        let sy = half_yaw_nwu.sin();

        let quat_nwu = normalize_quat([
            cy * cp * cr + sy * sp * sr,
            cy * cp * sr - sy * sp * cr,
            cy * sp * cr + sy * cp * sr,
            sy * cp * cr - cy * sp * sr,
        ]);

        Self { quat_nwu }
    }

    pub fn quat_nwu(self) -> [f64; 4] {
        self.quat_nwu
    }

    pub fn attitude_rad(self) -> [f64; 3] {
        let [w, x, y, z] = self.quat_nwu;

        let roll = (2.0 * (w * x + y * z)).atan2(1.0 - 2.0 * (x * x + y * y));

        let pitch_sin = (2.0 * (w * y - z * x)).clamp(-1.0, 1.0);

        let pitch = pitch_sin.asin();

        let yaw_nwu = (2.0 * (w * z + x * y)).atan2(1.0 - 2.0 * (y * y + z * z));

        [roll, pitch, wrap_angle(-yaw_nwu)]
    }

    pub fn integrate_body_rates(&mut self, rates_rpy_rad_s: [f64; 3], dt_seconds: f64) {
        let [roll_rate, pitch_rate, yaw_rate] = rates_rpy_rad_s;

        // QuadBench exposes Betaflight's conventional
        // roll / pitch / clockwise-yaw rates.
        //
        // The internal quaternion is body -> NWU,
        // where positive Z yaw is counter-clockwise,
        // hence the yaw sign inversion here.
        let omega_body = [0.0, roll_rate, pitch_rate, -yaw_rate];

        let derivative = quat_multiply(self.quat_nwu, omega_body).map(|value| value * 0.5);

        for (component, delta) in self.quat_nwu.iter_mut().zip(derivative) {
            *component += delta * dt_seconds;
        }

        self.quat_nwu = normalize_quat(self.quat_nwu);
    }

    pub fn thrust_direction_enu(self) -> [f64; 3] {
        let thrust_nwu = rotate_vector(self.quat_nwu, [0.0, 0.0, 1.0]);

        // NWU = North / West / Up
        // ENU = East / North / Up
        [-thrust_nwu[1], thrust_nwu[0], thrust_nwu[2]]
    }
}

fn quat_multiply(a: [f64; 4], b: [f64; 4]) -> [f64; 4] {
    let [aw, ax, ay, az] = a;

    let [bw, bx, by, bz] = b;

    [
        aw * bw - ax * bx - ay * by - az * bz,
        aw * bx + ax * bw + ay * bz - az * by,
        aw * by - ax * bz + ay * bw + az * bx,
        aw * bz + ax * by - ay * bx + az * bw,
    ]
}

fn normalize_quat(quat: [f64; 4]) -> [f64; 4] {
    let norm =
        (quat[0] * quat[0] + quat[1] * quat[1] + quat[2] * quat[2] + quat[3] * quat[3]).sqrt();

    if norm <= f64::EPSILON {
        return [1.0, 0.0, 0.0, 0.0];
    }

    quat.map(|value| value / norm)
}

fn rotate_vector(quat: [f64; 4], vector: [f64; 3]) -> [f64; 3] {
    let conjugate = [quat[0], -quat[1], -quat[2], -quat[3]];

    let vector_quat = [0.0, vector[0], vector[1], vector[2]];

    let rotated = quat_multiply(quat_multiply(quat, vector_quat), conjugate);

    [rotated[1], rotated[2], rotated[3]]
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
    fn attitude_round_trip_is_stable() {
        let orientation = Orientation::from_attitude_rad([
            20.0_f64.to_radians(),
            -15.0_f64.to_radians(),
            90.0_f64.to_radians(),
        ]);

        let attitude = orientation.attitude_rad();

        assert!((attitude[0].to_degrees() - 20.0).abs() < 0.001);

        assert!((attitude[1].to_degrees() + 15.0).abs() < 0.001);

        assert!((attitude[2].to_degrees() - 90.0).abs() < 0.001);
    }

    #[test]
    fn positive_roll_thrusts_east() {
        let orientation = Orientation::from_attitude_rad([20.0_f64.to_radians(), 0.0, 0.0]);

        assert!(orientation.thrust_direction_enu()[0] > 0.0);
    }

    #[test]
    fn positive_pitch_thrusts_north() {
        let orientation = Orientation::from_attitude_rad([0.0, 20.0_f64.to_radians(), 0.0]);

        assert!(orientation.thrust_direction_enu()[1] > 0.0);
    }

    #[test]
    fn quaternion_stays_normalized() {
        let mut orientation = Orientation::default();

        for _ in 0..10_000 {
            orientation.integrate_body_rates([4.0, -3.0, 2.0], 0.002);
        }

        let q = orientation.quat_nwu();

        let norm = (q[0] * q[0] + q[1] * q[1] + q[2] * q[2] + q[3] * q[3]).sqrt();

        assert!((norm - 1.0).abs() < 1e-10);
    }
}
