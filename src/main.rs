#![feature(slice_patterns)]

mod explore;
mod grammar;

fn main()
{
   let depth = 4;
   let nbIterations = 100;
   let score = explore::search(depth, nbIterations);
   println!("Reached {} in {} iterations.", score, nbIterations);
}
