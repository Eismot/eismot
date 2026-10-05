# Complete

**15 / 15. Final prize: $1,000,000**

## Final answer: A / An arbitrary value in the register's value domain

Lamport emphasizes that his original algorithm tolerates arbitrary values from reads overlapping writes in its register model, with each location written by one process. Non-overlapping reads still have specified behavior. This mathematical model does not make unsynchronized non-atomic C++ accesses legal; those can have undefined behavior.

[Source](https://lamport.azurewebsites.net/pubs/pubs.html)

[Restart](q01.md) / [Games](../../../README.md)
