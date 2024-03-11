use crate::utils::lerp2;



/// Gravitational constant
pub const G: f32 = 9.80665;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Engine {
  pub name: &'static str,
  /// Top speed of this engine. (Top speed axis)
  ///
  /// Multiply by 10 to get m/s.
  pub speed: [f32; 2],
  pub speed_cost: [f32; 2],
  /// Acceleration of this engine. (Maneuverability axis)
  ///
  /// Multiply by 50 to get m/s^2.
  pub thrust: [f32; 2],
  pub thrust_cost: [f32; 2],
  /// Burn duration, in seconds, per engine segment. (Burn duration axis)
  ///
  /// Multiply by the number of engine segments to get durn duration.
  pub burn_duration_per_segment: [f32; 2],
  pub burn_duration_cost: [f32; 2],
  pub thrust_angle: [f32; 2],
  pub turn_rate: [f32; 2],
  /// Top speed when strafing. (Top speed axis)
  ///
  /// Multiply by 10 to get m/s.
  pub strafe_speed: [f32; 2],
  /// Acceleration when strafing. (Maneuverability axis)
  ///
  /// Multiply by 50 to get m/s^2.
  pub strafe_thrust: [f32; 2]
}

impl Engine {
  pub fn setup_info(self, settings: EngineSettings, segments: usize) -> EngineSetupInfo {
    let settings = settings.normalize();
    let speed = lerp2(self.speed, settings.top_speed) * 10.0;
    let speed_cost = lerp2(self.speed_cost, settings.top_speed);
    let thrust = lerp2(self.thrust, settings.maneuverability) * 50.0;
    let thrust_cost = lerp2(self.thrust_cost, settings.maneuverability);
    let burn_duration = lerp2(self.burn_duration_per_segment, settings.burn_duration) * segments as f32;
    let burn_duration_cost = lerp2(self.burn_duration_cost, settings.burn_duration);
    let strafe_speed = lerp2(self.strafe_speed, settings.top_speed) * 10.0;
    let strafe_thrust = lerp2(self.strafe_thrust, settings.maneuverability) * 50.0;
    let cost = speed_cost + thrust_cost + burn_duration_cost;

    EngineSetupInfo { speed, thrust, burn_duration, strafe_speed, strafe_thrust, cost }
  }
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
pub struct EngineSetupInfo {
  pub speed: f32,
  pub thrust: f32,
  pub burn_duration: f32,
  pub strafe_speed: f32,
  pub strafe_thrust: f32,
  pub cost: f32
}

pub mod list {
  use super::*;

  pub const SGM1_ENGINE: Engine = Engine {
    name: "SGM-1 Engine",
    speed: [25.0, 40.0],
    speed_cost: [0.0, 0.0],
    thrust: [7.0, 15.0],
    thrust_cost: [0.0, 0.0],
    burn_duration_per_segment: [2.0, 8.0],
    burn_duration_cost: [0.0, 0.0],
    thrust_angle: [15.0, 40.0],
    turn_rate: [3.0, 7.0],
    strafe_speed: [4.0, 5.0],
    strafe_thrust: [0.1, 0.35],
  };

  pub const SGM2_ENGINE: Engine = Engine {
    name: "SGM-2 Engine",
    speed: [15.0, 35.0],
    speed_cost: [0.0, 0.0],
    thrust: [7.0, 15.0],
    thrust_cost: [0.0, 0.0],
    burn_duration_per_segment: [4.0, 20.0],
    burn_duration_cost: [0.0, 0.0],
    thrust_angle: [12.0, 30.0],
    turn_rate: [1.25, 3.0],
    strafe_speed: [3.0, 4.0],
    strafe_thrust: [0.4, 0.8],
  };

  pub const SGMH2_CRUISE_ENGINE: Engine = Engine {
    name: "SGM-H-2 Cruise Engine",
    speed: [15.0, 35.0],
    speed_cost: [0.0, 0.0],
    thrust: [4.0, 8.0],
    thrust_cost: [0.0, 0.0],
    burn_duration_per_segment: [40.0, 130.0],
    burn_duration_cost: [0.0, 0.0],
    thrust_angle: [8.0, 20.0],
    turn_rate: [0.75, 1.75],
    strafe_speed: [3.0, 4.0],
    strafe_thrust: [0.4, 0.8],
  };

  pub const SGMH2_SPRINT_ENGINE: Engine = Engine {
    name: "SGM-H-2 Sprint Engine",
    speed: [50.0, 100.0],
    speed_cost: [2.0, 5.0],
    thrust: [50.0, 100.0],
    thrust_cost: [0.0, 0.0],
    burn_duration_per_segment: [1.0, 3.0],
    burn_duration_cost: [0.0, 0.0],
    thrust_angle: [25.0, 45.0],
    turn_rate: [2.0, 5.0],
    strafe_speed: [3.0, 4.0],
    strafe_thrust: [0.4, 0.8],
  };

  pub const SGMH3_CRUISE_ENGINE: Engine = Engine {
    name: "SGM-H-3 Cruise Engine",
    speed: [10.0, 20.0],
    speed_cost: [0.0, 0.0],
    thrust: [12.0, 18.0],
    thrust_cost: [0.0, 0.0],
    burn_duration_per_segment: [80.0, 250.0],
    burn_duration_cost: [0.0, 0.0],
    thrust_angle: [8.0, 30.0],
    turn_rate: [0.5, 1.25],
    strafe_speed: [2.0, 4.0],
    strafe_thrust: [0.4, 0.8],
  };

  pub const SGMH3_SPRINT_ENGINE: Engine = Engine {
    name: "SGM-H-3 Sprint Engine",
    speed: [55.0, 100.0],
    speed_cost: [3.0, 10.0],
    thrust: [150.0, 250.0],
    thrust_cost: [0.0, 0.0],
    burn_duration_per_segment: [0.85, 2.75],
    burn_duration_cost: [0.0, 0.0],
    thrust_angle: [30.0, 60.0],
    turn_rate: [2.0, 5.0],
    strafe_speed: [3.0, 4.0],
    strafe_thrust: [0.4, 0.8],
  };

  pub const SGT3_ENGINE: Engine = Engine {
    name: "SGT-3 Engine",
    speed: [17.5, 30.0],
    speed_cost: [0.0, 0.0],
    thrust: [30.0, 50.0],
    thrust_cost: [0.0, 3.0],
    burn_duration_per_segment: [1.0, 4.5],
    burn_duration_cost: [0.0, 0.0],
    thrust_angle: [8.0, 30.0],
    turn_rate: [0.4, 1.0],
    strafe_speed: [2.0, 3.5],
    strafe_thrust: [1.0, 3.0],
  };

  pub const CM4_ENGINE: Engine = Engine {
    name: "CM-4 Engine",
    speed: [12.5, 27.5],
    speed_cost: [0.0, 0.0],
    thrust: [25.0, 75.0],
    thrust_cost: [0.0, 0.0],
    burn_duration_per_segment: [3.0, 40.0],
    burn_duration_cost: [0.0, 0.0],
    thrust_angle: [15.0, 30.0],
    turn_rate: [0.5, 1.5],
    strafe_speed: [2.0, 3.5],
    strafe_thrust: [2.0, 6.0],
  };
}
