// SPDX-License-Identifier: GPL-2.0-only
//! Keep exact predicates and aligned signs so a rejected projected root remains explainable.
use super::super::signs::RootSignEvaluation;
use super::polynomial::ExactCoefficientPolynomial;
use std::cmp::Ordering;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum MembershipRule {
    Zero,
    Nonzero,
    Nonnegative,
    Nonpositive,
    Positive,
}
impl MembershipRule {
    pub fn accepts(self, sign: Ordering) -> bool {
        match self {
            Self::Zero => sign == Ordering::Equal,
            Self::Nonzero => sign != Ordering::Equal,
            Self::Nonnegative => sign != Ordering::Less,
            Self::Nonpositive => sign != Ordering::Greater,
            Self::Positive => sign == Ordering::Greater,
        }
    }
}
pub(super) struct MembershipQuery {
    pub polynomial: ExactCoefficientPolynomial,
    pub rule: MembershipRule,
}
pub(super) struct QueryCertificate {
    pub query: MembershipQuery,
    pub evaluation: RootSignEvaluation,
}
pub(super) struct MembershipCertificate {
    pub accepted: Vec<bool>,
    pub queries: Vec<QueryCertificate>,
}
impl MembershipCertificate {
    pub fn new(count: usize) -> Self {
        Self {
            accepted: vec![true; count],
            queries: Vec::new(),
        }
    }
    pub fn append(&mut self, query: MembershipQuery, evaluation: RootSignEvaluation) {
        for (accepted, sign) in self.accepted.iter_mut().zip(evaluation.signs()) {
            *accepted &= query.rule.accepts(*sign);
        }
        self.queries.push(QueryCertificate { query, evaluation });
    }
}
impl MembershipQuery {
    pub fn new(polynomial: ExactCoefficientPolynomial, rule: MembershipRule) -> Self {
        Self { polynomial, rule }
    }
}
