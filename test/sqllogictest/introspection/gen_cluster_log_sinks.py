#!/usr/bin/env python3

# Copyright Materialize, Inc. and contributors. All rights reserved.
#
# Use of this software is governed by the Business Source License
# included in the LICENSE file at the root of this repository.
#
# As of the Change Date specified in that file, in accordance with
# the Business Source License, use of this software will be governed
# by the Apache License, Version 2.0.

from pathlib import Path

SLT_PATH = Path(__file__).parent.resolve() / "cluster_log_sinks.slt"
SLT = SLT_PATH.open("wt", newline="\n")


def emit(string: str) -> None:
    SLT.write(string)


emit(
    """\
# Copyright Materialize, Inc. and contributors. All rights reserved.
#
# Use of this software is governed by the Business Source License
# included in the LICENSE file at the root of this repository.
#
# As of the Change Date specified in that file, in accordance with
# the Business Source License, use of this software will be governed
# by the Apache License, Version 2.0.

#######################################################
# This file is generated by gen_cluster_log_sinks.py. #
# Don't modify it directly!                           #
#######################################################

"""
)

emit(
    """\
# Start from a pristine server state
reset-server

"""
)

SRC = [
    "mz_active_peeks",
    "mz_arrangement_batches_internal",
    "mz_arrangement_records_internal",
    "mz_arrangement_sharing",
    "mz_arrangement_sharing_internal",
    "mz_arrangement_sizes",
    "mz_compute_exports",
    "mz_compute_frontiers",
    "mz_compute_import_frontiers",
    "mz_dataflow_addresses",
    "mz_dataflow_channels",
    "mz_dataflow_channel_operators",
    "mz_dataflow_operator_dataflows",
    "mz_dataflow_operator_reachability",
    "mz_dataflow_operator_reachability_internal",
    "mz_dataflow_operators",
    "mz_dataflows",
    "mz_message_counts",
    "mz_message_counts_received_internal",
    "mz_message_counts_sent_internal",
    "mz_peek_durations_histogram",
    "mz_peek_durations_histogram_internal",
    "mz_records_per_dataflow",
    "mz_records_per_dataflow_global",
    "mz_records_per_dataflow_operator",
    "mz_scheduling_elapsed",
    "mz_scheduling_elapsed_internal",
    "mz_scheduling_parks_histogram",
    "mz_scheduling_parks_histogram_internal",
    "mz_worker_compute_delays_histogram",
    "mz_worker_compute_delays_histogram_internal",
    "mz_worker_compute_dependencies",
    "mz_worker_compute_frontiers",
    "mz_worker_compute_import_frontiers",
]
STAR_OVERRIDE = {
    "mz_dataflow_addresses": "id,worker_id",
    "mz_dataflow_operator_reachability_internal": "port,worker_id,update_type",
    "mz_dataflow_operator_reachability": "port,worker_id,update_type",
}


def query_empty(q: str) -> str:
    return f"query T\n{q};\n----\n\n"


def stmt_ok(q: str) -> str:
    return f"statement ok\n{q};\n\n"


def equal(postfix: str) -> str:
    res = ""
    for x in SRC:
        p = query_empty(
            f"SELECT * FROM ((SELECT * FROM mz_internal.{x}) EXCEPT (SELECT * FROM mz_internal.{x}_{postfix}))"
        )
        p += query_empty(
            f"SELECT * FROM ((SELECT * FROM mz_internal.{x}_{postfix}) EXCEPT (SELECT * FROM mz_internal.{x}))"
        )
        if x in STAR_OVERRIDE:
            p = p.replace("*", STAR_OVERRIDE[x])
        res += p
    return res


emit(
    """\
# Check that no log source has been created initially
query T
SELECT COUNT(*) FROM mz_sources WHERE name LIKE 'mz_active_peeks_%';
----
3

"""
)

emit(stmt_ok("CREATE CLUSTER c1 REPLICAS (r (SIZE '1'))"))
emit(
    """\
query T
SELECT COUNT(*) FROM mz_sources WHERE name LIKE 'mz_active_peeks_%';
----
4

"""
)
emit(stmt_ok("CREATE TABLE t1(f1 int, f2 int)"))
emit(stmt_ok("INSERT INTO t1 VALUES (1,1),(2,3),(4,5)"))
emit(stmt_ok("CREATE MATERIALIZED VIEW ma1 AS SELECT COUNT(*) FROM t1"))
emit(equal("1"))

emit(stmt_ok("SET CLUSTER TO c1"))
emit(stmt_ok("CREATE MATERIALIZED VIEW ma2 AS SELECT COUNT(*) FROM t1"))
emit(equal("4"))

emit(stmt_ok("CREATE CLUSTER c2 REPLICAS (r1 (SIZE '1'), r2 (SIZE '1'))"))
emit(
    """\
query T
SELECT COUNT(*) FROM mz_sources WHERE name LIKE 'mz_active_peeks_%';
----
6

"""
)

emit(stmt_ok("set cluster = c2"))
emit(stmt_ok("set cluster_replica = r1"))
emit(equal("5").rstrip() + "\n")