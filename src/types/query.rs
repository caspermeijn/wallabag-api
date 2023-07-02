// Copyright 2021 Pablo Baeyens <pbaeyens31+github@gmail.com>
// SPDX-License-Identifier: Apache-2.0 OR MIT

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
// Represents parameters for the entry checking endpoint.
pub(crate) struct EntriesExistParams {
    pub return_id: usize,
    pub urls: Vec<String>,
}
