use std::fmt;
use std::str::FromStr;



#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Engine {
  pub speed: [f32; 2],
  pub speed_cost: [f32; 2],
  pub thrust: [f32; 2],
  pub thrust_cost: [f32; 2],
  pub flight_time_per_increment: [f32; 2],
  pub flight_time_cost: [f32; 2],
  pub max_thrust_angle: [f32; 2],
  pub turn_rate: [f32; 2],
  pub max_strafe_speed: [f32; 2],
  pub strafe_thrust: [f32; 2]
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EngineSettings {
  pub top_speed: f32,
  pub burn_duration: f32,
  pub maneuverability: f32
}

impl EngineSettings {
  pub const fn from_array(a: [f32; 3]) -> Self {
    EngineSettings { top_speed: a[0], burn_duration: a[1], maneuverability: a[2] }
  }

  pub const fn into_array(self) -> [f32; 3] {
    [self.top_speed, self.burn_duration, self.maneuverability]
  }

  pub fn normalize(self) -> Self {
    let a = self.top_speed.max(0.0);
    let b = self.burn_duration.max(0.0);
    let c = self.maneuverability.max(0.0);
    let sum = a + b + c;
    if sum <= 0.0 {
      EngineSettings {
        top_speed: 1.0 / 3.0,
        burn_duration: 1.0 / 3.0,
        maneuverability: 1.0 / 3.0
      }
    } else {
      EngineSettings {
        top_speed: a / sum,
        burn_duration: b / sum,
        maneuverability: c / sum
      }
    }
  }
}

pub mod list {
  use super::*;

}
