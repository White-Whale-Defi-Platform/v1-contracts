# Memory

The memory contract represents an abstraction around the use and storage of contract and asset addresses.
The contract has two internal maps and provided two types of Raw query calls. These are methods implemented in the Memory struct.
With this request-response model around addresses and assets we gain a small piece of assurance against human error such
as mistyped addresses as well as gaining the ability to have many dapps requesting asset info from a common source.
We are working with external public partners to enshure the registered addresses are correct.


# Tests
The test cases covered by this contract are located in [the README file under src/tests/](src/tests/README.md).
