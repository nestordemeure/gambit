#![feature(slice_patterns)]

mod explore;
mod grammar;
mod distribution;
mod result;

mod test;
use explore::search;
use distribution::thompson_max::ThompsonMax;
use test::grammar2019::State;
use result::single::SingleResult;

fn main()
{
   let depth = 4;
   let nb_iterations = 100_000;
   let result = search::<State, ThompsonMax, SingleResult<State>>(depth, nb_iterations);
   println!("Reached {:?} in {} iterations.", result, nb_iterations);
}
