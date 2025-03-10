use crate::{GitHubContext, utils};
use serde_yaml::Value;
use std::collections::BTreeMap;
use std::path::Path;

use serde_yaml::Value;

use crate::GitHubContext;

/// Representation of a job loaded from the `src/ci/github-actions/jobs.yml` file.
#[derive(serde::Deserialize, Debug, Clone)]
pub struct Job {
    /// Name of the job, e.g. mingw-check
    pub name: String,
    /// GitHub runner on which the job should be executed
    pub os: String,
    pub env: BTreeMap<String, Value>,
    /// Should the job be only executed on a specific channel?
    #[serde(default)]
    pub only_on_channel: Option<String>,
    /// Do not cancel the whole workflow if this job fails.
    #[serde(default)]
    pub continue_on_error: Option<bool>,
    /// Free additional disk space in the job, by removing unused packages.
    #[serde(default)]
    pub free_disk: Option<bool>,
}

impl Job {
    /// By default, the Docker image of a job is based on its name.
    /// However, it can be overridden by its IMAGE environment variable.
    pub fn image(&self) -> String {
        self.env
            .get("IMAGE")
            .map(|v| v.as_str().expect("IMAGE value should be a string").to_string())
            .unwrap_or_else(|| self.name.clone())
    }

    fn is_linux(&self) -> bool {
        self.os.contains("ubuntu")
    }
}

#[derive(serde::Deserialize, Debug)]
struct JobEnvironments {
    #[serde(rename = "pr")]
    pr_env: BTreeMap<String, Value>,
    #[serde(rename = "try")]
    try_env: BTreeMap<String, Value>,
    #[serde(rename = "auto")]
    auto_env: BTreeMap<String, Value>,
}

#[derive(serde::Deserialize, Debug)]
pub struct JobDatabase {
    #[serde(rename = "pr")]
    pub pr_jobs: Vec<Job>,
    #[serde(rename = "try")]
    pub try_jobs: Vec<Job>,
    #[serde(rename = "auto")]
    pub auto_jobs: Vec<Job>,

    /// Shared environments for the individual run types.
    envs: JobEnvironments,
}

impl JobDatabase {
    fn find_auto_job_by_name(&self, name: &str) -> Option<Job> {
        self.auto_jobs.iter().find(|j| j.name == name).cloned()
    }
}

pub fn load_job_db(path: &Path) -> anyhow::Result<JobDatabase> {
    let db = utils::read_to_string(path)?;
    let mut db: Value = serde_yaml::from_str(&db)?;

    // We need to expand merge keys (<<), because serde_yaml can't deal with them
    // `apply_merge` only applies the merge once, so do it a few times to unwrap nested merges.
    db.apply_merge()?;
    db.apply_merge()?;

    let db: JobDatabase = serde_yaml::from_value(db)?;
    Ok(db)
}

/// Representation of a job outputted to a GitHub Actions workflow.
#[derive(serde::Serialize, Debug)]
struct GithubActionsJob {
    /// The main identifier of the job, used by CI scripts to determine what should be executed.
    name: String,
    /// Helper label displayed in GitHub Actions interface, containing the job name and a run type
    /// prefix (PR/try/auto).
    full_name: String,
    os: String,
    env: BTreeMap<String, serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    continue_on_error: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    free_disk: Option<bool>,
}

/// Skip CI jobs that are not supposed to be executed on the given `channel`.
fn skip_jobs(jobs: Vec<Job>, channel: &str) -> Vec<Job> {
    jobs.into_iter()
        .filter(|job| {
            job.only_on_channel.is_none() || job.only_on_channel.as_deref() == Some(channel)
        })
        .collect()
}

/// Type of workflow that is being executed on CI
#[derive(Debug)]
pub enum RunType {
    /// Workflows that run after a push to a PR branch
    PullRequest,
    /// Try run started with @bors try
    TryJob { custom_jobs: Option<Vec<String>> },
    /// Merge attempt workflow
    AutoJob,
}

