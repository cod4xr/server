// SPDX-License-Identifier: AGPL-3.0-or-later
use cod4x_script::legacy::{
    LegacyComparison, LegacyNumber, LegacyVector, LegacyVectorError, LegacyVectorOperand,
    LegacyVectorOperation, LegacyVectorPair,
};

#[derive(Debug)]
pub enum EvaluationFailure {
    Domain(LegacyVectorError),
    Unexpected(String),
}

impl std::fmt::Display for EvaluationFailure {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Domain(error) => std::fmt::Display::fmt(error, formatter),
            Self::Unexpected(message) => formatter.write_str(message),
        }
    }
}

pub trait VectorEvaluator {
    fn index(
        object: LegacyVector,
        index: LegacyVectorOperand,
    ) -> Result<LegacyNumber, EvaluationFailure>;
    fn construct(values: [LegacyNumber; 3]) -> Result<LegacyVector, EvaluationFailure>;
    fn arithmetic(
        operation: LegacyVectorOperation,
        left: LegacyVectorOperand,
        right: LegacyVectorOperand,
    ) -> Result<LegacyVector, EvaluationFailure>;
    fn compare(
        operation: LegacyComparison,
        left: LegacyVectorOperand,
        right: LegacyVectorOperand,
    ) -> Result<bool, EvaluationFailure>;
}

pub struct HelperEvaluator;

impl VectorEvaluator for HelperEvaluator {
    fn index(
        object: LegacyVector,
        index: LegacyVectorOperand,
    ) -> Result<LegacyNumber, EvaluationFailure> {
        match index {
            LegacyVectorOperand::Number(index) => {
                object.index(index).map_err(EvaluationFailure::Domain)
            }
            LegacyVectorOperand::Vector(_) => Err(EvaluationFailure::Unexpected(
                "Standalone index helper accepts numeric index inputs only".into(),
            )),
        }
    }
    fn construct(values: [LegacyNumber; 3]) -> Result<LegacyVector, EvaluationFailure> {
        Ok(LegacyVector::from_numbers(values))
    }

    fn arithmetic(
        operation: LegacyVectorOperation,
        left: LegacyVectorOperand,
        right: LegacyVectorOperand,
    ) -> Result<LegacyVector, EvaluationFailure> {
        let pair = LegacyVectorPair::coerce(left, right).map_err(EvaluationFailure::Domain)?;
        operation.evaluate(pair).map_err(EvaluationFailure::Domain)
    }

    fn compare(
        operation: LegacyComparison,
        left: LegacyVectorOperand,
        right: LegacyVectorOperand,
    ) -> Result<bool, EvaluationFailure> {
        LegacyVectorPair::coerce(left, right)
            .and_then(|pair| pair.compare(operation))
            .map_err(EvaluationFailure::Domain)
    }
}
