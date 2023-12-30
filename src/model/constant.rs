pub static NUMBER_OF_SEATS: u16 = 217;

#[cfg(test)]
mod test {
  use super::*;

  #[test]
  fn test_number_of_seats() {
    assert_ne!(NUMBER_OF_SEATS, 216);
    assert_eq!(NUMBER_OF_SEATS, 217);
    assert_ne!(NUMBER_OF_SEATS, 218);
  }
}
