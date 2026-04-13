//! Projects Entity Definitions

use atlas_core::schema::SchemaBuilder;
use atlas_core::schema::WorkflowBuilder;
use atlas_shared::EntityDefinition;

/// Task entity with workflow
pub fn task_definition() -> EntityDefinition {
    let workflow = WorkflowBuilder::new("task_workflow", "todo")
        .initial_state("todo", "To Do")
        .working_state("in_progress", "In Progress")
        .working_state("in_review", "In Review")
        .final_state("done", "Done")
        .final_state("cancelled", "Cancelled")
        .transition("todo", "in_progress", "start")
        .transition("in_progress", "in_review", "submit_review")
        .transition("in_review", "done", "approve")
        .transition("in_review", "in_progress", "request_changes")
        .transition("todo", "cancelled", "cancel")
        .transition("in_progress", "cancelled", "cancel")
        .build();

    SchemaBuilder::new("tasks", "Task")
        .plural_label("Tasks")
        .table_name("proj_tasks")
        .description("Project tasks")
        .icon("check-square")
        .required_string("title", "Title")
        .rich_text("description", "Description")
        .reference("project_id", "Project", "projects")
        .reference("assignee_id", "Assignee", "employees")
        .reference("parent_task_id", "Parent Task", "tasks")
        .enumeration("priority", "Priority", vec![
            "critical", "high", "medium", "low"
        ])
        .enumeration("task_type", "Type", vec![
            "task", "bug", "feature", "improvement", "story"
        ])
        .integer("estimated_hours", "Estimated Hours")
        .integer("actual_hours", "Actual Hours")
        .date("due_date", "Due Date")
        .date("start_date", "Start Date")
        .integer("progress", "Progress %")
        .workflow(workflow)
        .build()
}

/// Timesheet entity
pub fn timesheet_definition() -> EntityDefinition {
    let workflow = WorkflowBuilder::new("timesheet_workflow", "draft")
        .initial_state("draft", "Draft")
        .working_state("submitted", "Submitted")
        .final_state("approved", "Approved")
        .final_state("rejected", "Rejected")
        .transition("draft", "submitted", "submit")
        .transition("submitted", "approved", "approve")
        .transition("submitted", "rejected", "reject")
        .transition("rejected", "draft", "revise")
        .build();

    SchemaBuilder::new("timesheets", "Timesheet")
        .plural_label("Timesheets")
        .table_name("proj_timesheets")
        .description("Employee timesheets")
        .icon("clock")
        .reference("employee_id", "Employee", "employees")
        .reference("project_id", "Project", "projects")
        .reference("task_id", "Task", "tasks")
        .date("date", "Date")
        .decimal("hours", "Hours", 5, 2)
        .string("description", "Description")
        .enumeration("entry_type", "Type", vec![
            "regular", "overtime", "holiday", "sick", "vacation"
        ])
        .boolean("billable", "Billable")
        .workflow(workflow)
        .build()
}

/// Milestone entity
pub fn milestone_definition() -> EntityDefinition {
    SchemaBuilder::new("milestones", "Milestone")
        .plural_label("Milestones")
        .table_name("proj_milestones")
        .description("Project milestones")
        .icon("flag")
        .required_string("name", "Milestone Name")
        .reference("project_id", "Project", "projects")
        .date("target_date", "Target Date")
        .date("actual_date", "Actual Date")
        .enumeration("status", "Status", vec![
            "planned", "in_progress", "completed", "overdue"
        ])
        .currency("budget", "Milestone Budget", "USD")
        .rich_text("description", "Description")
        .build()
}
