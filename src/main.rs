#![feature(slice_patterns)]

mod explore;
mod grammar;
mod test;
use test::grammar2019;

fn main()
{
   let depth = 4;
   let nb_iterations = 200_000;
   let score = explore::search::<grammar2019::State>(depth, nb_iterations);
   println!("Reached {} in {} iterations.", score, nb_iterations);
}
