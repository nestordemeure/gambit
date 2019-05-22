use super::Distribution;
use rand::Rng;

#[derive(Clone)]
pub struct Optional<Distr: Distribution>
{
   nb_visit: u64,
   distribution: Distr
}

impl<UnderlyingScoreType, Distr> Distribution for Optional<Distr>
   where Distr: Distribution<ScoreType = UnderlyingScoreType>
{
   type ScoreType = Option<UnderlyingScoreType>;

   fn new() -> Self
   {
      Optional { nb_visit: 0, distribution: Distr::new() }
   }

   fn nb_visit(&self) -> u64
   {
      self.nb_visit
   }

   fn update(&mut self, score_opt: Self::ScoreType)
   {
      self.nb_visit += 1;
      if let Some(score) = score_opt
      {
         self.distribution.update(score);
      }
   }

   /// returns a random score
   fn score<RNG: Rng>(&self, default_distribution: &Self, rng: &mut RNG) -> f64
   {
      let nb_score = self.distribution.nb_visit();
      let probability_valid_formula = rng.gen_ratio((nb_score + 1) as u32, (self.nb_visit + 2) as u32); // laplacian smoothing
      match probability_valid_formula
      {
         false => std::f64::NEG_INFINITY,
         true if nb_score == 0 => default_distribution.distribution.score(&default_distribution.distribution, rng),
         true => self.distribution.score(&default_distribution.distribution, rng)
      }
   }
}
