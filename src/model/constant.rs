pub static NUMBER_OF_SEATS: u16 = 217;

//TEST

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_number_of_seats() {
    assert_eq!(NUMBER_OF_SEATS, 217);
  }
}
