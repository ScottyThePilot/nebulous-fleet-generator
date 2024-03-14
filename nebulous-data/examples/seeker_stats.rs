use nebulous_data::data::missiles::seekers::SeekerStrategyEntry;

fn main() {
  let display = nebulous_data::utils::anonymous_fmt_display(|f| {
    for entry in SeekerStrategyEntry::get_entries_cached() {
      let defeat_probability = entry.get_defeat_probability_default();
      let defeat_probability_no_ewar = entry.get_defeat_probability_default_no_ewar();

      let min_cost = entry.seeker_strategy.min_cost();
      let max_cost = entry.seeker_strategy.max_cost();

      writeln!(f, "### `{}`", entry.seeker_strategy)?;
      match min_cost == max_cost {
        true => writeln!(f, "- Cost: **{min_cost:.2}**")?,
        false => writeln!(f, "- Cost: **{min_cost:.2} - {max_cost:.2}**")?
      };

      writeln!(f, "- Defeat probability: **{:.2}%**", defeat_probability * 100.0)?;
      writeln!(f, "- Defeat probability (no EWAR): **{:.2}%**", defeat_probability_no_ewar * 100.0)?;

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

  let writer = std::fs::File::create("./nebulous-data/seeker_stats.md").expect("error");
  nebulous_data::utils::adapt_fmt(writer, &display).expect("error");
}
