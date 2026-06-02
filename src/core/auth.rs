// Authentication Module
// Handles Microsoft/Xbox Live OAuth authentication flow for Minecraft
//
// Flow:
// 1. User authorizes via Microsoft OAuth 2.0
// 2. Exchange auth code for Microsoft access token
// 3. Authenticate with Xbox Live to get XBL token
// 4. Authenticate with XSTS to get XSTS token
// 5. Authenticate with Minecraft services to get MC token
// 6. Fetch Minecraft profile

use serde::{Deserialize, Serialize};
use reqwest::Client;
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use std::sync::{Arc, Mutex};
use log::{info, warn, error, debug};

// OAuth 2.0 Configuration
const CLIENT_ID: &str = "a60a8e54-8c3c-4b25-9fe4-8ec4e616c410";
const REDIRECT_URI: &str = "http://127.0.0.1:8080/";
const SCOPE: &str = "XboxLive.signin offline_access";
const LISTEN_PORT: u16 = 8080;

// Microsoft OAuth Endpoints
const MS_AUTH_URL: &str = "https://login.microsoftonline.com/consumers/oauth2/v2.0/authorize";
const MS_TOKEN_URL: &str = "https://login.microsoftonline.com/consumers/oauth2/v2.0/token";

// Xbox Live Authentication Endpoints
const XBL_AUTH_URL: &str = "https://user.auth.xboxlive.com/user/authenticate";
const XSTS_AUTH_URL: &str = "https://xsts.auth.xboxlive.com/xsts/authorize";

// Minecraft Services Endpoints
const MC_AUTH_URL: &str = "https://api.minecraftservices.com/authentication/login_with_xbox";
const MC_PROFILE_URL: &str = "https://api.minecraftservices.com/minecraft/profile";

/// Minecraft user profile information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinecraftProfile {
    /// Unique player ID (without dashes)
    pub id: String,
    /// Player username (skin name)
    pub name: String,
    /// Player skins (optional)
    #[serde(rename = "skins")]
    pub skins: Option<Vec<Skin>>,
}

/// Minecraft skin information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skin {
    /// Skin ID
    pub id: String,
    /// Skin state (active/inactive)
    pub state: String,
    /// Skin texture URL
    pub url: String,
    /// Skin variant (classic/slim)
    #[serde(rename = "variant")]
    pub variant: Option<String>,
}

/// OAuth tokens from Microsoft authentication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthTokens {
    /// Access token for API calls
    pub access_token: String,
    /// Refresh token for obtaining new access tokens
    pub refresh_token: String,
    /// Token expiration time in seconds
    pub expires_in: u64,
}

/// Xbox Live authentication response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XboxLiveAuthResponse {
    /// Token issue timestamp
    #[serde(rename = "IssueInstant")]
    pub issue_instant: String,
    /// Token expiration timestamp
    #[serde(rename = "NotAfter")]
    pub not_after: String,
    /// Xbox Live token
    #[serde(rename = "Token")]
    pub token: String,
    /// Display claims containing user hash
    #[serde(rename = "DisplayClaims")]
    pub display_claims: DisplayClaims,
}

/// Display claims container
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayClaims {
    /// Xbox User Identity claims
    pub xui: Vec<XuiClaim>,
}

/// Xbox User Identity claim
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XuiClaim {
    /// User hash (used in XBL3.0 authentication)
    pub uhs: String,
}

/// XSTS authentication response (same structure as Xbox Live)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XSTSAuthResponse {
    /// Token issue timestamp
    #[serde(rename = "IssueInstant")]
    pub issue_instant: String,
    /// Token expiration timestamp
    #[serde(rename = "NotAfter")]
    pub not_after: String,
    /// XSTS token
    #[serde(rename = "Token")]
    pub token: String,
    /// Display claims containing user hash
    #[serde(rename = "DisplayClaims")]
    pub display_claims: DisplayClaims,
}

/// Minecraft authentication response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinecraftAuthResponse {
    /// Minecraft access token
    pub access_token: String,
    /// Token expiration time in seconds
    pub expires_in: u64,
    /// Token type (usually "Bearer")
    pub token_type: String,
}

