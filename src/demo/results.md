# Demo

There is two way to confirm that we are better than the state of the art :
- Run our code and the state of the art methods on the same machine, the same examples and, if possible, the programming language (or use a proxy such as number of calls to the evaluation function).
However this requires a huge compatibility between all methods or the ability to reimplement them and use proper parameters (which is highly non trivial for genetic algorithms).
- Run our code on problem solved using methods from the state of the art and show that we are better (run faster for equivalent results or provide better results all with less or simpler parameters).

One can get datasets from [archive.ics.uci.edu](https://archive.ics.uci.edu/ml/datasets.php).
Running different methods on a regression or feature learning task for all suitable datasets would be a good way to test a method.

Here is a compilation of interesting examples that have been or might be explored.

## Kepler's third law

Rederive Kepler's formula from the periods and distances.
It is interesting to note that we find formulas that have a smaller error than the actual correct formula (probably due to the low precision of our input data).

error: 0.004013
period = sqrt(distance^3)

error: **-0.002402**
period = sqrt(((distance ^ 3) + sin((1 - sin((cos(distance) + distance))))))

error: -0.002729
period = sqrt(distance^3 + cos(distance + cos(sin(3) - distance)))

## Prime generating polynomials

Searching for formulas that produces primes number for all integers from 0 to score.
We accept negativ primes but refuse a polynomial that would raise the same value twice.

See [Predicting Prime Numbers Using Cartesian Genetic Programming]() for a reference tackling the problem with genetic programming and [Wolfram:Prime-Generating Polynomial](http://mathworld.wolfram.com/Prime-GeneratingPolynomial.html) for a table of results (which might not be up to date, I should check the publication referencing those papers).
If the table is up to date, we have found a polynomial that does as well as the best known polynomial while having only integer coeffiscients (and no fractions).

prime number: **56**
formula: (-1 + (x + (((-31 + x) * (-24 + (x + (x + -24)))) + x)))
simplified: 2*x² - 108*x + 1487

prime number: 55
formula: (-19 + ((x + (1 + -19)) * (x + (x + (9 * -8))))) 
simplified: 2x² -108x + 1277

## 2019

Find a formula that equal 2019 and is made only of '1', '*' and '+' (the shortest one uses 23 '1').
The problem is surprisingly hard to solve but the algorithm ends up converging to a solution.

## Regexp

Search for a regexp able to match numbers from a set of building blocks (comma, sign, digits, optional group and the concatenation of groups) that needs to be properly combined.

The reward function is the number of examples (from a set of matching and non matching strings) it gets rights.
The sparsity of the reward function (which could be improved uppon by adding more examples and insuring that they cover a wide array of incorrect statements) makes the problem ill-funded for our approach.

Nevertheless, most of the time, the algorithm converges on the regexp a human would write and satisfies all examples : `[[+|-]]?[0-9]+[,[0-9]+]?`

## Iris dataset Representation learning

We want to fund a set of coordinates such that labeled data are easily separable in those coordinates.

As an evaluation function, we count the number of points that are strictly closer to their label's mean than to the mean of any other label.
As this scoring function is binary, it might be low on informations to orient the search if there are few datapoints.
(I experimented with a different evaluation function that tries to quantify the quality of a result even if it is imperfect but the results were worse)

Another good simple classifier would be a gaussian classifier (that first a gaussian on each label and then associate a label with the closest gaussian).
This classifier can deal with non spherical labels and spread-up points.
We could then maximize `proba_correct_label - sum proba_incorrect_label` (sum or max).

missclassified points: **1/150**
formula: (petal_width, petal_length^2 / (sqrt(sepal_width) * sepal_length))

It is notable that one of the coordinates is usually much simpler than the other.
Evolving the coordinates one after the other might procude better results.

We are obviously overfitting here.
A nice property of the pareto front is that it keeps simpler (and hopeffuly more general) but less efficient formula.
In a realistic settup, we would validate our pareto front on a validation set in order to remove overfitted propositions (they are expected to lose their advantages once the dataset is changed).

## Learn a neural network architecture

[DeepSwarm](https://github.com/Pattio/DeepSwarm) uses Ant Colony Optimisation to search for a good neural network architecture. It should not be too hard to use our alforithm instead.

## Distilling Free-Form Natural Laws from Experimental Data

They propose a way to find natural law in data taken from physical systems (which is very nice).
https://science.sciencemag.org/content/324/5923/81.full

They give their data and runtime.

We could reproduce their results if we have an eval function that looks at partial derivativs.

## biological logical circuits

I could reimplement the concept with monte carlo tree search instead of genetic algorithm.
It is a nice example (and could examplify why the grammar is useful) but it brings little more.
