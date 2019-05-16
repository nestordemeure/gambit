
use super::Distribution;
use rand::Rng;

/// does not store informations
pub struct RandomSearch {}

impl Distribution for RandomSearch
{
   type ScoreType = Option<f64>;
   
   fn new() -> RandomSearch
   {
      RandomSearch {}
   }

   fn update(&mut self, _score: Self::ScoreType) {}

   /// returns a random score
   fn score<RNG: Rng>(&self, _default_distribution: &RandomSearch, rng: &mut RNG) -> f64
   {
      rng.gen()
   }
}