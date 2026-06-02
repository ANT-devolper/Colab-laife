//! DISC profile derivation. A DISC result is four dimension scores (executor = D,
//! communicator = I, planner = S, analyst = C). The primary profile is the highest
//! dimension and the secondary the second highest; we derive them at read time
//! rather than storing them. Pure (no I/O), so it is unit-testable without Docker.

/// The four DISC dimension scores.
pub struct DiscScores {
    /// Dominance (D).
    pub executor: i32,
    /// Influence (I).
    pub communicator: i32,
    /// Steadiness (S).
    pub planner: i32,
    /// Conscientiousness (C).
    pub analyst: i32,
}

/// The derived primary and secondary profiles, as dimension names.
#[derive(Debug, PartialEq, Eq)]
pub struct DiscProfile {
    pub primary: &'static str,
    pub secondary: &'static str,
}

/// Derives the primary (highest) and secondary (second highest) dimension. Ties
/// break by a fixed dimension order: executor, communicator, planner, analyst.
pub fn profile(scores: &DiscScores) -> DiscProfile {
    let mut dims = [
        ("executor", scores.executor),
        ("communicator", scores.communicator),
        ("planner", scores.planner),
        ("analyst", scores.analyst),
    ];
    // `sort_by` is stable, so equal scores keep the fixed order above.
    dims.sort_by(|a, b| b.1.cmp(&a.1));
    DiscProfile {
        primary: dims[0].0,
        secondary: dims[1].0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn picks_the_two_highest_dimensions() {
        let scores = DiscScores {
            executor: 10,
            communicator: 25,
            planner: 5,
            analyst: 18,
        };

        assert_eq!(
            profile(&scores),
            DiscProfile {
                primary: "communicator",
                secondary: "analyst",
            }
        );
    }

    #[test]
    fn the_top_dimension_can_be_executor() {
        let scores = DiscScores {
            executor: 30,
            communicator: 1,
            planner: 2,
            analyst: 3,
        };

        let result = profile(&scores);
        assert_eq!(result.primary, "executor");
        assert_eq!(result.secondary, "analyst");
    }

    #[test]
    fn ties_break_by_the_fixed_dimension_order() {
        // All equal: order is executor, communicator, planner, analyst.
        let scores = DiscScores {
            executor: 7,
            communicator: 7,
            planner: 7,
            analyst: 7,
        };

        assert_eq!(
            profile(&scores),
            DiscProfile {
                primary: "executor",
                secondary: "communicator",
            }
        );
    }
}
