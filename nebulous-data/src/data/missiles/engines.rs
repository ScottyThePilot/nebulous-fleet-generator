use crate::utils::lerp2;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};



/// Gravitational constant.
pub const G: f32 = 9.80665;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Engine {
  pub name: &'static str,
  /// Top speed of this engine, in m/s. (Top speed axis)
  pub speed: [f32; 2],
  /// The cost associated with the top speed axis.
  pub speed_cost: [f32; 2],
  /// Acceleration of this engine. in m/s^2. (Maneuverability axis)
  pub thrust: [f32; 2],
  /// The cost associated with the maneuverability axis.
  pub thrust_cost: [f32; 2],
  /// Burn duration, in seconds, per engine segment. (Burn duration axis)
  ///
  /// Multiply by the number of engine segments to get durn duration.
  pub burn_duration_per_segment: [f32; 2],
  /// The cost associated with the burn duration axis.
  pub burn_duration_cost: [f32; 2],
  /// Unknown, likely used to derive turn rate.
  pub thrust_angle: [f32; 2],
  /// Unknown, likely used to derive turn rate.
  pub turn_rate: [f32; 2],
  /// Top speed when strafing, in m/s. (Top speed axis)
  pub strafe_speed: [f32; 2],
  /// Acceleration when strafing, in m/s^2. (Maneuverability axis)
  pub strafe_thrust: [f32; 2]
}

impl Engine {
  pub fn setup_info(self, settings: EngineSettings, segments: usize) -> EngineSetupInfo {
    let settings = settings.normalize();
    let speed = lerp2(self.speed, settings.top_speed);
    let speed_cost = lerp2(self.speed_cost, settings.top_speed);
    let thrust = lerp2(self.thrust, settings.maneuverability);
    let thrust_cost = lerp2(self.thrust_cost, settings.maneuverability);
    let burn_duration = lerp2(self.burn_duration_per_segment, settings.burn_duration) * segments as f32;
    let burn_duration_cost = lerp2(self.burn_duration_cost, settings.burn_duration);
    let strafe_speed = lerp2(self.strafe_speed, settings.top_speed);
    let strafe_thrust = lerp2(self.strafe_thrust, settings.maneuverability);
    let cost = speed_cost + thrust_cost + burn_duration_cost;

    EngineSetupInfo { speed, thrust, burn_duration, strafe_speed, strafe_thrust, cost }
  }
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
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
      EngineSettings::default()
    } else {
      EngineSettings {
        top_speed: a / sum,
        burn_duration: b / sum,
        maneuverability: c / sum
      }
    }
  }
}

impl Default for EngineSettings {
  fn default() -> Self {
    EngineSettings {
      top_speed: 1.0 / 3.0,
      burn_duration: 1.0 / 3.0,
      maneuverability: 1.0 / 3.0
    }
  }
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct EngineSetupInfo {
  pub speed: f32,
  pub thrust: f32,
  pub burn_duration: f32,
  pub strafe_speed: f32,
  pub strafe_thrust: f32,
  pub cost: f32
}

impl EngineSetupInfo {
  /// The time it takes for this engine to accelerate to top speed.
  pub fn acceleration_time(self) -> f32 {
    self.speed / self.thrust
  }

  /// The distance this missile will travel while accelerating to top speed.
  pub fn acceleration_distance(self) -> f32 {
    (self.thrust / 2.0) * self.acceleration_time().powi(2)
  }

  /// The distance this missile will travel while at top speed.
  pub fn cruise_distance(self) -> f32 {
    (self.burn_duration - self.acceleration_time()) * self.speed
  }

  /// The maxiumum range of this engine.
  pub fn max_range(self) -> f32 {
    self.acceleration_distance() + self.cruise_distance()
  }
}

pub mod list {
  use super::*;

  pub const SGM1_ENGINE: Engine = Engine {
    name: "SGM-1 Engine",
    speed: [250.0, 400.0],
    speed_cost: [0.0, 0.0],
    thrust: [350.0, 750.0],
    thrust_cost: [0.0, 0.0],
    burn_duration_per_segment: [2.0, 8.0],
    burn_duration_cost: [0.0, 0.0],
    thrust_angle: [15.0, 40.0],
    turn_rate: [3.0, 7.0],
    strafe_speed: [40.0, 50.0],
    strafe_thrust: [5.0, 17.5]
  };

