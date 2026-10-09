# Congratulations!

<picture><source media="(prefers-color-scheme: dark)" srcset="../../assets/trivia-victory-dark.svg"><source media="(prefers-color-scheme: light)" srcset="../../assets/trivia-victory.svg"><img src="../../assets/trivia-victory.svg" alt="Congratulations! 15 of 15 answered. Final prize: $1,000,000."></picture>

**15 / 15. Final prize: $1,000,000**

All five difficulty tiers cleared. No more questions. Just bragging rights.

[Play again](q01.md) / [Choose another run](../../README.md)

<details>
<summary>Final answer &amp; source</summary>

**C. Arrays embedded in structures had no convenient place for a separately initialized base pointer**

Earlier array semantics materialized a pointer cell. Adding arrays inside structures raised storage and initialization problems. Ritchie instead generated the pointer when an array appeared in an expression. Modern C retains array-to-pointer conversion with specified exceptions; arrays and pointers are still distinct types.

[Source](https://www.nokia.com/bell-labs/about/dennis-m-ritchie/chist.html)

</details>

---

[![MIT license](https://img.shields.io/badge/License-MIT-116329?logo=opensourceinitiative&logoColor=white)](../../LICENSE)
[![Minimum Rust version: 1.81](https://img.shields.io/badge/Rust-1.81%2B-000000?logo=rust&logoColor=white)](../../Cargo.toml)
