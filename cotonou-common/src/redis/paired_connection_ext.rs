use std::pin::Pin;

use futures::Future;
use redis_async::{
    client::{paired::SendFuture, PairedConnection},
    resp::{FromResp, RespValue},
    resp_array,
};

pub trait PairedConnectionExt {
    fn lpush<E>(&self, key: &str, element: E) -> SendFuture<i32>
    where
        RespValue: std::convert::From<E>;

    fn rpush<E>(&self, key: &str, element: E) -> SendFuture<i32>
    where
        RespValue: std::convert::From<E>;

    fn lpush_multiple<E>(&self, key: &str, elements: &[E]) -> SendFuture<i32>
    where
        RespValue: std::convert::From<E>,
        E: Clone;

    fn rpush_multiple<E>(&self, key: &str, elements: &[E]) -> SendFuture<i32>
    where
        RespValue: std::convert::From<E>,
        E: Clone;

    fn lpop<'a, E>(
        &'a self,
        key: &'a str,
        count: usize,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<E>, redis_async::error::Error>> + 'a>>
    where
        E: FromResp;

    fn publish<C, M>(&self, channel: C, message: M) -> SendFuture<i32>
    where
        RespValue: std::convert::From<C>,
        RespValue: std::convert::From<M>;

    fn del(&self, key: &str) -> SendFuture<i32>;
    fn select(&self, index: u8) -> SendFuture<String>;
}

impl PairedConnectionExt for PairedConnection {
    fn lpush<E>(&self, key: &str, element: E) -> SendFuture<i32>
    where
        RespValue: std::convert::From<E>,
    {
        self.send::<i32>(resp_array!["LPUSH", key, element])
    }

    fn rpush<E>(&self, key: &str, element: E) -> SendFuture<i32>
    where
        RespValue: std::convert::From<E>,
    {
        self.send::<i32>(resp_array!["RPUSH", key, element])
    }

    fn lpush_multiple<E>(&self, key: &str, elements: &[E]) -> SendFuture<i32>
    where
        RespValue: std::convert::From<E>,
        E: Clone,
    {
        self.send::<i32>(RespValue::Array(
            ["LPUSH".into(), key.into()]
                .into_iter()
                .chain(elements.iter().map(|e| RespValue::from(e.clone())))
                .collect(),
        ))
    }

    fn rpush_multiple<E>(&self, key: &str, elements: &[E]) -> SendFuture<i32>
    where
        RespValue: std::convert::From<E>,
        E: Clone,
    {
        self.send::<i32>(RespValue::Array(
            ["RPUSH".into(), key.into()]
                .into_iter()
                .chain(elements.iter().map(|e| RespValue::from(e.clone())))
                .collect(),
        ))
    }

    fn lpop<'a, E>(
        &'a self,
        key: &'a str,
        count: usize,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<E>, redis_async::error::Error>> + 'a>>
    where
        E: FromResp,
    {
        let fut = async move {
            let values = self
                .send::<RespValue>(resp_array!["LPOP", key, count.to_string()])
                .await?;

            match values {
                RespValue::Nil => Ok(Vec::new()),
                RespValue::Array(_) => Ok(Vec::<E>::from_resp(values)?),
                _ => Err(redis_async::error::Error::Resp(
                    "Unexpected resp value type".to_owned(),
                    Some(values),
                )),
            }
        };

        Box::pin(fut)
    }

    fn publish<C, M>(&self, channel: C, message: M) -> SendFuture<i32>
    where
        RespValue: std::convert::From<C>,
        RespValue: std::convert::From<M>,
    {
        self.send::<i32>(resp_array!["PUBLISH", channel, message])
    }

    fn del(&self, key: &str) -> SendFuture<i32> {
        self.send::<i32>(resp_array!["DEL", key])
    }

    fn select(&self, index: u8) -> SendFuture<String> {
        self.send::<String>(resp_array!["SELECT", index.to_string()])
    }
}