/// Complete account session with all tokens and profile info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountSession {
    /// Minecraft user profile
    pub minecraft_profile: MinecraftProfile,
    /// Minecraft access token
    pub access_token: String,
    /// Microsoft refresh token for re-authentication
    pub refresh_token: String,
    /// Xbox Live token
    pub xbl_token: String,
    /// XSTS token
    pub xsts_token: String,
    /// Xbox User Identity hash
    pub xbl_uhs: String,
}

/// Generates the Microsoft OAuth authorization URL.
/// The state parameter is used to prevent CSRF attacks.
pub fn get_auth_url(state: &str) -> String {
    format!(
        "{}?client_id={}&response_type=code&redirect_uri={}&scope={}&state={}&prompt=select_account",
        MS_AUTH_URL,
        CLIENT_ID,
        urlencoding::encode(REDIRECT_URI),
        urlencoding::encode(SCOPE),
        state
    )
}

/// Exchanges an authorization code for Microsoft access and refresh tokens.
///
/// # Arguments
/// * `code` - The authorization code from the OAuth callback
///
/// # Returns
/// * `Ok(AuthTokens)` - The access and refresh tokens
/// * `Err(String)` - Error message if the exchange fails
pub async fn exchange_code_for_tokens(code: &str) -> Result<AuthTokens, String> {
    info!("Exchanging authorization code for tokens...");
    let client = Client::new();
    let mut params = HashMap::new();
    params.insert("client_id", CLIENT_ID);
    params.insert("code", code);
    params.insert("grant_type", "authorization_code");
    params.insert("redirect_uri", REDIRECT_URI);

    let response = client
        .post(MS_TOKEN_URL)
        .form(&params)
        .send()
        .await
        .map_err(|e| {
            error!("Failed to exchange code: {}", e);
            format!("Failed to exchange code: {}", e)
        })?;

    if !response.status().is_success() {
        let error_text = response.text().await.unwrap_or_default();
        error!("Token exchange failed with status: {}", error_text);
        return Err(format!("Token exchange failed: {}", error_text));
    }

    let token_response: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse token response: {}", e))?;

    info!("Successfully obtained Microsoft tokens");
    Ok(AuthTokens {
        access_token: token_response["access_token"]
            .as_str()
            .ok_or("Missing access_token")?
            .to_string(),
        refresh_token: token_response["refresh_token"]
            .as_str()
            .ok_or("Missing refresh_token")?
            .to_string(),
        expires_in: token_response["expires_in"]
            .as_u64()
            .unwrap_or(3600),
    })
}

/// Refreshes an expired Microsoft access token using the refresh token.
///
/// # Arguments
/// * `refresh_token` - The refresh token from previous authentication
///
/// # Returns
/// * `Ok(AuthTokens)` - New access and refresh tokens
/// * `Err(String)` - Error message if the refresh fails
pub async fn refresh_access_token(refresh_token: &str) -> Result<AuthTokens, String> {
    let client = Client::new();
    let mut params = HashMap::new();
    params.insert("client_id", CLIENT_ID);
    params.insert("refresh_token", refresh_token);
    params.insert("grant_type", "refresh_token");

    let response = client
        .post(MS_TOKEN_URL)
        .form(&params)
        .send()
        .await
        .map_err(|e| format!("Failed to refresh token: {}", e))?;

    if !response.status().is_success() {
        let error_text = response.text().await.unwrap_or_default();
        return Err(format!("Token refresh failed: {}", error_text));
    }

    let token_response: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse refresh response: {}", e))?;

    Ok(AuthTokens {
        access_token: token_response["access_token"]
            .as_str()
            .ok_or("Missing access_token")?
            .to_string(),
        refresh_token: token_response["refresh_token"]
            .as_str()
            .or(Some(refresh_token))
            .unwrap()
            .to_string(),
        expires_in: token_response["expires_in"]
            .as_u64()
            .unwrap_or(3600),
    })
}

