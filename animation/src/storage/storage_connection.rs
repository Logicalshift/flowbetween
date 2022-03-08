use super::storage_command::*;
use super::storage_response::*;

use flo_stream::*;
use futures::prelude::*;
use futures::stream::{BoxStream};

///
/// Represents a storage command publisher and its connected response stream
///
pub struct StorageConnection {
    storage_requests:   Publisher<Vec<StorageCommand>>, 
    storage_responses:  BoxStream<'static, Vec<StorageResponse>>
}

impl StorageConnection {
    ///
    /// Creates a new storage connection
    ///
    pub fn new(storage_requests: Publisher<Vec<StorageCommand>>, storage_responses: BoxStream<'static, Vec<StorageResponse>>) -> StorageConnection {
        StorageConnection {
            storage_requests,
            storage_responses
        }
    }

    ///
    /// Sends a request to the storage layer
    ///
    pub fn request<'a, Commands: 'a+IntoIterator<Item=StorageCommand>>(&'a mut self, request: Commands) -> impl 'a+Future<Output=Option<Vec<StorageResponse>>> {
        async move {
            self.storage_requests.publish(request.into_iter().collect()).await;
            self.storage_responses.next().await
        }
    }

    ///
    /// Sends a single request that produces a single response to the storage layer
    ///
    pub fn request_one<'a>(&'a mut self, request: StorageCommand) -> impl 'a+Future<Output=Option<StorageResponse>> {
        async move {
            self.request(vec![request]).await
                .and_then(|mut result| result.pop())
        }
    }
}