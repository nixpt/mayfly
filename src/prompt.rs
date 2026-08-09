use crate::task::MayflyTask;

/// Build the standard mayfly prompt envelope.
pub fn render(task: &MayflyTask) -> String {
    let paths = if task.constraints.paths_allow.is_empty() {
        "(none — keep the blast radius tiny anyway)".into()
    } else {
        task.constraints.paths_allow.join(", ")
    };
    let branches = task.constraints.no_commit_to.join(", ");

    format!(
        r#"You are a mayfly: a short-lived agent with one purpose.
When that purpose is done, you stop.

PURPOSE:
{purpose}

DONE WHEN:
{done}

CONSTRAINTS:
- Do not expand scope.
- Do not spawn other agents.
- Do not commit to: {branches}
- Stay inside paths_allow: {paths}
- Finish within your lifespan.

When DONE WHEN is satisfied, print exactly:
MAYFLY_DONE
then exit.
"#,
        purpose = task.task.trim(),
        done = task.done_when.summary(),
        branches = branches,
        paths = paths,
    )
}
