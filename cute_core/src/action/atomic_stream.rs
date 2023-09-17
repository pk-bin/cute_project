use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio_stream::Stream;

pub struct NoneMapper {
    close_token: Arc<RwLock<bool>>,
}

impl Default for NoneMapper {
    fn default() -> Self {
        Self {
            close_token: Arc::new(RwLock::new(false)),
        }
    }
}

impl NoneMapper {
    pub async fn register(
        &mut self,
        proc: &mut dyn crate::CuteProc,
        data: &[u8],
    ) -> Result<(), std::io::Error> {
        /// 이전에 사용중이던 모든 쓰레드의 close token 값 변경.
        let mut close_token = self.close_token.write().await;
        *close_token = true;
        /// drop 시켜서 현재와의 연결성 해제.
        drop(close_token);

        /// 새롭게 구성하여 call에 형성되는 stream 에 넣기.
        self.close_token = Arc::new(RwLock::new(false));
        let _ = proc.open(data);
        proc.valid_check()
    }

    pub async fn call(
        &mut self,
        proc: Box<dyn crate::CuteProc>,
    ) -> Pin<Box<dyn Stream<Item = Result<Vec<u8>, std::io::Error>> + Send>> {
        let clone_close_token = self.close_token.clone();
        Box::pin(async_stream::stream! {
            while !*clone_close_token.read().await {
                yield proc.call()
            }
        })
    }
}
