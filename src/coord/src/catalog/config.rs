// Copyright Materialize, Inc. and contributors. All rights reserved.
//
// Use of this software is governed by the Business Source License
// included in the LICENSE file.
//
// As of the Change Date specified in that file, in accordance with
// the Business Source License, use of this software will be governed
// by the Apache License, Version 2.0.

use std::time::Duration;

use mz_build_info::BuildInfo;
use mz_dataflow_types::sources::AwsExternalId;
use mz_ore::metrics::MetricsRegistry;
use mz_sql::plan::ComputeInstanceIntrospectionConfig;

use crate::catalog::storage;
use crate::persistcfg::PersisterWithConfig;

/// Configures a catalog.
#[derive(Debug)]
pub struct Config<'a> {
    /// The connection to the SQLite database.
    pub storage: storage::Connection,
    /// Whether to enable experimental mode.
    pub experimental_mode: Option<bool>,
    /// Whether to enable safe mode.
    pub safe_mode: bool,
    /// Whether to enable introspection for the local compute instance.
    pub local_compute_introspection: Option<ComputeInstanceIntrospectionConfig>,
    /// Information about this build of Materialize.
    pub build_info: &'static BuildInfo,
    /// An [External ID][] to use for all AWS AssumeRole operations.
    ///
    /// [External ID]: https://docs.aws.amazon.com/IAM/latest/UserGuide/id_roles_create_for-user_externalid.html
    pub aws_external_id: AwsExternalId,
    /// Timestamp frequency to use for CREATE SOURCE
    pub timestamp_frequency: Duration,
    /// Function to generate wall clock now; can be mocked.
    pub now: mz_ore::now::NowFn,
    /// Whether or not to skip catalog migrations.
    pub skip_migrations: bool,
    /// The registry that catalog uses to report metrics.
    pub metrics_registry: &'a MetricsRegistry,
    /// Whether or not to prevent user indexes from being considered for use.
    pub disable_user_indexes: bool,
    /// A runtime for the `persist` crate alongside its configuration.
    pub persister: &'a PersisterWithConfig,
}