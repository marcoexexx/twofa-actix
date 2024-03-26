use actix_web::{post, web, HttpResponse, Responder};
use chrono::Utc;
use serde_json::json;
use totp_rs::{Algorithm, Secret, TOTP};
use uuid::Uuid;

use crate::models::{
    AppState, DisableOPTSchema, GenerateOPTSchema, User, UserLoginSchema,
    UserRegisterSchema, VerifyOPTSschema,
};
use crate::response::{GenericResponse, UserData, UserResponse};
use crate::utils::generate_base32_string;

fn user_to_response(user: &User) -> UserData {
    UserData {
        id: user.id.to_owned().unwrap(),
        name: user.name.clone(),
        email: user.email.clone(),
        opt_base32: user.opt_base32.clone(),
        opt_enabled: user.opt_enabled.unwrap(),
        opt_verified: user.opt_verified.unwrap(),
        opt_auth_url: user.opt_auth_url.clone(),
        created_at: user.created_at.unwrap(),
        updated_at: user.updated_at.unwrap(),
    }
}

#[post("/auth/register")]
async fn register_user_handler(
    body: web::Json<UserRegisterSchema>,
    data: web::Data<AppState>,
) -> impl Responder {
    let mut db = data.db.lock().expect("Unable to get database");

    for user in db.iter() {
        if user.email == body.email.to_lowercase() {
            let err_response = GenericResponse {
                status: String::from("fail"),
                message: format!(
                    "User with email: {} already exists",
                    user.email
                ),
            };
            return HttpResponse::Conflict().json(err_response);
        }
    }

    let uuid = Uuid::new_v4();
    let datetime = Utc::now();

    let user = User {
        id: Some(String::from(uuid)),
        email: body.email.clone().to_lowercase(),
        name: body.name.clone(),
        password: body.password.clone(),
        opt_enabled: Some(false),
        opt_verified: Some(false),
        opt_base32: None,
        opt_auth_url: None,
        created_at: Some(datetime),
        updated_at: Some(datetime),
    };

    db.push(user);

    HttpResponse::Ok().json(json!({
        "status": "success",
        "message": "Registered successfully, please login"
    }))
}

#[post("/auth/login")]
async fn login_handler(
    body: web::Json<UserLoginSchema>,
    data: web::Data<AppState>,
) -> impl Responder {
    let db = data.db.lock().expect("Unable to get database");

    let user = db
        .iter()
        .find(|user| user.email == body.email.to_lowercase());

    if let Some(user) = user {
        let user = user.clone();

        let res = UserResponse {
            status: String::from("success"),
            user: user_to_response(&user),
        };

        return HttpResponse::Ok().json(res);
    }

    HttpResponse::BadRequest().json(json!({
      "status": "fail",
      "message": "Invalid email or password"
    }))
}

#[post("/auth/opt/generate")]
async fn generate_opt_handler(
    body: web::Json<GenerateOPTSchema>,
    data: web::Data<AppState>,
) -> impl Responder {
    let mut db = data.db.lock().expect("Unable to get database");

    let user = db
        .iter_mut()
        .find(|user| user.id == Some(String::from(&body.user_id)));

    if let Some(user) = user {
        let base32_string = generate_base32_string();

        let totp = TOTP::new(
            Algorithm::SHA1,
            6,
            1,
            30,
            Secret::Encoded(base32_string).to_bytes().unwrap(),
        )
        .expect("Unable to create topt");

        let opt_base32 = totp.get_secret_base32();
        let email = body.email.clone();
        let issuer = "Marco";
        let opt_auth_url = format!("optauth://totp/{issuer}:{email}?secret={opt_base32}&issuer={issuer}");

        user.opt_base32 = Some(opt_base32.clone());
        user.opt_auth_url = Some(opt_auth_url.clone());

        return HttpResponse::Ok().json(json!({
          "base32": opt_base32,
          "opt_auth_url": opt_auth_url
        }));
    }

    let res = GenericResponse {
        status: String::from("fail"),
        message: format!("No user wit Id: {} found", body.user_id),
    };

    HttpResponse::NotFound().json(res)
}

