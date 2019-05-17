#![feature(slice_patterns)]
#![feature(specialization)]

mod explore;
mod grammar;
mod distribution;
mod result;

mod test;
use explore::search;
use test::grammar2019::State;

fn main()
{
   let depth = 4;
   let nb_iterations = 10_000;
   let result = search::<State, distribution::ThompsonMax, result::ParetoFront<State>>(depth, nb_iterations);
   println!("Result obtained in {} iterations: {}", nb_iterations, result);
}
