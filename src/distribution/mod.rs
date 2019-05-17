use rand::Rng;

pub mod thompson_max;
pub mod ucb_tuned;
pub mod random;
pub mod thompson_max_option;
pub mod ucb_tuned_option;
pub mod random_option;

pub use thompson_max::ThompsonMax;
pub use ucb_tuned::UcbTuned;
pub use random::RandomSearch;
pub use thompson_max_option::ThompsonMaxOption;
pub use ucb_tuned_option::UcbTunedOption;
pub use random_option::RandomSearchOption;

pub trait Distribution
{
   type ScoreType;
   
   /// returns a default distribution
   fn new() -> Self;

   /// adds a score to the distribution
   fn update(&mut self, score: Self::ScoreType);

   /// produces a score from the distribution
   fn score<RNG: Rng>(&self, default_distribution: &Self, rng: &mut RNG) -> f64;
}
