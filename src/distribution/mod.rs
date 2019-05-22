pub mod thompson_max;
pub mod ucb_tuned;
pub mod random;
pub mod option;

use rand::Rng;
pub use thompson_max::ThompsonMax;
pub use ucb_tuned::UcbTuned;
pub use random::RandomSearch;
pub use option::Optional;

pub trait Distribution: Clone
{
   type ScoreType;

   /// returns a default distribution
   fn new() -> Self;

   /// returns the number of times a given node has been visited
   fn nb_visit(&self) -> u64;

   /// adds a score to the distribution
   fn update(&mut self, score: Self::ScoreType);

   /// produces a score from the distribution
   fn score<RNG: Rng>(&self, default_distribution: &Self, rng: &mut RNG) -> f64;
}
