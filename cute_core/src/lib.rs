use std::pin::Pin;
use tokio_stream::Stream;

/// cute core 에서 사용할 mapper trait.
pub trait CuteMapper {
    /// 동작 및 실행 결과 가져옴.
    fn execute(&mut self, name: String, buffer: Box<Vec<u8>>) -> Result<Vec<u8>, std::io::Error>;
}

/// Cute Proc. Mapper 를 담기 위한 container. Generic 으로는 외부의 Default 가 정의된 Context 가질 수 있다.
///
/// 해당 trait 이 비동기 상태에서일 경우를 반영해서 sync 추가.
pub trait CuteProc: Send + Sync + 'static {
    /// 데이터 출력 요청을 보냄. 보낼 때는 Parameter 정보 요청. 무조건. bincode serde 사용.
    fn open(&mut self, bytes: &[u8]) -> Result<(), std::io::Error>;
    /// virtual method. valid check.
    fn valid_check(&self) -> Result<(), std::io::Error> {
        Ok(())
    }
    /// 데이터 결과를 반환 함. 무조건. bincode serde 사용.
    fn call(&mut self) -> Result<Vec<u8>, std::io::Error>;
}

/// CuteMap
pub struct CuteMap {
    unary_name_map: std::collections::HashMap<String, action::CuteProcType>,
    stream_name_map: std::collections::HashMap<String, action::CuteProcType>,
    unary_map: action::cache_unary::Mapper,
    atomic_stream: action::atomic_stream::NoneMapper,
    multi_stream: action::multi_stream::NoneMapper,
}

impl CuteMap {
    /// Map을 생성.
    pub fn new() -> Self {
        Self {
            unary_name_map: Default::default(),
            stream_name_map: Default::default(),
            unary_map: Default::default(),
            atomic_stream: Default::default(),
            multi_stream: Default::default(),
        }
    }

    /// 작업을 등록해준다.
    pub fn register(
        &mut self,
        name: String,
        cute_type: action::CuteProcType,
    ) -> Result<(), std::io::Error> {
        match cute_type {
            action::CuteProcType::CacheUnary(_, _) => {
                self.unary_name_map.entry(name).or_insert(cute_type);
            }
            _ => {
                self.stream_name_map.entry(name).or_insert(cute_type);
            }
        }
        Ok(())
    }
    /// unary 작업을 open 및 실행한다.
    pub async fn open_unary(
        &mut self,
        name: String,
        data: &[u8],
    ) -> Result<Vec<u8>, std::io::Error> {
        match self.unary_name_map.get(&*name) {
            None => Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "not found unary key",
            )),
            Some(proc_type) => match proc_type {
                action::CuteProcType::CacheUnary(proc_name, creator) => {
                    let proc = creator();
                    self.unary_map.register(proc_name.clone(), proc).await;
                    self.unary_map.call(proc_name.clone(), data).await
                }
                _ => Err(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "is not unary process",
                )),
            },
        }
    }

    pub async fn open_stream(
        &mut self,
        name: String,
        is_atomic: bool,
        data: &[u8],
    ) -> Result<Pin<Box<dyn Stream<Item = Result<Vec<u8>, std::io::Error>> + Send>>, std::io::Error>
    {
        match self.stream_name_map.get_mut(&*name) {
            None => Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "do not find proc name",
            )),
            Some(proc_type) => match proc_type {
                action::CuteProcType::AtomicStream(_, creator) => {
                    if is_atomic {
                        let mut proc = creator();
                        self.atomic_stream.register(proc.as_mut(), data).await?;
                        Ok(self.atomic_stream.call(proc).await)
                    } else {
                        Err(std::io::Error::new(
                            std::io::ErrorKind::NotFound,
                            "Cute Process Type Error",
                        ))
                    }
                }
                action::CuteProcType::MultiStream(_, creator) => {
                    if !is_atomic {
                        let mut proc = creator();
                        self.multi_stream.register(proc.as_mut(), data).await?;
                        Ok(self.multi_stream.call(proc).await)
                    } else {
                        Err(std::io::Error::new(
                            std::io::ErrorKind::NotFound,
                            "Cute Process Type Error",
                        ))
                    }
                }
                _ => Err(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "Cute Process Type Error",
                )),
            },
        }
    }
}
pub mod action;
