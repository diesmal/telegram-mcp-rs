use grammers_client::types::Chat;
use grammers_tl_types::enums::InputPeer;

pub fn test(chat: Chat) -> InputPeer {
    chat.pack().to_input_peer()
}
