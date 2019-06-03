// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

mod common;
use crate::common::*;

use iso8601;

use glean_core::metrics::*;
use glean_core::ping::PingMaker;
use glean_core::{CommonMetricData, Glean, Lifetime};

fn set_up_basic_ping() -> (Glean, PingMaker, PingType, tempfile::TempDir) {
    let (t, tmpname) = tempdir();
    let mut glean = Glean::new(&tmpname, GLOBAL_APPLICATION_ID, true).unwrap();
    let ping_maker = PingMaker::new();
    let ping_type = PingType::new("store1", true);
    glean.register_ping_type(&ping_type);

    // Record something, so the ping will have data
    let metric = BooleanMetric::new(CommonMetricData {
        name: "boolean_metric".into(),
        category: "telemetry".into(),
        send_in_pings: vec!["store1".into()],
        disabled: false,
        lifetime: Lifetime::User,
    });
    metric.set(&glean, true);

    (glean, ping_maker, ping_type, t)
}

#[test]
fn ping_info_must_contain_a_nonempty_start_and_end_time() {
    let (glean, ping_maker, ping_type, _t) = set_up_basic_ping();

    let content = ping_maker.collect(glean.storage(), &ping_type).unwrap();
    let ping_info = content["ping_info"].as_object().unwrap();

    let start_time_str = ping_info["start_time"].as_str().unwrap();
    let start_time_date = iso8601_to_chrono(&iso8601::datetime(start_time_str).unwrap());

    let end_time_str = ping_info["end_time"].as_str().unwrap();
    let end_time_date = iso8601_to_chrono(&iso8601::datetime(end_time_str).unwrap());

    assert!(start_time_date <= end_time_date);
}

#[test]
fn get_ping_info_must_report_all_the_required_fields() {
    let (glean, ping_maker, ping_type, _t) = set_up_basic_ping();

    let content = ping_maker.collect(glean.storage(), &ping_type).unwrap();
    let ping_info = content["ping_info"].as_object().unwrap();

    assert_eq!("store1", ping_info["ping_type"].as_str().unwrap());
    assert!(ping_info.get("start_time").is_some());
    assert!(ping_info.get("end_time").is_some());
    assert!(ping_info.get("seq").is_some());
}

#[test]
fn get_client_info_must_report_all_the_available_data() {
    let (glean, ping_maker, ping_type, _t) = set_up_basic_ping();

    let content = ping_maker.collect(glean.storage(), &ping_type).unwrap();
    let client_info = content["client_info"].as_object().unwrap();

    client_info["telemetry_sdk_build"].as_str().unwrap();
}

// SKIPPED from glean-ac: collect() must report a valid ping with the data from the engines
// This test doesn't really make sense with rkv

#[test]
fn collect_must_report_none_when_no_data_is_stored() {
    // NOTE: This is a behavior change from glean-ac which returned an empty
    // string in this case. As this is an implementation detail and not part of
    // the public API, it's safe to change this.

    let (mut glean, ping_maker, ping_type, _t) = set_up_basic_ping();

    let unknown_ping_type = PingType::new("unknown", true);
    glean.register_ping_type(&ping_type);

    assert!(ping_maker
        .collect(glean.storage(), &unknown_ping_type)
        .is_none());
}

#[test]
#[ignore] // sequence numbers aren't implemented
fn seq_number_must_be_sequential() {
    let (glean, ping_maker, _ping_type, _t) = set_up_basic_ping();

    let metric = BooleanMetric::new(CommonMetricData {
        name: "boolean_metric".into(),
        category: "telemetry".into(),
        send_in_pings: vec!["store2".into()],
        disabled: false,
        lifetime: Lifetime::User,
    });
    metric.set(&glean, true);

    for i in 0..=1 {
        for ping_name in ["store1", "store2"].iter() {
            let ping_type = PingType::new(*ping_name, true);
            let content = ping_maker.collect(glean.storage(), &ping_type).unwrap();
            let seq_num = content["ping_info"]["seq"].as_i64().unwrap();
            // Ensure sequence numbers in different stores are independent of
            // each other
            assert_eq!(i, seq_num);
        }
    }
}
