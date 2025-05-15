use std::collections::HashMap;
use handle_errors::Error;

#[derive(Debug)]
pub struct Pagination {
    pub start: usize,
    pub end: usize,
}

// This one was my idea
impl Pagination {
    pub fn sanitize(mut self) -> Self {
        if self.start > self.end {
            let saved_start = self.start;
            self.start = self.end;
            self.end = saved_start;
        }
        self
    }
    pub fn saturate(mut self, max_len: usize) -> Self {
        if self.end > max_len {
            println!("Saturating! Level: {}", max_len);
            self.end = max_len;
            println!("Start: {} End: {}", self.start, self.end);
        }
        self
    }
}

pub fn extract_pagination(params: HashMap<String, String>) -> Result<Pagination, Error> {
    if params.contains_key("start") && params.contains_key("end") {
        let mut pagination = Pagination {
            start: params
                .get("start")
                .unwrap()
                .parse::<usize>()
                .map_err(Error::ParseError)?,
            end: params
                .get("end")
                .unwrap()
                .parse::<usize>()
                .map_err(Error::ParseError)?,
        };
        // println!("{:?}", pagination);
        pagination = pagination.sanitize();
        // println!("{:?}", pagination);
        return Ok(pagination);
    }
    Err(Error::MissingParameters)
}