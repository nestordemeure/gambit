# Gambit

**This is a work in progress.**

## Symbolic regression using Monte-Carlo tree search

Gambit is a Rust library to do [Symbolic regression](https://en.wikipedia.org/wiki/Symbolic_regression) using [Monte-Carlo tree search](http://mcts.ai/about/).

In practice, given a grammar and a way to evaluate a formula that respects the grammar, it can optimise structures such as :

- a mathematical formula
- a program
- the architecture of an artificial neural network
- ...

The classical approach to solve this problem is to use [Genetic algorithms](https://en.wikipedia.org/wiki/Genetic_algorithm), however they tend to be very sensitiv to their parameters and (as a consequence) slow.

## Example

*TODO*

## How does it work

*TODO*

## References

The interface has been influenced by the very good [gramEvol](https://github.com/fnoorian/gramEvol) R package (which uses genetic algorithm in order to perform symbolic regression).

Tristan Cazenave previously explored the use of Monte-Carlo tree search to perform symbolic regression.
However, in his [work](https://www.lamsade.dauphine.fr/~cazenave/papers/MCExpression.pdf), he showed that, while promising, the [UCT algorithm](http://mcts.ai/about/) performs poorly for the task.
We solve the problem with an alternativ algorithm that perform significantly better on our benchmarks.

