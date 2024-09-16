use nebulous_data::data::missiles::seekers::{
  SeekerStrategyEntry,
  COUNTERMEASURE_PROBABILITIES_VS_ALLIANCE,
  COUNTERMEASURE_PROBABILITIES_VS_PROTECTORATE
};

fn main() {
  let mut entries = Vec::new();
  for entry in SeekerStrategyEntry::get_entries_cached() {
    if entry.seeker_strategy.len().get() >= 3 { continue };

    let mut success_probability = 1.0 - entry.get_defeat_probability_default();
    let success_probability_alliance = 1.0 - entry.get_defeat_probability(COUNTERMEASURE_PROBABILITIES_VS_ALLIANCE);
    let success_probability_protectorate = 1.0 - entry.get_defeat_probability(COUNTERMEASURE_PROBABILITIES_VS_PROTECTORATE);
    let success_probability_faction_specific = f32::max(success_probability_alliance, success_probability_protectorate);
    if success_probability_faction_specific > success_probability {
      success_probability = f32::max(success_probability + 0.1, success_probability_faction_specific);
    };

    let score = (20.0 - entry.seeker_strategy.min_cost()) * success_probability;

    entries.push((entry, score));
  };

  entries.sort_by(|(_, a), (_, b)| {
    f32::total_cmp(a, b).reverse()
  });

  let display = nebulous_data::utils::anonymous_fmt_display(|f| {
    for &(entry, score) in entries.iter() {
      let defeat_probability = entry.get_defeat_probability_default();

      let min_cost = entry.seeker_strategy.min_cost();
      let max_cost = entry.seeker_strategy.max_cost();

      writeln!(f, "### `{}`", entry.seeker_strategy)?;

      writeln!(f, "- Score: **{score:.2}**")?;

      match min_cost == max_cost {
        true => writeln!(f, "- Cost: **{min_cost:.2}**")?,
        false => writeln!(f, "- Cost: **{min_cost:.2} - {max_cost:.2}**")?
      };

      writeln!(f, "- Defeat probability: **{:.2}%**", defeat_probability * 100.0)?;

      if entry.countermeasure_methods.is_empty() {
        writeln!(f, "- Cannot be defeated")?;
      } else {
        writeln!(f, "- Can be defeated by:")?;
        for &countermeasures in entry.countermeasure_methods.iter() {
          writeln!(f, "  - {countermeasures}")?;
        };
      };

      writeln!(f)?;
    };

    Ok(())
  });

  let writer = std::fs::File::create("./nebulous-data/seeker_ranking.md").expect("error");
  nebulous_data::utils::adapt_fmt(writer, &display).expect("error");
}
