Feature: Transition tracing fields with ordinary instrument
  A downstream transition boundary instrumented with plain
  tracing::instrument records the contracted transition fields on its span.

  Scenario: A fallible transition that succeeds records its outcome
    Given a fallible boundary in state "Normalize" handling a "table_row" input
    When the boundary returns the outcome "stay"
    Then the boundary span records the transition name "handle_line"
    And the span records state "Normalize", event "table_row", and outcome "stay" as strings
    And the span does not record a transition error
    And the span records exactly the expected fields

  Scenario: A fallible transition that fails records its error category
    Given a fallible boundary in state "Normalize" handling a "table_row" input
    When the boundary returns the error category "unterminated_row"
    Then the boundary span records the transition error "unterminated_row" as a string
    And this fixture records no transition outcome
    And the span records exactly the expected fields

  Scenario: An infallible transition does not declare an error field
    Given an infallible boundary in state "TightCodeSpan" handling a "fence" input
    When the boundary returns the outcome "move"
    Then the span records state "TightCodeSpan", event "fence", and outcome "move" as strings
    And the span does not declare a transition error
    And the span records exactly the expected fields

  Scenario: An undeclared late field is lost
    Given a boundary that declares only the transition name
    When the boundary records the outcome "stay"
    Then the boundary span exists but does not record a transition outcome
    And the span records exactly the expected fields

  Scenario: An error propagated before recording leaves no error field
    Given a fallible boundary that propagates errors with the question-mark operator
    When the boundary fails before recording its error category
    Then the boundary returns an error
    And the boundary span exists but does not record a transition error
    And the span records exactly the expected fields
