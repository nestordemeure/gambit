#![feature(slice_patterns)]
#[macro_use]
extern crate lazy_static;
mod grammar;

use gambit::distribution;
use gambit::search::{search, memory_limited_search, memory_limited_search_optional, nested_search};
use gambit::result::{ParetoFront, Single, DisplayProgress};
//use grammar::grammar2019::State;
//use grammar::kepler::State;
use grammar::prime::State;

fn main()
{
   let depth = 4;
   let nb_iterations = 10_000;
   let free_memory = 900;
   //let result = search::<State, distribution::ThompsonMax, Single<State>>(depth, nb_iterations);
   let result =
      memory_limited_search::<State, distribution::ThompsonMax, DisplayProgress<Single<State>>>(depth,
                                                                                        nb_iterations,
                                                                                        free_memory);
   //let result = nested_search::<State, distribution::ThompsonMax, Single<State>>(depth, nb_iterations, free_memory);
   println!("Result obtained in {} iterations: {}", nb_iterations, result);
}
