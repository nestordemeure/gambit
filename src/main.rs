#![feature(slice_patterns)]

mod explore;
mod grammar;

fn main()
{
   let depth = 4;
   let nb_iterations = 100_000;
   let score = explore::search(depth, nb_iterations);
   println!("Reached {} in {} iterations.", score, nb_iterations);
}
