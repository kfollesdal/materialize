// Copyright Materialize, Inc. and contributors. All rights reserved.
//
// Use of this software is governed by the Business Source License
// included in the LICENSE file.
//
// As of the Change Date specified in that file, in accordance with
// the Business Source License, use of this software will be governed
// by the Apache License, Version 2.0.

// BEGIN LINT CONFIG
// DO NOT EDIT. Automatically generated by bin/gen-lints.
// Have complaints about the noise? See the note in misc/python/materialize/cli/gen-lints.py first.
#![allow(unknown_lints)]
#![allow(clippy::style)]
#![allow(clippy::complexity)]
#![allow(clippy::large_enum_variant)]
#![allow(clippy::mutable_key_type)]
#![allow(clippy::stable_sort_primitive)]
#![allow(clippy::map_entry)]
#![allow(clippy::box_default)]
#![allow(clippy::drain_collect)]
#![warn(clippy::bool_comparison)]
#![warn(clippy::clone_on_ref_ptr)]
#![warn(clippy::no_effect)]
#![warn(clippy::unnecessary_unwrap)]
#![warn(clippy::dbg_macro)]
#![warn(clippy::todo)]
#![warn(clippy::wildcard_dependencies)]
#![warn(clippy::zero_prefixed_literal)]
#![warn(clippy::borrowed_box)]
#![warn(clippy::deref_addrof)]
#![warn(clippy::double_must_use)]
#![warn(clippy::double_parens)]
#![warn(clippy::extra_unused_lifetimes)]
#![warn(clippy::needless_borrow)]
#![warn(clippy::needless_question_mark)]
#![warn(clippy::needless_return)]
#![warn(clippy::redundant_pattern)]
#![warn(clippy::redundant_slicing)]
#![warn(clippy::redundant_static_lifetimes)]
#![warn(clippy::single_component_path_imports)]
#![warn(clippy::unnecessary_cast)]
#![warn(clippy::useless_asref)]
#![warn(clippy::useless_conversion)]
#![warn(clippy::builtin_type_shadow)]
#![warn(clippy::duplicate_underscore_argument)]
#![warn(clippy::double_neg)]
#![warn(clippy::unnecessary_mut_passed)]
#![warn(clippy::wildcard_in_or_patterns)]
#![warn(clippy::crosspointer_transmute)]
#![warn(clippy::excessive_precision)]
#![warn(clippy::overflow_check_conditional)]
#![warn(clippy::as_conversions)]
#![warn(clippy::match_overlapping_arm)]
#![warn(clippy::zero_divided_by_zero)]
#![warn(clippy::must_use_unit)]
#![warn(clippy::suspicious_assignment_formatting)]
#![warn(clippy::suspicious_else_formatting)]
#![warn(clippy::suspicious_unary_op_formatting)]
#![warn(clippy::mut_mutex_lock)]
#![warn(clippy::print_literal)]
#![warn(clippy::same_item_push)]
#![warn(clippy::useless_format)]
#![warn(clippy::write_literal)]
#![warn(clippy::redundant_closure)]
#![warn(clippy::redundant_closure_call)]
#![warn(clippy::unnecessary_lazy_evaluations)]
#![warn(clippy::partialeq_ne_impl)]
#![warn(clippy::redundant_field_names)]
#![warn(clippy::transmutes_expressible_as_ptr_casts)]
#![warn(clippy::unused_async)]
#![warn(clippy::disallowed_methods)]
#![warn(clippy::disallowed_macros)]
#![warn(clippy::disallowed_types)]
#![warn(clippy::from_over_into)]
// END LINT CONFIG
// Disallow usage of `unwrap()`.
#![warn(clippy::unwrap_used)]

//! This crate is responsible for durable storing and modifying the catalog contents.

use std::fmt;
use std::fmt::Formatter;
use std::time::Duration;

use mz_controller::clusters::ReplicaLogging;
use mz_proto::TryFromProtoError;
use mz_sql::catalog::CatalogError as SqlCatalogError;
use mz_stash::StashError;

pub use crate::objects::{
    Cluster, ClusterConfig, ClusterReplica, ClusterVariant, ClusterVariantManaged, Database, Item,
    ReplicaConfig, ReplicaLocation, Role, Schema, SystemObjectMapping,
};
pub use crate::stash::{Connection, ALL_COLLECTIONS};
pub use crate::transaction::Transaction;
pub use initialize::{
    AUDIT_LOG_COLLECTION, CLUSTER_COLLECTION, CLUSTER_INTROSPECTION_SOURCE_INDEX_COLLECTION,
    CLUSTER_REPLICA_COLLECTION, COMMENTS_COLLECTION, CONFIG_COLLECTION, DATABASES_COLLECTION,
    DEFAULT_PRIVILEGES_COLLECTION, ID_ALLOCATOR_COLLECTION, ITEM_COLLECTION, ROLES_COLLECTION,
    SCHEMAS_COLLECTION, SETTING_COLLECTION, STORAGE_USAGE_COLLECTION,
    SYSTEM_CONFIGURATION_COLLECTION, SYSTEM_GID_MAPPING_COLLECTION, SYSTEM_PRIVILEGES_COLLECTION,
    TIMESTAMP_COLLECTION,
};

mod stash;
mod transaction;

pub mod builtin;
pub mod initialize;
pub mod objects;

const DATABASE_ID_ALLOC_KEY: &str = "database";
const SCHEMA_ID_ALLOC_KEY: &str = "schema";
const USER_ROLE_ID_ALLOC_KEY: &str = "user_role";
const USER_CLUSTER_ID_ALLOC_KEY: &str = "user_compute";
const SYSTEM_CLUSTER_ID_ALLOC_KEY: &str = "system_compute";
const USER_REPLICA_ID_ALLOC_KEY: &str = "replica";
const SYSTEM_REPLICA_ID_ALLOC_KEY: &str = "system_replica";
pub const AUDIT_LOG_ID_ALLOC_KEY: &str = "auditlog";
pub const STORAGE_USAGE_ID_ALLOC_KEY: &str = "storage_usage";

#[derive(Debug)]
pub enum Error {
    Catalog(SqlCatalogError),
    Stash(StashError),
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Error::Catalog(e) => write!(f, "{e}"),
            Error::Stash(e) => write!(f, "{e}"),
        }
    }
}

impl From<SqlCatalogError> for Error {
    fn from(e: SqlCatalogError) -> Self {
        Self::Catalog(e)
    }
}

impl From<StashError> for Error {
    fn from(e: StashError) -> Self {
        Self::Stash(e)
    }
}

impl From<TryFromProtoError> for Error {
    fn from(e: TryFromProtoError) -> Error {
        Error::Stash(mz_stash::StashError::from(e))
    }
}

#[derive(Clone, Debug)]
pub struct BootstrapArgs {
    pub default_cluster_replica_size: String,
    pub builtin_cluster_replica_size: String,
    pub bootstrap_role: Option<String>,
}

pub(crate) fn builtin_cluster_replica_config(bootstrap_args: &BootstrapArgs) -> ReplicaConfig {
    ReplicaConfig {
        location: ReplicaLocation::Managed {
            size: bootstrap_args.builtin_cluster_replica_size.clone(),
            availability_zone: None,
            disk: false,
        },
        logging: default_logging_config(),
        idle_arrangement_merge_effort: None,
    }
}

pub(crate) fn default_logging_config() -> ReplicaLogging {
    ReplicaLogging {
        log_logging: false,
        interval: Some(Duration::from_secs(1)),
    }
}