#[post("/auth/opt/verify")]
async fn verify_opt_handler(
    body: web::Json<VerifyOPTSschema>,
    data: web::Data<AppState>,
) -> impl Responder {
    let mut db = data.db.lock().expect("Unable to get database");

    let user = db
        .iter_mut()
        .find(|user| user.id == Some(String::from(&body.user_id)));

    if let Some(user) = user {
        let opt_base32 =
            user.opt_base32.clone().expect("User opt is none");

        let topt = TOTP::new(
            Algorithm::SHA1,
            6,
            1,
            30,
            Secret::Encoded(opt_base32).to_bytes().unwrap(),
        )
        .expect("Unable to create topt");

        let is_valid = topt.check_current(&body.token).unwrap();

        if !is_valid {
            let res = GenericResponse {
                status: String::from("fail"),
                message: String::from(
                    "Token is invalid or user doesn't exist",
                ),
            };

            return HttpResponse::Forbidden().json(res);
        }

        user.opt_enabled = Some(true);
        user.opt_verified = Some(true);

        return HttpResponse::Ok().json(json!({
          "opt_verified": true,
          "user": user_to_response(user)
        }));
    }

    let res = GenericResponse {
        status: String::from("fail"),
        message: format!("No user with Id: {} found", body.user_id),
    };

    HttpResponse::NotFound().json(json!(res))
}

#[post("/auth/opt/validate")]
async fn validate_opt_handler(
    body: web::Json<VerifyOPTSschema>,
    data: web::Data<AppState>,
) -> impl Responder {
    let db = data.db.lock().expect("Unable to get database");

    let user = db
        .iter()
        .find(|user| user.id == Some(String::from(&body.user_id)));

    if let Some(user) = user {
        if !user.opt_enabled.expect("Failed user opt_enabled is none") {
            let res = GenericResponse {
                status: String::from("fail"),
                message: String::from("2FA not enabled"),
            };

            return HttpResponse::Forbidden().json(res);
        }

        let opt_base32 = user
            .opt_base32
            .clone()
            .expect("Failed to get user opt_base32, it's none");

        let totp = TOTP::new(
            Algorithm::SHA1,
            6,
            1,
            30,
            Secret::Encoded(opt_base32).to_bytes().unwrap(),
        )
        .expect("Unable to create topt");

        let is_valid = totp.check_current(&body.token).unwrap();

        if !is_valid {
            return HttpResponse::Forbidden().json(json!({
              "status": "fail",
              "message": "Token is invalid or user doesn't exist"
            }));
        }

        return HttpResponse::Ok().json(json!({ "opt_valid": true }));
    }

    let res = GenericResponse {
        status: String::from("fail"),
        message: format!("No user with Id: {} found", body.user_id),
    };

    HttpResponse::NotFound().json(json!(res))
}

#[post("/auth/opt/disable")]
async fn disable_opt_handler(
    body: web::Json<DisableOPTSchema>,
    data: web::Data<AppState>,
) -> impl Responder {
    let mut db = data.db.lock().expect("Unable to get database");

    let user = db
        .iter_mut()
        .find(|user| user.id == Some(String::from(&body.user_id)));

    if let Some(user) = user {
        user.opt_enabled = Some(false);
        user.opt_verified = Some(false);
        user.opt_auth_url = None;
        user.opt_base32 = None;

        return HttpResponse::Ok()
            .json(json!({ "user": user_to_response(user), "opt_disabled": true }));
    }

    HttpResponse::NotFound().json(json!({}))
}

pub fn config(conf: &mut web::ServiceConfig) {
    let scope = web::scope("/api")
        .service(register_user_handler)
        .service(login_handler)
        .service(generate_opt_handler)
        .service(verify_opt_handler)
        .service(validate_opt_handler)
        .service(disable_opt_handler);

    conf.service(scope);
}
