use std::str::FromStr;

use stq_router::RouteParser;
use stq_static_resources::TemplateVariant;
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
    StoreModerationStatusForUser,
    BaseProductModerationStatusForUser,
    StoreModerationStatusForModerator,
    BaseProductModerationStatusForModerator,
    Roles,
    RoleById { id: RoleId },
    RolesByUserId { user_id: UserId },
    Templates { template: TemplateVariant },
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

    router.add_route(r"^/users/stores/update-moderation-status$", || Route::StoreModerationStatusForUser);
    router.add_route(r"^/users/base_products/update-moderation-status", || {
        Route::BaseProductModerationStatusForUser
    });
    router.add_route(r"^/moderators/stores/update-moderation-status$", || {
        Route::StoreModerationStatusForModerator
    });
    router.add_route(r"^/moderators/base_products/update-moderation-status$", || {
        Route::BaseProductModerationStatusForModerator
    });

    router.add_route(r"^/roles$", || Route::Roles);

    router.add_route_with_params(r"^/roles/by-user-id/(\d+)$", |params| {
        params
            .get(0)
            .and_then(|string_id| string_id.parse().ok())
            .map(|user_id| Route::RolesByUserId { user_id })
    });

    router.add_route_with_params(r"^/roles/by-id/([a-zA-Z0-9-]+)$", |params| {
        params
            .get(0)
            .and_then(|string_id| string_id.parse().ok())
            .map(|id| Route::RoleById { id })
    });

    router.add_route_with_params(r"^/templates/([a-zA-Z-_]+)$", |params| {
        params
            .get(0)
            .and_then(|string_template| TemplateVariant::from_str(string_template).ok())
            .map(|template| Route::Templates { template })
    });

    router
}
