# Changelog

## 0.4.0
* bugfix: .env-example suggested SIGN_BITS instead of UNUSED_BITS, but it didn't work, added env variable support for both and additional CLI long parameter "--sign-bits"  
* feature: Input parameter validation:
  *  --sequence-bits, --node-id-bits maximum value 16.
  *  --sequence-bits, --node-id-bits minimum value 1.
  *  --unused-bits, --sign-bits maximum value 8.
* feature: new CLI short parameters
  * --number = "-n", --micros-ten-power = "-m", --node-id-bits = "-w" (as in '[w]orker bits'), --sequence-bits = "-s", --node-id = "-i", --unused-bits = "-u"
* feature: new long parameter equivalent to --number: "--quantity", and short "-q"
* feature: input validation, output warning message and exit gracefully if a number/quantity of zero ("0") ids is requested
* feature: ERROR if conflicting pairs of parameters are given: --number & --quantity or --unused-bits & --sign-bits
* feature: fully populate help messages along with values for defaults, minimum, maximum and ranges. 
* feature: Timer for --debug option now calculates directly time per generated id alongside total running time.
* feature: changed default minimum cooldown to 200 ns (down from 1500ns). 

## 0.3.0

* Replace chrono with time ^0.3. Addresses [RUSTSEC-2020-0071](https://rustsec.org/advisories/RUSTSEC-2020-0071.html): Potential Segfault in Time Crate.
* Fix parameter precedence: Command-line values take higher precedence to .env and environment variables.
* Document detail on benchmarking procedure.
