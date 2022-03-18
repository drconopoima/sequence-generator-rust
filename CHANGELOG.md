# Changelog

## 0.3.0

* Replace chrono with time ^0.3. Addresses [RUSTSEC-2020-0071](https://rustsec.org/advisories/RUSTSEC-2020-0071.html): Potential Segfault in Time Crate.
* Fix parameter precedence: Command-line values take higher precedence to .env and environment variables.
* Document detail on benchmarking procedure.
