#![feature(slice_patterns)]
#![feature(box_patterns)]
#![feature(specialization)]
#![allow(dead_code)]
#![allow(unused_imports)]

mod tools;
mod grammar;
mod distribution;
mod search;
mod result;
mod memory;
mod test;

use search::{search, search2, search_optional, memory_limited_search};
use test::grammar2019::State;
use result::{ParetoFront, DisplayProgress, Single};

fn main()
{
   let depth = 4;
   let nb_iterations = 20_000;
   //let free_memory = 900;
   //let result = search::<State, distribution::ThompsonMax, Single<State>>(depth, nb_iterations);
   let result = search2::<State, distribution::ThompsonMax, Single<State>>(depth, nb_iterations);
   //let result = memory_limited_search::<State, distribution::ThompsonMax, Single<State>>(depth, nb_iterations, free_memory);
   //let result = search_optional::<State, distribution::ThompsonMax, Single<State>>(depth, nb_iterations);
   println!("Result obtained in {} iterations: {}", nb_iterations, result);
}

/*
   10000 iter
   base memory use: 275Mo (211 on paper)
   no formula memory use: 175Mo
   no formula boxed: 145Mo (127 on paper)
   no formula, box to node: 137Mo (103 on paper)
   37% of trees are nodes with this grammar
*/