/// Authenticates with Xbox Live using a Microsoft access token.
///
/// # Arguments
/// * `ms_token` - Microsoft access token
///
/// # Returns
/// * `Ok((token, uhs))` - Xbox Live token and user hash
/// * `Err(String)` - Error message if authentication fails
pub async fn authenticate_xbox_live(ms_token: &str) -> Result<(String, String), String> {
    info!("Authenticating with Xbox Live...");
    let client = Client::new();
    
    let payload = serde_json::json!({
        "Properties": {
            "AuthMethod": "RPS",
            "SiteName": "user.auth.xboxlive.com",
            "RpsTicket": format!("d={}", ms_token)
        },
        "RelyingParty": "http://auth.xboxlive.com",
        "TokenType": "JWT"
    });

    let response = client
        .post(XBL_AUTH_URL)
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .json(&payload)
        .send()
        .await
        .map_err(|e| {
            error!("Xbox Live auth request failed: {}", e);
            format!("Xbox Live auth failed: {}", e)
        })?;

    if !response.status().is_success() {
        let error_text = response.text().await.unwrap_or_default();
        error!("Xbox Live auth failed with status: {}", error_text);
        return Err(format!("Xbox Live auth failed: {}", error_text));
    }

    let xbl_response: XboxLiveAuthResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse Xbox Live response: {}", e))?;

    let uhs = xbl_response.display_claims.xui
        .first()
        .ok_or("No xui claims found")?
        .uhs
        .clone();

    info!("Xbox Live authentication successful");
    Ok((xbl_response.token, uhs))
}

/// Authenticates with XSTS (Xbox Security Token Service) using an Xbox Live token.
///
/// # Arguments
/// * `xbl_token` - Xbox Live token
///
/// # Returns
/// * `Ok((token, uhs))` - XSTS token and user hash
/// * `Err(String)` - Error message if authentication fails
pub async fn authenticate_xsts(xbl_token: &str) -> Result<(String, String), String> {
    info!("Authenticating with XSTS...");
    let client = Client::new();
    
    let payload = serde_json::json!({
        "Properties": {
            "SandboxId": "RETAIL",
            "UserTokens": [xbl_token]
        },
        "RelyingParty": "rp://api.minecraftservices.com/",
        "TokenType": "JWT"
    });

    let response = client
        .post(XSTS_AUTH_URL)
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .json(&payload)
        .send()
        .await
        .map_err(|e| {
            error!("XSTS auth request failed: {}", e);
            format!("XSTS auth failed: {}", e)
        })?;

    if !response.status().is_success() {
        let error_text = response.text().await.unwrap_or_default();
        
        // Handle specific Xbox Live error codes
        if error_text.contains("2148916233") {
            error!("Xbox Live account is a child account (under 18)");
            return Err("Xbox Live account is a child account (under 18) and requires adult approval".to_string());
        } else if error_text.contains("2148916238") {
            error!("Xbox Live account is banned");
            return Err("Xbox Live account is banned".to_string());
        }
        
        error!("XSTS auth failed: {}", error_text);
        return Err(format!("XSTS auth failed: {}", error_text));
    }

    let xsts_response: XSTSAuthResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse XSTS response: {}", e))?;

    let uhs = xsts_response.display_claims.xui
        .first()
        .ok_or("No xui claims found")?
        .uhs
        .clone();

    Ok((xsts_response.token, uhs))
}

/// Authenticates with Minecraft services using XSTS token.
///
/// # Arguments
/// * `xsts_token` - XSTS token
/// * `xbl_uhs` - Xbox User Identity hash
///
/// # Returns
/// * `Ok(MinecraftAuthResponse)` - Minecraft access token
/// * `Err(String)` - Error message if authentication fails
pub async fn authenticate_minecraft(xsts_token: &str, xbl_uhs: &str) -> Result<MinecraftAuthResponse, String> {
    info!("Authenticating with Minecraft services...");
    let client = Client::new();
    
    let payload = serde_json::json!({
        "identityToken": format!("XBL3.0 x={};{}", xbl_uhs, xsts_token)
    });

    let response = client
        .post(MC_AUTH_URL)
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .json(&payload)
        .send()
        .await
        .map_err(|e| {
            error!("Minecraft auth request failed: {}", e);
            format!("Minecraft auth failed: {}", e)
        })?;

    if !response.status().is_success() {
        let error_text = response.text().await.unwrap_or_default();
        error!("Minecraft auth failed: {}", error_text);
        return Err(format!("Minecraft auth failed: {}", error_text));
    }

    let mc_response: MinecraftAuthResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse Minecraft response: {}", e))?;

    info!("Minecraft authentication successful");
    Ok(mc_response)
}

