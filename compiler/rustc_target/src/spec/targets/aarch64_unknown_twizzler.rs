use crate::spec::{Cc, LinkerFlavor, Lld, StackProbeType, Target, TargetMetadata, TargetOptions};

const LINKER_SCRIPT: &str = include_str!("./aarch64_unknown_twizzler_linker_script.ld");

pub(crate) fn target() -> Target {
    let mut base = crate::spec::base::twizzler::opts(false);
    base.pre_link_args
        .get_mut(&LinkerFlavor::Gnu(Cc::Yes, Lld::Yes))
        .unwrap()
        .push("--target=aarch64-unknown-twizzler".into());
    base.pre_link_args
        .get_mut(&LinkerFlavor::Gnu(Cc::Yes, Lld::No))
        .unwrap()
        .push("--target=aarch64-unknown-twizzler".into());
    base.stack_probes = StackProbeType::Inline;

    Target {
        llvm_target: "aarch64-unknown-twizzler".into(),
        pointer_width: 64,
        data_layout: "e-m:e-i8:8:32-i16:16:32-i64:64-i128:128-n32:64-S128-Fn32".into(),
        arch: "aarch64".into(),
        options: TargetOptions {
            max_atomic_width: Some(128),
            // this option requires linkers where `linker_is_gnu` is true.
            link_script: Some(LINKER_SCRIPT.into()),
            ..base
        },
        metadata: TargetMetadata {
            description: Some(std::borrow::Cow::Borrowed("aarch64 Twizzler")),
            tier: None,
            host_tools: None,
            std: Some(true),
        },
    }
}
