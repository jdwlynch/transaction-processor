# transaction-processor
 The 'transaction-processor' is a toy financial transactions engine designed to explore 
 the intricacies of handling five different transaction types:
* Deposits
* Withdrawals
* Disputes
* Resolutions
* Chargebacks

The details of the rules surrounding these functions have been intentionally omitted for confidentiality.
## Overview
The transaction-processor is a fintech related simulation of common financial transactions. 
It is a Rust specific implementation primarily using the csv and serde, and rust decimal crates. Examples from these crates were used as the starting point.
The engine takes in a csv in the formatted as shown below:

|   types    |  client  |    tx    |  amount  |
|------------|----------|----------|----------|
| deposit    |    1     |    1     |   1.0    |
| deposit    |    2     |    2     |   3.0    |
| deposit    |    1     |    3     |   1.5    |
|withdrawal  |    1     |    4     |   1.5    |
|withdrawal  |    1     |    5     |   1.5    |

output is printed to std out in the following format:

|   client   |available |   held   |  total   |
|------------|----------|----------|----------|
| 1          |    3.4   |   1.0503 |   3.4503 |
| 2          |    2.1   |   0.0    |   2.1    |

## Usage

```ignore
$ cargo run --test input-file.csv < output-file.csv
```
See the test-inputs directory for sample input files.

## Assumptions
In addition to the defined transaction rules, the following assumptions were made:
* Disputes are only valid against deposits given the wording *clients available funds should decrease by the amount disputed*
* As long as transactions are well-formed and validated, extra dummy data is safe to ignore
* Accounts cannot be unlocked during the execution of this program
* Our partners will not try to overload our system, so it is safe to omit record length and number of record checks (assumed for simplicity and brevity)
* A ledger can't store dispute related transactions (dispute, resolve, chargeback), so they are outputted to INFO logs and can be stored if needed in future refinements.

## Testing
Test cases were documented privately and omitted for confidentiality. They are available on request.

Test driven development was a good candidate for this challenge, however in the interests of time, testing happened along-side development and afterward.
While there are public modules in the code, the key marker is output versus input. Depending on where this sits within a system
that testing could be viewed as black box integration testing or in the case of the challenge, isolated end-to-end testing.

While end-to-end testing has its drawbacks for large distributed systems, the primary deliverable of this exercise is correct and properly formatted output.
Therefore, testing output against input was prioritized. Test files in the directory are named against what they test. Errors are logged to stderr and log level
is configured through the RUSTLOG environment variable. Setting a value of error and info will demonstrate the test results.

Every case that fails generates ERROR output.

The type system is used extensively. Match statements prefer to exclude the _ case such that if items were added in the future and missed, the match
would catch this at compile time.

## Panic
Panic statements are used where it is impossible for the code to fail. This represents a critical malfunction of the transaction-processor which would
result in missed transactions, lost funds, or other serious faults. It is thought that a system with this kind of error should shut-down rather than
risk client accounts and funds.

## Errors
Errors are handled in error.rs using the thiserror crate. Errors are logged to stderr, and are explicitly checked for, caught, and logged.

## Future Work
Taking this forward as if it were a large scale production system, I would consider some imporvements not limited to the following:

* A pipeline to execute auditing, clippy checks, documentation checks, unit and integration testing
* Robust unit testing (input-output testing finishes the challenge, but is not maintainable)
* Asynchronously generating transactions actioned by client hash matching transaction processors (similar to Kafka) to remove the I/O bottleneck at scale
* Creating traits, at least for the transaction feed such that any source of transactions could implement the interface
* More familiarity with Rust design patterns as the processor became messier due to design errors which didn't agree with the borrow checker
* More strict input validation including record length and number of records
* Smarter csv validation using its errors more effectively, such as breaking the whole program if an improper header is found
* Improved explicit test names and more focused test files
* Add a unique ID to every transaction and include dispute related transactions in the ledger

Overall this was a fun, and enlightening project. The power of Rust is incredible, and coming from C++ very much appreciated.