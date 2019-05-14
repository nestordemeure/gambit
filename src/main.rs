#![feature(slice_patterns)]

mod explore;
mod grammar;
mod distribution;
mod test;
use explore::search;
use distribution::thompson_max::ThompsonMax;
use test::grammar2019::State;

fn main()
{
   let depth = 4;
   let nb_iterations = 200_000;
   let score = search::<State, ThompsonMax>(depth, nb_iterations);
   println!("Reached {} in {} iterations.", score, nb_iterations);
}
