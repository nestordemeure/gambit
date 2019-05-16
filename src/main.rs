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
   let nb_iterations = 1000;
   let result = search::<State, distribution::ThompsonMax, result::ParetoFront<State>>(depth, nb_iterations);
   println!("Result obtained in {} iterations: {}", nb_iterations, result);
}

/*
   NOTES:
   
   how to deal with illegal formulas
   using options lets me design things properly but it is a waste of time and memory if all formulas are legal
   the ideal would be to have the ability to go with one or the other depending on the grammar
   
   we could have an output type for the grammar which is either option<f64> or f64
   and priors for both
*/