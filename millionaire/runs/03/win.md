# Congratulations!

<picture><source media="(prefers-color-scheme: dark)" srcset="../../../assets/trivia-victory-dark.svg"><source media="(prefers-color-scheme: light)" srcset="../../../assets/trivia-victory.svg"><img src="../../../assets/trivia-victory.svg" alt="Congratulations! 15 of 15 answered. Final prize: $1,000,000."></picture>

**15 / 15. Final prize: $1,000,000**

All five difficulty tiers cleared. No more questions. Just bragging rights.

[Play again](q01.md) / [Choose another run](../../../README.md)

<details>
<summary>Final answer &amp; source</summary>

**A. An arbitrary value in the register's value domain**

Lamport emphasizes that his original algorithm tolerates arbitrary values from reads overlapping writes in its register model, with each location written by one process. Non-overlapping reads still have specified behavior. This mathematical model does not make unsynchronized non-atomic C++ accesses legal; those can have undefined behavior.

[Source](https://lamport.azurewebsites.net/pubs/pubs.html)

</details>

---

[![MIT license](https://img.shields.io/badge/License-MIT-116329?logo=opensourceinitiative&logoColor=white)](../../../LICENSE)
[![Minimum Rust version: 1.81](https://img.shields.io/badge/Rust-1.81%2B-000000?logo=rust&logoColor=white)](../../../Cargo.toml)
