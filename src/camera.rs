use nalgebra_glm::Vec3;
use std::f32::consts::PI;

pub struct Camera {
  pub eye: Vec3,
  pub center: Vec3,
  pub up: Vec3,
  pitch: f32,
  yaw: f32,
}

impl Camera {
  pub fn new(eye: Vec3, center: Vec3, up: Vec3) -> Self {
    Camera {
      eye,
      center,
      up,
      pitch: 0.0,
      yaw: 0.0,
    }
  }

  pub fn get_forward(&self) -> Vec3 {
    (self.center - self.eye).normalize()
  }

  pub fn get_right(&self) -> Vec3 {
    self.get_forward().cross(&self.up).normalize()
  }

  pub fn move_forward(&mut self, amount: f32) {
    let forward = self.get_forward();
    self.eye += forward * amount;
    self.center += forward * amount;
  }

  pub fn move_right(&mut self, amount: f32) {
    let right = self.get_right();
    self.eye += right * amount;
    self.center += right * amount;
  }

  pub fn move_up(&mut self, amount: f32) {
    self.eye += self.up * amount;
    self.center += self.up * amount;
  }

  pub fn rotate_pitch(&mut self, angle: f32) {
    self.pitch += angle;
    self.pitch = self.pitch.clamp(-PI/2.0 + 0.1, PI/2.0 - 0.1);
    self.update_center();
  }

  fn update_center(&mut self) {
    let forward = Vec3::new(
      self.yaw.cos() * self.pitch.cos(),
      self.pitch.sin(),
      self.yaw.sin() * self.pitch.cos()
    ).normalize();
    
    self.center = self.eye + forward;
  }

  pub fn zoom(&mut self, amount: f32) {
    let forward = self.get_forward();
    self.eye += forward * amount;
  }
}
