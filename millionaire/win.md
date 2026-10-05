# Complete

**15 / 15. Final prize: $1,000,000**

## Final answer: A / Equality of the current value hides an intervening change that may invalidate the thread's assumptions

Compare-and-swap checks the current value, not its history. A change away and back can invalidate assumptions even though the comparison succeeds. Version tags can detect changes when designed to avoid relevant wraparound; pointer-based structures may also need safe memory reclamation. Stronger memory ordering alone does not remove ABA.

[Source](https://en.wikipedia.org/wiki/ABA_problem)

[Restart](q01.md) / [Games](../README.md)
