use jsonwebtoken::{decode, DecodingKey, Validation};
use reqwest::header::HEADER_AUTHORIZATION;

fn get_jwt_token(req: &Request) -> Option<String> {
    if let Some(cookie) = req.cookies().get("token") {
        return Some(cookie.value().to_owned());
    }

    if let Some(header) = req.headers().get(HEADER_AUTHORIZATION) {
        if let Ok(header_str) = header.to_str() {
            return header_str.split(' ').nth(1).map(|s| s.to_owned());
        }
    }

    None
}

async fn verify(req: Request, res: Response) -> Result<(), Error> {
    let token = get_jwt_token(&req).ok_or_else(|| Error::new("Token not found"))?;

    let secret_key = match env::var("SECRET_KEY") {
        Ok(key) => key,
        Err(_) => return Err(Error::new("SECRET_KEY not set")),
    };

    let token_data = decode::<Claims>(
        &token,
        &DecodingKey::from_secret(secret_key.as_ref()),
        &Validation::default(),
    );

    match token_data {
        Ok(data) => {
            req.extensions_mut().insert(data.claims);
            Ok(())
        }
        Err(e) => {

            Err(Error::new("Failed to decode token"))
        }
    }
}
