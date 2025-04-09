use crate::project::Project;
use nixcode_macros::tool;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(JsonSchema, Serialize, Deserialize)]
pub struct CodeReviewPrompt {}

#[tool("Get code review instructions")]
pub async fn get_code_review_instructions(
    _: CodeReviewPrompt,
    _: Arc<Project>,
) -> serde_json::Value {
    let prompt = r#"Please act as an AI assistant performing a code review for a given GitHub Pull Request. Follow these steps precisely:

1.  **Understand the Context:** Begin by thoroughly reading and understanding the title and the full description of the provided GitHub Pull Request. Grasp the intended purpose and scope of the changes.
2.  **Analyze the Diff:** Carefully examine the code changes (the diff) introduced by this Pull Request compared to its designated base branch. Identify what has been added, removed, or modified.
3.  **Contextual Codebase Check:** Evaluate how the proposed changes fit within the broader context of the existing codebase. To perform this accurately, simulate the following local review process:
    * Assume you perform a `git pull` on the repository to get the latest state of the base branch.
    * Assume you then check out the specific branch associated with this Pull Request (`git checkout <pr-branch-name>`).
    * Continue your review analysis as if operating on this local checkout, specifically assessing whether the changes align with the project's established structure, architecture, and coding style conventions.
4.  **Review Focus Areas:** During your detailed analysis, prioritize the following key aspects:
    * **Correctness:** Verify that the implemented changes are functionally correct according to the PR description and achieve the intended goal without introducing regressions.
    * **Simplicity (KISS):** Assess adherence to the "Keep It Simple, Stupid" principle. Is the solution straightforward and avoids unnecessary complexity?
    * **Repetition (DRY):** Check for adherence to the "Don't Repeat Yourself" principle. Identify any redundant code patterns or logic that could be abstracted or refactored.
5.  **Provide Feedback:** After completing your review and formulating your conclusions, add a constructive comment directly to the Pull Request on GitHub:
    * If you identify specific issues or areas for improvement based on the criteria above (Correctness, KISS, DRY, style, architecture), clearly list each point.
    * Crucially, for *each* identified issue, provide a concrete suggestion detailing *how* the code could be improved or the problem resolved.
    * If the code is well-implemented, adheres to the principles, integrates well, and requires no changes, state this clearly in your comment, confirming approval from your review perspective."#;

    serde_json::json!(prompt)
}
