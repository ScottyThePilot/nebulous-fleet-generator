use itertools::Itertools;

/// `P(A or B or C or N ...)`
pub fn probability_any(list: &[f32]) -> f32 {
  match list {
    &[] => f32::NAN,
    &[a] => a,
    &[a, b] => (a + b) - (a * b),
    &[a, b, c] => {
      (a + b + c) - (a * b) - (a * c) - (b * c) + (a * b * c)
    },
    list => {
      let mut sum = list.iter()
        .copied().sum::<f32>();
      for i in 2..=list.len() {
        let sign = if i % 2 == 0 { -1.0 } else { 1.0 };
        for m in list.iter().copied().combinations(i) {
          sum += m.into_iter().product::<f32>() * sign;
        };
      };

      sum
    }
  }
}
