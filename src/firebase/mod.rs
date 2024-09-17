use std::collections::HashMap;
use diesel_async::RunQueryDsl;
use fireauth::FireAuth;
use firebase_rs::Firebase;
use crate::auth::{Credentials, ID};
use crate::backend::DBConnection;
use crate::error::{APIError, APIResult};
use crate::schema::user_data;
use crate::user_data::UserData;

/*
const firebaseConfig = {
  apiKey: "AIzaSyCi8_8p1CJXnZji5h8fjgvbR9bdDHPr4YQ",
  authDomain: "ropelab-e17f9.firebaseapp.com",
  databaseURL: "https://ropelab-e17f9-default-rtdb.europe-west1.firebasedatabase.app",
  projectId: "ropelab-e17f9",
  storageBucket: "ropelab-e17f9.appspot.com",
  messagingSenderId: "90216435446",
  appId: "1:90216435446:web:e5f4b0daf7d767cce7e6d7"
};
 */

const FIREBASE_URI: &str = "https://ropelab-e17f9-default-rtdb.europe-west1.firebasedatabase.app";
const FIREBASE_API_KEY: &str = "AIzaSyCi8_8p1CJXnZji5h8fjgvbR9bdDHPr4YQ";

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct FirebaseUserData {
    pub activePercent: i32,
    pub email: String,
    pub experienceText: String,
    pub fetname: String,
    pub foundUsText: String,
    pub goalText: String,
    pub name: String,
    pub open: bool,
    pub passivePercent: i32,
    pub rolePercent: i32,
    pub showExperience: bool,
    pub showName: bool,
    pub showOpen: bool,
    pub showRole: bool,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct FirebaseEvent {
    pub archive: bool,
    pub attended: Option<HashMap<String, bool>>,
    pub date: String,
    pub eid: i32,
    pub guests: HashMap<String, i32>,
    pub registered: Vec<String>,
    pub slots: i32,
    pub text: String,
    pub visible: bool,
    pub waiting: Option<Vec<String>>,
}

pub async fn firebase_login_user(credentials: Credentials) -> APIResult<String> {
    let auth = FireAuth::new(FIREBASE_API_KEY.to_string());
    
    let response = auth
        .sign_in_email(&credentials.email, &credentials.password, true)
        .await
        .map_err(|_| APIError::InvalidCredentials)?;

    Ok(response.local_id)
}

pub async fn firebase_get_user_data(firebase_user_id: &str) -> APIResult<FirebaseUserData> {
    let firebase = Firebase::auth(FIREBASE_URI, FIREBASE_API_KEY)
        .map_err(APIError::internal)?;
    
    let user_data = firebase.at(&format!("users/{firebase_user_id}")).get::<FirebaseUserData>().await
        .map_err(APIError::internal)?;
    
    Ok(user_data)
}

pub async fn firebase_is_user_verified(firebase_user_id: &str) -> APIResult<bool> {
    let firebase = Firebase::auth(FIREBASE_URI, FIREBASE_API_KEY)
        .map_err(APIError::internal)?;

    let verified = firebase.at(&format!("approved/{firebase_user_id}")).get::<bool>().await
        .map_err(APIError::internal)?;

    Ok(verified)
}

pub async fn get_firebase_events() -> APIResult<Vec<FirebaseEvent>> {
    let firebase = Firebase::auth(FIREBASE_URI, FIREBASE_API_KEY)
        .map_err(APIError::internal)?;

    let events = firebase.at(&format!("events")).get::<Vec<Option<FirebaseEvent>>>().await
        .map_err(APIError::internal)?
        .into_iter()
        .filter_map(|e| e)
        .collect();

    Ok(events)
}

pub async fn firebase_is_user_new(firebase_user_id: &str) -> APIResult<bool> {
    let events = get_firebase_events().await?;
    
    let mut found = false;
    for event in events {
        for test_id in event.registered {
            if test_id == firebase_user_id {
                found = true;
            }
        }
    }
    
    Ok(!found)
}

pub async fn insert_user_data_from_firebase(u_id: ID, firebase_user_data: FirebaseUserData, new: bool, conn: &mut DBConnection) -> APIResult<()> {
    let user_data = UserData {
        user_id: u_id,
        name: firebase_user_data.name,
        fetlife_name: firebase_user_data.fetname,
        experience_text: firebase_user_data.experienceText,
        found_us_text: firebase_user_data.foundUsText,
        goal_text: firebase_user_data.goalText,
        role_factor: firebase_user_data.rolePercent as f64,
        open: firebase_user_data.open,
        show_name: firebase_user_data.showName,
        show_role: firebase_user_data.showRole,
        show_open: firebase_user_data.showOpen,
        show_fetlife: false,
        new,
    };
    
    diesel::insert_into(user_data::table)
        .values(&user_data)
        .execute(&mut conn.0)
        .await
        .map_err(APIError::internal)?;
    
    Ok(())
}

