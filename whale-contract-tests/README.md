# Whale Contract Tests

A small project to give almost end-to-end automated tests for Whale contracts

Rather than just testing a unit of code, these tests define a Scenario and a number of requirements for that scenario in plain english defined as steps. Each step may have some associated test code with it

The current framework used is `pytest` via the `pytest-bdd` package. Another alternative would be `behave`.

To run tests:

```bash
pytest tests/test_gov/test_governance.py --gherkin-terminal-reporter --feature tests/features -v
```

To run tests AND see the print statements output; (good for debug)

```bash
pytest tests/test_gov/test_governance.py -s
```

The option `--gherkin-terminal-reporter` is used to print out the features to the terminal as well as showing whether the tests pass.