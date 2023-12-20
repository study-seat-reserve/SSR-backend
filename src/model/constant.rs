pub static NUMBER_OF_SEATS: u16 = 217;

//TEST
#[cfg(test)]
mod test {
  use super::*;

  #[test]
  fn test_number_of_seats() {
    assert_eq!(NUMBER_OF_SEATS, 217);
    assert_eq!(NUMBER_OF_SEATS + 1, 218);
    assert_eq!(NUMBER_OF_SEATS - 1, 216);
  }
}
