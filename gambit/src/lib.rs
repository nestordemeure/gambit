#![feature(slice_patterns)]
#![feature(box_patterns)]
pub mod tools;
pub mod grammar;
pub mod distribution;
pub mod memory;
pub mod search;
pub mod result;

#[cfg(test)]
mod tests
{
   #[test]
   fn it_works()
   {
      assert_eq!(2 + 2, 4);
   }
}
