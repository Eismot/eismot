# Complete

**15 / 15. Final prize: $1,000,000**

## Final answer: C / Arrays embedded in structures had no convenient place for a separately initialized base pointer

Earlier array semantics materialized a pointer cell. Adding arrays inside structures raised storage and initialization problems. Ritchie instead generated the pointer when an array appeared in an expression. Modern C retains array-to-pointer conversion with specified exceptions; arrays and pointers are still distinct types.

[Source](https://www.nokia.com/bell-labs/about/dennis-m-ritchie/chist.html)

[Restart](q01.md) / [Games](../../README.md)