/// Fetches the Minecraft user profile using the access token.
///
/// # Arguments
/// * `mc_token` - Minecraft access token
///
/// # Returns
/// * `Ok(MinecraftProfile)` - User profile information
/// * `Err(String)` - Error message if the request fails
pub async fn get_minecraft_profile(mc_token: &str) -> Result<MinecraftProfile, String> {
    info!("Fetching Minecraft profile...");
    let client = Client::new();

    let response = client
        .get(MC_PROFILE_URL)
        .header("Authorization", format!("Bearer {}", mc_token))
        .send()
        .await
        .map_err(|e| {
            error!("Failed to get Minecraft profile: {}", e);
            format!("Failed to get Minecraft profile: {}", e)
        })?;

    if !response.status().is_success() {
        let error_text = response.text().await.unwrap_or_default();
        error!("Failed to get Minecraft profile: {}", error_text);
        return Err(format!("Failed to get Minecraft profile: {}", error_text));
    }

    let profile: MinecraftProfile = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse Minecraft profile: {}", e))?;

    info!("Minecraft profile fetched: {} ({})", profile.name, profile.id);
    Ok(profile)
}

/// Performs the complete login flow from authorization code to Minecraft profile.
///
/// # Arguments
/// * `code` - The OAuth authorization code
///
/// # Returns
/// * `Ok(AccountSession)` - Complete session with all tokens and profile
/// * `Err(String)` - Error message if any step fails
pub async fn full_login_flow(code: &str) -> Result<AccountSession, String> {
    info!("Starting full login flow...");
    
    // Step 1: Exchange code for Microsoft tokens
    debug!("Step 1: Exchanging code for Microsoft tokens");
    let ms_tokens = exchange_code_for_tokens(code).await?;
    
    // Step 2: Authenticate with Xbox Live
    debug!("Step 2: Authenticating with Xbox Live");
    let (xbl_token, _xbl_uhs) = authenticate_xbox_live(&ms_tokens.access_token).await?;
    
    // Step 3: Authenticate with XSTS
    debug!("Step 3: Authenticating with XSTS");
    let (xsts_token, xsts_uhs) = authenticate_xsts(&xbl_token).await?;
    
    // Step 4: Authenticate with Minecraft
    debug!("Step 4: Authenticating with Minecraft");
    let mc_auth = authenticate_minecraft(&xsts_token, &xsts_uhs).await?;
    
    // Step 5: Get Minecraft profile
    debug!("Step 5: Fetching Minecraft profile");
    let profile = get_minecraft_profile(&mc_auth.access_token).await?;
    
    info!("Login flow completed successfully for user: {}", profile.name);
    
    Ok(AccountSession {
        minecraft_profile: profile,
        access_token: mc_auth.access_token,
        refresh_token: ms_tokens.refresh_token,
        xbl_token,
        xsts_token,
        xbl_uhs: xsts_uhs,
    })
}

/// Refreshes an existing session using the refresh token.
///
/// # Arguments
/// * `refresh_token` - The refresh token from previous authentication
///
/// # Returns
/// * `Ok(AccountSession)` - Refreshed session with new tokens
/// * `Err(String)` - Error message if refresh fails
pub async fn refresh_session(refresh_token: &str) -> Result<AccountSession, String> {
    // Step 1: Refresh Microsoft token
    let ms_tokens = refresh_access_token(refresh_token).await?;
    
    // Step 2: Authenticate with Xbox Live
    let (xbl_token, _xbl_uhs) = authenticate_xbox_live(&ms_tokens.access_token).await?;
    
    // Step 3: Authenticate with XSTS
    let (xsts_token, xsts_uhs) = authenticate_xsts(&xbl_token).await?;
    
    // Step 4: Authenticate with Minecraft
    let mc_auth = authenticate_minecraft(&xsts_token, &xsts_uhs).await?;
    
    // Step 5: Get Minecraft profile
    let profile = get_minecraft_profile(&mc_auth.access_token).await?;
    
    Ok(AccountSession {
        minecraft_profile: profile,
        access_token: mc_auth.access_token,
        refresh_token: ms_tokens.refresh_token,
        xbl_token,
        xsts_token,
        xbl_uhs: xsts_uhs,
    })
}

