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
        update_time: Duration,
    ) {
        self.proc_map
            .entry(name)
            .or_insert(MapperObject::new(proc, update_time));
    }

    pub async fn call(&self, name: String, data: &[u8]) -> Result<Vec<u8>, std::io::Error> {
        match &mut self.proc_map.get(&*name) {
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
    duration: Duration,
    time_check: std::time::Instant,
    cache_value: Option<Vec<u8>>,
}

impl MapperObject {
    fn new(proc: Box<dyn crate::CuteProc>, duration: Duration) -> Self {
        Self {
            proc,
            duration,
            time_check: Instant::now(),
            cache_value: None,
        }
    }

    fn check(&self) -> Result<(), std::io::Error> {
        if self.time_check.elapsed().as_micros() > self.duration.as_micros() {
            Ok(())
        } else {
            Err(std::io::Error::new(std::io::ErrorKind::WouldBlock, ""))
        }
    }

    fn call(&mut self, data: &[u8]) -> Result<Vec<u8>, std::io::Error> {
        let _ = self.check()?;
        let _ = self.proc.open(data)?;
        let _ = self.proc.valid_check()?;
        match self.proc.call() {
            Ok(result) => {
                self.cache_value.insert = Some(result.clone());
                self.time_check = Instant::now();
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
