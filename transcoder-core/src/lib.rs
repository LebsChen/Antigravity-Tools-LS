pub mod proto {
    pub mod exa {
        pub mod browser_pb { tonic::include_proto!("exa.browser_pb"); }
        pub mod cascade_plugins_pb { tonic::include_proto!("exa.cascade_plugins_pb"); }
        pub mod chat_client_server_pb { tonic::include_proto!("exa.chat_client_server_pb"); }
        pub mod chat_pb { tonic::include_proto!("exa.chat_pb"); }
        pub mod code_edit { pub mod code_edit_pb { tonic::include_proto!("exa.code_edit.code_edit_pb"); } }
        pub mod codeium_common_pb { tonic::include_proto!("exa.codeium_common_pb"); }
        pub mod context_module_pb { tonic::include_proto!("exa.context_module_pb"); }
        pub mod cortex_pb { tonic::include_proto!("exa.cortex_pb"); }
        pub mod diff_action_pb { tonic::include_proto!("exa.diff_action_pb"); }
        pub mod extension_server_pb { tonic::include_proto!("exa.extension_server_pb"); }
        pub mod index_pb { tonic::include_proto!("exa.index_pb"); }
        pub mod jetski_cortex_pb { tonic::include_proto!("exa.jetski_cortex_pb"); }
        pub mod language_server_pb { tonic::include_proto!("exa.language_server_pb"); }
        pub mod opensearch_clients_pb { tonic::include_proto!("exa.opensearch_clients_pb"); }
        pub mod reactive_component_pb { tonic::include_proto!("exa.reactive_component_pb"); }
        pub mod unified_state_sync_pb { tonic::include_proto!("exa.unified_state_sync_pb"); }
    }
    pub mod gemini_coder { tonic::include_proto!("gemini_coder"); }
    pub mod google {
        pub mod internal { pub mod cloud { pub mod code { pub mod v1internal { tonic::include_proto!("google.internal.cloud.code.v1internal"); } } } }
        pub mod rpc { tonic::include_proto!("google.rpc"); }
    }
}

pub mod common;
pub mod constants;
pub mod stats;
pub mod version;
pub mod provisioner;
pub mod tools;
pub mod mappers;
pub mod openai;
pub mod anthropic;
pub mod gemini;
pub mod ide;
pub mod cascade;

pub mod transcoder {
    pub use crate::common::{LsConnectionInfo, parse_model_enum_string};
    pub use crate::openai::{OpenAIChatRequest, OpenAITool, OpenAIFunction, OpenAIMessage};
    pub use crate::anthropic::{AnthropicMessageRequest, AnthropicTool, AnthropicMessage, AnthropicStreamEvent};
    pub use crate::gemini::{GeminiContentRequest, GeminiToolWrapper, GeminiFunctionDeclaration, GeminiContent, GeminiPart};
    
    // 导出 Mappers 相关定义
    pub use crate::mappers::{ProtocolMapper, MapperChunk, StreamMetadata};
    pub use crate::mappers::engine::handle_generic_stream;
    pub use crate::mappers::openai::OpenAiMapper;
    pub use crate::mappers::anthropic::AnthropicMapper;
    pub use crate::mappers::gemini::GeminiMapper;
    
    // 导出统计相关
    pub use crate::stats::{StatsManager, TokenStatsEntry, TrafficLog};

    // 导出版本信息相关
    pub use crate::version::{VersionManager, AntigravityVersionInfo};
    pub use crate::provisioner::{AssetProvisioner, ProvisionedAssets, ProvisioningStrategy};

    // 导出 Extension Server 服务定义，以便在 cli-server 中实现
    pub use crate::proto::exa::extension_server_pb::extension_server_service_server::{
        ExtensionServerService, ExtensionServerServiceServer
    };
    pub use crate::proto::exa::extension_server_pb::*;
    pub use crate::proto::exa::extension_server_pb::UnifiedStateSyncUpdate;
    pub use crate::ide::switch_account;
}
