use std::fmt;
use std::str::FromStr;

use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not};



macro_rules! try_opt {
  ($expr:expr) => (if let Some(__v) = $expr { __v } else { return None });
}

type KeyBytes = [u8; 16];

/// Within the fleet files, a few fields are 22-character-long alphanumeric strings.
/// These are actually 16 bytes (128 bits) encoded as Base64 (particularly URL-safe Base64).
///
/// This struct represents those Base64 strings.
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Key {
  inner: KeyBytes
}

impl Key {
  /// After testing a very large number of keys from fleet files generated by the game,
  /// a few bits in the decoded bytes appear to always be left as 0's for some reason.
  ///
  /// This has 0's set at those bits, and 1's elsewhere.
  ///
  /// You can use [`Key::is_masked`] to test if a key matches this mask.
  pub const MASK: Self = key!("_________0-__________w");

  pub const fn is_masked(self) -> bool {
    const INV_MASK: Key = Key::MASK.not();
    self.and(INV_MASK).to_u128() == 0
  }

  #[inline]
  pub const fn mask(self) -> Self {
    self.and(Self::MASK)
  }

  pub const fn to_bytes(self) -> KeyBytes {
    self.inner
  }

  pub fn to_str(self, buf: &mut [u8; 22]) -> &str {
    let mut key = u128::from_be_bytes(self.inner);
    buf[21] = encode_byte(((key & 0b11) as u8) << 4);
    key >>= 2;

    for slot in buf[..21].iter_mut().rev() {
      *slot = encode_byte((key & 0b111111) as u8);
      key >>= 6;
    };

    std::str::from_utf8(buf).unwrap()
  }

  pub const fn from_bytes(bytes: KeyBytes) -> Self {
    Self { inner: bytes }
  }

  pub const fn from_str_unchecked(s: &str) -> Self {
    match Self::from_str(s) {
      Some(key) => key,
      None => panic!("failed to decode key")
    }
  }

  pub const fn from_str(s: &str) -> Option<Self> {
    let s = s.as_bytes();
    if s.len() != 22 { return None };
    let (&last_ch, mut s) = try_opt!(s.split_last());
    if !matches!(last_ch, b'A' | b'Q' | b'g' | b'w') { return None };
    let last_value = try_opt!(decode_byte(last_ch));

    let mut inner = 0u128;
    while let Some((&ch, rest)) = s.split_first() {
      let value = try_opt!(decode_byte(ch));
      inner <<= 6;
      inner |= value as u128;
      s = rest;
    };

    inner <<= 2;
    inner |= (last_value >> 4) as u128;

    Some(Key {
      inner: inner.to_be_bytes()
    })
  }

  #[inline]
  pub const fn or(self, other: Self) -> Self {
    Self::from_u128(self.to_u128() | other.to_u128())
  }

  #[inline]
  pub const fn and(self, other: Self) -> Self {
    Self::from_u128(self.to_u128() & other.to_u128())
  }

  #[inline]
  pub const fn xor(self, other: Self) -> Self {
    Self::from_u128(self.to_u128() ^ other.to_u128())
  }

  #[inline]
  pub const fn not(self) -> Self {
    Self::from_u128(!self.to_u128())
  }

  #[inline]
  const fn to_u128(self) -> u128 {
    u128::from_ne_bytes(self.inner)
  }

  #[inline]
  const fn from_u128(num: u128) -> Self {
    Self { inner: u128::to_ne_bytes(num) }
  }
}

impl fmt::Debug for Key {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let value = u128::from_ne_bytes(self.inner);
    write!(f, "Key({value:032x})")
  }
}

impl fmt::Display for Key {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let mut buf = [0; 22];
    let s = self.to_str(&mut buf);
    f.write_str(s)?;
    Ok(())
  }
}

impl FromStr for Key {
  type Err = KeyFromStrError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    Key::from_str(s).ok_or(KeyFromStrError)
  }
}

#[derive(Debug, Error, Clone, Copy)]
#[error("failed to decode key")]
pub struct KeyFromStrError;

xml::impl_deserialize_nodes_parse!(Key);
xml::impl_serialize_nodes_display!(Key);

macro_rules! impl_bit_binop {
  ($Trait:ident, $function:ident, $TraitAssign:ident, $function_assign:ident, $base:ident) => {
    impl $Trait for Key {
      type Output = Key;

      fn $function(self, rhs: Self) -> Self::Output {
        self.$base(rhs)
      }
    }

    impl $TraitAssign for Key {
      fn $function_assign(&mut self, rhs: Self) {
        *self = self.$base(rhs);
      }
    }
  };
}

