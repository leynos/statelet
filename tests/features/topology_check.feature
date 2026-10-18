Feature: Preserve the pre-macro workspace boundary
  The topology check admits exactly one runtime package and no declared edge
  to the macro package, until the macro baseline gate admits one.

  Scenario: The initial runtime package passes
    Given metadata for the single statelet runtime package
    When the topology policy is checked
    Then the topology is accepted

  Scenario: An inactive renamed macro edge is still forbidden
    Given runtime metadata declaring an optional renamed statelet-macros dependency
    When the topology policy is checked
    Then the topology is rejected with the macro baseline gate explanation

  Scenario: A second workspace package needs a policy change
    Given metadata with an additional workspace member
    When the topology policy is checked
    Then the topology is rejected with a workspace expansion explanation
