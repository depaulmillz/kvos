pub trait Set {
    type Key;
    fn new() -> Self;
    fn contains(&self, key: &Self::Key) -> bool;
    fn insert(&mut self, key: &Self::Key) -> bool;
    fn remove(&mut self, key: &Self::Key) -> bool;
    fn size(&self)->usize;
}
extern crate alloc;
use alloc::vec::Vec;
pub struct SimpleSet<T> {
  pub data: Vec<T>,
}

impl<T: PartialEq + Clone> SimpleSet<T> {
  pub fn new() -> Self {
      Self { data: Vec::new() }
  }

  pub fn insert(&mut self, item: &T) -> bool {
      if !self.contains(&item) {
          self.data.push(item.clone());
          true
      } else {
          false
      }
  }

  pub fn remove(&mut self, item: &T) -> bool {
      if let Some(pos) = self.data.iter().position(|x| x == item) {
          self.data.swap_remove(pos);
          true
      } else {
          false
      }
  }

  pub fn contains(&self, item: &T) -> bool {
      self.data.iter().any(|x| x == item)
  }

  pub fn size(&self) -> usize {
    self.data.len()
}
}
