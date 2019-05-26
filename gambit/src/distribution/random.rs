use super::Distribution;
use rand::Rng;

/// does not store informations
#[derive(Clone)]
pub struct RandomSearch {}

impl Distribution for RandomSearch
{
   type ScoreType = f64;

   fn new() -> RandomSearch
   {
      RandomSearch {}
   }
   
   /// returns a dummy value
   fn nb_visit(&self) -> u64
   {
      1
   }

   fn update(&mut self, _score: Self::ScoreType) {}

   /// returns a random score
   fn score<RNG: Rng>(&self, _default_distribution: &RandomSearch, rng: &mut RNG) -> f64
   {
      rng.gen()
   }
}