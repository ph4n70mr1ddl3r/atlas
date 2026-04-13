//! HCM Entity Markers
//! 
//! These types serve as markers for HCM entities.
//! The actual entity definitions are stored in the database
//! and loaded via the schema engine.

use atlas_core::schema::SchemaBuilder;
use atlas_shared::{EntityDefinition, FieldType};

/// Generate the Employee entity definition
pub fn employee_definition() -> EntityDefinition {
    SchemaBuilder::new("employees", "Employee")
        .plural_label("Employees")
        .table_name("hcm_employees")
        .description("Employee records")
        .icon("user")
        .string("employee_number", "Employee Number")
        .required_string("first_name", "First Name")
        .required_string("last_name", "Last Name")
        .email("email", "Email")
        .date("hire_date", "Hire Date")
        .reference("department_id", "Department", "departments")
        .reference("position_id", "Position", "positions")
        .reference("manager_id", "Manager", "employees")
        .enumeration("status", "Status", vec!["active", "inactive", "on_leave", "terminated"])
        .build()
}

/// Generate the Department entity definition
pub fn department_definition() -> EntityDefinition {
    SchemaBuilder::new("departments", "Department")
        .plural_label("Departments")
        .table_name("hcm_departments")
        .icon("building")
        .required_string("name", "Department Name")
        .required_string("code", "Code")
        .reference("parent_id", "Parent Department", "departments")
        .build()
}