/// Maximum number of custom try jobs that can be requested in a single
/// `@bors try` request.
const MAX_TRY_JOBS_COUNT: usize = 20;

fn calculate_jobs(
    run_type: &RunType,
    db: &JobDatabase,
    channel: &str,
) -> anyhow::Result<Vec<GithubActionsJob>> {
    let (jobs, prefix, base_env) = match run_type {
        RunType::PullRequest => (db.pr_jobs.clone(), "PR", &db.envs.pr_env),
        RunType::TryJob { custom_jobs } => {
            let jobs = if let Some(custom_jobs) = custom_jobs {
                if custom_jobs.len() > MAX_TRY_JOBS_COUNT {
                    return Err(anyhow::anyhow!(
                        "It is only possible to schedule up to {MAX_TRY_JOBS_COUNT} custom jobs, received {} custom jobs",
                        custom_jobs.len()
                    ));
                }

                let mut jobs = vec![];
                let mut unknown_jobs = vec![];
                for custom_job in custom_jobs {
                    if let Some(job) = db.find_auto_job_by_name(custom_job) {
                        jobs.push(job);
                    } else {
                        unknown_jobs.push(custom_job.clone());
                    }
                }
                if !unknown_jobs.is_empty() {
                    return Err(anyhow::anyhow!(
                        "Custom job(s) `{}` not found in auto jobs",
                        unknown_jobs.join(", ")
                    ));
                }
                jobs
            } else {
                db.try_jobs.clone()
            };
            (jobs, "try", &db.envs.try_env)
        }
        RunType::AutoJob => (db.auto_jobs.clone(), "auto", &db.envs.auto_env),
    };
    let jobs = skip_jobs(jobs, channel);
    let jobs = jobs
        .into_iter()
        .map(|job| {
            let mut env: BTreeMap<String, serde_json::Value> = crate::yaml_map_to_json(base_env);
            env.extend(crate::yaml_map_to_json(&job.env));
            let full_name = format!("{prefix} - {}", job.name);

            GithubActionsJob {
                name: job.name,
                full_name,
                os: job.os,
                env,
                continue_on_error: job.continue_on_error,
                free_disk: job.free_disk,
            }
        })
        .collect();

    Ok(jobs)
}

pub fn calculate_job_matrix(
    db: JobDatabase,
    gh_ctx: GitHubContext,
    channel: &str,
) -> anyhow::Result<()> {
    let run_type = gh_ctx.get_run_type().ok_or_else(|| {
        anyhow::anyhow!("Cannot determine the type of workflow that is being executed")
    })?;
    eprintln!("Run type: {run_type:?}");

    let jobs = calculate_jobs(&run_type, &db, channel)?;
    if jobs.is_empty() {
        return Err(anyhow::anyhow!("Computed job list is empty"));
    }

    let run_type = match run_type {
        RunType::PullRequest => "pr",
        RunType::TryJob { .. } => "try",
        RunType::AutoJob => "auto",
    };

    eprintln!("Output");
    eprintln!("jobs={jobs:?}");
    eprintln!("run_type={run_type}");
    println!("jobs={}", serde_json::to_string(&jobs)?);
    println!("run_type={run_type}");

    Ok(())
}

pub fn find_linux_job<'a>(jobs: &'a [Job], name: &str) -> anyhow::Result<&'a Job> {
    let Some(job) = jobs.iter().find(|j| j.name == name) else {
        let available_jobs: Vec<&Job> = jobs.iter().filter(|j| j.is_linux()).collect();
        let mut available_jobs =
            available_jobs.iter().map(|j| j.name.to_string()).collect::<Vec<_>>();
        available_jobs.sort();
        return Err(anyhow::anyhow!(
            "Job {name} not found. The following jobs are available:\n{}",
            available_jobs.join(", ")
        ));
    };
    if !job.is_linux() {
        return Err(anyhow::anyhow!("Only Linux jobs can be executed locally"));
    }

    Ok(job)
}
