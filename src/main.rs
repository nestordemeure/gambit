#![feature(slice_patterns)]
#![feature(specialization)]

mod grammar;
mod distribution;
mod search;
mod result;
mod test;

use search::search;
use test::grammar2019::State;
use result::{ParetoFront, DisplayProgress};

fn main()
{
   let depth = 4;
   let nb_iterations = 10_000;
   let result = search::<State, distribution::ThompsonMax, DisplayProgress<ParetoFront<State>>>(depth, nb_iterations);
   println!("Result obtained in {} iterations: {}", nb_iterations, result);
}
