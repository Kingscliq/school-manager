use axum::{
    Router,
    middleware::from_fn_with_state,
    routing::{get, post},
};

use crate::{
    auth::{
        handlers::{
            create_school_handler, get_all_schools_handler, get_all_users_handler, login_handler,
            me_handler, register_handler,
        },
        middleware::auth_middleware,
    },
    models::AppStore,
};

pub fn auth_routes(store: AppStore) -> Router<AppStore> {
    let protected = Router::new()
        .route("/me", get(me_handler))
        .route("/users", get(get_all_users_handler))
        .route("/schools", get(get_all_schools_handler))
        .route_layer(from_fn_with_state(store, auth_middleware));

    Router::new()
        .route("/schools", post(create_school_handler))
        .route("/register", post(register_handler))
        .route("/login", post(login_handler))
        .merge(protected)
}
