use std::time::{Duration, Instant};

#[derive(Default)]
pub struct Mapper {
    proc_map: std::collections::HashMap<String, MapperObject>,
}

impl Mapper {
    pub async fn register(
        &mut self,
        name: String,
        proc: Box<dyn crate::CuteProc>,
    ) {
        self.proc_map
            .entry(name)
            .or_insert(MapperObject::new(proc));
    }

    pub async fn call(&self, name: String, data: &[u8]) -> Result<Vec<u8>, std::io::Error> {
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
    proc: Box<dyn crate::CuteProc>,
    cache_param : Option<Vec<u8>>,
    cache_value: Vec<u8>,
}

impl MapperObject {
    fn new(proc: Box<dyn crate::CuteProc>) -> Self {
        Self {
            proc,
            cache_param : None,
            cache_value: None,
        }
    }

    fn call(&mut self, data: &[u8]) -> Result<Vec<u8>, std::io::Error> {
        if let Some(cached_param) = self.cache_param.as_ref() {
            if cached_param.as_slice() == data {
                return Err(std::io::Error::new(std::io::ErrorKind::Other, "Data and cache_value are the same."));
            }
        }

        self.cache_param =  Some(data.to_vec());
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

    fn get_cache(&self) -> Result<Vec<u8>, std::io::Error> {
        match self.cache_value.clone() {
            None => Err(std::io::Error::new(
                std::io::ErrorKind::WouldBlock,
                "do not cache data",
            )),
            Some(obj) => Ok(obj),
        }
    }
}
