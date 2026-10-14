// SPDX-License-Identifier: GPL-2.0-only
//! Complete candidate isolation is retained alongside decisions, including rejected roots.
use super::super::RootIsolation;
use super::{
    membership_certificate::MembershipCertificate, polynomial::ExactCoefficientPolynomial,
};
pub(super) struct CandidateCertificate {
    pub polynomial: ExactCoefficientPolynomial,
    pub isolation: RootIsolation,
    pub membership: MembershipCertificate,
}
impl CandidateCertificate {
    pub fn new(
        polynomial: ExactCoefficientPolynomial,
        isolation: RootIsolation,
        membership: MembershipCertificate,
    ) -> Self {
        Self {
            polynomial,
            isolation,
            membership,
        }
    }
}
