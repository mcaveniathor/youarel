use crate::util::{AppState, encode, ShortenReq, ShortenEntry, de_get, ser_insert };
use axum::{
    http::StatusCode, Form,
    response::{ Html, Redirect, Response, IntoResponse, Result},
    extract::{State,Path},
};

const HEAD: &str = r#"
<!doctype html>
    <html lang="en">
        <head>
         <meta name="viewport" content="width=device-width, initial-scale=1">
        <!-- FONT
        –––––––––––––––––––––––––––––––––––––––––––––––––– -->
        <link href="//fonts.googleapis.com/css?family=Raleway:400,300,600" rel="stylesheet" type="text/css">

        <link rel="stylesheet" href="assets/css/skeleton-main.css">
        <link rel="stylesheet" href="assets/css/skeleton-dark.css">
        </head>
"#;




pub async fn redirect(State(state): State<AppState>, Path(key): Path<String>) -> Result<Redirect> {
    let entry_opt: Option<ShortenEntry> = de_get(state.db.clone(), &key).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    match entry_opt {
        Some(mut entry) => {
            if let Some(accesses) = entry.accesses {
                if accesses == 0 {
                    info!("No remaining access for key {}. Removing entry from database.", &key);
                    state.db.remove(&key).map_err(|e| e.to_string())?;
                }
                else {
                    let a = accesses-1;
                    debug!("Decremented access counter for key {}. Remaining: {:?}", &key, a);
                    entry.accesses = Some(a);
                    state.db.insert(&key, bincode::serialize(&entry).map_err(|e| e.to_string())?).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
                }
            }
            Ok(Redirect::to(&entry.long_url.to_string()))
        },
        _ => Err((StatusCode::NOT_FOUND, format!("Shortened URL not found: {}",key)).into())
    }
    /*
    match state.db.get(key) {
        Ok(Some(entry_bincode)) => {
            let mut entry: ShortenEntry = bincode::deserialize(&entry_bincode[..]).map_err(|_| { return StatusCode::INTERNAL_SERVER_ERROR.into_response(); }).unwrap();
                match entry.long_url.to_string() {
                Ok(u_str) => {
                    if let Some(accesses) = entry.accesses_remaining {
                        entry.accesses_remaining -= 1;
                        info!("Decreme
                        state.db.insert(&key, bincode::serialize(entry).unwrap());
                    }
                    Redirect::to(&u_str).into_response()
                }
                Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        },
        Ok(None) => {
            StatusCode::NOT_FOUND.into_response()

        },
        Err(_) => {
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        },
    }
    */
}

pub async fn accept_form(State(state): State<AppState>, Form(shorten_req): Form<ShortenReq>) -> Response {
    let length = shorten_req.length.unwrap_or_else(|| state.default_length);
    let encoded = encode(shorten_req.long_url.as_str(), length); // The encoded value is used as the database key as well
    match ser_insert::<&str,ShortenEntry,ShortenEntry>(state.db, &encoded, &shorten_req.into()) {
        Ok(_) => {
            let mut htmlstr = HEAD.to_string();
                htmlstr += r#"
               <body>
                <div class="container">
                    <div class="row">
                        <div class="six columns">
                            <h2>Short URL:</h2>
                        </div>
                        <div class="six columns">
                        "#;

            htmlstr += &format!("https://{}:{}/",state.hostname, state.port);
            htmlstr += &encoded;
            htmlstr += r#"
                        </div>
                    </div>
                </div>
                </body>
            </html>
            "#;
        Html(htmlstr).into_response()
        },
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
    }
}
pub async fn root() -> Html<String> {
    let mut htmlstr = HEAD.to_string();
    htmlstr += r#"
            <body>
            <div class="container">
                <form action="/" method="post">
                <div class="row">
                    <div class="six columns">
                        <label for="long_url">
                            Long URL
                            <input type="text" name="long_url" id="long_url">
                        </label>
                    </div>
                    <div class="six columns">
                        <label for="accesses">
                        Number of uses before link expires [optional]
                        <input type="number" name="accesses" id="accesses">
                        </label>
                    </div>
                </div>
                <div class="row">
                    <input class="button-primary u-full-width" type="submit" value="Shorten">
                <div>
                </form>
            </div>
            </body>
        </html>
        "#;
    Html(htmlstr)
}

