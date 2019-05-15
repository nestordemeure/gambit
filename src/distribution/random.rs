
use super::Distribution;
use rand::Rng;

/// does not store informations
pub struct RandomDecision {}

impl Distribution for RandomDecision
{
   fn new() -> RandomDecision
   {
      RandomDecision {}
   }

   fn update(&mut self, _score_opt: Option<f64>) {}

   /// returns a random score
   fn score<RNG: Rng>(&self, _default_distribution: &RandomDecision, rng: &mut RNG) -> f64
   {
      rng.gen()
   }
}