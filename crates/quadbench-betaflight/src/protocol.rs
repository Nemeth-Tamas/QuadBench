pub const RC_CHANNEL_COUNT: usize = 16;
pub const MOTOR_COUNT: usize = 4;

pub const RC_PACKET_SIZE: usize = 40;
pub const FDM_PACKET_SIZE: usize = 144;
pub const MOTOR_PACKET_SIZE: usize = 16;

const GRAVITY_MPS2: f64 = 9.80665;

const HOME_LAT_DEG: f64 = 47.0;
const HOME_LON_DEG: f64 = 19.0;
const HOME_ALT_M: f64 = 120.0;

const METRES_PER_DEGREE: f64 = 111_319.49;

const RZ_NEG_90: [f64; 4] = [
    std::f64::consts::FRAC_1_SQRT_2,
    0.0,
    0.0,
    -std::f64::consts::FRAC_1_SQRT_2,
];

#[derive(Debug, Clone, Copy)]
pub struct FdmState {
    pub angular_velocity_rpy: [f64; 3],
    pub linear_acceleration_xyz: [f64; 3],
    pub orientation_quat: [f64; 4],
    pub velocity_xyz: [f64; 3],
    pub position_xyz: [f64; 3],
    pub pressure_pa: f64,
}

impl Default for FdmState {
    fn default() -> Self {
        Self::from_body_kinematics([0.0; 3], [0.0; 3], [0.0; 3], [0.0; 3], [0.0; 3])
    }
}

