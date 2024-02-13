use nebulous_data::missiles::{SeekerStrategy, CountermeasureProbabilities};

fn main() {
  let mut out = String::new();

  let no_ewar = CountermeasureProbabilities {
    radar_jamming: 0.0,
    comms_jamming: 0.0,
    ..Default::default()
  };

  for &seeker_strategy in SeekerStrategy::LIST {
    let seeker_layouts = seeker_strategy.get_layouts();
    let average_guidance_quality = seeker_layouts.iter()
      .map(|&layout| layout.guidance_quality())
      .sum::<f32>() / seeker_layouts.len() as f32;
    let minimum_cost = seeker_layouts.iter()
      .map(|&layout| layout.cost())
      .min_by(f32::total_cmp)
      .expect("infallible");
    let decoy_probability = seeker_strategy.get_decoy_probability(Default::default());
    let decoy_probability_no_ewar = seeker_strategy.get_decoy_probability(no_ewar);

    out.push_str(&format!("### `{}`\n", seeker_strategy));
    out.push_str(&format!("Minimum Cost: **{:.2}**\n", minimum_cost));
    out.push_str(&format!("Guidance quality: **{:.2}**\n", average_guidance_quality));
    out.push_str(&format!("Decoy probability: **{:.2}%**\n", decoy_probability * 100.0));
    out.push_str(&format!("Decoy probability (no EWAR): **{:.2}%**\n", decoy_probability_no_ewar * 100.0));
    out.push_str("Can be decoyed by:\n");
    for &decoy_combination in seeker_strategy.decoyed_by {
      out.push_str("- ");
      for (i, &countermeasure) in decoy_combination.iter().enumerate() {
        if i != 0 { out.push_str(" + ") };
        out.push_str(countermeasure.to_str());
      };
      out.push_str("\n");
    };
    out.push_str("\n");
  };

  std::fs::write("./seeker_stats.md", out).unwrap();
}
