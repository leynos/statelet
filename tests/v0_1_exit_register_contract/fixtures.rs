//! Canonical Markdown fixtures shared by v0.1 exit-register contract scenarios.

/// Builds the complete valid register used as a baseline for negative controls.
pub(super) fn valid_register() -> String {
    "<!-- exit-register:begin -->\n| B1 verdict | B2 verdict | Exit | Gate | Reachable |\n| --- | --- \
     | --- | --- | --- |\n| Falsified  | Falsified  | E1 ship nothing          | G2   | yes       |\n| \
     Falsified  | Held       | E1 ship nothing          | G2   | no        |\n| Held       | \
     Falsified  | E2 ship conventions only | G3   | yes       |\n| Held       | Held       | \
     E3 ship macro            | G3   | yes       |\n<!-- exit-register:end -->"
        .to_owned()
}
