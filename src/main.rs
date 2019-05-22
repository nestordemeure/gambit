#![feature(slice_patterns)]
#![feature(box_patterns)]
#![allow(dead_code)]
#![allow(unused_imports)]

mod tools;
mod grammar;
mod distribution;
mod memory;
mod search;
mod result;
mod test;

use search::{search, memory_limited_search, nested_search};
use test::grammar2019::State;
use result::{ParetoFront, Single, DisplayProgress};

fn main()
{
   let depth = 4;
   let nb_iterations = 10_000;
   let free_memory = 900;
   let result = search::<State, distribution::UcbTuned, Single<State>>(depth, nb_iterations);
   //let result = memory_limited_search::<State, distribution::ThompsonMax, Single<State>>(depth, nb_iterations, free_memory);
   //let result = nested_search::<State, distribution::ThompsonMax, Single<State>>(depth, nb_iterations, free_memory);
   println!("Result obtained in {} iterations: {}", nb_iterations, result);
}
