use std::time::Instant; 

use crate::proxy::router::MatchedRoute; 
use crate::filter::rate_limit::RateLimitDecision; 


pub struct RequestContext { 
    pub route: Option<MatchedRoute>, 
    pub start: Instant, 
    pub rate_limited: bool, 
    pub rate_limit_decision: Option<RateLimitDecision>, 
}

impl RequestContext { 
    pub fn new() -> Self { 
        Self { 
            route: None, 
            start: Instant::now(), 
            rate_limited: false, 
            rate_limit_decision: None, 
        }
    }
}