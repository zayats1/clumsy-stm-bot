use libm::cosf;
pub fn angle_to_speed(speed: f32, angle: f32) -> i16 {
    (speed * cosf(angle)) as i16
}