/// Starts a local HTTP server to receive the OAuth callback.
/// Listens on port 8080 for the authorization code.
///
/// # Returns
/// * `Ok(String)` - The authorization code from the callback
/// * `Err(String)` - Error message if the server fails or times out
pub async fn wait_for_auth_code() -> Result<String, String> {
    info!("Starting local callback server on port {}...", LISTEN_PORT);
    
    // Try to bind to the port
    let listener = TcpListener::bind(format!("127.0.0.1:{}", LISTEN_PORT))
        .map_err(|e| {
            warn!("Port {} is busy: {}", LISTEN_PORT, e);
            format!("Port {} is busy: {}", LISTEN_PORT, e)
        })?;
    
    // Set non-blocking mode for async compatibility
    listener.set_nonblocking(true)
        .map_err(|e| format!("Failed to set non-blocking: {}", e))?;
    
    info!("Callback server listening on http://127.0.0.1:{}", LISTEN_PORT);
    
    // Shared state for the received code
    let code: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
    let code_clone = code.clone();
    
    // Spawn a blocking task to handle incoming connections
    let listener_handle = tokio::task::spawn_blocking(move || {
        for stream in listener.incoming() {
            match stream {
                Ok(mut stream) => {
                    let mut reader = BufReader::new(&stream);
                    let mut request_line = String::new();
                    reader.read_line(&mut request_line).unwrap_or_default();
                    
                    debug!("Received request: {}", request_line.trim());
                    
                    // Check if this is the callback request (root path with code parameter)
                    if request_line.contains("GET /") && request_line.contains("code=") {
                        // Extract the authorization code from the query string
                        if let Some(code_part) = request_line.split("code=").nth(1) {
                            let code = code_part.split("&").next().unwrap_or("").to_string();
                            if !code.is_empty() {
                                info!("Authorization code received");
                                
                                // Store the code
                                let mut locked = code_clone.lock().unwrap();
                                *locked = Some(code);
                                
                                // Send success response to browser
                                let response = "HTTP/1.1 200 OK\r\n\
                                    Content-Type: text/html; charset=utf-8\r\n\
                                    \r\n\
                                    <html><body style='font-family: sans-serif; text-align: center; padding: 50px;'>\
                                    <h1>Login Successful!</h1>\
                                    <p>You can close this window and return to the launcher.</p>\
                                    <script>setTimeout(() => window.close(), 2000);</script>\
                                    </body></html>";
                                
                                stream.write_all(response.as_bytes()).unwrap_or_default();
                                stream.flush().unwrap_or_default();
                                break;
                            }
                        }
                        
                        // Send error response if code is missing
                        let error_response = "HTTP/1.1 400 Bad Request\r\n\
                            Content-Type: text/html; charset=utf-8\r\n\
                            \r\n\
                            <html><body style='font-family: sans-serif; text-align: center; padding: 50px;'>\
                            <h1>Missing authorization code</h1>\
                            <p>Please try again.</p>\
                            </body></html>";
                        
                        stream.write_all(error_response.as_bytes()).unwrap_or_default();
                        stream.flush().unwrap_or_default();
                    }
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    // No connection available, sleep briefly and retry
                    std::thread::sleep(std::time::Duration::from_millis(100));
                    continue;
                }
                Err(_) => break,
            }
        }
    });
    
    // Wait for the code with a timeout
    let timeout = std::time::Duration::from_secs(300); // 5 minutes
    let start = std::time::Instant::now();
    
    debug!("Waiting for authorization code (timeout: 5 minutes)...");
    
    loop {
        // Check if code was received
        {
            let locked = code.lock().unwrap();
            if let Some(code) = locked.as_ref() {
                info!("Authorization code received, closing callback server");
                listener_handle.abort();
                return Ok(code.clone());
            }
        }
        
        // Check for timeout
        if start.elapsed() > timeout {
            warn!("Authentication timed out after 5 minutes");
            listener_handle.abort();
            return Err("Authentication timed out after 5 minutes".to_string());
        }
        
        // Sleep briefly before checking again
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
}

/// Performs the complete automatic login flow.
/// Opens browser and waits for callback to receive the authorization code.
///
/// # Returns
/// * `Ok(AccountSession)` - Complete session with all tokens and profile
/// * `Err(String)` - Error message if any step fails
pub async fn auto_login_flow() -> Result<AccountSession, String> {
    info!("Starting automatic login flow...");
    let code = wait_for_auth_code().await?;
    full_login_flow(&code).await
}