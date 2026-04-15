//! HCM Entity Definitions
//! 
//! Entity definitions for the Human Capital Management domain.
//! These are used to register entities in the schema engine.

use atlas_core::schema::{SchemaBuilder, WorkflowBuilder};
use atlas_shared::EntityDefinition;

/// Generate the Employee entity definition with lifecycle workflow
pub fn employee_definition() -> EntityDefinition {
    let workflow = WorkflowBuilder::new("employee_lifecycle", "onboarding")
        .initial_state("onboarding", "Onboarding")
        .working_state("active", "Active")
        .working_state("on_leave", "On Leave")
        .working_state("offboarding", "Offboarding")
        .final_state("terminated", "Terminated")
        .transition("onboarding", "active", "activate")
        .transition("active", "on_leave", "start_leave")
        .transition("on_leave", "active", "end_leave")
        .transition("active", "offboarding", "start_offboarding")
        .transition("active", "onboarding", "rehire")
        .transition("offboarding", "terminated", "terminate")
        .build();

    SchemaBuilder::new("employees", "Employee")
        .plural_label("Employees")
        .table_name("hcm_employees")
        .description("Employee records")
        .icon("user")
        .string("employee_number", "Employee Number")
        .required_string("first_name", "First Name")
        .required_string("last_name", "Last Name")
        .string("middle_name", "Middle Name")
        .email("email", "Email")
        .phone("phone", "Phone")
        .date("date_of_birth", "Date of Birth")
        .date("hire_date", "Hire Date")
        .date("termination_date", "Termination Date")
        .reference("department_id", "Department", "departments")
        .reference("position_id", "Position", "positions")
        .reference("manager_id", "Manager", "employees")
        .enumeration("employment_type", "Employment Type", vec![
            "full_time", "part_time", "contractor", "intern"
        ])
        .enumeration("status", "Status", vec!["active", "inactive", "on_leave", "terminated"])
        .address("address", "Address")
        .workflow(workflow)
        .build()
}

/// Generate the Department entity definition
pub fn department_definition() -> EntityDefinition {
    SchemaBuilder::new("departments", "Department")
        .plural_label("Departments")
        .table_name("hcm_departments")
        .description("Organizational departments")
        .icon("building")
        .required_string("name", "Department Name")
        .required_string("code", "Code")
        .reference("parent_id", "Parent Department", "departments")
        .reference("manager_id", "Manager", "employees")
        .string("cost_center", "Cost Center")
        .boolean_default("is_active", "Active", true)
        .build()
}

/// Generate the Position entity definition
pub fn position_definition() -> EntityDefinition {
    SchemaBuilder::new("positions", "Position")
        .plural_label("Positions")
        .table_name("hcm_positions")
        .description("Job positions within the organization")
        .icon("briefcase")
        .required_string("title", "Job Title")
        .required_string("code", "Code")
        .reference("department_id", "Department", "departments")
        .enumeration("level", "Job Level", vec![
            "entry", "mid", "senior", "lead", "manager", "director", "executive"
        ])
        .currency("min_salary", "Minimum Salary", "USD")
        .currency("max_salary", "Maximum Salary", "USD")
        .boolean_default("is_active", "Active", true)
        .build()
}
