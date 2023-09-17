/// cute proc type
pub enum CuteProcType {
    /// 일정 시간 동안 반환한 결과를 memory 상에 쥐고 있으며 유저가 얼마나 빨리 요청하든 memory 상의 결과 반환.
    CacheUnary(String, fn() -> Box<dyn crate::CuteProc>),
    /// 해당 stream 은 단 하나만 열려야 되는 경우 사용.
    AtomicStream(String, fn() -> Box<dyn crate::CuteProc>),
    /// 해당 stream 은 여러개가 동시에 돌아도 상관없는 경우 사용.
    MultiStream(String, fn() -> Box<dyn crate::CuteProc>),
}

impl CuteProcType {
    pub fn new_cache_unary(name: String, creator: fn() -> Box<dyn crate::CuteProc>) -> Self {
        Self::CacheUnary(name, creator)
    }

    pub fn new_atomic_stream(name: String, creator: fn() -> Box<dyn crate::CuteProc>) -> Self {
        Self::AtomicStream(name, creator)
    }

    pub fn new_multi_stream(name: String, creator: fn() -> Box<dyn crate::CuteProc>) -> Self {
        Self::MultiStream(name, creator)
    }
}

pub(crate) mod atomic_stream;
pub mod cache_unary;
pub(crate) mod multi_stream;
