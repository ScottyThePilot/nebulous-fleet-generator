use bytemuck::Contiguous;
use itertools::Itertools;

use std::ops::RangeInclusive;



#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContiguousEnumValues<T: Contiguous> {
  inner: RangeInclusive<T::Int>
}

impl<T: Contiguous> ContiguousEnumValues<T> {
  pub const fn new() -> Self {
    ContiguousEnumValues {
      inner: T::MIN_VALUE..=T::MAX_VALUE
    }
  }
}

impl<T> Iterator for ContiguousEnumValues<T>
where T: Contiguous, RangeInclusive<T::Int>: Iterator<Item = T::Int> {
  type Item = T;

  fn next(&mut self) -> Option<Self::Item> {
    self.inner.next().map(|int| T::from_integer(int).unwrap())
  }

  fn size_hint(&self) -> (usize, Option<usize>) {
    self.inner.size_hint()
  }

  fn count(self) -> usize where Self: Sized {
    self.inner.count()
  }

  fn fold<B, F>(self, init: B, f: F) -> B
  where Self: Sized, F: FnMut(B, Self::Item) -> B {
    self.inner.map(|int| T::from_integer(int).unwrap()).fold(init, f)
  }
}

impl<T> DoubleEndedIterator for ContiguousEnumValues<T>
where T: Contiguous, RangeInclusive<T::Int>: DoubleEndedIterator<Item = T::Int> {
  fn next_back(&mut self) -> Option<Self::Item> {
    self.inner.next_back().map(|int| T::from_integer(int).unwrap())
  }

  fn rfold<B, F>(self, init: B, f: F) -> B
  where Self: Sized, F: FnMut(B, Self::Item) -> B {
    self.inner.map(|int| T::from_integer(int).unwrap()).rfold(init, f)
  }
}

impl<T> ExactSizeIterator for ContiguousEnumValues<T>
where T: Contiguous, RangeInclusive<T::Int>: ExactSizeIterator<Item = T::Int> {
  fn len(&self) -> usize {
    self.inner.len()
  }
}



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

#[inline]
pub fn lerp<T: Lerp<Factor>, Factor: LerpFactor>(a: T, b: T, factor: Factor) -> T {
  Lerp::lerp(a, b, factor)
}

#[inline]
pub fn lerp2<T: Lerp<Factor>, Factor: LerpFactor>(v: [T; 2], factor: Factor) -> T {
  let [a, b] = v;
  Lerp::lerp(a, b, factor)
}

pub trait Lerp<Factor: LerpFactor>: Sized {
  /// Linearly interpolates between two values.
  ///
  /// When `factor` is 0, the result will be the value of `from`.
  ///
  /// When `factor` is 1, the result will be the value of `to`.
  fn lerp(from: Self, to: Self, factor: Factor) -> Self;

  /// Linearly interpolates between a list of values.
  /// Returns `None` if the provided list is empty or `factor` is out of bounds.
  fn lerp_slice(slice: &[Self], factor: Factor) -> Option<Self> where Self: Clone {
    LerpFactor::lerp_slice(slice, factor)
  }

  fn lerp_slice_normalized(slice: &[Self], factor: Factor) -> Option<Self> where Self: Clone {
    LerpFactor::lerp_slice_normalized(slice, factor)
  }
}

pub trait LerpFactor: Copy {
  fn lerp_slice<T: Lerp<Self> + Clone>(slice: &[T], factor: Self) -> Option<T>;

  fn lerp_slice_normalized<T: Lerp<Self> + Clone>(slice: &[T], factor: Self) -> Option<T>;
}

macro_rules! impl_lerp_float {
  ($Float:ty) => {
    impl Lerp<$Float> for $Float {
      fn lerp(from: Self, to: Self, factor: $Float) -> Self {
        from.mul_add(1.0 - factor, to * factor)
      }
    }

    impl<T, const N: usize> Lerp<$Float> for [T; N]
    where T: Lerp<$Float> + Clone {
      fn lerp(from: Self, to: Self, factor: $Float) -> Self {
        from.into_iter().zip(to.into_iter())
          .map(|(from, to)| T::lerp(from, to, factor))
          .collect::<Vec<T>>()
          .try_into().ok()
          .expect("infallible")
      }
    }

    impl LerpFactor for $Float {
      fn lerp_slice<T: Lerp<Self> + Clone>(slice: &[T], factor: Self) -> Option<T> {
        if slice.is_empty() { return None };
        let lower = factor.floor() as usize;
        let upper = factor.ceil() as usize;
        if lower == upper {
          slice.get(lower).cloned()
        } else {
          let lower = slice.get(lower).cloned()?;
          let upper = slice.get(upper).cloned()?;
          let factor = factor.rem_euclid(1.0);
          Some(T::lerp(lower, upper, factor))
        }
      }

      fn lerp_slice_normalized<T: Lerp<Self> + Clone>(slice: &[T], factor: Self) -> Option<T> {
        let factor = factor.clamp(0.0, 1.0) * ((slice.len() - 1) as $Float);
        LerpFactor::lerp_slice(slice, factor)
      }
    }
  };
}

impl_lerp_float!(f32);
impl_lerp_float!(f64);

#[macro_export]
macro_rules! any {
  ($($expr:expr),* $(,)?) => ($($expr ||)* false);
}

#[macro_export]
macro_rules! all {
  ($($expr:expr),* $(,)?) => ($($expr &&)* true);
}
