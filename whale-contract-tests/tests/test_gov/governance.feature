Feature: Governance
    The contract used for voting on Polls and staking Whale token

    Scenario: Creating a Poll and Voting
        Given I'm a Whale token holder
        And I have staked whale

        When I attempt to create a simple text proposal
        And I submit the proposal 

        Then the Poll should be created
        And I should be able to vote on it

    Scenario: Staking Whale Tokens
        Given I'm a Whale token holder
        And I have some available whale

        When I attempt to stake some whale tokens to be able to vote on polls
        And I submit the staking tx

        Then I should be able to stake
        And I should be able to unstake the same amount
