pub const RC_CHANNEL_COUNT: usize = 16;
pub const MOTOR_COUNT: usize = 4;

pub const RC_PACKET_SIZE: usize = 40;
pub const FDM_PACKET_SIZE: usize = 144;
pub const MOTOR_PACKET_SIZE: usize = 16;

const GRAVITY_MPS2: f64 = 9.80665;

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
        Self {
            timestamp,
            angular_velocity_rpy: [0.0; 3],
            linear_acceleration_xyz: [0.0, 0.0, -GRAVITY_MPS2],
            orientation_quat: [
                std::f64::consts::FRAC_1_SQRT_2,
                0.0,
                0.0,
                -std::f64::consts::FRAC_1_SQRT_2,
            ],
            velocity_xyz: [0.0; 3],
            position_xyz: [19.0, 47.0, 120.0],
            pressure_pa: 101_325.0,
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
