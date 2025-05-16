use std::collections::HashMap;
use handle_errors::Error;

/// Pagination struct that is getting extracted
/// from query params
#[derive(Debug)]
pub struct Pagination {
    /// The index of the first item that has to be returned
    pub start: usize,
    /// The index of the last item that has to be returned
    pub end: usize,
}

/// Pagination methods. These two were my idea!
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

/// Extract query parameters from the `/questions` route
/// # Example query
/// GET requests to this route can have a pagination attached so we just
/// return the questions we need
/// `\questions?start=1&end=10`
pub fn extract_pagination(params: HashMap<String, String>) 
    -> Result<Pagination, Error> {
        // Could be improved in the future
        if params.contains_key("start") && params.contains_key("end") {
            let mut pagination = Pagination {
                // Takes the "start" parameter in the query
                // and tries to convert it to a number
                start: params
                    .get("start")
                    .unwrap()
                    .parse::<usize>()
                    .map_err(Error::ParseError)?,
                // Takes the "end" parameter in the query
                // and tries to convert it to a number
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