use indexmap::IndexMap; 

fn insert_if_missing( 
                      headers: &mut IndexMap<String, String>, 
                      name: &str, 
                      value: &str, 
) { 
    if !headers.contains_key(name) { 
        headers.insert(name.to_string(), value.to_string()); 
    }
}

pub fn apply_security_headers( 
                               headers: &mut IndexMap<String, String>, 
                               enabled: bool, 
) { 
    if !enabled { 
        return; 
    }

    insert_if_missing(headers, "x-content-type-options", "nosniff"); 
    insert_if_missing(headers, "x-frame-options", "DENY"); 
    insert_if_missing(headers, "referrer-policy", "no-referrer"); 
}