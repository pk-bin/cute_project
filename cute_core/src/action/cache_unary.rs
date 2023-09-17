#[derive(Default)]
/// 설명 x
pub struct Mapper {
    /// hashmap 관리되는 proc 들 정의.
    proc_map: std::collections::HashMap<String, MapperObject>,
}

impl Mapper {
    /// 등록
    pub async fn register(&mut self, name: String, proc: Box<dyn crate::CuteProc>) {
        self.proc_map.entry(name).or_insert(MapperObject::new(proc));
    }

    /// 실행.
    pub async fn call(&mut self, name: String, data: &[u8]) -> Result<Vec<u8>, std::io::Error> {
        match self.proc_map.get_mut(&*name) {
            None => Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "do not find name",
            )),
            Some(obj) => match obj.call(data) {
                Ok(result) => Ok(result),
                Err(_) => obj.get_cache(),
            },
        }
    }
}

struct MapperObject {
    /// process container
    proc: Box<dyn crate::CuteProc>,
    /// parameter 임시 저장소.
    cache_param: Option<Vec<u8>>,
    /// result value 임시 저장소.
    cache_value: Vec<u8>,
}

impl MapperObject {
    /// 생성.
    fn new(proc: Box<dyn crate::CuteProc>) -> Self {
        Self {
            proc,
            cache_param: None,
            cache_value: vec![],
        }
    }

    /// 호출. 이때 proc 의 open -> valid_check -> call 수행.
    fn call(&mut self, data: &[u8]) -> Result<Vec<u8>, std::io::Error> {
        if let Some(cached_param) = self.cache_param.as_ref() {
            if cached_param.as_slice() == data {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Data and cache_value are the same.",
                ));
            }
        }

        self.cache_param = Some(data.to_vec());
        let _ = self.proc.open(data)?;
        let _ = self.proc.valid_check()?;
        match self.proc.call() {
            Ok(result) => {
                self.cache_value = result.clone();
                Ok(result.clone())
            }
            Err(e) => Err(e),
        }
    }

    /// 임시 저장된 result_value 반환 함수.
    fn get_cache(&self) -> Result<Vec<u8>, std::io::Error> {
        Ok(self.cache_value.clone())
    }
}
