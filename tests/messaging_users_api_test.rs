use reqwest;
use serde_json::json;
use tokio;

const BASE_URL: &str = "http://localhost:4646/m";
const SCHOOL_TOKEN: &str = "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJpZCI6IjY4ZWU1MWI3MzA4YWMyODg3ODM5MDg1YyIsImNyZWF0b3JfaWQiOiI2OTEwMjZjZTNkMDNhN2M0ZWI2ZTYwZmYiLCJuYW1lIjoiQUthZ2VyYSBoaWdoIHNjaG9vbCIsInVzZXJuYW1lIjoiYWthZ2VyYV9oaWdoX3NjaG9vIiwibG9nbyI6Imh0dHBzOi8vcmVzLmNsb3VkaW5hcnkuY29tL2RxZm14a3dyei9pbWFnZS91cGxvYWQvdjE3NjA0NDg5NTAvdXBsb2FkXzE3NjA0NDg5NDcucG5nLnBuZyIsInNjaG9vbF90eXBlIjoiUHJpdmF0ZSIsImFmZmlsaWF0aW9uIjpudWxsLCJkYXRhYmFzZV9uYW1lIjoic2Nob29sXzY4ZWU1MWI3MzA4YWMyODg3ODM5MDg1YyIsImNyZWF0ZWRfYXQiOiIyMDI1LTEwLTE0VDEzOjM1OjUxLjkwOTAzMjYwMFoiLCJtZW1iZXIiOnsidXNlcl90eXBlIjoiVVNFUiIsIl9pZCI6IjY5MTAyNmNlM2QwM2E3YzRlYjZlNjBmZiIsIm5hbWUiOiJIQUtJWkFZRVpVIEpFQU4gREUgRElFVSIsImVtYWlsIjoiaGFraXpheWV6dWplYW5kZWRpZXVAZ21haWwuY29tIiwidXNlcm5hbWUiOiJfaGFraXpheWV6dV9kaWV1X2plYW5fZGVfMTU2IiwicGFzc3dvcmRfaGFzaCI6IiRhcmdvbjJpZCR2PTE5JG09MTk0NTYsdD0yLHA9MSRPZmtqM09XTlgrUjVERnpLWCtwL3NnJHJWYWdIczRTSmhzWnYyNXVUdnhtK25DRWxrUU14RHd4eUlrdVRWbDB5VVUiLCJyb2xlIjoiU0NIT09MU1RBRkYiLCJpbWFnZV9pZCI6InVybF91cGxvYWRfMTc2MjY2NjI3MCIsImltYWdlIjoiaHR0cHM6Ly9yZXMuY2xvdWRpbmFyeS5jb20vZHFmbXhrd3J6L2ltYWdlL3VwbG9hZC92MTc2MjY2NjI3NC91cmxfdXBsb2FkXzE3NjI2NjYyNzAuanBnIiwiYmFja2dyb3VuZF9pbWFnZXMiOm51bGwsImJpbyI6bnVsbCwiZGlzYWJsZSI6ZmFsc2UsInBob25lIjoiKzI1MDc4ODgxNjk0MCIsImFkZHJlc3MiOnsiY291bnRyeSI6IlJ3YW5kYSIsInByb3ZpbmNlIjoiS2lnYWxpIENpdHkiLCJkaXN0cmljdCI6Ikdhc2FibyIsInNlY3RvciI6bnVsbCwiY2VsbCI6bnVsbCwidmlsbGFnZSI6bnVsbCwic3RhdGUiOm51bGwsInN0cmVldCI6bnVsbCwiY2l0eSI6bnVsbCwicG9zdGFsX2NvZGUiOm51bGwsImdvb2dsZV9tYXBfdXJsIjpudWxsfSwic29jaWFsX21lZGlhIjpbeyJwbGF0Zm9ybSI6Ik90aGVyIiwidXJsIjoiaHR0cHM6Ly9zb3NoZ3Rocy5vcmcifV0sInByZWZlcnJlZF9jb21tdW5pY2F0aW9uX21ldGhvZCI6WyJDaGF0IiwiRW1haWwiLCJTbXMiLCJWaWRlb0NhbGwiLCJDYWxsIl0sImdlbmRlciI6Ik1BTEUiLCJhZ2UiOm51bGwsImxhbmd1YWdlc19zcG9rZW4iOlsiRW5nbGlzaCIsIktpbnlhcndhbmRhIl0sImhvYmJpZXNfaW50ZXJlc3RzIjpudWxsLCJkcmVhbV9jYXJlZXIiOm51bGwsInNwZWNpYWxfc2tpbGxzIjpudWxsLCJoZWFsdGhfb3JfbGVhcm5pbmdfbm90ZXMiOm51bGwsImN1cnJlbnRfc2Nob29sX2lkIjoiNjhlZTUxYjczMDhhYzI4ODc4MzkwODVjIiwic2Nob29scyI6WyI2OTEwMjg5NTNkMDNhN2M0ZWI2ZTYxMDEiLCI2OGVlNTFiNzMwOGFjMjg4NzgzOTA4NWMiXSwiYWNjZXNzaWJsZV9jbGFzc2VzIjpudWxsLCJmYXZvcml0ZV9zdWJqZWN0c19jYXRlZ29yeSI6bnVsbCwicHJlZmVycmVkX3N0dWR5X3N0eWxlcyI6bnVsbCwiZ3VhcmRpYW5faW5mbyI6bnVsbCwic3BlY2lhbF9zdXBwb3J0X25lZWRlZCI6bnVsbCwibGVhcm5pbmdfY2hhbGxlbmdlcyI6bnVsbCwidGVhY2hpbmdfbGV2ZWwiOm51bGwsImVtcGxveW1lbnRfdHlwZSI6IkZ1bGxUaW1lIiwidGVhY2hpbmdfc3RhcnRfZGF0ZSI6bnVsbCwieWVhcnNfb2ZfZXhwZXJpZW5jZSI6bnVsbCwiZWR1Y2F0aW9uX2xldmVsIjoiTWFzdGVyIiwiY2VydGlmaWNhdGlvbnNfdHJhaW5pbmdzIjpbIkZpcnN0QWlkIiwiVGVhY2hpbmdDZXJ0aWZpY2F0ZSIsIkNvbXB1dGVyTGl0ZXJhY3kiLCJMZWFkZXJzaGlwVHJhaW5pbmciLCJTYWZldHlUcmFpbmluZyIsIkxhbmd1YWdlUHJvZmljaWVuY3kiLCJDaGlsZFByb3RlY3Rpb24iLCJNYW5hZ2VtZW50VHJhaW5pbmciLCJDb3Vuc2VsaW5nVHJhaW5pbmciLCJNZW50b3JzaGlwUHJvZ3JhbSIsIlRlY2huaWNhbENlcnRpZmljYXRpb24iXSwicHJlZmVycmVkX2FnZV9ncm91cCI6bnVsbCwicHJvZmVzc2lvbmFsX2dvYWxzIjpudWxsLCJhdmFpbGFiaWxpdHlfc2NoZWR1bGUiOlt7ImRheSI6Ik1vbiIsInRpbWVfcmFuZ2UiOnsic3RhcnQiOiIwOTowMDowMCIsImVuZCI6IjE3OjAwOjAwIn19LHsiZGF5IjoiVHVlIiwidGltZV9yYW5nZSI6eyJzdGFydCI6IjA5OjAwOjAwIiwiZW5kIjoiMTc6MDA6MDAifX0seyJkYXkiOiJXZWQiLCJ0aW1lX3JhbmdlIjp7InN0YXJ0IjoiMDk6MDA6MDAiLCJlbmQiOiIxNzowMDowMCJ9fSx7ImRheSI6IlRodSIsInRpbWVfcmFuZ2UiOnsic3RhcnQiOiIwOTowMDowMCIsImVuZCI6IjE3OjAwOjAwIn19LHsiZGF5IjoiRnJpIiwidGltZV9yYW5nZSI6eyJzdGFydCI6IjA5OjAwOjAwIiwiZW5kIjoiMTc6MDA6MDAifX1dLCJkZXBhcnRtZW50IjoiQWRtaW5pc3RyYXRpb24iLCJqb2JfdGl0bGUiOiJNYW5hZ2VyIiwidGVhY2hpbmdfc3R5bGUiOm51bGwsImNyZWF0ZWRfYXQiOiIyMDI1LTExLTA5VDA1OjI5OjUwLjgzODg5NzYwMFoiLCJ1cGRhdGVkX2F0IjoiMjAyNS0xMi0xOVQyMjoxMTozNS4zNTQ5NDY4MDBaIn0sImV4cCI6MTc3MjYzMzc4NiwiaWF0IjoxNzcyMDI4OTg2fQ.vEndgk-MFSvM1N3KY3DYtrwNeRJvjeeE86E9tQzaxX0";
const AUTH_TOKEN: &str = "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJ1c2VyIjp7ImlkIjoiNjkxMDI2Y2UzZDAzYTdjNGViNmU2MGZmIiwibmFtZSI6IkhBS0laQVlFWlUgSkVBTiBERSBESUVVIiwiZW1haWwiOiJoYWtpemF5ZXp1amVhbmRlZGlldUBnbWFpbC5jb20iLCJ1c2VybmFtZSI6Il9oYWtpemF5ZXp1X2RpZXVfamVhbl9kZV8xNTYiLCJpbWFnZSI6Imh0dHBzOi8vcmVzLmNsb3VkaW5hcnkuY29tL2RxZm14a3dyei9pbWFnZS91cGxvYWQvdjE3NjI2NjYyNzQvdXJsX3VwbG9hZF8xNzYyNjY2MjcwLmpwZyIsInBob25lIjoiKzI1MDc4ODgxNjk0MCIsInJvbGUiOiJTQ0hPT0xTVEFGRiIsImdlbmRlciI6Ik1BTEUiLCJkaXNhYmxlIjpmYWxzZSwiY3VycmVudF9zY2hvb2xfaWQiOiI2OGVlNTFiNzMwOGFjMjg4NzgzOTA4NWMiLCJzY2hvb2xzIjpbIjY5MTAyODk1M2QwM2E3YzRlYjZlNjEwMSIsIjY4ZWU1MWI3MzA4YWMyODg3ODM5MDg1YyJdLCJhY2Nlc3NpYmxlX2NsYXNzZXMiOm51bGwsImlhdCI6MTc3MjAyODk4NSwiZXhwIjoxNzcyNjMzNzg1fSwiZXhwIjoxNzcyNjMzNzg1LCJpYXQiOjE3NzIwMjg5ODV9.8k70i902o4g6EI6gZkef4p5wdEOVvqUL-ClhZXvGLH0";

