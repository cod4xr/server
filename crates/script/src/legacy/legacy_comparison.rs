// SPDX-License-Identifier: AGPL-3.0-or-later
/// Legacy numeric comparisons; these deliberately differ from Rust float equality.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyComparison {
    /// Exact integer equality, or binary32 difference below the source's double
    /// 0.000001 threshold. Equal infinities/NaNs therefore compare false.
    Equal,
    /// Negates legacy equality, including its tolerance and non-finite behavior.
    NotEqual,
    /// Strict numeric less-than after legacy pair coercion.
    Less,
    /// Strict numeric greater-than after legacy pair coercion.
    Greater,
    /// Negates greater-than; unordered NaN comparisons therefore produce true.
    LessOrEqual,
    /// Negates less-than; unordered NaN comparisons therefore produce true.
    GreaterOrEqual,
}

impl LegacyComparison {
    pub(super) fn integers(self, left: i32, right: i32) -> bool {
        match self {
            Self::Equal => left == right,
            Self::NotEqual => left != right,
            Self::Less => left < right,
            Self::Greater => left > right,
            Self::LessOrEqual => left <= right,
            Self::GreaterOrEqual => left >= right,
        }
    }

    pub(super) fn floats(self, left: f32, right: f32) -> bool {
        match self {
            Self::Equal => float_equal(left, right),
            Self::NotEqual => !float_equal(left, right),
            Self::Less => left < right,
            Self::Greater => left > right,
            Self::LessOrEqual => {
                let greater = left > right;
                !greater // Source negates Scr_EvalGreater, including unordered operands.
            }
            Self::GreaterOrEqual => {
                let less = left < right;
                !less // Source negates Scr_EvalLess rather than using a direct >=.
            }
        }
    }
}

fn float_equal(left: f32, right: f32) -> bool {
    // Source subtracts in binary32, then promotes to double for fabs/threshold.
    // A binary32 1e-6 threshold would change the nearest boundary's result.
    f64::from((left - right).abs()) < 0.000_001
}
