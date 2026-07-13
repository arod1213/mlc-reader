use crate::{mutations::works::WorkInfo, types::Party};

pub enum WorkError {
    Overclaim(f64),
    Underclaim(f64),
    MissingReleases,
    InDispute,
}

pub fn validate_work(work: WorkInfo) -> Vec<WorkError> {
    let mut errors = vec![];
    if work.releases.is_empty() {
        errors.push(WorkError::MissingReleases);
    }
    if work.in_dispute {
        errors.push(WorkError::InDispute);
    }
    // nest parties into hierachies
    // writer -> vec<Publisher>
    // sum shares based on writer?

    errors
}