#[tokio::main]
async fn main() {
    println!("=== Testing Messaging Users API ===\n");
    
    let client = reqwest::Client::new();
    
    // Test 1: Upload Public Key
    println!("Test 1: Upload Public Key");
    println!("POST {}/users/public-key", BASE_URL);
    
    let upload_body = json!({
        "public_key": "-----BEGIN PUBLIC KEY-----\nMIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEA...\n-----END PUBLIC KEY-----",
        "key_algorithm": "RSA-2048"
    });
    
    match client
        .post(format!("{}/users/public-key", BASE_URL))
        .header("School-Token", SCHOOL_TOKEN)
        .header("Authorization", format!("Bearer {}", AUTH_TOKEN))
        .header("Content-Type", "application/json")
        .json(&upload_body)
        .send()
        .await
    {
        Ok(response) => {
            let status = response.status();
            println!("Status: {}", status);
            
            match response.text().await {
                Ok(body) => {
                    println!("Response: {}", body);
                    if status.is_success() {
                        println!("✓ Upload successful\n");
                    } else {
                        println!("✗ Upload failed\n");
                    }
                }
                Err(e) => println!("Error reading response: {}\n", e),
            }
        }
        Err(e) => println!("✗ Request failed: {}\n", e),
    }
    
    // Test 2: Get Public Keys (single user)
    println!("Test 2: Get Public Keys (single user)");
    let user_id = "691026ce3d03a7c4eb6e60ff";
    println!("GET {}/users/public-keys?user_ids={}", BASE_URL, user_id);
    
    match client
        .get(format!("{}/users/public-keys", BASE_URL))
        .header("School-Token", SCHOOL_TOKEN)
        .header("Authorization", format!("Bearer {}", AUTH_TOKEN))
        .query(&[("user_ids", user_id)])
        .send()
        .await
    {
        Ok(response) => {
            let status = response.status();
            println!("Status: {}", status);
            
            match response.text().await {
                Ok(body) => {
                    println!("Response: {}", body);
                    if status.is_success() {
                        println!("✓ Get public keys successful\n");
                    } else {
                        println!("✗ Get public keys failed\n");
                    }
                }
                Err(e) => println!("Error reading response: {}\n", e),
            }
        }
        Err(e) => println!("✗ Request failed: {}\n", e),
    }
    
    // Test 3: Get Public Keys (multiple users)
    println!("Test 3: Get Public Keys (multiple users)");
    let user_ids = "691026ce3d03a7c4eb6e60ff,68ee51b7308ac28878390850";
    println!("GET {}/users/public-keys?user_ids={}", BASE_URL, user_ids);
    
    match client
        .get(format!("{}/users/public-keys", BASE_URL))
        .header("School-Token", SCHOOL_TOKEN)
        .header("Authorization", format!("Bearer {}", AUTH_TOKEN))
        .query(&[("user_ids", user_ids)])
        .send()
        .await
    {
        Ok(response) => {
            let status = response.status();
            println!("Status: {}", status);
            
            match response.text().await {
                Ok(body) => {
                    println!("Response: {}", body);
                    if status.is_success() {
                        println!("✓ Get multiple public keys successful\n");
                    } else {
                        println!("✗ Get multiple public keys failed\n");
                    }
                }
                Err(e) => println!("Error reading response: {}\n", e),
            }
        }
        Err(e) => println!("✗ Request failed: {}\n", e),
    }
    
    // Test 4: Get Public Keys without user_ids parameter (should fail)
    println!("Test 4: Get Public Keys without user_ids (should fail with 400)");
    println!("GET {}/users/public-keys", BASE_URL);
    
    match client
        .get(format!("{}/users/public-keys", BASE_URL))
        .header("School-Token", SCHOOL_TOKEN)
        .header("Authorization", format!("Bearer {}", AUTH_TOKEN))
        .send()
        .await
    {
        Ok(response) => {
            let status = response.status();
            println!("Status: {}", status);
            
            match response.text().await {
                Ok(body) => {
                    println!("Response: {}", body);
                    if status == 400 {
                        println!("✓ Correctly returned 400 Bad Request\n");
                    } else {
                        println!("✗ Expected 400 but got {}\n", status);
                    }
                }
                Err(e) => println!("Error reading response: {}\n", e),
            }
        }
        Err(e) => println!("✗ Request failed: {}\n", e),
    }
    
    // Test 5: Get Public Keys with invalid user IDs
    println!("Test 5: Get Public Keys with invalid user IDs");
    let invalid_ids = "invalid_id_1,invalid_id_2";
    println!("GET {}/users/public-keys?user_ids={}", BASE_URL, invalid_ids);
    
    match client
        .get(format!("{}/users/public-keys", BASE_URL))
        .header("School-Token", SCHOOL_TOKEN)
        .header("Authorization", format!("Bearer {}", AUTH_TOKEN))
        .query(&[("user_ids", invalid_ids)])
        .send()
        .await
    {
        Ok(response) => {
            let status = response.status();
            println!("Status: {}", status);
            
            match response.text().await {
                Ok(body) => {
                    println!("Response: {}", body);
                    if status == 400 {
                        println!("✓ Correctly returned 400 for invalid IDs\n");
                    } else {
                        println!("✗ Expected 400 but got {}\n", status);
                    }
                }
                Err(e) => println!("Error reading response: {}\n", e),
            }
        }
        Err(e) => println!("✗ Request failed: {}\n", e),
    }
    
    println!("=== All tests completed ===");
}
