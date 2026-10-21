use std::path::Prefix;

use crate::mc::{ Decode, DecodeError, Direction, Parse, PhasePayload };
use crate::mc::types::{ Bool, Byte, Either, Int, Long, PrefixedArray, PrefixedOptional, String, UUID, UnsignedByte, VarInt };

#[derive(Debug)]
pub enum Payload {
    // Serverbound
    ClientInformation{
        locale: String,
        view_distance: Byte,
        chat_mode: VarInt,
        char_colors: Bool,
        skin_parts: UnsignedByte,
        main_hand: VarInt,
        text_filtering: Bool,
        server_listings: Bool,
        particle_status: VarInt
    },
    CookieResponse {
        identifier: String,
        payload: PrefixedOptional<PrefixedArray<Byte>>
    },
    PluginMessageServer { channel: String },
    FinishConfigurationAcknowledge(),
    KeepServerAlive { id: Long },
    Pong { id: Int },
    ResourcePackResponse { uuid: UUID, result: VarInt },
    KnownPacksServer {
        packs: PrefixedArray<(String, String, String)>
    },
    CustomClickAction { 
        id: String,
        payload: String
    },
    AcceptCodeOfConduct(),
    
    // Clientbound
    CookieRequest { key: String },
    PluginMessageClient { 
        channel: String,
        payload: String 
    },
    Disconnect { reason: String },
    FinishConfiguration(),
    KeepClientAlive { id: Long },
    Ping { id: Long },
    ResetChat(),
    RegistryData { 
        id: String, 
        entries: PrefixedArray<(String, PrefixedOptional<String>)>
    },
    RemoveResourcePack { uuid: PrefixedOptional<UUID> },
    AddResourcePack { 
        uuid: UUID,
        url: String,
        hash: String,
        forced: Bool,
        prompt_message: PrefixedOptional<String>
    },
    PostEffects { post_effects: PrefixedArray<String> },
    StoreCookie {
        key: String,
        payload: PrefixedArray<Byte>
    },
    Transfer { host: String, port: VarInt },
    FeatureFlags { flags: PrefixedArray<String> },
    UpdateTags { 
        registries: PrefixedArray<(String, PrefixedArray<String>)>
    },
    KnownPacksClient {
        packs: PrefixedArray<(String, String, String)>
    },
    CustomReportDetails {
        details: PrefixedArray<(String, String)>
    },
    ServerLinks {
        links: PrefixedArray<(Either<VarInt, String>, String)>
    },
    ClearDialog(),
    ShowDialog {
        dialog: String
    },
    CodeOfConduct { code_of_conduct: String }
}

impl Parse for Payload {
    fn parse(
        bytes: &mut &[u8],
        direction: Direction,
        id: VarInt
    ) -> Result<PhasePayload, DecodeError> {
        use Direction::{ Clientbound, Serverbound };
        use Payload::*;

        let id = id.0;

        let payload = match (direction, id) {
            // Serverbound
            (Serverbound, 0) => ClientInformation { 
                locale: String::decode(bytes)?, 
                view_distance: Byte::decode(bytes)?,
                chat_mode: VarInt::decode(bytes)?,
                char_colors: Bool::decode(bytes)?,
                skin_parts: UnsignedByte::decode(bytes)?,
                main_hand: VarInt::decode(bytes)?,
                text_filtering: Bool::decode(bytes)?,
                server_listings: Bool::decode(bytes)?,
                particle_status: VarInt::decode(bytes)?
            },
            (Serverbound, 1) => CookieResponse { 
                identifier: String::decode(bytes)?,
                payload: PrefixedOptional::<PrefixedArray::<Byte>>::decode(bytes)?
            },
            (Serverbound, 2) => PluginMessageServer { channel: String::decode(bytes)? },
            (Serverbound, 3) => FinishConfigurationAcknowledge(),
            (Serverbound, 4) => KeepServerAlive { id: Long::decode(bytes)? },
            (Serverbound, 5) => Pong { id: Int::decode(bytes)? },
            (Serverbound, 6) => ResourcePackResponse {
                uuid: UUID::decode(bytes)?,
                result: VarInt::decode(bytes)?
            },
            (Serverbound, 7) => KnownPacksServer { 
                packs: PrefixedArray::<(String, String, String)>::decode(bytes)?
            },
            (Serverbound, 8) => CustomClickAction {
                id: String::decode(bytes)?,
                payload: String::decode(bytes)?
            },
            (Serverbound, 9) => AcceptCodeOfConduct(),

            // Clientbound
            (Clientbound, 0) => CookieRequest { key: String::decode(bytes)? },
            (Clientbound, 1) => PluginMessageClient { 
                channel: String::decode(bytes)?,
                payload: String::decode(bytes)? 
            },
            (Clientbound, 2) => Disconnect { reason: String::decode(bytes)? },
            (Clientbound, 3) => FinishConfiguration(),
            (Clientbound, 4) => KeepServerAlive { id: Long::decode(bytes)? },
            (Clientbound, 5) => Ping { id: Long::decode(bytes)? },
            (Clientbound, 6) => ResetChat(),
            (Clientbound, 7) => RegistryData { 
                id: String::decode(bytes)?,
                entries: PrefixedArray::<(String, PrefixedOptional::<String>)>::decode(bytes)?
            },
            (Clientbound, 8) => RemoveResourcePack { uuid: PrefixedOptional::<UUID>::decode(bytes)? },
            (Clientbound, 9) => AddResourcePack { 
                uuid: UUID::decode(bytes)?,
                url: String::decode(bytes)?,
                hash: String::decode(bytes)?,
                forced: Bool::decode(bytes)?,
                prompt_message: PrefixedOptional::<String>::decode(bytes)?
            },
            (Clientbound, 10) => PostEffects { 
                post_effects: PrefixedArray::<String>::decode(bytes)?
            },
            (Clientbound, 11) => StoreCookie { 
                key: String::decode(bytes)?,
                payload: PrefixedArray::<Byte>::decode(bytes)?
            },
            (Clientbound, 12) => Transfer { 
                host: String::decode(bytes)?,
                port: VarInt::decode(bytes)?
            },
            (Clientbound, 13) => FeatureFlags { flags: PrefixedArray::<String>::decode(bytes)? },
            (Clientbound, 14) => UpdateTags {
                registries: PrefixedArray::<(String, PrefixedArray::<String>)>::decode(bytes)?
            },
            (Clientbound, 15) => KnownPacksClient {
                packs: PrefixedArray::<(String, String, String)>::decode(bytes)?
            },
            (Clientbound, 16) => CustomReportDetails {
                details: PrefixedArray::<(String, String)>::decode(bytes)?
            },
            (Clientbound, 17) => ServerLinks {
                links: PrefixedArray::<(Either::<VarInt, String>, String)>::decode(bytes)?
            },
            (Clientbound, 18) => ClearDialog(),
            (Clientbound, 19) => ShowDialog { dialog: String::decode(bytes)? },
            (Clientbound, 20) => CodeOfConduct { code_of_conduct: String::decode(bytes)? },

            _ => Err(DecodeError::MalformedBytes)?
        };

        Ok(PhasePayload::Configuration(payload))
    }
}
