use rand::Rng;

pub mod thompson_max;
pub use thompson_max::ThompsonMax;
pub mod ucb_tuned;
pub use ucb_tuned::UcbTuned;
pub mod random;
pub use random::RandomDecision;

pub trait Distribution
{
   /// returns a default distribution
   fn new() -> Self;

   /// adds a score to the distribution
   fn update(&mut self, score_opt: Option<f64>);

   /// produces a score from the distribution
   fn score<RNG: Rng>(&self, default_distribution: &Self, rng: &mut RNG) -> f64;
}