
use super::Distribution;
use rand::Rng;

/// stores information gotten during previous runs
pub struct ThompsonMax
{
   nb_score: u64,
   sum_scores: f64,
   max_score: f64
}

impl ThompsonMax
{
   /// uses the prior sample a potential score
   fn sample<RNG: Rng>(&self, rng: &mut RNG) -> f64
   {
      let e = f64::exp(1.);
      let k = self.nb_score as f64;
      let mean = self.sum_scores / k;
      let sup = f64::ln(k + e) * self.max_score;
      // TODO max > mean but max*log(k) could be < mean !!
      // rng.gen_range(mean, sup)
      mean + (sup - mean)*rng.gen::<f64>()
   }
}

impl Distribution for ThompsonMax
{
   type ScoreType = f64;
   
   /// returns a default, empty, prior
   fn new() -> ThompsonMax
   {
      ThompsonMax { nb_score: 0, sum_scores: 0., max_score: std::f64::NEG_INFINITY }
   }
   
   fn nb_visit(&self) -> u64
   {
      self.nb_score
   }

   /// adds a score to the prior
   fn update(&mut self, score: Self::ScoreType)
   {
      self.nb_score += 1;
      self.sum_scores += score;
      if score > self.max_score
      {
         self.max_score = score;
      }
   }

   /// gives a score to the node, we will take the node with the maximum score
   fn score<RNG: Rng>(&self, _default_distribution: &ThompsonMax, mut rng: &mut RNG) -> f64
   {
      if self.nb_score == 0
      {
         std::f64::INFINITY
      }
      else
      {
         self.sample(&mut rng)
      }
   }
}