impl_bit_binop!(BitAnd, bitand, BitAndAssign, bitand_assign, and);
impl_bit_binop!(BitOr, bitor, BitOrAssign, bitor_assign, or);
impl_bit_binop!(BitXor, bitxor, BitXorAssign, bitxor_assign, xor);

impl Not for Key {
  type Output = Key;

  fn not(self) -> Self::Output {
    self.not()
  }
}



const fn encode_byte(ch: u8) -> u8 {
  b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_"[ch as usize]
}

const fn decode_byte(ch: u8) -> Option<u8> {
  match ch {
    b'A'..=b'Z' => Some((ch - b'A') + 0),
    b'a'..=b'z' => Some((ch - b'a') + 26),
    b'0'..=b'9' => Some((ch - b'0') + 52),
    b'-' => Some(62),
    b'_' => Some(63),
    _ => None
  }
}

#[cfg(test)]
mod tests {
  use base64::DecodeSliceError;
  use base64::engine::Engine;
  use base64::engine::general_purpose::URL_SAFE_NO_PAD as ENGINE;

  use super::*;

  #[test]
  fn test_key_samples() {
    let mut mask = Key::from_u128(0);
    for &sample in SAMPLES {
      let key_w = key_from_str_w(sample).unwrap();
      let key = Key::from_str(sample).unwrap();
      assert_eq!(key_w, key);

      let key_str_w = key_to_string_w(key_w);
      let key_str = key.to_string();
      assert_eq!(key_str_w, key_str);

      mask |= key;
    };
  }

  #[test]
  fn test_key_roundtrip() {
    for key1 in keys() {
      let key_str1 = key1.to_string();
      let key2 = Key::from_str(&key_str1).unwrap();
      let key_str2 = key2.to_string();
      assert_eq!(key1, key2);
      assert_eq!(key_str1, key_str2);
    };
  }

  #[test]
  fn test_mask() {
    let mut mask = Key::from_u128(0);
    for key in keys() {
      assert!(key.is_masked());
      mask |= key;
    };

    assert_eq!(Key::MASK, mask);
  }

  fn keys() -> impl Iterator<Item = Key> {
    use crate::utils::ContiguousExt;
    crate::data::hulls::HullKey::values()
      .flat_map(|hull_key| hull_key.hull().sockets)
      .map(|socket| socket.save_key)
  }

  fn key_from_str_w(s: &str) -> Result<Key, DecodeSliceError> {
    let mut out = [0; 16];
    ENGINE.decode_slice(s, &mut out)?;
    Ok(Key { inner: out })
  }

  fn key_to_string_w(key: Key) -> String {
    ENGINE.encode(key.inner)
  }

