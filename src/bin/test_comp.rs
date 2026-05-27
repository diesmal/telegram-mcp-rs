use grammers_client::message::InputMessage;
use grammers_client::peer::Dialog;
use grammers_session::types::PeerId;

fn main() {
    let msg = InputMessage::new().text("hello");
    let dialog: Option<Dialog> = None;
    if let Some(d) = dialog {
        let p = d.peer;
        let id: i64 = p.id().bot_api_dialog_id();
        let name: String = p.name().unwrap_or("").to_string();
    }
}
