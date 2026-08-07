//! Objects, fields, types and validation, held as run time data.
//!
//! Two halves so far. [`field_type`] is the closed set of types an operator may
//! choose from, listed once, each row saying what it stores, what it accepts
//! and how it orders. [`definition`] is what an object and a field are, and the
//! rules a definition has to satisfy before anything tries to realise it.
//!
//! What is not here yet is named rather than left to be discovered. An object
//! definition is not a record with a history of its own, adding and renaming
//! and hiding are not separate operations with separate permissions, a picklist
//! value in use cannot be counted because nothing here reads records, and there
//! is no file a definition round trips through. Each of those needs a layer
//! that does not exist: #19 for the history, #21 for the permissions, and the
//! store for the counting. #17 stays open on them.

pub mod definition;
pub mod field_type;

pub use definition::{
    COLUMNS_PER_TABLE, FIELDS_PER_OBJECT, FieldDefinition, IDENTIFIER_CHARACTERS, ObjectDefinition,
    Refusal, Rule, TableSlots,
};
pub use field_type::{FIELD_TYPES, FieldType, Sort, Spec, Validation};
