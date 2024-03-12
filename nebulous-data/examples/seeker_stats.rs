use nebulous_data::data::missiles::seekers::SeekerStrategyEntry;

fn main() {
  let mut out = String::new();
  for entry in SeekerStrategyEntry::get_entries_cached() {
    let defeat_probability = entry.get_defeat_probability_default();
    let defeat_probability_no_ewar = entry.get_defeat_probability_default_no_ewar();

    let min_cost = entry.seeker_strategy.min_cost();
    let max_cost = entry.seeker_strategy.max_cost();

    out.push_str(&format!("### `{}`\n", entry.seeker_strategy));
    if min_cost == max_cost {
      out.push_str(&format!("Cost: **{min_cost:.2}**\n"));
    } else {
      out.push_str(&format!("Cost: **{min_cost:.2} - {max_cost:.2}**\n"));
    };
    out.push_str(&format!("Defeat probability: **{:.2}%**\n", defeat_probability * 100.0));
    out.push_str(&format!("Defeat probability (no EWAR): **{:.2}%**\n", defeat_probability_no_ewar * 100.0));
    if entry.countermeasure_methods.is_empty() {
      out.push_str("Cannot be defeated!\n");
    } else {
      out.push_str("Can be defeated by:\n");
      for &countermeasures in entry.countermeasure_methods.iter() {
        out.push_str(&format!("- {countermeasures}\n"));
      };
    };

    out.push_str("\n");
  };

  std::fs::write("./seeker_stats.md", out).unwrap();
}
