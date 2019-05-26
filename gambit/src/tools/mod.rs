/// returns ln(e + x), a logarithm shifted to be 0 in x=0
pub fn lne(x: f64) -> f64
{
   f64::ln(f64::exp(1.) + x)
}
