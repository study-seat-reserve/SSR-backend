pub static NUMBER_OF_SEATS: u16 = 217;

#[cfg(test)]
mod test {
  use super::*;

  #[test]
  fn test_number_of_seats() {
    assert_eq!(NUMBER_OF_SEATS, 217);
  }
}
