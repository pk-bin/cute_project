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
        let mut close_token = self.close_token.write().await;
        *close_token = true;
        drop(close_token);

        self.close_token = Arc::new(RwLock::new(false));
        let _ = proc.open(data);
        proc.valid_check()
    }

    pub async fn call(
        &mut self,
        mut proc: Box<dyn crate::CuteProc>,
    ) -> Pin<Box<dyn Stream<Item = Result<Vec<u8>, std::io::Error>> + Send>> {
        let clone_close_token = self.close_token.clone();
        Box::pin(async_stream::stream! {
            while !*clone_close_token.read().await {
                yield proc.call()
            }
        })
    }
}
