Feature: Keep user-facing examples executable
  Every fenced example in a user-facing document is loaded from its published
  source and run as part of the test suite.

  Scenario: README examples run from published text
    Given the published README
    When its documented examples run
    Then every example passes and has a unique identifier

  Scenario: User guide examples run from published text
    Given the published user's guide
    When its documented examples run
    Then every example passes and has a unique identifier