impl FdmState {
    pub fn from_body_kinematics(
        attitude_rpy_rad: [f64; 3],
        angular_velocity_rpy_rad_s: [f64; 3],
        linear_acceleration_enu_mps2: [f64; 3],
        velocity_enu_mps: [f64; 3],
        position_enu_m: [f64; 3],
    ) -> Self {
        let [roll, pitch, yaw] = attitude_rpy_rad;

        let q_nwu = quat_from_euler_betaflight(roll, pitch, yaw);

        let orientation_quat = quat_conjugate_x_180(quat_multiply(RZ_NEG_90, q_nwu));

        let specific_force_world_nwu = [
            linear_acceleration_enu_mps2[1],
            -linear_acceleration_enu_mps2[0],
            linear_acceleration_enu_mps2[2] + GRAVITY_MPS2,
        ];

        let specific_force_body = quat_rotate_inverse(q_nwu, specific_force_world_nwu);

        let latitude_scale = METRES_PER_DEGREE;

        let longitude_scale = METRES_PER_DEGREE * HOME_LAT_DEG.to_radians().cos();

        let true_lon = HOME_LON_DEG + position_enu_m[0] / longitude_scale;

        let true_lat = HOME_LAT_DEG + position_enu_m[1] / latitude_scale;

        let packet_lon = 2.0 * HOME_LON_DEG - true_lon;

        let packet_lat = 2.0 * HOME_LAT_DEG - true_lat;

        Self {
            angular_velocity_rpy: [
                angular_velocity_rpy_rad_s[0],
                -angular_velocity_rpy_rad_s[1],
                -angular_velocity_rpy_rad_s[2],
            ],
            linear_acceleration_xyz: [
                -specific_force_body[0],
                -specific_force_body[1],
                -specific_force_body[2],
            ],
            orientation_quat,
            velocity_xyz: velocity_enu_mps,
            position_xyz: [packet_lon, packet_lat, HOME_ALT_M + position_enu_m[2]],
            pressure_pa: 101_325.0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct FdmPacket {
    pub timestamp: f64,
    pub angular_velocity_rpy: [f64; 3],
    pub linear_acceleration_xyz: [f64; 3],
    pub orientation_quat: [f64; 4],
    pub velocity_xyz: [f64; 3],
    pub position_xyz: [f64; 3],
    pub pressure_pa: f64,
}

impl FdmPacket {
    pub fn stationary(timestamp: f64) -> Self {
        Self::from_state(timestamp, FdmState::default())
    }

    pub fn from_state(timestamp: f64, state: FdmState) -> Self {
        Self {
            timestamp,
            angular_velocity_rpy: state.angular_velocity_rpy,
            linear_acceleration_xyz: state.linear_acceleration_xyz,
            orientation_quat: state.orientation_quat,
            velocity_xyz: state.velocity_xyz,
            position_xyz: state.position_xyz,
            pressure_pa: state.pressure_pa,
        }
    }

    pub fn encode(self) -> [u8; FDM_PACKET_SIZE] {
        let mut output = [0_u8; FDM_PACKET_SIZE];
        let mut offset = 0;

        write_f64(&mut output, &mut offset, self.timestamp);

        for value in self.angular_velocity_rpy {
            write_f64(&mut output, &mut offset, value);
        }

        for value in self.linear_acceleration_xyz {
            write_f64(&mut output, &mut offset, value);
        }

        for value in self.orientation_quat {
            write_f64(&mut output, &mut offset, value);
        }

        for value in self.velocity_xyz {
            write_f64(&mut output, &mut offset, value);
        }

        for value in self.position_xyz {
            write_f64(&mut output, &mut offset, value);
        }

        write_f64(&mut output, &mut offset, self.pressure_pa);

        output
    }
}

pub fn encode_rc_packet(
    timestamp: f64,
    channels: &[u16; RC_CHANNEL_COUNT],
) -> [u8; RC_PACKET_SIZE] {
    let mut output = [0_u8; RC_PACKET_SIZE];

    output[0..8].copy_from_slice(&timestamp.to_le_bytes());

    for (index, value) in channels.iter().enumerate() {
        let offset = 8 + index * 2;

        output[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
    }

    output
}

pub fn decode_motor_packet(input: &[u8]) -> Option<[f32; MOTOR_COUNT]> {
    if input.len() < MOTOR_PACKET_SIZE {
        return None;
    }

    let mut motors = [0.0_f32; MOTOR_COUNT];

    for (index, motor) in motors.iter_mut().enumerate() {
        let offset = index * 4;

        *motor = f32::from_le_bytes([
            input[offset],
            input[offset + 1],
            input[offset + 2],
            input[offset + 3],
        ]);
    }

    Some(motors)
}

fn quat_from_euler_betaflight(roll: f64, pitch: f64, yaw: f64) -> [f64; 4] {
    let cr = (roll / 2.0).cos();
    let sr = (roll / 2.0).sin();

    let cp = (pitch / 2.0).cos();
    let sp = (pitch / 2.0).sin();

    let cy = (-yaw / 2.0).cos();
    let sy = (-yaw / 2.0).sin();

    [
        cy * cp * cr + sy * sp * sr,
        cy * cp * sr - sy * sp * cr,
        cy * sp * cr + sy * cp * sr,
        sy * cp * cr - cy * sp * sr,
    ]
}

fn quat_conjugate_x_180(quat: [f64; 4]) -> [f64; 4] {
    [quat[0], quat[1], -quat[2], -quat[3]]
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

fn quat_rotate_inverse(quat: [f64; 4], vector: [f64; 3]) -> [f64; 3] {
    let conjugate = [quat[0], -quat[1], -quat[2], -quat[3]];

    let vector_quat = [0.0, vector[0], vector[1], vector[2]];

    let rotated = quat_multiply(quat_multiply(conjugate, vector_quat), quat);

    [rotated[1], rotated[2], rotated[3]]
}

fn write_f64(output: &mut [u8], offset: &mut usize, value: f64) {
    let bytes = value.to_le_bytes();

    output[*offset..*offset + 8].copy_from_slice(&bytes);

    *offset += 8;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rc_packet_has_expected_layout() {
        let channels = std::array::from_fn(|index| 1_000 + index as u16);

        let packet = encode_rc_packet(12.5, &channels);

        assert_eq!(packet.len(), RC_PACKET_SIZE,);

        assert_eq!(f64::from_le_bytes(packet[0..8].try_into().unwrap(),), 12.5,);

        assert_eq!(u16::from_le_bytes([packet[8], packet[9],]), 1_000,);

        assert_eq!(u16::from_le_bytes([packet[38], packet[39],]), 1_015,);
    }

    #[test]
    fn stationary_fdm_is_exactly_144_bytes() {
        let packet = FdmPacket::stationary(1.0).encode();

        assert_eq!(packet.len(), FDM_PACKET_SIZE,);
    }

    #[test]
    fn motor_packet_decodes_four_floats() {
        let expected = [0.1_f32, 0.25, 0.5, 1.0];

        let mut packet = [0_u8; MOTOR_PACKET_SIZE];

        for (index, value) in expected.iter().enumerate() {
            let offset = index * 4;

            packet[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
        }

        assert_eq!(decode_motor_packet(&packet), Some(expected),);
    }
}
