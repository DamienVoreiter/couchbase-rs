pub mod error;
pub mod options;
pub mod results;

use crate::api::error::{CouchbaseError, ErrorContext, CouchbaseResult};
use crate::api::options::{GetOptions, QueryOptions, UpsertOptions};
use crate::api::results::{GetResult, MutationResult, QueryResult};
use crate::io::request::{UpsertRequest, GetRequest, QueryRequest, Request};
use crate::io::Core;
use futures::channel::oneshot;
use std::sync::Arc;
use serde::Serialize;
use serde_json::to_vec;

pub struct Cluster {
    core: Arc<Core>,
}

impl Cluster {
    pub fn connect<S: Into<String>>(connection_string: S, username: S, password: S) -> Self {
        Cluster {
            core: Arc::new(Core::new(
                connection_string.into(),
                username.into(),
                password.into(),
            )),
        }
    }

    pub fn bucket<S: Into<String>>(&self, name: S) -> Bucket {
        let name = name.into();
        self.core.open_bucket(name.clone());
        Bucket::new(self.core.clone(), name)
    }

    pub async fn query<S: Into<String>>(
        &self,
        statement: S,
        options: QueryOptions,
    ) -> CouchbaseResult<QueryResult> {
        let (sender, receiver) = oneshot::channel();
        self.core.send(Request::Query(QueryRequest::new(
            statement.into(),
            options,
            sender,
        )));
        receiver.await.unwrap()
    }
}

pub struct Bucket {
    name: String,
    core: Arc<Core>,
}

impl Bucket {
    pub(crate) fn new(core: Arc<Core>, name: String) -> Self {
        Self { name, core }
    }

    pub fn default_collection(&self) -> Collection {
        Collection::new(self.core.clone())
    }
}

pub struct Scope {}

impl Scope {}

pub struct Collection {
    core: Arc<Core>,
}

impl Collection {
    pub(crate) fn new(core: Arc<Core>) -> Self {
        Self { core }
    }

    pub async fn get<S: Into<String>>(
        &self,
        id: S,
        options: GetOptions,
    ) -> CouchbaseResult<GetResult> {
        let (sender, receiver) = oneshot::channel();
        self.core
            .send(Request::Get(GetRequest::new(id.into(), options, sender)));
        receiver.await.unwrap()
    }

    pub async fn upsert<S: Into<String>, T>(
        &self,
        id: S,
        content: T,
        options: UpsertOptions,
    ) -> CouchbaseResult<MutationResult> where T: Serialize {
        let serialized = match to_vec(&content) {
            Ok(v) => v,
            Err(e) => return Err(CouchbaseError::EncodingFailure {
                ctx: ErrorContext::default(),
                source: e.into(),
            }),
        };

        let (sender, receiver) = oneshot::channel();
        self.core
            .send(Request::Upsert(UpsertRequest::new(id.into(), serialized, options, sender)));
        receiver.await.unwrap()
    }
}

#[derive(Debug)]
pub struct MutationToken {
    partition_uuid: u64,
    sequence_number: u64,
    partition_id: u16,
}

impl MutationToken {
    pub fn new(partition_uuid: u64,
        sequence_number: u64,
        partition_id: u16) -> Self {
        Self { partition_uuid, sequence_number, partition_id }
    }

    pub fn partition_uuid(&self) -> u64 {
        self.partition_uuid
    }

    pub fn sequence_number(&self) -> u64 {
        self.sequence_number
    }

    pub fn partition_id(&self) -> u16 {
        self.partition_id
    }

    pub fn bucket_name(&self) -> &String {
        todo!()
    }
}
