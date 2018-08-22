use stq_router::RouteParser;
use stq_types::*;

/// List of all routes with params for the app
#[derive(Clone, Debug, PartialEq)]
pub enum Route {
    SimpleMail,
    OrderUpdateStateForUser,
    TemplateOrderUpdateStateForUser,
    OrderUpdateStateForStore,
    TemplateOrderUpdateStateForStore,
    OrderCreateForUser,
    TemplateOrderCreateForUser,
    OrderCreateForStore,
    TemplateOrderCreateForStore,
    EmailVerificationForUser,
    TemplateEmailVerificationForUser,
    PasswordResetForUser,
    TemplatePasswordResetForUser,
    ApplyPasswordResetForUser,
    TemplateApplyPasswordResetForUser,
    ApplyEmailVerificationForUser,
    TemplateApplyEmailVerificationForUser,
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
    // TemplateOrderUpdateStateForUser
    router.add_route(r"^/users/template-order-update-state$", || Route::TemplateOrderUpdateStateForUser);
    // OrderUpdateStateForStore
    router.add_route(r"^/stores/order-update-state$", || Route::OrderUpdateStateForStore);
    // TemplateOrderUpdateStateForStore
    router.add_route(r"^/stores/template-order-update-state$", || Route::TemplateOrderUpdateStateForStore);
    // OrderCreateForUser
    router.add_route(r"^/users/order-create$", || Route::OrderCreateForUser);
    // TemplateOrderCreateForUser
    router.add_route(r"^/users/template-order-create$", || Route::TemplateOrderCreateForUser);
    // OrderCreateForStore
    router.add_route(r"^/stores/order-create$", || Route::OrderCreateForStore);
    // TemplateOrderCreateForStore
    router.add_route(r"^/stores/template-order-create$", || Route::TemplateOrderCreateForStore);
    // EmailVerificationForUser
    router.add_route(r"^/users/email-verification$", || Route::EmailVerificationForUser);
    // TemplateEmailVerificationForUser
    router.add_route(r"^/users/template-email-verification$", || Route::TemplateEmailVerificationForUser);
    // ApplyEmailVerificationForUser
    router.add_route(r"^/users/apply-email-verification$", || Route::ApplyEmailVerificationForUser);
    // TemplateApplyEmailVerificationForUser
    router.add_route(r"^/users/template-apply-email-verification$", || {
        Route::TemplateApplyEmailVerificationForUser
    });
    // PasswordResetForUser
    router.add_route(r"^/users/password-reset$", || Route::PasswordResetForUser);
    // TemplatePasswordResetForUser
    router.add_route(r"^/users/template-password-reset$", || Route::TemplatePasswordResetForUser);
    // ApplyPasswordResetForUser
    router.add_route(r"^/users/apply-password-reset$", || Route::ApplyPasswordResetForUser);
    // TemplateApplyPasswordResetForUser
    router.add_route(r"^/users/template-apply-password-reset$", || {
        Route::TemplateApplyPasswordResetForUser
    });

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
