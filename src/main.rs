#![feature(slice_patterns)]
#![feature(specialization)]

mod grammar;
mod distribution;
mod search;
mod result;
mod memory;
mod test;

use search::{search, search_optional};
use test::grammar2019::State;
use result::{ParetoFront, DisplayProgress, Single};

fn main()
{
   let depth = 4;
   let nb_iterations = 10_000;
   //let result = search::<State, distribution::ThompsonMax, Single<State>>(depth, nb_iterations);
   let result = search_optional::<State, distribution::ThompsonMax, Single<State>>(depth, nb_iterations);
   println!("Result obtained in {} iterations: {}", nb_iterations, result);
}
