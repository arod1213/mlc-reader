use crate::mutations::works::WorkInfo;

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

    let share_total = work.parties.iter().fold(0.0, |acc, x| {
        x.shares.iter().map(|x| x.share).sum::<f64>() + acc
    });

    // tolerance 0.5
    if share_total > 100.5 {
        errors.push(WorkError::Overclaim(share_total));
    } else if share_total < 99.5 {
        errors.push(WorkError::Underclaim(share_total));
    }
    // nest parties into hierachies
    // writer -> vec<Publisher>
    // sum shares based on writer?

    errors
}
