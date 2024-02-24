use itertools::Itertools;

/// `P(A or B or C or N ...)`
pub(crate) fn probability_any(list: &[f32]) -> f32 {
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

#[macro_export]
macro_rules! zsize {
  ($expr:expr) => (match std::num::NonZeroUsize::new($expr) {
    Option::Some(__v) => __v,
    Option::None => panic!("value is zero")
  });
}

macro_rules! size_op {
  ($vis:vis fn $name:ident [$t:tt]) => (
    #[inline] $vis const fn $name(self, rhs: Self) -> Self {
      Size { x: self.x $t rhs.x, y: self.x $t rhs.y, z: self.x $t rhs.z }
    }
  );
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Size {
  pub x: usize,
  pub y: usize,
  pub z: usize
}

impl Size {
  #[inline]
  pub const fn new(x: usize, y: usize, z: usize) -> Self {
    Size { x, y, z }
  }

  #[inline]
  pub const fn from_array(array: [usize; 3]) -> Self {
    Size { x: array[0], y: array[1], z: array[2] }
  }

  #[inline]
  pub const fn into_array(self) -> [usize; 3] {
    [self.x, self.y, self.z]
  }

  size_op!(pub fn add [+]);
  size_op!(pub fn sub [-]);
  size_op!(pub fn div [/]);
  size_op!(pub fn mul [*]);
  size_op!(pub fn rem [%]);

  #[inline]
  pub const fn volume(self) -> usize {
    self.x * self.y * self.z
  }
}
