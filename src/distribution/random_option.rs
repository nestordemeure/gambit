
use super::Distribution;
use rand::Rng;

/// does not store informations
pub struct RandomSearchOption {}

impl Distribution for RandomSearchOption
{
   type ScoreType = Option<f64>;
   
   fn new() -> RandomSearchOption
   {
      RandomSearchOption {}
   }

   fn update(&mut self, _score: Self::ScoreType) {}

   /// returns a random score
   fn score<RNG: Rng>(&self, _default_distribution: &RandomSearchOption, rng: &mut RNG) -> f64
   {
      rng.gen()
   }
}