  pub const SGM2_ENGINE: Engine = Engine {
    name: "SGM-2 Engine",
    speed: [150.0, 350.0],
    speed_cost: [0.0, 0.0],
    thrust: [350.0, 750.0],
    thrust_cost: [0.0, 0.0],
    burn_duration_per_segment: [4.0, 20.0],
    burn_duration_cost: [0.0, 0.0],
    thrust_angle: [12.0, 30.0],
    turn_rate: [1.25, 3.0],
    strafe_speed: [30.0, 40.0],
    strafe_thrust: [20.0, 40.0]
  };

  pub const SGMH2_CRUISE_ENGINE: Engine = Engine {
    name: "SGM-H-2 Cruise Engine",
    speed: [150.0, 350.0],
    speed_cost: [0.0, 0.0],
    thrust: [200.0, 400.0],
    thrust_cost: [0.0, 0.0],
    burn_duration_per_segment: [40.0, 130.0],
    burn_duration_cost: [0.0, 0.0],
    thrust_angle: [8.0, 20.0],
    turn_rate: [0.75, 1.75],
    strafe_speed: [30.0, 40.0],
    strafe_thrust: [20.0, 40.0]
  };

  pub const SGMH2_SPRINT_ENGINE: Engine = Engine {
    name: "SGM-H-2 Sprint Engine",
    speed: [500.0, 1000.0],
    speed_cost: [2.0, 5.0],
    thrust: [2500.0, 5000.0],
    thrust_cost: [0.0, 0.0],
    burn_duration_per_segment: [1.0, 3.0],
    burn_duration_cost: [0.0, 0.0],
    thrust_angle: [25.0, 45.0],
    turn_rate: [2.0, 5.0],
    strafe_speed: [30.0, 40.0],
    strafe_thrust: [20.0, 40.0]
  };

  pub const SGMH3_CRUISE_ENGINE: Engine = Engine {
    name: "SGM-H-3 Cruise Engine",
    speed: [100.0, 200.0],
    speed_cost: [0.0, 0.0],
    thrust: [600.0, 900.0],
    thrust_cost: [0.0, 0.0],
    burn_duration_per_segment: [80.0, 250.0],
    burn_duration_cost: [0.0, 0.0],
    thrust_angle: [8.0, 30.0],
    turn_rate: [0.5, 1.25],
    strafe_speed: [20.0, 40.0],
    strafe_thrust: [20.0, 40.0]
  };

  pub const SGMH3_SPRINT_ENGINE: Engine = Engine {
    name: "SGM-H-3 Sprint Engine",
    speed: [550.0, 1000.0],
    speed_cost: [3.0, 10.0],
    thrust: [7500.0, 12500.0],
    thrust_cost: [0.0, 0.0],
    burn_duration_per_segment: [0.85, 2.75],
    burn_duration_cost: [0.0, 0.0],
    thrust_angle: [30.0, 60.0],
    turn_rate: [2.0, 5.0],
    strafe_speed: [30.0, 40.0],
    strafe_thrust: [20.0, 40.0]
  };

  pub const SGT3_ENGINE: Engine = Engine {
    name: "SGT-3 Engine",
    speed: [175.0, 300.0],
    speed_cost: [0.0, 0.0],
    thrust: [1500.0, 2500.0],
    thrust_cost: [0.0, 3.0],
    burn_duration_per_segment: [1.0, 4.5],
    burn_duration_cost: [0.0, 0.0],
    thrust_angle: [8.0, 30.0],
    turn_rate: [0.4, 1.0],
    strafe_speed: [20.0, 35.0],
    strafe_thrust: [50.0, 150.0]
  };

  pub const CM4_ENGINE: Engine = Engine {
    name: "CM-4 Engine",
    speed: [125.0, 275.0],
    speed_cost: [0.0, 0.0],
    thrust: [1250.0, 3750.0],
    thrust_cost: [0.0, 0.0],
    burn_duration_per_segment: [3.0, 40.0],
    burn_duration_cost: [0.0, 0.0],
    thrust_angle: [15.0, 30.0],
    turn_rate: [0.5, 1.5],
    strafe_speed: [20.0, 35.0],
    strafe_thrust: [100.0, 300.0]
  };
}
