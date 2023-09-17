use std::pin::Pin;
use tokio_stream::Stream;

#[derive(Default)]
pub struct NoneMapper {}

impl NoneMapper {
    pub async fn register(
        &mut self,
        proc: &mut dyn crate::CuteProc,
        data: &[u8],
    ) -> Result<(), std::io::Error> {
        let _ = proc.open(data)?;
        proc.valid_check()
    }

    pub async fn call(
        &mut self,
        mut proc: Box<dyn crate::CuteProc>,
    ) -> Pin<Box<dyn Stream<Item = Result<Vec<u8>, std::io::Error>> + Send>> {
        Box::pin(async_stream::stream! {
            loop {
                yield proc.call()
            }
        })
    }
}
