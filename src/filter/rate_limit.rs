use std::collections::HashMap; 
use std::sync::Mutex; 
use std::time::Duration; 
use std::time::Instant; 

use crate::config::schema::RateLimitConfig; 

pub struct RateLimiter { 
    clients: Mutex<HashMap<String, ClientCounter>>, 
}

struct ClientCounter { 
    window_start: Instant, 
    count: u64, 
}

#[derive(Debug, Clone)] 
pub struct RateLimitDecision { 
    pub allowed: bool, 
    pub limit: Option<u64>, 
    pub remaining: Option<u64>, 
    pub reset_after_secs: Option<u64>, 
    pub retry_after_secs: Option<u64>, 
}

impl RateLimiter { 
    pub fn new() -> Self { 
        Self { 
            clients: Mutex::new(HashMap::new()), 
        }
    }
    pub fn check(&self, client_ip: &str,
                 bucket: &str, 
                 config: Option<&RateLimitConfig>, 
    ) -> RateLimitDecision { 
        let Some(config) = config else { 
            return RateLimitDecision { 
                allowed: true, 
                limit: None, 
                remaining: None, 
                reset_after_secs: None, 
                retry_after_secs: None, 
            }; 
        };
        let key = format!("{bucket}:{client_ip}"); 

        let limit = config.requests; 

        let window = Duration::from_secs(config.window_seconds); 

        let mut clients = self.clients.lock().expect("rate limiter mutex poisoned"); 

        let now = Instant::now(); 

        let counter = clients.entry(key).or_insert(ClientCounter { 
            window_start: now, 
            count: 0, 
        }); 

        let elapsed = now.duration_since(counter.window_start); 

        if elapsed >=window{ 
            counter.window_start = now; 
            counter.count = 0; 
        } 

        let elapsed = now.duration_since(counter.window_start); 
        let reset_after_secs = window.saturating_sub(elapsed).as_secs().max(1); 

        if counter.count >= limit { 
            return RateLimitDecision { 
                allowed: false, 
                limit: Some(limit), 
                remaining: Some(0), 
                reset_after_secs: Some(reset_after_secs), 
                retry_after_secs: Some(reset_after_secs), 
            }; 
        } 

        counter.count += 1; 
        let remaining = limit.saturating_sub(counter.count); 
        RateLimitDecision { 
            allowed: true, 
            limit: Some(limit), 
            remaining: Some(remaining), 
            reset_after_secs: Some(reset_after_secs), 
            retry_after_secs: None, 
        } 
    } 
}