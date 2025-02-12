// .moon/toolchain.yml

use crate::language_platform::PlatformType;
use crate::toolchain::*;
use rustc_hash::FxHashMap;
use schematic::{validate, Config};
use version_spec::UnresolvedVersionSpec;
use warpgate_api::PluginLocator;

#[cfg(feature = "proto")]
use std::path::Path;

#[cfg(feature = "proto")]
use crate::{inherit_tool, inherit_tool_without_version, is_using_tool_version};

/// Configures an individual platform.
#[derive(Clone, Config, Debug, PartialEq)]
#[config(allow_unknown_fields)]
pub struct ToolConfig {
    /// Location of the WASM plugin to use.
    #[setting(required)]
    pub plugin: Option<PluginLocator>,

    /// The version to download, install, and run tasks with.
    pub version: Option<UnresolvedVersionSpec>,

    /// Arbitrary configuration that'll be passed to the WASM plugin.
    #[setting(flatten)]
    pub config: FxHashMap<String, serde_json::Value>,
}

/// Configures all tools and platforms required for tasks.
/// Docs: https://moonrepo.dev/docs/config/toolchain
#[derive(Clone, Config, Debug)]
pub struct ToolchainConfig {
    #[setting(
        default = "https://moonrepo.dev/schemas/toolchain.json",
        rename = "$schema"
    )]
    pub schema: String,

    /// Extends another toolchain configuration file. Supports a relative
    /// file path or a secure URL.
    #[setting(extend, validate = validate::extends_string)]
    pub extends: Option<String>,

    /// Configures and enables the Bun platform.
    #[setting(nested)]
    pub bun: Option<BunConfig>,

    /// Configures and enables the Deno platform.
    #[setting(nested)]
    pub deno: Option<DenoConfig>,

    /// Configures and enables the Node.js platform.
    #[setting(nested)]
    pub node: Option<NodeConfig>,

    /// Configures and enables the Rust platform.
    #[setting(nested)]
    pub rust: Option<RustConfig>,

    /// Configures and enables the TypeScript platform.
    #[setting(nested)]
    pub typescript: Option<TypeScriptConfig>,
    // TODO: enable once platforms are live!
    // #[setting(flatten, nested)]
    // pub tools: FxHashMap<Id, ToolConfig>,
}

impl ToolchainConfig {
    pub fn get_enabled_platforms(&self) -> Vec<PlatformType> {
        let mut tools = vec![];

        if self.bun.is_some() {
            tools.push(PlatformType::Bun);
        }

        if self.deno.is_some() {
            tools.push(PlatformType::Deno);
        }

        if self.node.is_some() {
            tools.push(PlatformType::Node);
        }

        if self.rust.is_some() {
            tools.push(PlatformType::Rust);
        }

        tools
    }
}

#[cfg(feature = "proto")]
impl ToolchainConfig {
    inherit_tool!(BunConfig, bun, "bun", inherit_proto_bun);

    inherit_tool!(DenoConfig, deno, "deno", inherit_proto_deno);

    inherit_tool!(NodeConfig, node, "node", inherit_proto_node);

    inherit_tool!(RustConfig, rust, "rust", inherit_proto_rust);

    inherit_tool_without_version!(
        TypeScriptConfig,
        typescript,
        "typescript",
        inherit_proto_typescript
    );

    pub fn should_install_proto(&self) -> bool {
        is_using_tool_version!(self, bun);
        is_using_tool_version!(self, deno);
        is_using_tool_version!(self, node);
        is_using_tool_version!(self, node, pnpm);
        is_using_tool_version!(self, node, yarn);
        is_using_tool_version!(self, rust);

        // Special case
        if self
            .node
            .as_ref()
            .is_some_and(|config| config.npm.version.is_some())
        {
            return true;
        }

        false
    }

    pub fn inherit_proto(&mut self, proto_config: &proto_core::ProtoConfig) -> miette::Result<()> {
        self.inherit_proto_bun(proto_config)?;
        self.inherit_proto_deno(proto_config)?;
        self.inherit_proto_node(proto_config)?;
        self.inherit_proto_rust(proto_config)?;
        self.inherit_proto_typescript(proto_config)?;

        if let Some(node_config) = &mut self.node {
            node_config.inherit_proto(proto_config)?;
        }

        Ok(())
    }

    pub fn load<R: AsRef<Path>, P: AsRef<Path>>(
        workspace_root: R,
        path: P,
        proto_config: &proto_core::ProtoConfig,
    ) -> miette::Result<ToolchainConfig> {
        use crate::validate::check_yml_extension;
        use moon_common::color;
        use schematic::ConfigLoader;

        let mut result = ConfigLoader::<ToolchainConfig>::new()
            .set_help(color::muted_light(
                "https://moonrepo.dev/docs/config/toolchain",
            ))
            .set_root(workspace_root)
            .file_optional(check_yml_extension(path.as_ref()))?
            .load()?;

        result.config.inherit_proto(proto_config)?;

        Ok(result.config)
    }

    pub fn load_from<R: AsRef<Path>>(
        workspace_root: R,
        proto_config: &proto_core::ProtoConfig,
    ) -> miette::Result<ToolchainConfig> {
        use moon_common::consts;

        let workspace_root = workspace_root.as_ref();

        Self::load(
            workspace_root,
            workspace_root
                .join(consts::CONFIG_DIRNAME)
                .join(consts::CONFIG_TOOLCHAIN_FILENAME),
            proto_config,
        )
    }
}