  // Generated from random bytes
  const SAMPLES: &[&str] = &[
    "keE8mLOSiyo3bY4df_ft_w", "Nsne_LepZW660K7j73jGFg", "g8r2Az4V6YrkMPeRgx527Q", "VU-eC72x6880l_w2yt8QXw",
    "fcTrEIwXrF5GkGhhOOtsMQ", "E5Yiyr5csEKhV1GOGCFH2Q", "06XqA5oika-YzcTzQ8n9lw", "DNu4zJXq1bBv3sZ0-wJnLw",
    "IY_ayi9F90tJjUmzyC842g", "wBE91Ql_TJE5wTb6OLb2Qg", "fDpR396MwJzmkpuXCe7IrQ", "1lQ7sEPfLmtvd7UKxk87TQ",
    "rY6Vv5s2zl3mdkwc9kSUAA", "_tqmMv3bfZGjaw24eqozHg", "WohVhJWVLOni1H2ltjqEYg", "5pR5o5L7NE0Pke3U6lNOIQ",
    "PwpR9kD-o4_s0XEDBsSu8Q", "jUHNgz9X9V1FNZpGdjTQvw", "2Sc2bSIBhgp-AJuzzEM1sA", "ClUu6Bj6aTCcDAo5QnHncw",
    "gWSEDIg5a6JCWesPNz5suw", "mMuardKuwMmppqhtjtDBIA", "sFTnw8blFM6trV29HSJZpg", "uz8n05cK_vJ-JInlawQ9DA",
    "cIZSVIwOPEW5gZoZz6O6jw", "SqnommvURRwWIvapgVoocQ", "F91vg18e1pu4Mpr1PG_Qug", "azJ6cbrrRxmrZY8-6QFCyg",
    "NO75xA3lndgCp-ZDYHoASQ", "Q50rgsIU9Iz6AR2b30KKLg", "VkssSG1pHUCqnN_67I8siA", "WYXZmlr5gEjIxZJOKPiJCQ",
    "NLecROlStPRb3kqXBsa-qg", "wyB-yiEGk8meYoJacis8gA", "h-YvSF4iVdUPKga3BG5zRQ", "6MrR2bddXXGzHjuCb4B_Pg",
    "JnzU_YyN4gH1t7rCL6_UZQ", "w4atO2Oi1DWSQkNEzAJbNA", "kVoq9mdpHGiHve-TVoV9pQ", "qtZD-99NqwOng1tyLROKOg",
    "ub8Hnh9Nzw8KirDLn9XpKg", "bC1vnu1_jOhdQ2neLRYScg", "qlYYvFJpDnHR_iQaQXcVRw", "8AuGDaJ-qFvUjkLBiiG4PQ",
    "iH15CxKmW96ThR4dY2vS4Q", "-_9O5C6PfedFGLhnbLQmbw", "KvpWuNc98ja8Y0MHpP_J8Q", "jMCHEe_Ag11izzkGy0G67g",
    "Wq-vh8ch39bXBWqSiMCr7w", "pK2ZqZ-etq5laRxN7v_b1g", "eMp0dWfTID3AupU0_qU5MA", "LqtF7GF-jLweNg3qBGTAyg",
    "pBf91KtKXLQ4xKYW_OYGuQ", "W0YDITcuSw9Us46GSA_NnQ", "YPxqdHn864jxvLGCz1KrLw", "rP3_Zi6FBKvY-qF6qNhhqQ",
    "NBoHG_8tbvuFXGuepx-Yhw", "kwLkY6GDLAkM1rhjwbSqBg", "Ak3vdvwWwt_A4ke6MpNqlg", "A-1Zo17olP9jxkdV75PXBw",
    "FIFcT9UtTXrTcLnKejkfhg", "hMo1bsP6IKgslp6gTytIRg", "waXyHluJnKPeN7Y-oPBhIA", "rAEctHVJ0EBpsZkDEv2lrg",
    "4rsMGQfez06j6SkjdxTQzQ", "LVEcNmijzUuftLYx601EbQ", "iwaiKL3dgIaFq4Lv00H8ZA", "fxi0ovJyMN0Y71NecmSDuQ",
    "Lzt49GQsXAgnIT6DRDr0Uw", "04geUqxOyupyl8B1gDyBzg", "9z_w0WkBbg10ewB7YeDk_A", "dQJEjSUE56BPQra7IGD2VA",
    "uvyRaQ5AgDGX1rkugyx8Cg", "5I9g0ioUUjTGmNuyI1Z69A", "NYBD7SRuQPwVPbkXEhh6cw", "BnD94HxpYdyiXDW6NFxZsQ",
    "3I0KLLv8quJbL7pmnvEVQQ", "gqxG6HxJM4vrmLBTHkgl4w", "ae4NGLXFxtaCLA9OjbNdLw", "s2c-k5fDgobmQrX6tpIgZg",
    "RC9WIkE-FbytYEHKQxwFMA", "7_mqWPa9gjOhjRyP7NO-KA", "Sxzx2L2MyoxGF2bh936wqQ", "_yhEWWhZsI4H1QydrGiZ8Q",
    "7iil0osPmdZoKSHoy2uPBw", "YbjbMXoRcmOZwIu0f4h7JQ", "GbARHo0GTF56EREk3pxE3A", "lirdp-BwcHtx7NBQ--fUjA",
    "XSVUz_BZ28o69gg-FLRXkQ", "EqzKwmXA2ogV1tthgl0HRQ", "-6yr4aTlUpNPrZmMxyB-NQ", "UfInfQieLkdUnqKS39RUhQ",
    "gzoRurlU570l9KcWGZr5Lg", "amqz75fUkCk3gc_OPbE4yA", "FJpK9-bBoEQCtA5qMSyeQw", "iMGfrROtikL6RJavhpQubw",
    "ue32VbNeH49NRpGHxoS8nA", "r_1h_T2D9mMFM7LFzijkOg", "vMzH0tS3Klj28gV9ZGNQgQ", "gn3aVnik6NZ1pBQKSeOCjA",
    "Nb3RcT1OmedWDcA3WauFqw", "8YL_qHuSOM-x09SKOyvn1Q", "d2wb-GZHbC5OuQQMgZ8XgQ", "227XsTxd1oCQ3RJghsoe_Q",
    "FIPW4__Uue4uQ72yhd2O6w", "vs-vXUfsyHyTGIN2TBSBvw", "EIpbpCiwcdtWsstwD7GdOA", "VNltpAaepClvKgjpFVmMOw",
    "8yUTZ-P5_avcChU4F_PMaA", "RVDpMH0NlSGYK21qwkfjCg", "IddB4au9dE01mH4xvcXRzA", "LcntODdRH9-fSNqKYMUZGg",
    "sNMm60qoIwdERaZYk9mJKw", "FhbKIgcGMfbLFw1ofaD-mw", "3Gx47ng6_3VnRXeZv6zl-g", "VMzJaE1djisRCEMac02WIQ",
    "7fePe67GazroHsTZ9_MQkA", "LOkuHJI5ztbHBBEodhCV5A", "7Dr136dU19oI5KzzHR2T1A", "wSq0TUHhnczXtTBX3iJ8Ng",
    "EdVPWlFtx_cTZPiDTdklsg", "KiSJEUeR48r5zFkZ7ZGsFA", "bI7-dUvbrQ9Qsb9MFOKfEw", "7NyjKahGoITp2RgImGSJ7g",
    "9kFLv5I2akujV6_WrRBClQ", "gjQ18w_nKf8KTktbOEgPPw", "Q5ewnOpGulX8o8jeSOLuwg", "y_RhJp6_6OgVxOiH45Bakg",
    "9zGaXwtZQNC46YcdRrOSzA", "4Jx4S7X0z-5vehIjbEBWXg", "07MCP_1DwsnEPmo9Vg5eFA", "g-GzGuHJJBnDeSMriHYPtQ",
    "UmOU0gLLeO95_qU34kuYVA", "GzKhjmFMkaOvVGDzenPoxw", "Ws6Ey6whqq_AkXbHZglkNQ", "qLDF1MZXq-AUA511sqitsQ",
    "WlYQHpNqQ56idML53JgOZw", "ztGv_ptLNKShQVz03nWH6Q", "PWEXCPO9RrF_tVTKoAFnWg", "lDLc2hOIe_sqTJI8i8oxEw",
    "Hk39lJOunHniO8qGPqVKXA", "rt-yhwS_UMRALzO-gXhYjg", "6fwpRSDrC83ydzSNPNMXhA", "fWMFsEVKOo9n819HIbQHhA",
    "5X8Sy7bqnnFFFn4fwVbBpQ", "3uALnSnTe16SHrMwTPy1FQ", "nFCk_4e53jrvzscl8DPECQ", "IAaDH0btzfErNXyXq8dRaA",
    "WeGCZXjwG63f2NZQ3mAHLw", "DalHPbgHucveltybvYnOQw", "C7mdEGz-qZ57AJrlhbmRgg", "q-dYMYJAQeqbgwt7RQC5yg",
    "GTrvhHnEvextS_4xrKYa9A", "ia-UX68tc0tzvksgFf9-BQ", "9nQwENtLgr5jVa7mvAeedg", "0mBdpMZAwEdwVWzKupZBgw",
    "q08qkCBaBwghPl6JWS8mOQ", "9xFRgSr9nZI7-XBhcDByWw", "IvOYSotXnW5hLxwh0vEqhg", "s-vpKqNMSzMT9ODu6Q27ig",
    "5ZGNm0kT5nXY36rSSuwkGQ", "o9uC9Gn4ohJFyHD6-UmFMg", "KoW81FMPmZbIuN5twty9qw", "phIhthWlQBiN1qQ9P2KYew",
    "kwIDMmRx70inaKr9qf0jiw", "XGaIz1jwOZewMll8N9QflA", "PEIzF2v0mx97blBS96QlJQ", "NiDuqUTgJGPh5YrQw-s99w",
    "-1ZgXknN3C2G7cn98CbZ1A", "f-hS0NosDDtSU8V5ZcasBw", "aNJgoUJmmfLDHzkiwsoEtw", "dsTw4AlUyTYQknnztBH86A",
    "dTKy6EgIGkN5kIMLsJAjBw", "j3wNhTfwLDX5eU23mTv63w", "wH7ItVROuoFzlXGjT029Aw", "9O05jqgHsu5IaBx77e4I7Q",
    "5a7fPwc3mMiR123oESfGUQ", "SwI8viDDitHt35IP1mbQBw", "T8B30atxBv6t1FEHM_GCcg", "78RXrue10oUE8XyF_82SLg",
    "uDXt0ShFB5b1tHIBXEZf_g", "moUWPG8ypQynfahkpVP5gw", "fElDlSQZCUHg22W2c-Iq6Q", "yl3nFBNE6E0_9Skp-L6fYQ",
    "WISbx632kzFXjnYBfpu0EA", "X0DNLUee5WM6uWZACCY-Eg", "n_0cofngAWqtOLmOIE8c3g", "BtcKH6ai6l2mLL_se653ZQ",
    "ExPj6wv0NDwPicxtHEJ8ow", "3GG2s-Ah6X6bB4KHqbsBbg", "X1MkR79TR9dvNYxkCrhksw", "XOl65gR5qwfxY3J0O9c2lA",
    "qFjDCFduucEDK76f1TQSVA", "YF0xn0bsaXCOc3Z4sLgqxQ", "js8dndJgHIryMPP17eUeWA", "73aI-JzSKab9fbsgswbA_w",
    "nNXYXK1-10Ts6-Zg3EQtdQ", "EuE1-tBlLVpFInPnoYzNFw", "KKTLKefBtfNh1wUuq7m-7A", "fj6vcTizO7qX7USdejswVg",
    "SAZnzxP-k20fxsmfVM7rgA", "XmMjgO8q_BXLZoJc8cqdXg", "e7XAdl5DX1o9fSaMCLb5pQ", "KJp01ciSEh6RbP4O_L91vQ",
    "xKdowWrHy9TkuzCZMzRa9Q", "hwfKLpKMAJYP1w4htOf9cQ", "RNIPhJq2tvQFH99wf5S51w", "CUomiFn5SUjVArAGeXOyaQ",
    "kTmkdlWskEY64Zo836adEw", "YAR8xte4H3j8Xt6KYj5-KQ", "Z6LkzXbtFBYDSqJ90GDD5w", "NUaVhzVIPfmAbbK_NkvQYQ",
    "WJY-xoxJnyeI1v42fv8Dng", "NkSFwnfDowSUThGsnO9OGA", "Z0bf-_aFQBleA9GRSgGmTA", "gbgt6rzNOJ9nkTUQwp6eJA",
    "foaEb-d9sSid47D6EzoxcQ", "4xo0KCrAmo2qSRdi_MWqvw", "h1z6kwpFNtwAwHZvCv9FhA", "J9WI0PoqZgHggnPOuCAN3w",
    "muGehPgt6dlqqmu8Wa2X2g", "ny26ojk-La0VJZY9hT9Bfw", "8w_deyRjHPbsc9YFS2mTaA", "44GAAbDxYNbuWKP9F_0Hyg",
    "HKAUKTtYMsqkPVR-oUrdSw", "HHxJztZLrTBxv9GJyV-pRw", "x1AHC02tCE9IKS_EmXgYIQ", "LJ363lqExxULngJGYJ3xmA",
    "iUXM_DYcDD8wlmd_yLEqeA", "iYtqvsk56XgVHAEwEaedWQ", "m0Xt7HRmXBHbJ4oDeO8lRw", "pTV27MzrKtBUuREi96uU6g",
    "VSzMvjeTrihz82-rH5nTxQ", "PE6Q537H-E2oYCQlC5GLvQ", "NrRocvUCZorZQUdjlFBhRA", "tiuPyVX9Zksph8-i-m93Zg",
    "02qdEmeu6Slf-n1K4o3Mbw", "KB5uTjmxJEjDWbensCXhDg", "ug-0iJfgMycpKNzfZi8w_w", "w3dnBeGDWJUrfkEuZOGhqw",
    "7H9Gi1o2Iyf1IOpbR0qRkA", "xLApWSHSBcVVBzRFs7V_7w", "zjtR8skhSQVw3oPu5GzrWA", "DZdIy1PLJn0piZcE4gBLoA",
    "XzJ_jQG9SnqDut3e6wQavw", "HnqJdLwr5I5SsD26dkg8Gg", "ZJwLJ_hPvTixf8KwPNamxw", "OTbJkHUgDALWBFeIy-RxuA",
    "-da7CAU1xZD1AvCWt2bOtg", "fBkZjaABojK4t4pLYJ-b9Q", "WLTMTYQn8iFVlG06-aq8Ag", "wHuXMvjfCElgfAphcirRiQ",
    "g_8C307Zzr3y1fdCjXGqEQ", "8ALv8EHsVYg9sLsskRzQUQ", "JXdwxNLGGIfemMdWLXad9Q", "HgPODloSyYj-E-CQqfea2Q"
  ];
}
