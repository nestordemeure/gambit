# Symbolic regression
### Exploring **non** genetic algorithms for symbolic regression.

**This is the R&D branch. It contains F# code to quickly iterate on designs and formulas in order to test ideas and find what works and what doesn't.** 

## TODO

study distibution of rewards in [mean;maximum] as a function of heigh (can be computed for each formula that is evaluated).
I am expecting a shift linear in height from mean to max as height increase (wich would validate the ThompsonMax formula).
Confirming this hypothesis and characterizing the shift could lead to a better formula and would give credance to the existing formula.

## Design

### Specificities

This particular problem calls for some adaptations :
- Dealing with invalid formula.
- The branch choosing algorithm.
- A way to constrain memory usage.

### Choosing your own algorithm

I use record types to encapsulate both types and operations on them (a poor man trait system).

You can define your problem with grammar and a child selecion algorithm with context.

### Memory consumption

We currently keep tree for all explored state while classical monte carlo tree search only expand by a single node per call.
The classical approach might be a slight waste but it does help a lot in reducing memory consumption which would be needed for an implementation at scale.

Implement expand algorithm that expand a single new node (and discard the rest of the expansion) to keep memory usage constrained.
If memory is still a constraint, we could kill cold node when we need hot ones

See [Memory Bounded Monte Carlo Tree Search](http://mcts.ai/edpowley/papers/powley_aiide17.pdf) for a paper on the problem.
solutions are :
- limiting the height of the tree
- expanding a node only when it has been visited n times in order to slow down the expansion
- killing cold nodes

A way to kill cold node is to give an index to every node that associate it with the date of last visit (in nb iterations).
When we want to free some space, we *resorb* the node that has not been seen for the longuest time.
For that we need a way to resorb a node : just reexpand it and keep the old context (which can be done by mutating the childrens (one just need not to forget to delete their index from the table))
We can resorb the node tha maximize (time_since_last_visit, nb_visit) (nb_visit is there in order to target node closer to the root).

Traditional monte carlo expansion (building sequence at random until terminaison) cannot be used since it can led to accidentaly huge sequences (given enough iterations, it has a significant probability of crashing the memory).

## Experiments

### Grammar

The grammar (expand operation) has a huge impact on the results.
With no doubts, the more free/unconstrained a grammar is and the best the results will be.

### Once vs Depth

When we reach a leaf, we can complete the formula as soon as possible (*Once*) or continue at random until we reach the end or a given length (*Depth*).

*Depth* is either equivalent or better by an order of magnitude depending on the tests.
There seem to be a sweet spot to the depth (which maximizes performances), a good balance between exploring and wasting time by going too far.

*Once* seems to be a lot more sensitiv to the grammar formulation.

Overall *Once* seem to be a bad idea.

### Nested vs From root

The search can always start from the root (*From Root*) or we can, at regular interval (maxIter/targetFormulaMaxSize), set the root to its best children (*Nested*).
'Best' being the best in mean (which performs better than max).

As long as the Nesting does not appens too soon, it makes the search a lot faster (for a similar asymptotic result).
UCB seem to benefit more than thompson sampling, maybe because it is better at spotting the best branch in constrained time.

Pruning methods seem to be worth researching further.

### Random vs UCB vs Thompson

Thompson sampling tend to perform better than UCB which is better than random search only on half the examples.
However those differences are dwarfed by the other criteria (Once/Depth, Nested/FromRoot).

UCB1-Tuned has no C parameter (it is deduced from the empirical variance), does not require scaling of the mean and is much better than UCB.
Does does not affect it and it even manage to catch up with the stacked approach (making it the best contender so far) which implies that its criteria is efficient at prunning.

Using a normal law for Thompson sampling (instead of a uniform modeled after the mean) gives a small gain.
Using a normal law + optimistic sampling (best out of two) gives a sensible gain but we are still far from UCBTuned.
A normal law plus rejection sampling to get over mean+sd gives similar results (however, when we ask for 2*sd performances degrades below classical thompson sampling)
Using a uniform law on the maximum found in a child so far gives very go result (not competitiv with nested but noticeable)
A uniform law on [min;max] makes it competitv with nested search but not better.
A uniform law on [mean;max] makes it better than nested search but not UCB-tuned.
A uniform law on [mean;mean+k*sd] makes it less efficient.
Sampling from [mean;2*max] gets it start 'slow' as a nested search but later catch up to UCBTuned
(sampling from [0;2*max] is sampling from max with a width of max, sampling from mean;2*max could be explained in a similar fashion
and thus sampling from [max-sd;max+sd could work])
using the estimate from the frequentist german tank problem for [mean;max] makes it a bit worse than UCBtuned
sampling from the estimator for the german tank problem gives a curve that is close to the nested one but then loses
sampling from max+random*max_std does not work well
sampling from (mean+max / 2)+random*max_std gives results remarcably close to the random nested approach

overall mean seem to be useful when max is not reliable (few iterations) and vice versa (you want to go for max in the end game) -> which is precisely what UCBtuned already does

using a UCBtuned inspired thompson by with a uniform random second term is worse than nested

using thompson with a uniform value in [mean;log(nbValue)*max] gives competitiv results on both benchmarks wich is interesting.

A logical final step would be to metaoptimise the scoring function using symbolic regression to find a function that performs best on a maximum of benchmarks.

### Memory

#### Tests

I experimented no different techniques to reduce memory usage.

Expanding by only one nod per run does degrade slightly the quality of the algorithm but reduce sensibly (by a factor 2 or 3) the memory usage.

Expanding a node only after k visit is simple to implement (using an information that should be in most prior anyway) but it does not garantee that the tree will not explode.
In practice it slow the starts and degrades the result by what seem to be a factor proportional to k (just like expanding by only one node but multiplied by k) but it reduces the memory usage very efficiently.

Recycling nodes by cutting nodes that have not been visited for a long time seems to deeply degrade the quality.
I am unsure wether it is a bug (the implementation is, too, complex) or a consequence of the algorithm.
It, however, garantees that the memory usage will be constant which is nice.
An other implementation that always cut the oldest node show a much smaller degradation in quality (hitting at a bug in the previous implementation) but it still is sensibly worse.

Contracting as many node (selected with the argmin) as we create garantees a constant memory usage and should be easy to parallelize (which is not the case for recycling).
It starts faster than expanding after k visits (which is logical isnce it starts being identical to a classical algorithm) but later converge toward the same asymptot.

deleting nodes (starting with the one with the lowest mean) means that we will not losing time researching alreaddy dropped nodes.
However it degrades results significatively.

Deleting all main children but the best (similar to cazenave's nested search) when we hit our memory budget (we cannot do it later) lets us continue as long as we have not consummed the whole tree (producing formula that are longueur with time) but tends to quickly cut all improvements.

flattening nodes that are not terminal and have only nodes as children could reduce memory cunsumption by increasing the branch factor, it seem to silightly degrade performances on the kepler example but the memory gain quickly decreases with the size of the tree.

Currently, the biggest bang for the bucks would be to only expand a node after k visit (with k=1 as a default).
It let us use k=0 if we want no limits but put breakes on the system if needed

putting states options in the node instead of formulas in the leafs seem to slightly increase memory use.
it is counterintuitive since we expect to have as many nodes as leafs and thus state*n size vs formula*n size.
(but, since we are using an enum, leafs are just as heavy as nodes)

an idea is to have a limited number of nodes but to fuse some of them using a hash (two nodes are fused if a hash of the current formula give the same index for both).
in this setting the one armed bandit would have attrocious distributions to learn but we can garantee a fixed memory use while the performances should go from optimal to equivalent to a random search (when all distribution become uniform).
A limit is that, in this setting, managing the progressive expanding of the tree is tricky (which matters a lot to the performances).
using thompson sampling with a normal prior (which express the combinaison of many small influences), we get accetable results.

The simplest solution is maybe, when we hit our memory limit, to stop adding nodes.
We are then doing random searches as soon as we quit the leafs but we keep updating the prior in the previously expanded nodes.
Ideally we need to method to keep accepting longer formulas.

An idea, not yet tried, is to have only one node per state but to have a neural network trained (using each evaluation as we do them) to evaluate formulas containing non terminal (returning an evaluation and a margin of error).
To choose a child, we would use evaluate each potential child with the neural network and use its output to choose one.
(taking either the best or a sample the distribution for thompson sampling)
(some neural network can learn a distribution, it might be appropriate here)

#### The solution

The final solution is to do the search normaly as long as you have a memory budget.

Once your budget is spent, you stop creating new nodes but you keep exploring and updating the priors.

For a given tree, you can compute a growth factor which is 'average_formula_length / log(1 + nb_evaluations)'.
For each leaf, you memorise how many times it was visited which lets you compute how far you can explore it using the formula 'depth = growthFactor * log(1 + nb_visit)'.
(the growth factor is a function of the overall shape of the tree and lets us emulate the speed without an historic of expanded nodes)

As a side note, once in memory limited settings, one as to be careful no to give full priority to the exploration of leafs (as this could trap us in local minimums).

This approach gives us a fixed memory budget and the ability to keep learning potentially forever.
In practice, the learning does deteriorates as soon as you quit known nodes but the asymptotic behaviours stays good and the approach is competitiv with non memorylimited algorithms.

### Reward distribution

A corrected reward is a reward minus the mean before the reward (our expected return).
In this space getting a positi number is getting more than expected and vice versa.

A normalized reward is a corrected reward divided by (max-mean).
In this space 0 is what we expected and 1 getting the maximum so far.

When we observe the distribution of corrected rewards as a function of the number of visit in a node, we notice that (after a short period of initialisation) it is not a function of the number of visit.

When we observe the distribution of normalized reward, we notice a shift to the right (toward the max) which seem explainable by the fact that the system is not adversarial : each node underneath the current node is doing its best to help and produce a better result.
Thus, it might be interresting to study the distribution of the x-mean

