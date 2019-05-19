
use super::Distribution;
use rand::Rng;

/// stores information gotten during previous runs
pub struct UcbTuned
{
   nb_score: u64,
   sum_scores: f64,
   sum_squared_score: f64
}

impl UcbTuned
{
   /// returns the mean score so far
   fn mean(&self) -> f64 
   {
      if self.nb_score == 0
      {
         std::f64::INFINITY
      }
      else
      {
         self.sum_scores / (self.nb_score as f64)
      }
   }
   
   /// returns the variance of the scores so far
   fn var(&self) -> f64 
   {
      if self.nb_score < 2 
      {
         0.
      } 
      else
      {
         let mean = self.mean();
         let var = self.sum_squared_score / (self.nb_score as f64 - 1.) - mean*mean;
         var.abs() // could be negativ due to numerical unstability
         // TODO https://en.wikipedia.org/wiki/Algorithms_for_calculating_variance
      }
   }
}

impl Distribution for UcbTuned
{
   type ScoreType = f64;
   
   /// returns a default, empty, distribution
   fn new() -> UcbTuned
   {
      UcbTuned { nb_score: 0, sum_scores: 0., sum_squared_score: 0. }
   }

   fn nb_visit(&self) -> u64
   {
      self.nb_score
   }

   /// adds a score to the distribution
   fn update(&mut self, score: Self::ScoreType)
   {
      self.nb_score += 1;
      self.sum_scores += score;
      self.sum_squared_score += score*score;
   }

   /// gives a score to the node, we will take the node with the maximum score
   fn score<RNG: Rng>(&self, default_distribution: &UcbTuned, mut _rng: &mut RNG) -> f64
   {
      if self.nb_score == 0
      {
         std::f64::INFINITY
      }
      else
      {
         let fathers_nb_visit = default_distribution.nb_score as f64;
         let child_nb_visit = self.nb_score as f64;
         let c = self.var() + (2. * fathers_nb_visit.ln() / child_nb_visit).sqrt();
         self.mean() + (c * fathers_nb_visit.ln() / child_nb_visit).sqrt()
      }
   }
}