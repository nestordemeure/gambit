#![feature(slice_patterns)]

mod explore;
mod grammar;
mod distribution;
mod result;

mod test;
use explore::search;
use test::grammar2019::State;
use grammar::Grammar;

fn main()
{
   let depth = 4;
   let nb_iterations = 100_000;
   let result = search::<State, distribution::ThompsonMax, result::Single<State>>(depth, nb_iterations);
   let score = result.score;
   let formula = result.formula;
   println!("Reached {} with '{}' in {} iterations.", score, State::to_string(&formula), nb_iterations);
}
