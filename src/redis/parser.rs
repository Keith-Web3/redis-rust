#[derive(Debug, Clone)]

pub enum RedisData {
    String(String),
    Int(isize),
    Error(String),
    BulkString(String),
    NullBulkString(()),
    Array(Vec<RedisData>),
    Null(()),
}

impl RedisData {
    pub fn as_string(&self) -> Result<String, ()> {
        match self {
            RedisData::String(val) => Ok(val.to_string()),
            RedisData::BulkString(val) => Ok(val.to_string()),
            RedisData::Error(val) => Ok(val.to_string()),
            RedisData::Int(val) => Ok(val.to_string()),
            _ => Err(()),
        }
    }
    pub fn as_int(&self) -> Result<&isize, ()> {
        match self {
            RedisData::Int(val) => Ok(val),
            _ => Err(()),
        }
    }
    pub fn as_arr(&self) -> Result<&Vec<Self>, ()> {
        match self {
            RedisData::Array(val) => Ok(val),
            _ => Err(()),
        }
    }

    pub fn is_null(&self) -> bool {
        match self {
            RedisData::NullBulkString(_) => true,
            RedisData::Null(_) => true,
            _ => false,
        }
    }
    pub fn is_string(&self, compare_to: Option<&str>) -> bool {
        match self {
            RedisData::String(val) => {
                if let Some(other_string) = compare_to {
                    other_string == val
                } else {
                    true
                }
            }
            _ => false,
        }
    }
    pub fn parses_to_string(&self, compare_to: Option<&str>) -> bool {
        self.is_bulk_string(compare_to) || self.is_string(compare_to)
    }
    pub fn is_bulk_string(&self, compare_to: Option<&str>) -> bool {
        match self {
            RedisData::BulkString(val) => {
                if let Some(other_string) = compare_to {
                    other_string == val
                } else {
                    true
                }
            }
            _ => false,
        }
    }
    pub fn is_err(&self) -> bool {
        match self {
            RedisData::Error(_) => true,
            _ => false,
        }
    }
    pub fn is_int(&self, compare_to: Option<&isize>) -> bool {
        match self {
            RedisData::Int(val) => {
                if let Some(other_number) = compare_to {
                    other_number == val
                } else {
                    true
                }
            }
            _ => false,
        }
    }
    pub fn is_arr(&self) -> bool {
        match self {
            RedisData::Array(_) => true,
            _ => false,
        }
    }
}
