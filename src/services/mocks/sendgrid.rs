use models::SendGridPayload;
use services::sendgrid::SendgridService;
use services::types::ServiceFuture;

pub struct SendgridServiceMock;

impl SendgridService for SendgridServiceMock {
    fn send(&self, _payload: SendGridPayload) -> ServiceFuture<()> {
        Box::new(::futures::future::ok(()))
    }
}
