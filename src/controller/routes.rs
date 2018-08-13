use stq_router::RouteParser;
use stq_types::*;

/// List of all routes with params for the app
#[derive(Clone, Debug, PartialEq)]
pub enum Route {
    SimpleMail,
    OrderUpdateStateForUser,
    OrderUpdateStateForStore,
    OrderCreateForUser,
    OrderCreateForStore,
    EmailVerificationForUser,
    PasswordResetForUser,
    ApplyPasswordResetForUser,
    ApplyEmailVerificationForUser,
    UserRoles,
    UserRole(UserId),
    DefaultRole(UserId),
}

pub fn create_route_parser() -> RouteParser<Route> {
    let mut router = RouteParser::default();

    // Simple Mail
    router.add_route(r"^/simple-mail$", || Route::SimpleMail);
    // OrderUpdateStateForUser
    router.add_route(r"^/users/order-update-state$", || Route::OrderUpdateStateForUser);
    // OrderUpdateStateForStore
    router.add_route(r"^/stores/order-update-state$", || Route::OrderUpdateStateForStore);
    // OrderCreateForUser
    router.add_route(r"^/users/order-create$", || Route::OrderCreateForUser);
    // OrderCreateForStore
    router.add_route(r"^/stores/order-create$", || Route::OrderCreateForStore);
    // EmailVerificationForUser
    router.add_route(r"^/users/email-verification$", || Route::EmailVerificationForUser);
    // ApplyEmailVerificationForUser
    router.add_route(r"^/users/apply-email-verification$", || Route::ApplyEmailVerificationForUser);
    // PasswordResetForUser
    router.add_route(r"^/users/password-reset$", || Route::PasswordResetForUser);
    // ApplyPasswordResetForUser
    router.add_route(r"^/users/apply-password-reset$", || Route::ApplyPasswordResetForUser);

    // User_roles Routes
    router.add_route(r"^/user_roles$", || Route::UserRoles);

    // User_roles/:id route
    router.add_route_with_params(r"^/user_roles/(\d+)$", |params| {
        params
            .get(0)
            .and_then(|string_id| string_id.parse::<i32>().ok())
            .map(UserId)
            .map(Route::UserRole)
    });

    // roles/default/:id route
    router.add_route_with_params(r"^/roles/default/(\d+)$", |params| {
        params
            .get(0)
            .and_then(|string_id| string_id.parse::<i32>().ok())
            .map(UserId)
            .map(Route::DefaultRole)
    });
    router
}
